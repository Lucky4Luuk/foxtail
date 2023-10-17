use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use glow::*;

pub struct Framebuffer {
    fbo: glow::Framebuffer,
    tex: glow::Texture,
    rbo: glow::Renderbuffer,
    gl: Arc<Context>,
    shader_bound: Arc<AtomicBool>,
    default_fb_shader: Arc<super::shader::Shader>,
    mesh: super::mesh::Mesh,
    size: (usize, usize),
}

impl super::Drawable for Framebuffer {
    fn draw(&self) -> Result<(), super::RenderError> {
        if self.shader_bound.load(Ordering::Acquire) {
            self.bind_tex(0);
            self.mesh.draw()?;
            self.unbind_tex();
        } else {
            self.default_fb_shader.while_bound(|_| {
                self.bind_tex(0);
                self.mesh.draw()?;
                self.unbind_tex();
                Ok(())
            })?;
        }
        Ok(())
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.tex);
            self.gl.delete_renderbuffer(self.rbo);
        }
    }
}

impl Framebuffer {
    fn create_fb(gl: Arc<Context>, size: (i32, i32)) -> (NativeFramebuffer, NativeTexture, NativeRenderbuffer) {
        let fbo = unsafe { gl.create_framebuffer().map_err(|e| error!("{}", e)).expect("Failed to create framebuffer!") };
        let (tex, rbo) = unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, Some(fbo));

            let tex = gl.create_texture().map_err(|e| error!("{}", e)).expect("Failed to create framebuffer color attachment!");
            gl.bind_texture(TEXTURE_2D, Some(tex));
            gl.tex_image_2d(TEXTURE_2D, 0, RGBA32F as i32, size.0, size.1, 0, RGBA, UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);
            gl.bind_texture(TEXTURE_2D, None);
            gl.framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, Some(tex), 0);

            let rbo = gl.create_renderbuffer().map_err(|e| error!("{}", e)).expect("Failed to create framebuffer renderbuffer!");
            gl.bind_renderbuffer(RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(RENDERBUFFER, DEPTH24_STENCIL8, size.0, size.1);
            gl.bind_renderbuffer(RENDERBUFFER, None);
            gl.framebuffer_renderbuffer(FRAMEBUFFER, DEPTH_STENCIL_ATTACHMENT, RENDERBUFFER, Some(rbo));

            let fb_status = gl.check_framebuffer_status(FRAMEBUFFER);
            if fb_status != FRAMEBUFFER_COMPLETE {
                error!("Incomplete framebuffer! Code: {}", fb_status);
                panic!("Incomplete framebuffer!");
            }
            gl.bind_framebuffer(FRAMEBUFFER, None);

            (tex, rbo)
        };
        (fbo, tex, rbo)
    }

    pub fn with_resolution(renderer: &super::Renderer, size: (i32, i32)) -> Self {
        let gl = renderer.gl.clone();
        let (fbo, tex, rbo) = Self::create_fb(gl.clone(), size);
        super::gl_error(&gl);
        Self {
            fbo,
            tex,
            rbo,
            gl,
            shader_bound: renderer.shader_bound.clone(),
            default_fb_shader: renderer.default_fb_shader.clone(),
            mesh: super::mesh::Mesh::quad(renderer),
            size: (size.0 as usize, size.1 as usize),
        }
    }

    pub fn new(renderer: &super::Renderer) -> Self {
        let size = renderer.size();
        Self::with_resolution(renderer, (size.width as i32, size.height as i32))
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.tex);
            self.gl.delete_renderbuffer(self.rbo);
        }
        let (fbo, tex, rbo) = Self::create_fb(self.gl.clone(), size);
        super::gl_error(&self.gl);
        self.fbo = fbo;
        self.tex = tex;
        self.rbo = rbo;
        self.size = (size.0 as usize, size.1 as usize);
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn bind_tex(&self, location: u32) {
        unsafe {
            self.gl.active_texture(location);
            self.gl.bind_texture(TEXTURE_2D, Some(self.tex));
        }
    }

    pub fn unbind_tex(&self) {
        unsafe {
            self.gl.bind_texture(TEXTURE_2D, None);
        }
    }

    fn bind(&self) {
        unsafe {
            self.gl.bind_framebuffer(FRAMEBUFFER, Some(self.fbo));
        }
    }

    fn unbind(&self) {
        unsafe {
            self.gl.bind_framebuffer(FRAMEBUFFER, None);
        }
    }

    pub fn clear(&self) {
        unsafe {
            self.gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT | STENCIL_BUFFER_BIT);
        }
    }

    /// Runs a closure while the framebuffer is bound
    pub fn while_bound<F: FnOnce() -> Result<(), super::RenderError>>(&self, f: F) -> Result<(), super::RenderError> {
        let mut og_viewport = [0i32; 4];
        unsafe { self.gl.get_parameter_i32_slice(glow::VIEWPORT, &mut og_viewport); }
        unsafe { self.gl.viewport(0, 0, self.size.0 as i32, self.size.1 as i32); }
        self.bind();
        f()?;
        self.unbind();
        unsafe { self.gl.viewport(og_viewport[0], og_viewport[1], og_viewport[2], og_viewport[3]); }
        Ok(())
    }
}
