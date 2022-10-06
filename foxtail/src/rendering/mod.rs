use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use winit::window::Window;
use raw_gl_context::{GlConfig, GlContext};
use glow::*;

pub mod render_pass;
pub mod mesh;
pub mod shader;
pub mod buffer;

#[derive(Debug)]
pub enum RenderError {
    Generic,
}

pub trait Drawable {
    fn draw(&self) -> Result<(), RenderError>;
}

const VS:       &'static str = include_str!("shaders/vs.glsl");
const FB_FS:    &'static str = include_str!("shaders/fb_fs.glsl");

pub(crate) fn gl_error(gl: &Context) {
    // if cfg!(debug_assertions) {}
    let err = unsafe { gl.get_error() };
    if err == 0 { return; }
    error!("[{}] {}!", err, match err {
        INVALID_ENUM => "Invalid enum",
        INVALID_VALUE => "Invalid value",
        INVALID_OPERATION => "Invalid operation",
        STACK_OVERFLOW => "Stack overflow",
        STACK_UNDERFLOW => "Stack underflow",
        OUT_OF_MEMORY => "Out of memory",
        INVALID_FRAMEBUFFER_OPERATION => "Invalid framebuffer operation",
        _ => "Unknown OpenGL error",
    });
}

pub struct Renderer {
    size: winit::dpi::PhysicalSize<u32>,
    pub(crate) context: GlContext,
    pub(crate) is_context_current: bool,
    pub gl: Arc<Context>,
    pub(crate) shader_bound: Arc<AtomicBool>,

    pub(crate) default_fb_shader: Arc<shader::Shader>,
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
        let shader_bound = Arc::new(AtomicBool::new(false));

        let default_fb_shader = shader::Shader::new_from_gl(gl.clone(), shader_bound.clone(), VS, FB_FS);

        Self {
            size: size,
            context: context,
            is_context_current: true,
            gl: gl,
            shader_bound: shader_bound,

            default_fb_shader: Arc::new(default_fb_shader),
        }
    }

    pub(crate) fn gl_make_current(&mut self) {
        self.context.make_current();
        self.is_context_current = true;
    }

    pub(crate) fn gl_make_not_current(&mut self) {
        self.context.make_not_current();
        self.is_context_current = false;
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            unsafe {
                self.gl.viewport(0,0, new_size.width as i32, new_size.height as i32);
            }
        }
    }

    pub fn fence(&self) {
        unsafe {
            self.gl.memory_barrier(glow::ALL_BARRIER_BITS);
        }
    }

    pub fn start_frame(&mut self) -> Result<(), RenderError> {
        self.gl_make_current();
        unsafe {
            self.gl.clear_color(0.2,0.2,0.2,1.0);
            self.gl.clear(COLOR_BUFFER_BIT);
        }
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), RenderError> {
        self.context.swap_buffers();
        self.gl_make_not_current();
        Ok(())
    }
}
