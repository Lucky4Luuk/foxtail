use std::sync::Arc;
use glow::*;

pub struct FixedSizeBuffer<T> {
    buf: NativeBuffer,
    size: usize,
    gl: Arc<Context>,
    bound_loc: Option<u32>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FixedSizeBuffer<T> {
    pub fn new(renderer: &super::Renderer, count: usize) -> Self {
        let gl = renderer.gl.clone();
        let size = std::mem::size_of::<T>() * count;
        let buf = unsafe {
            let b = gl.create_buffer().expect("Failed to create buffer!");
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(b));
            gl.buffer_storage(glow::SHADER_STORAGE_BUFFER, size as i32, None, glow::DYNAMIC_STORAGE_BIT | glow::MAP_WRITE_BIT | glow::MAP_PERSISTENT_BIT | glow::MAP_COHERENT_BIT);
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);
            b
        };
        Self {
            buf: buf,
            size: size,
            gl: gl,
            bound_loc: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn bind(&mut self, location: u32) {
        self.bound_loc = Some(location);
        unsafe {
            self.gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, location, Some(self.buf));
        }
    }

    pub fn unbind(&mut self) {
        if let Some(loc) = self.bound_loc {
            unsafe {
                self.gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, loc, None);
            }
            self.bound_loc = None;
        } else {
            trace!("Attempting to unbind unbound buffer!");
        }
    }
}
