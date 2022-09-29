use std::sync::Arc;

use winit::window::Window;
use raw_gl_context::{GlConfig, GlContext};
use glow::*;

pub mod render_pass;
pub mod mesh;
pub mod shader;

#[derive(Debug)]
pub enum RenderError {
    Generic,
}

pub trait Drawable {
    /// Should only be called while a shader is bound
    fn draw(&self) -> Result<(), RenderError>;
}

pub struct Renderer {
    size: winit::dpi::PhysicalSize<u32>,
    pub(crate) context: GlContext,
    pub(crate) is_context_current: bool,
    pub(crate) gl: Arc<Context>,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let mut conf = GlConfig::default();
        conf.version = (4,5);
        let context = GlContext::create(window, conf).expect("Failed to create OpenGL context!");
        let gl = unsafe {
            context.make_current();
            let gl = Context::from_loader_function(|symbol| context.get_proc_address(symbol) as *const _);
            Arc::new(gl)
        };

        Self {
            size: size,
            context: context,
            is_context_current: true,
            gl: gl,
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.context.make_current();
            unsafe {
                self.gl.viewport(0,0, new_size.width as i32, new_size.height as i32);
            }
            self.context.make_not_current();
        }
    }

    pub fn start_frame(&mut self) -> Result<(), RenderError> {
        self.context.make_current();
        self.is_context_current = true;
        unsafe {
            self.gl.clear_color(0.2,0.2,0.2,1.0);
            self.gl.clear(COLOR_BUFFER_BIT);
        }
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), RenderError> {
        self.context.swap_buffers();
        self.context.make_not_current();
        self.is_context_current = false;
        Ok(())
    }
}
