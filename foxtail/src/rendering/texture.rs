use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use glow::*;

pub enum TextureFormat {
    R,
    RG,
    RGB,
    RGBA,
}

impl TextureFormat {
    fn to_gl_format(&self) -> u32 {
        match self {
            Self::RGB => RGB,
            Self::RGBA => RGBA,
            _ => unimplemented!(),
        }
    }

    fn to_gl_internal_format(&self) -> i32 {
        match self {
            Self::RGB => RGB32F as i32,
            Self::RGBA => RGBA32F as i32,
            _ => unimplemented!(),
        }
    }
}

pub enum TextureFiltering {
    Linear,
    Nearest,
}

impl TextureFiltering {
    fn to_gl(&self) -> i32 {
        match self {
            Self::Linear => LINEAR as i32,
            Self::Nearest => NEAREST as i32,
        }
    }

    fn to_gl_mipmap(&self) -> i32 {
        match self {
            Self::Linear => LINEAR_MIPMAP_LINEAR as i32,
            Self::Nearest => NEAREST_MIPMAP_NEAREST as i32,
        }
    }
}

pub struct TextureSettings {
    pub width: usize,
    pub height: usize,
    pub format: TextureFormat,
    pub filtering: TextureFiltering,
    pub mipmap: bool,
}

pub struct Texture {
    tex: glow::Texture,
    settings: TextureSettings,
    gl: Arc<Context>,
    shader_bound: Arc<AtomicBool>,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.tex);
        }
    }
}

impl Texture {
    pub fn new(renderer: &super::Renderer, settings: TextureSettings, pixels: Option<&[u8]>) -> Self {
        let gl = renderer.gl.clone();
        let tex = new_tex(gl.clone(), &settings, pixels);
        super::gl_error(&gl);
        Self {
            tex,
            settings,
            gl,
            shader_bound: renderer.shader_bound.clone(),
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.settings.width, self.settings.height)
    }

    pub fn resize(&mut self, size: (usize, usize), pixels: Option<&[u8]>) {
        self.settings.width = size.0;
        self.settings.height = size.1;
        let tex = new_tex(self.gl.clone(), &self.settings, pixels);
        self.tex = tex;
    }

    fn bind_tex(&self, location: u32) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + location);
            self.gl.bind_texture(TEXTURE_2D, Some(self.tex));
        }
    }

    fn unbind_tex(&self) {
        unsafe {
            self.gl.bind_texture(TEXTURE_2D, None);
        }
    }

    /// Runs a closure while the framebuffer is bound
    pub fn while_bound<F: FnOnce() -> Result<(), super::RenderError>>(&self, location: u32, f: F) -> Result<(), super::RenderError> {
        if self.shader_bound.load(Ordering::Acquire) == false {
            panic!("No shader bound, but you are trying to bind a texture!");
        }
        self.bind_tex(location);
        f()?;
        self.unbind_tex();
        Ok(())
    }
}

fn new_tex(gl: Arc<Context>, settings: &TextureSettings, pixels: Option<&[u8]>) -> glow::Texture {
    let tex = unsafe {
        let tex = gl.create_texture().map_err(|e| error!("{}", e)).expect("Failed to create texture!");
        gl.bind_texture(TEXTURE_2D, Some(tex));
        gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_BASE_LEVEL, 0);
        gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAX_LEVEL, 5);
        gl.tex_parameter_f32(TEXTURE_2D, TEXTURE_LOD_BIAS, -1.8);
        gl.tex_image_2d(TEXTURE_2D, 0, settings.format.to_gl_internal_format(), settings.width as i32, settings.height as i32, 0, settings.format.to_gl_format(), UNSIGNED_BYTE, pixels);
        if settings.mipmap { gl.generate_texture_mipmap(tex); }
        // Regular filtering
        gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, if settings.mipmap { settings.filtering.to_gl_mipmap() } else { settings.filtering.to_gl() });
        gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, settings.filtering.to_gl());
        gl.bind_texture(TEXTURE_2D, None);
        tex
    };
    tex
}
