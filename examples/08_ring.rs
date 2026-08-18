use ancorix::prelude::*;

struct Demo {
    ring: Handle<Shader>,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        let bytes = include_bytes!("../assets/shaders/ring.frag.spv");

        Self {
            ring: ctx.assets.shader(bytes),
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let center = ctx.window.size() / 2.0;
        let t = ctx.time.elapsed();

        ctx.draw.clear(Rgba::WHITE);

        let spin = Transform2D {
            rotation: t,
            origin: v2!(0.5),
            scale: v2!(1.0 + (t * 1.5).sin() * 0.25),
        };

        ctx.draw.shader_ex(
            self.ring,
            Rect::from_center(center, v2!(320.0)),
            spin,
            rgba!("#22d3ee"),
            &(), // no parameters beyond what Surface already gives the shader
        );
    }
}

fn main() {
    Window::new("Ancorix: ring shader", 1280, 720).run::<Demo>();
}
