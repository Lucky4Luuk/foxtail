#[macro_use] extern crate log;
use foxtail::prelude::*;

const VS: &'static str = include_str!("../shaders/vs.glsl");
const FS: &'static str = include_str!("../shaders/fs.glsl");

pub struct Demo {
    mesh: Mesh,
    shader: Shader,
    framebuffer: Framebuffer,
}

impl Demo {
    fn new(ctx: &Context) -> Self {
        ctx.set_window_title("Foxtail demo");
        trace!("Demo created!");
        let mesh = Mesh::quad(&ctx);
        let shader = Shader::new(&ctx, VS, FS);
        let fb = Framebuffer::new(&ctx);
        Self {
            mesh: mesh,
            shader: shader,
            framebuffer: fb,
        }
    }
}

impl App for Demo {
    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, _ctx: &mut Context) {
        let _ = self.framebuffer.while_bound(|| {
            self.framebuffer.clear();
            self.shader.while_bound(|_| {
                self.mesh.draw()?;
                Ok(())
            })
        });
        let _ = self.framebuffer.while_bound(|| {
            self.framebuffer.draw()
        });
        let _ = self.framebuffer.draw();
    }

    fn on_resize(&mut self, size: (i32, i32)) {
        self.framebuffer.resize(size);
    }
}

fn main() {
    foxtail::run(|ctx| Demo::new(ctx))
}
