use ancorix::prelude::*;

struct Demo;

impl App for Demo {
    fn init(_ctx: &mut Ctx) -> Self {
        Self
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        ctx.draw.clear(Rgba::WHITE);

        let shape = Rect::from_center(ctx.window.size() / 2.0, v2!(200.0));
        ctx.draw.rect(shape, Rgba::CYAN);
    }
}

fn main() {
    Window::new("Ancorix: rect", 1280, 720).run::<Demo>();
}
