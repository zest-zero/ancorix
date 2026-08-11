use ancorix::prelude::*;

const SPEED: f32 = 400.0;

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
        let step = SPEED * ctx.time.dt();

        // one key at a time, asked for by name
        if ctx.input.is_pressed(Key::A) {
            self.pos.x -= step;
        }
        if ctx.input.is_pressed(Key::D) {
            self.pos.x += step;
        }
        if ctx.input.is_pressed(Key::W) {
            self.pos.y -= step;
        }
        if ctx.input.is_pressed(Key::S) {
            self.pos.y += step;
        }

        ctx.draw.clear(rgba!("#1e1e1e"));

        let square = Rect::from_center(self.pos, v2!(90.0));
        ctx.draw.rect(square, Rgba::CYAN);
    }
}

fn main() {
    Window::new("Ancorix: keys", 1280, 720).run::<Demo>();
}
