use std::sync::Arc;
use glow::*;
use crate::prelude::FixedSizeBuffer;

pub struct AtomicCounter {
    buf: NativeBuffer,
    read_buf: FixedSizeBuffer<u32>,
    gl: Arc<Context>,
    bound_loc: Option<u32>,
}

impl AtomicCounter {
    pub fn new(renderer: &super::Renderer) -> Self {
        let gl = renderer.gl.clone();
        let buf = unsafe {
            let b = gl.create_buffer().expect("Failed to create atomic counter buffer!");
            gl.bind_buffer(glow::ATOMIC_COUNTER_BUFFER, Some(b));
            gl.buffer_data_u8_slice(glow::ATOMIC_COUNTER_BUFFER, &[0u8; 4], glow::DYNAMIC_DRAW);
            gl.bind_buffer(glow::ATOMIC_COUNTER_BUFFER, None);
            b
        };
        // TODO: Make this storage host-only with flags?
        let read_buf: FixedSizeBuffer<u32> = FixedSizeBuffer::new(renderer, 1);
        let obj = Self {
            buf: buf,
            read_buf,
            gl: gl,
            bound_loc: None,
        };

        obj.reset(0);

        obj
    }

    pub fn reset(&self, value: u32) {
        let bytes = value.to_le_bytes();
        unsafe {
            self.gl.bind_buffer(glow::ATOMIC_COUNTER_BUFFER, Some(self.buf));
            self.gl.buffer_data_u8_slice(glow::ATOMIC_COUNTER_BUFFER, &bytes, glow::DYNAMIC_DRAW);
            self.gl.bind_buffer(glow::ATOMIC_COUNTER_BUFFER, None);
        }
    }

    /// WARNING: Extremely slow!
    pub fn read(&self) -> u32 {
        let mut bytes = [0u8; 4];
        unsafe {
            self.gl.bind_buffer(glow::ATOMIC_COUNTER_BUFFER, Some(self.buf));
            self.gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.read_buf.buf()));

            self.gl.copy_buffer_sub_data(glow::ATOMIC_COUNTER_BUFFER, glow::SHADER_STORAGE_BUFFER, 0,0, 4);

            self.gl.get_buffer_sub_data(glow::SHADER_STORAGE_BUFFER, 0, &mut bytes);

            self.gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);
            self.gl.bind_buffer(glow::ATOMIC_COUNTER_BUFFER, None);
        }
        u32::from_le_bytes(bytes)
    }

    pub fn bind(&mut self, location: u32) {
        self.bound_loc = Some(location);
        unsafe {
            self.gl.bind_buffer_base(glow::ATOMIC_COUNTER_BUFFER, location, Some(self.buf));
        }
    }

    pub fn unbind(&mut self) {
        if let Some(loc) = self.bound_loc {
            unsafe {
                self.gl.bind_buffer_base(glow::ATOMIC_COUNTER_BUFFER, loc, None);
            }
            self.bound_loc = None;
        } else {
            trace!("Attempting to unbind unbound buffer!");
        }
    }
}
