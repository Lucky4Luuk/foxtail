use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use glow::*;

pub struct Mesh {
    vbo: NativeBuffer,
    vao: NativeVertexArray,
    vert_count: i32,
    gl: Arc<Context>,
    shader_bound: Arc<AtomicBool>,
}

impl super::Drawable for Mesh {
    /// Panics if no shader is bound!
    fn draw(&self) -> Result<(), super::RenderError> {
        puffin::profile_function!();
        if self.shader_bound.load(Ordering::Acquire) != true {
            panic!("No shader bound! Use `shader.while_bound` or similar!");
        }
        unsafe {
            // self.gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(TRIANGLE_FAN, 0, self.vert_count);
            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

impl Mesh {
    pub fn quad(renderer: &super::Renderer) -> Self {
        unsafe {
            let quad_vertices: [f32; 32] = [
                // Position    // Color     // UV
                -1.0,-1.0,0.0, 1.0,1.0,1.0, 0.0,0.0,
                 1.0,-1.0,0.0, 1.0,1.0,1.0, 1.0,0.0,
                 1.0, 1.0,0.0, 1.0,1.0,1.0, 1.0,1.0,
                -1.0, 1.0,0.0, 1.0,1.0,1.0, 0.0,1.0,
            ];
            let quad_vertices_u8: &[u8] = core::slice::from_raw_parts(
                quad_vertices.as_ptr() as *const u8,
                quad_vertices.len() * core::mem::size_of::<f32>(),
            );

            let gl = renderer.gl.clone();
            trace!("GL cloned!");

            let vbo = gl.create_buffer().expect("Failed to create VBO!");
            gl.bind_buffer(ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, quad_vertices_u8, STATIC_DRAW);

            let vao = gl.create_vertex_array().expect("Failed to create VAO!");
            gl.bind_vertex_array(Some(vao));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, (8 * core::mem::size_of::<f32>()) as i32, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, (8 * core::mem::size_of::<f32>()) as i32, (3 * core::mem::size_of::<f32>()) as i32);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, FLOAT, false, (8 * core::mem::size_of::<f32>()) as i32, (6 * core::mem::size_of::<f32>()) as i32);

            gl.bind_buffer(ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Self {
                vbo: vbo,
                vao: vao,
                vert_count: 4,
                gl: gl,
                shader_bound: renderer.shader_bound.clone(),
            }
        }
    }
}
