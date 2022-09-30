#[macro_use] extern crate log;

use std::ops::Deref;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoopProxy, EventLoopBuilder},
    window::{WindowBuilder, Window},
};

pub mod prelude;
pub mod rendering;

pub trait App {
    fn event(&mut self, event: &WindowEvent) -> bool { false }
    fn update(&mut self, ctx: &Context) {}
    fn render(&mut self, ctx: &Context) {}
    fn on_resize(&mut self, size: (i32, i32)) {}
}

#[derive(Debug)]
pub enum EngineEvent {
    SetTitle(String),
}

struct State<A: App> {
    app: A,
    renderer: rendering::Renderer,
    event_loop: EventLoopProxy<EngineEvent>,
}

impl<A: App> State<A> {
    fn new<F: Fn(&Context) -> A>(window: &Window, event_loop: EventLoopProxy<EngineEvent>, f: F) -> Self {
        let renderer = rendering::Renderer::new(window);

        let ctx = Context::new(&renderer, &event_loop);
        let app = f(&ctx);
        drop(ctx);

        Self {
            app: app,
            renderer: renderer,
            event_loop: event_loop,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.gl_make_current();
        self.renderer.resize(new_size);
        self.app.on_resize((new_size.width as i32, new_size.height as i32));
        self.renderer.gl_make_not_current();
    }

    // TODO: Actually process events here
    fn event(&mut self, event: &WindowEvent) -> bool {
        self.app.event(event)
    }

    fn update(&mut self) {
        if !self.renderer.is_context_current {
            self.renderer.gl_make_current();
        }
        let ctx = Context::new(&self.renderer, &self.event_loop);
        self.app.update(&ctx);
        drop(ctx);
        if self.renderer.is_context_current {
            self.renderer.gl_make_not_current();
        }
    }

    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size()
    }

    fn render(&mut self) -> Result<(), rendering::RenderError> {
        self.renderer.start_frame()?;
        let ctx = Context::new(&self.renderer, &self.event_loop);
        self.app.render(&ctx);
        self.renderer.end_frame()?;
        Ok(())
    }
}

// Contains references to parts of the current state, for use
// in the user facing API
pub struct Context<'c> {
    renderer: &'c rendering::Renderer,
    event_loop: &'c EventLoopProxy<EngineEvent>,
}

impl<'c> Context<'c> {
    fn new(renderer: &'c rendering::Renderer, event_loop: &'c EventLoopProxy<EngineEvent>) -> Self {
        Self {
            renderer: renderer,
            event_loop: event_loop,
        }
    }

    pub fn set_window_title<S: Into<String>>(&self, name: S) {
        self.event_loop.send_event(EngineEvent::SetTitle(name.into())).map_err(|e| error!("Event loop proxy error {}", e)).expect("The event loop closed!");
    }
}

impl<'c> Deref for Context<'c> {
    type Target = rendering::Renderer;
    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

pub fn run<A: App + 'static, F: Fn(&Context) -> A>(f: F) {
    pretty_env_logger::formatted_timed_builder().filter_level(log::LevelFilter::max()).init();

    let event_loop = EventLoopBuilder::<EngineEvent>::with_user_event().build();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window, event_loop.create_proxy(), f);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => if !state.event(event) {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                },
                _ => {}
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::UserEvent(engine_event) => {
            match engine_event {
                EngineEvent::SetTitle(title) => window.set_title(&title),
            }
        },
        Event::MainEventsCleared => {
            state.update();
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        },
        _ => {}
    });
}
