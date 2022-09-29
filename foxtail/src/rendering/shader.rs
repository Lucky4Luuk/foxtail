use std::sync::Arc;
use glow::*;

unsafe fn compile_stage(gl: &Context, stage: u32, src: &str) -> NativeShader {
    let shader = gl.create_shader(stage).expect("Failed to create shader!");
    gl.shader_source(shader, src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        error!("Shader compile error: {}", gl.get_shader_info_log(shader));
        panic!("Failed to compile shader!");
    }
    shader
}

pub struct Shader {
    program: NativeProgram,
    gl: Arc<Context>
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}

impl Shader {
    pub fn new(renderer: &super::Renderer, vs: &str, fs: &str) -> Self {
        unsafe {
            let gl = renderer.gl.clone();
            trace!("GL cloned!");

            let program = gl.create_program().expect("Failed to create shader program!");

            let vs_shader = compile_stage(&gl, VERTEX_SHADER, vs);
            let fs_shader = compile_stage(&gl, FRAGMENT_SHADER, fs);

            gl.attach_shader(program, vs_shader);
            gl.attach_shader(program, fs_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                error!("Program link error: {}", gl.get_program_info_log(program));
                panic!("Failed to link program!");
            }
            gl.detach_shader(program, vs_shader);
            gl.detach_shader(program, fs_shader);
            gl.delete_shader(vs_shader);
            gl.delete_shader(fs_shader);

            Self {
                program: program,
                gl: gl,
            }
        }
    }

    fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }

    fn unbind(&self) {
        unsafe {
            self.gl.use_program(None);
        }
    }

    /// Runs a closure while the shader is bound
    pub fn while_bound<F: FnOnce() -> Result<(), super::RenderError>>(&self, f: F) -> Result<(), super::RenderError> {
        self.bind();
        f()?;
        self.unbind();
        Ok(())
    }
}
