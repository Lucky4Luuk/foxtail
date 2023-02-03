use std::sync::{Arc, Mutex};

use egui_glow::winit::EguiGlow;
use egui_glow::ShaderVersion;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::event::WindowEvent;
use glow::Context;

pub use egui::Context as EguiContext;

pub struct FoxUi {
    egui: Mutex<EguiGlow>,
    window: Arc<Mutex<Window>>,
}

impl FoxUi {
    pub fn new<T>(event_loop: &EventLoop<T>, gl: Arc<Context>, window: Arc<Mutex<Window>>) -> Self {
        let egui = EguiGlow::new(&event_loop, gl, Some(ShaderVersion::Gl140));
        Self {
            egui: Mutex::new(egui),
            window: window,
        }
    }

    pub fn draw<F: FnMut(&egui::Context)>(&self, f: F) {
        let mut window_lock = self.window.lock().unwrap();
        let mut egui_lock = self.egui.lock().unwrap();
        egui_lock.run(&window_lock, f);
        egui_lock.paint(&mut window_lock);
    }

    pub fn event(&self, event: &WindowEvent) -> bool {
        let mut egui_lock = self.egui.lock().unwrap();
        egui_lock.on_event(event).consumed
    }
}
