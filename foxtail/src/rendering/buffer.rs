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
        trace!("Allocating buffer with size: {}b/{}kb/{}mb", size, size/1024, size/1024/1024);
        let buf = unsafe {
            let b = gl.create_buffer().expect("Failed to create buffer!");
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(b));
            gl.buffer_storage(glow::SHADER_STORAGE_BUFFER, size as i32, None, glow::DYNAMIC_STORAGE_BIT | glow::MAP_WRITE_BIT);
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

    pub fn write(&self, offset: usize, data: &[T]) {
        let t_size = std::mem::size_of::<T>();
        let offset_raw = offset * t_size;
        if offset_raw + data.as_ref().len() * t_size > self.size {
            panic!("Cannot write past buffer bounds!");
        }
        unsafe {
            let data_raw: &[u8] = std::slice::from_raw_parts(
                data.as_ref().as_ptr() as *const u8,
                data.as_ref().len() * t_size,
            );

            self.gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.buf));
            self.gl.buffer_sub_data_u8_slice(glow::SHADER_STORAGE_BUFFER, offset_raw as i32, data_raw);
            self.gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);
        }
    }

    pub fn write_slice<'f>(&'f self, writes: impl Iterator<Item = (usize, &'f T)>) {
        unsafe { self.gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.buf)); }
        for (offset, data) in writes {
            let t_size = std::mem::size_of::<T>();
            let offset_raw = offset * t_size;
            if offset_raw + t_size > self.size {
                panic!("Cannot write past buffer bounds!");
            }
            unsafe {
                let data_raw: &[u8] = std::slice::from_raw_parts(
                    data as *const T as *const u8,
                    t_size,
                );

                self.gl.buffer_sub_data_u8_slice(glow::SHADER_STORAGE_BUFFER, offset_raw as i32, data_raw);
            }
        }
        unsafe { self.gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None); }
    }

    pub fn size(&self) -> usize {
        self.size
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
