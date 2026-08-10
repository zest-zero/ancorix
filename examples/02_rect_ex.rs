use ancorix::prelude::*;

struct Demo {
    center: Vector2,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        Self {
            center: ctx.window.size() / 2.0,
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let t = ctx.time.elapsed();
        let breath = 1.0 + (t * 2.0).sin() * 0.3;

        ctx.draw.clear(rgba!("#1e1e1e"));

        ctx.draw.rect_ex(
            Rect::from_center(self.center, v2!(220.0)),
            Transform2D {
                rotation: t,
                origin: v2!(0.5),
                scale: v2!(breath),
            },
            Rgba::CYAN,
        );
    }
}

fn main() {
    Window::new("Ancorix: transforms", 1280, 720).run::<Demo>();
}
