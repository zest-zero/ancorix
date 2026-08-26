use ancorix::prelude::*;

struct Demo {
    pos: Vector2,
    speed: f32,
    font: Font,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        ctx.input.bind_keys("left", &[Key::Left, Key::A]);
        ctx.input.bind_keys("right", &[Key::Right, Key::D]);
        ctx.input.bind_keys("up", &[Key::Up, Key::W]);
        ctx.input.bind_keys("down", &[Key::Down, Key::S]);

        Self {
            pos: v2!(200.0),
            speed: 400.0,
            font: ctx.assets.builtin_font(2),
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let dir = ctx.input.action_vector("left", "right", "up", "down");
        self.pos += dir * self.speed * ctx.time.dt();

        ctx.draw.clear(Rgba::WHITE);

        let shape = Circle::new(self.pos, 40.0);
        ctx.draw.circle(shape, rgba!("#a855f7"));

        ctx.draw.text(
            &self.font,
            "controls: wasd | arrows",
            v2!(20.0, 20.0),
            Rgba::BLACK,
        );
    }
}

fn main() {
    Window::new("Ancorix: movement", 1280, 720).run::<Demo>();
}
