use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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

pub struct UniformInterface<'u> {
    bound_shader: &'u NativeProgram,
    gl: Arc<Context>,
}

impl<'u> UniformInterface<'u> {
    pub fn set_f32(&self, name: &str, val: f32) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_1_f32(loc.as_ref(), val); }
    }

    pub fn set_u32(&self, name: &str, val: u32) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_1_u32(loc.as_ref(), val); }
    }

    pub fn set_vec2(&self, name: &str, val: [f32; 2]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_2_f32(loc.as_ref(), val[0], val[1]); }
    }

    pub fn set_vec3(&self, name: &str, val: [f32; 3]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_3_f32(loc.as_ref(), val[0], val[1], val[2]); }
    }

    pub fn set_vec4(&self, name: &str, val: [f32; 4]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_4_f32(loc.as_ref(), val[0], val[1], val[2], val[3]); }
    }

    pub fn set_mat2(&self, name: &str, val: [f32; 2*2]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_matrix_2_f32_slice(loc.as_ref(), false, &val); }
    }

    pub fn set_mat3(&self, name: &str, val: [f32; 3*3]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_matrix_3_f32_slice(loc.as_ref(), false, &val); }
    }

    pub fn set_mat4(&self, name: &str, val: [f32; 4*4]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_matrix_4_f32_slice(loc.as_ref(), false, &val); }
    }
}

pub struct Shader {
    program: NativeProgram,
    gl: Arc<Context>,
    shader_bound: Arc<AtomicBool>,
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
        let gl = renderer.gl.clone();
        let shader_bound = renderer.shader_bound.clone();
        Self::new_from_gl(gl, shader_bound, vs, fs)
    }

    pub(crate) fn new_from_gl(gl: Arc<Context>, shader_bound: Arc<AtomicBool>, vs: &str, fs: &str) -> Self {
        unsafe {
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
                shader_bound: shader_bound,
            }
        }
    }

    fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.shader_bound.store(true, Ordering::Release);
        }
    }

    fn unbind(&self) {
        unsafe {
            self.gl.use_program(None);
            self.shader_bound.store(false, Ordering::Release);
        }
    }

    /// Runs a closure while the shader is bound
    pub fn while_bound<F: FnOnce(UniformInterface) -> Result<(), super::RenderError>>(&self, f: F) -> Result<(), super::RenderError> {
        self.bind();
        let uni = UniformInterface {
            bound_shader: &self.program,
            gl: self.gl.clone(),
        };
        f(uni)?;
        self.unbind();
        Ok(())
    }
}

pub struct ComputeShader {
    program: NativeProgram,
    gl: Arc<Context>,
    shader_bound: Arc<AtomicBool>,
}

impl Drop for ComputeShader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}

impl ComputeShader {
    pub fn new(renderer: &super::Renderer, cs: &str) -> Self {
        let gl = renderer.gl.clone();
        let shader_bound = renderer.shader_bound.clone();
        Self::new_from_gl(gl, shader_bound, cs)
    }

    pub(crate) fn new_from_gl(gl: Arc<Context>, shader_bound: Arc<AtomicBool>, cs: &str) -> Self {
        unsafe {
            let program = gl.create_program().expect("Failed to create shader program!");

            let cs_shader = compile_stage(&gl, COMPUTE_SHADER, cs);

            gl.attach_shader(program, cs_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                error!("Program link error: {}", gl.get_program_info_log(program));
                panic!("Failed to link program!");
            }
            gl.detach_shader(program, cs_shader);
            gl.delete_shader(cs_shader);

            Self {
                program: program,
                gl: gl,
                shader_bound: shader_bound,
            }
        }
    }

    pub fn set_uniforms<F: FnOnce(UniformInterface)>(&self, f: F) {
        self.bind();
        let uni = UniformInterface {
            bound_shader: &self.program,
            gl: self.gl.clone(),
        };
        f(uni);
        self.unbind();
    }

    fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.shader_bound.store(true, Ordering::Release);
        }
    }

    fn unbind(&self) {
        unsafe {
            self.gl.use_program(None);
            self.shader_bound.store(false, Ordering::Release);
        }
    }

    /// Dispatches the compute shader
    pub fn dispatch(&self, num_groups: [u32; 3]) {
        self.bind();
        unsafe {
            self.gl.dispatch_compute(num_groups[0], num_groups[1], num_groups[2]);
        }
        self.unbind();
    }
}
