#[macro_use] extern crate log;

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopBuilder},
    window::{WindowBuilder, Window, Fullscreen as WinitFullscreen},
    monitor::VideoMode,
};
use winit_input_helper::WinitInputHelper;
use gilrs::{Gilrs, Event as GilEvent};
use glow::HasContext;

pub use glow;

pub mod prelude;
pub mod rendering;

#[cfg(target_os = "windows")]
pub mod windows_perf_flags {
    #[no_mangle]
    pub static NvOptimusEnablement: u32 = 1;
    #[no_mangle]
    pub static AmdPowerXpressRequestHighPerformance: i32 = 1;
}

pub trait App {
    fn update(&mut self, _ctx: &Context) {}
    fn render(&mut self, _ctx: &Context) {}
    fn on_resize(&mut self, _size: (i32, i32)) {}
}

#[derive(Debug)]
pub enum Fullscreen {
    Borderless,
    Exclusive(VideoMode),
}

#[derive(Debug, PartialEq)]
pub enum CullSide {
    Front,
    Back,
}

#[derive(Debug)]
pub enum EngineEvent {
    SetTitle(String),
    SetMaximized(bool),
    SetMinimized(bool),
    SetFullscreen(Option<Fullscreen>),
    SetSize((u32, u32)),
}

struct State<A: App> {
    app: A,
    renderer: rendering::Renderer,
    fox_ui: foxtail_ui::FoxUi,
    event_loop: EventLoopProxy<EngineEvent>,

    video_modes: Vec<VideoMode>,
}

impl<A: App> State<A> {
    fn new<F: Fn(&Context) -> A>(
        window: Arc<Mutex<Window>>,
        event_loop: &EventLoop<EngineEvent>,
        f: F,
        input: &WinitInputHelper,
        gil_input: &Gilrs,
    ) -> Self {
        let mut renderer = rendering::Renderer::new(&window);

        let mut fox_ui = foxtail_ui::FoxUi::new(event_loop, renderer.gl.clone(), window.clone());
        let event_loop_proxy = event_loop.create_proxy();

        let video_modes = window.lock().unwrap().current_monitor().expect("No monitor detected!").video_modes().collect();

        renderer.start_frame().expect("Failed to create a frame!");
        let mut ctx = Context::new(&renderer, &event_loop_proxy, &mut fox_ui, &input, &gil_input, &video_modes);
        let app = f(&mut ctx);
        drop(ctx);
        renderer.end_frame().expect("Failed to end a frame!");

        Self {
            app,
            renderer,
            fox_ui,
            event_loop: event_loop_proxy,

            video_modes,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.gl_make_current();
        self.renderer.resize(new_size);
        self.app.on_resize((new_size.width as i32, new_size.height as i32));
        self.renderer.gl_make_not_current();
    }

    fn update(&mut self, input: &WinitInputHelper, gil_input: &Gilrs) {
        puffin::profile_function!();
        if !self.renderer.is_context_current {
            self.renderer.gl_make_current();
        }
        let ctx = Context::new(&self.renderer, &self.event_loop, &self.fox_ui, input, gil_input, &self.video_modes);
        self.app.update(&ctx);
        drop(ctx);
        if self.renderer.is_context_current {
            self.renderer.gl_make_not_current();
        }
    }

    fn render(&mut self, input: &WinitInputHelper, gil_input: &Gilrs) -> Result<(), rendering::RenderError> {
        puffin::profile_function!();
        self.renderer.start_frame()?;
        let ctx = Context::new(&self.renderer, &self.event_loop, &self.fox_ui, input, gil_input, &self.video_modes);
        self.app.render(&ctx);
        unsafe {
            self.renderer.gl.disable(glow::FRAMEBUFFER_SRGB);
        }
        self.renderer.end_frame()?;
        Ok(())
    }
}

// Contains references to parts of the current state, for use
// in the user facing API
pub struct Context<'c> {
    renderer: &'c rendering::Renderer,
    event_loop: &'c EventLoopProxy<EngineEvent>,
    fox_ui: &'c foxtail_ui::FoxUi,

    input: &'c winit_input_helper::WinitInputHelper,
    gil_input: &'c Gilrs,

    video_modes: &'c Vec<VideoMode>,
}

impl<'c> Context<'c> {
    fn new(
        renderer: &'c rendering::Renderer,
        event_loop: &'c EventLoopProxy<EngineEvent>,
        fox_ui: &'c foxtail_ui::FoxUi,
        input: &'c winit_input_helper::WinitInputHelper,
        gil_input: &'c Gilrs,
        video_modes: &'c Vec<VideoMode>
    ) -> Self {
        Self {
            renderer,
            event_loop,
            fox_ui,

            input,
            gil_input,

            video_modes,
        }
    }

    pub fn input(&self) -> &WinitInputHelper {
        self.input
    }

    pub fn gamepads(&self) -> gilrs::ConnectedGamepadsIterator {
        self.gil_input.gamepads()
    }

    pub fn video_modes(&self) -> &Vec<VideoMode> {
        self.video_modes
    }

    pub fn enable_alpha_blending(&self, enabled: bool) {
        if enabled {
            unsafe {
                self.renderer.gl.enable(glow::BLEND);
                self.renderer.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            }
        } else {
            unsafe { self.renderer.gl.disable(glow::BLEND); }
        }
    }

    pub fn enable_depth_buffer(&self, enabled: bool) {
        if enabled {
            unsafe { self.renderer.gl.enable(glow::DEPTH_TEST); }
        } else {
            unsafe { self.renderer.gl.disable(glow::DEPTH_TEST); }
        }
    }

    pub fn enable_backface_culling(&self, side: Option<CullSide>) {
        if let Some(side) = side {
            unsafe {
                self.renderer.gl.enable(glow::CULL_FACE);
                self.renderer.gl.cull_face(if side == CullSide::Front { glow::FRONT } else { glow::BACK });
                self.renderer.gl.front_face(glow::CW);
            }
        } else {
            unsafe { self.renderer.gl.disable(glow::CULL_FACE); }
        }
    }

    pub fn set_window_title<S: Into<String>>(&self, name: S) {
        self.event_loop.send_event(EngineEvent::SetTitle(name.into())).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }

    pub fn set_maximized(&self, maximized: bool) {
        self.event_loop.send_event(EngineEvent::SetMaximized(maximized)).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }

    pub fn set_minimized(&self, minimized: bool) {
        self.event_loop.send_event(EngineEvent::SetMinimized(minimized)).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }

    pub fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        self.event_loop.send_event(EngineEvent::SetFullscreen(fullscreen)).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }

    pub fn set_size(&self, size: (u32, u32)) {
        self.event_loop.send_event(EngineEvent::SetSize(size)).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }

    pub fn event_loop(&self) -> &EventLoopProxy<EngineEvent> {
        &self.event_loop
    }

    pub fn draw_ui<F: FnMut(&foxtail_ui::EguiContext)>(&self, f: F) {
        self.fox_ui.draw(f);
    }
}

impl<'c> Deref for Context<'c> {
    type Target = rendering::Renderer;
    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

pub fn run<A: App + 'static, F: Fn(&Context) -> A>(f: F) {
    // pretty_env_logger::formatted_timed_builder().filter_level(log::LevelFilter::max()).init();
    pretty_env_logger::formatted_timed_builder().filter_level(log::LevelFilter::Debug).init();

    let event_loop = EventLoopBuilder::<EngineEvent>::with_user_event().build();
    let window = Arc::new(Mutex::new(WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize::<u32>::new(1280u32, 720u32)).build(&event_loop).unwrap()));

    let mut input = WinitInputHelper::new();
    let mut gil_input = Gilrs::new().unwrap();

    let mut state = State::new(window.clone(), &event_loop, f, &input, &gil_input);

    event_loop.run(move |event, _, control_flow| {
        puffin::GlobalProfiler::lock().new_frame();

        while let Some(GilEvent { id, event, .. }) = gil_input.next_event() {
            match event {
                _ => {},
            }
        }

        let mut event_consumed = false;
        if let Event::WindowEvent { ref event, .. } = event {
            if state.fox_ui.event(&event) {
                event_consumed = true;
            }
        }
        if let Event::UserEvent(ref ue) = event {
            match ue {
                EngineEvent::SetTitle(title) => window.lock().unwrap().set_title(title),
                EngineEvent::SetMaximized(max) => window.lock().unwrap().set_maximized(*max),
                EngineEvent::SetMinimized(min) => window.lock().unwrap().set_minimized(*min),
                EngineEvent::SetFullscreen(full) => {
                    if let Some(fullscreen) = full {
                        match fullscreen {
                            Fullscreen::Borderless => window.lock().unwrap().set_fullscreen(Some(WinitFullscreen::Borderless(None))),
                            Fullscreen::Exclusive(mode) => window.lock().unwrap().set_fullscreen(Some(WinitFullscreen::Exclusive(mode.clone()))),
                        }
                    } else {
                        window.lock().unwrap().set_fullscreen(None);
                    }
                },
                EngineEvent::SetSize((width, height)) => {
                    window.lock().unwrap().set_inner_size::<winit::dpi::PhysicalSize<u32>>((*width, *height).into());
                }
            }
        }
        if !event_consumed {
            if input.update(&event) {
                if input.close_requested() || input.destroyed() { *control_flow = ControlFlow::Exit; }
                if let Some(size) = input.window_resized() {
                    state.resize(size);
                }
                state.update(&input, &gil_input);
                if let Err(e) = state.render(&input, &gil_input) {
                    error!("Render error occured!");
                    panic!("{:?}", e);
                }
            }
        }
    });
}
