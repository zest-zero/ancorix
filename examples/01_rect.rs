use ancorix::prelude::*;

struct Demo {
    pos: Vector2,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        Self {
            pos: ctx.window.size() / 2.0,
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        ctx.draw.clear(Rgba::WHITE);

        ctx.draw.rect(
            Rect {
                pos: self.pos - 100.0,
                size: v2!(200.0),
            },
            Rgba::CYAN,
        );
    }
}

fn main() {
    Window::new("Ancorix: rect", 1280, 720).run::<Demo>();
}
