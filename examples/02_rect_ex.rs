use ancorix::prelude::*;

struct Demo;

impl App for Demo {
    fn init(_ctx: &mut Ctx) -> Self {
        Self
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let t = ctx.time.elapsed();
        let center = ctx.window.size() / 2.0;

        let spin = Transform2D {
            rotation: t,
            origin: v2!(0.5),
            scale: v2!(1.0 + 0.15 * (t * 3.0).sin()),
        };

        ctx.draw.clear(Rgba::WHITE);

        ctx.draw.rect_ex(
            Rect::from_center(center, v2!(220.0)),
            spin,
            rgba!("#f38ba8"),
        );

        // pivot indicator
        ctx.draw.circle(Circle::new(center, 6.0), Rgba::BLACK);
    }
}

fn main() {
    Window::new("Ancorix: 02 rect ex", 1280, 720).run::<Demo>();
}
