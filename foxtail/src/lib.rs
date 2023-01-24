#[macro_use] extern crate log;

use std::ops::Deref;
use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopBuilder},
    window::{WindowBuilder, Window},
};
use winit_input_helper::WinitInputHelper;
use glow::HasContext;

pub mod prelude;
pub mod rendering;

pub trait App {
    fn event(&mut self, _input: &prelude::Input) {}
    fn update(&mut self, _ctx: &mut Context) {}
    fn render(&mut self, _ctx: &mut Context) {}
    fn on_resize(&mut self, _size: (i32, i32)) {}
}

#[derive(Debug)]
pub enum EngineEvent {
    SetTitle(String),
}

struct State<A: App> {
    app: A,
    renderer: rendering::Renderer,
    fox_ui: foxtail_ui::FoxUi,
    event_loop: EventLoopProxy<EngineEvent>,
}

impl<A: App> State<A> {
    fn new<F: Fn(&mut Context) -> A>(window: Arc<Window>, event_loop: &EventLoop<EngineEvent>, f: F) -> Self {
        let renderer = rendering::Renderer::new(&window);

        let mut fox_ui = foxtail_ui::FoxUi::new(event_loop, renderer.gl.clone(), window.clone());
        let event_loop_proxy = event_loop.create_proxy();

        let mut ctx = Context::new(&renderer, &event_loop_proxy, &mut fox_ui);
        let app = f(&mut ctx);
        drop(ctx);

        Self {
            app: app,
            renderer: renderer,
            fox_ui: fox_ui,
            event_loop: event_loop_proxy,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.gl_make_current();
        self.renderer.resize(new_size);
        self.app.on_resize((new_size.width as i32, new_size.height as i32));
        self.renderer.gl_make_not_current();
    }

    fn update(&mut self) {
        puffin::profile_function!();
        if !self.renderer.is_context_current {
            self.renderer.gl_make_current();
        }
        let mut ctx = Context::new(&self.renderer, &self.event_loop, &mut self.fox_ui);
        self.app.update(&mut ctx);
        drop(ctx);
        if self.renderer.is_context_current {
            self.renderer.gl_make_not_current();
        }
    }

    fn render(&mut self) -> Result<(), rendering::RenderError> {
        puffin::profile_function!();
        self.renderer.start_frame()?;
        let mut ctx = Context::new(&self.renderer, &self.event_loop, &mut self.fox_ui);
        self.app.render(&mut ctx);
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
    fox_ui: &'c mut foxtail_ui::FoxUi,
}

impl<'c> Context<'c> {
    fn new(renderer: &'c rendering::Renderer, event_loop: &'c EventLoopProxy<EngineEvent>, fox_ui: &'c mut foxtail_ui::FoxUi) -> Self {
        Self {
            renderer: renderer,
            event_loop: event_loop,
            fox_ui: fox_ui,
        }
    }

    pub fn set_window_title<S: Into<String>>(&self, name: S) {
        self.event_loop.send_event(EngineEvent::SetTitle(name.into())).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }

    pub fn event_loop(&self) -> &EventLoopProxy<EngineEvent> {
        &self.event_loop
    }

    pub fn draw_ui<F: FnMut(&foxtail_ui::EguiContext)>(&mut self, f: F) {
        self.fox_ui.draw(f);
    }
}

impl<'c> Deref for Context<'c> {
    type Target = rendering::Renderer;
    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

pub fn run<A: App + 'static, F: Fn(&mut Context) -> A>(f: F) {
    pretty_env_logger::formatted_timed_builder().filter_level(log::LevelFilter::max()).init();

    let event_loop = EventLoopBuilder::<EngineEvent>::with_user_event().build();
    let window = Arc::new(WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize::<u32>::new(1280u32, 720u32)).build(&event_loop).unwrap());

    let mut state = State::new(window.clone(), &event_loop, f);

    let mut input = WinitInputHelper::new();
    let mut input_captured = false;

    event_loop.run(move |event, _, control_flow| {
        puffin::GlobalProfiler::lock().new_frame();
        if let Event::WindowEvent { ref event, .. } = event {
            if state.fox_ui.event(&event) {
                input_captured = true;
            } else {
                input_captured = false;
            }
        }
        if let Event::UserEvent(ref ue) = event {
            match ue {
                EngineEvent::SetTitle(title) => window.set_title(title),
            }
        }
        if input.update(&event) {
            if input.quit() { *control_flow = ControlFlow::Exit; }
            if let Some(size) = input.window_resized() {
                state.resize(size);
            }
            if input_captured == false { state.app.event(&input); }
            state.update();
            if let Err(e) = state.render() {
                error!("Render error occured!");
                panic!("{:?}", e);
            }
        }
    });
}
