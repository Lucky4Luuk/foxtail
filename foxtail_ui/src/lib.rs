use std::sync::Arc;

use egui_glow::winit::EguiGlow;
use egui_glow::ShaderVersion;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::event::WindowEvent;
use glow::Context;

pub use egui::Context as EguiContext;

pub struct FoxUi {
    egui: EguiGlow,
    window: Arc<Window>,
}

impl FoxUi {
    pub fn new<T>(event_loop: &EventLoop<T>, gl: Arc<Context>, window: Arc<Window>) -> Self {
        let egui = EguiGlow::new(&event_loop, gl, Some(ShaderVersion::Gl140));
        Self {
            egui: egui,
            window: window,
        }
    }

    pub fn draw<F: FnMut(&egui::Context)>(&mut self, f: F) {
        self.egui.run(&self.window, f);
        self.egui.paint(&mut self.window);
    }

    pub fn event(&mut self, event: &WindowEvent) -> bool {
        self.egui.on_event(event).consumed
    }
}
