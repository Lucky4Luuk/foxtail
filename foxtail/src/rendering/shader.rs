use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use glow::*;

fn format_shader_errors(src: &str, log: &str) -> String {
    let src_split = src.lines().collect::<Vec<&str>>();
    let mut formatted_errors = String::new();
    for line in log.lines() {
        let line_split = line.split(":").collect::<Vec<&str>>();
        let line_number = line_split.get(0).map(|s| {
            let mut s = s.trim().to_string();
            s = s.replace("0(", "");
            s.pop();
            s.parse::<usize>().map(|i| i.wrapping_sub(1)).unwrap_or(0)
        }).unwrap_or(0);
        let err_code = line_split.get(1).map(|s| {
            let mut s = s.trim().to_string();
            s = s.replace("error ", "");
            s
        }).unwrap_or(String::from(""));
        let err_line = line_split.get(2).map(|s| s.trim()).unwrap_or("");

        let mut var_name: Option<String> = None;
        if &err_code == "C1503" && err_line.contains("undefined variable") {
            let mut var = err_line.replace("undefined variable", "").trim().to_string();
            var = var[1..].to_string();
            var.pop();
            var_name = Some(var);
        }

        formatted_errors.push_str("\x1b[1;31m");
        formatted_errors.push_str(line);
        formatted_errors.push_str("\x1b[0m");
        formatted_errors.push('\n');
        let min = line_number.wrapping_sub(1).min(line_number);
        let max = line_number.wrapping_add(1).max(line_number);
        let line_num_str = line_number.wrapping_add(1).to_string();
        for i in min..=max {
            formatted_errors.push_str("\x1b[1;36m");
            if i == line_number {
                formatted_errors.push_str(&format!(" {} | ", line_num_str));
            } else {
                for _ in 0..(line_num_str.len()+2) { formatted_errors.push(' '); }
                formatted_errors.push_str("| ");
            }
            formatted_errors.push_str("\x1b[22;0m");
            let mut code_line = src_split.get(i).map(|s| *s).unwrap_or("CODE NOT FOUND").to_string();
            if let Some(var_name) = var_name.as_ref() {
                code_line = code_line.replace(var_name, &format!("\x1b[1;33m{}\x1b[22;0m", var_name));
            }
            formatted_errors.push_str(&code_line);
            if i == line_number { formatted_errors.push_str("\x1b[1;31m <- Error occurs here\x1b[22;0m"); }
            formatted_errors.push('\n');
        }
        formatted_errors.push('\n');
    }
    formatted_errors
}

unsafe fn compile_stage(gl: &Context, name: &str, stage: u32, src: &str) -> NativeShader {
    let shader = gl.create_shader(stage).expect("Failed to create shader!");
    gl.shader_source(shader, src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        let log = gl.get_shader_info_log(shader);
        error!("Shader compile error: {}", log);
        let formatted_errors = format_shader_errors(src, &log);
        panic!("Failed to compile shader (`{}`)! Errors:\n{}", name, formatted_errors);
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

    pub fn set_u32(&self, name: &str, val: u32) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_1_u32(loc.as_ref(), val); }
    }

    pub fn set_uvec2(&self, name: &str, val: [u32; 2]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_2_u32(loc.as_ref(), val[0], val[1]); }
    }

    pub fn set_uvec3(&self, name: &str, val: [u32; 3]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_3_u32(loc.as_ref(), val[0], val[1], val[2]); }
    }

    pub fn set_uvec4(&self, name: &str, val: [u32; 4]) {
        let loc = unsafe { self.gl.get_uniform_location(*self.bound_shader, name) };
        unsafe { self.gl.uniform_4_u32(loc.as_ref(), val[0], val[1], val[2], val[3]); }
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
    pub fn new(renderer: &super::Renderer, (vs, vs_name): (&str, &str), (fs, fs_name): (&str, &str)) -> Self {
        let gl = renderer.gl.clone();
        let shader_bound = renderer.shader_bound.clone();
        Self::new_from_gl(gl, shader_bound, vs, vs_name, fs, fs_name)
    }

    pub(crate) fn new_from_gl(gl: Arc<Context>, shader_bound: Arc<AtomicBool>, vs: &str, vs_name: &str, fs: &str, fs_name: &str) -> Self {
        unsafe {
            let program = gl.create_program().expect("Failed to create shader program!");

            let vs_shader = compile_stage(&gl, &vs_name, VERTEX_SHADER, vs);
            let fs_shader = compile_stage(&gl, &fs_name, FRAGMENT_SHADER, fs);

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
    pub fn new(renderer: &super::Renderer, (cs, cs_name): (&str, &str)) -> Self {
        let gl = renderer.gl.clone();
        let shader_bound = renderer.shader_bound.clone();
        Self::new_from_gl(gl, shader_bound, cs, cs_name)
    }

    pub(crate) fn new_from_gl(gl: Arc<Context>, shader_bound: Arc<AtomicBool>, cs: &str, cs_name: &str) -> Self {
        unsafe {
            let program = gl.create_program().expect("Failed to create shader program!");

            let cs_shader = compile_stage(&gl, cs_name, COMPUTE_SHADER, cs);

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

    /// Dispatches the compute shader
    pub fn dispatch(&self, num_groups: [u32; 3]) {
        unsafe {
            self.gl.dispatch_compute(num_groups[0], num_groups[1], num_groups[2]);
        }
    }
}
