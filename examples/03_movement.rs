use ancorix::prelude::*;

struct Demo {
    pos: Vector2,
    speed: f32,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        ctx.input.bind_keys("left", &[Key::Left, Key::A]);
        ctx.input.bind_keys("right", &[Key::Right, Key::D]);
        ctx.input.bind_keys("up", &[Key::Up, Key::W]);
        ctx.input.bind_keys("down", &[Key::Down, Key::S]);

        Self {
            pos: ctx.window.size() / 2.0,
            speed: 500.0,
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let dir = ctx.input.action_vector("left", "right", "up", "down");
        self.pos += dir * self.speed * ctx.time.dt();

        ctx.draw.clear(rgba!("#1e1e1e"));

        ctx.draw.circle(
            Circle {
                pos: self.pos,
                radius: 40.0,
            },
            Rgba::PURPLE,
        );
    }
}

fn main() {
    Window::new("Ancorix: movement", 1280, 720).run::<Demo>();
}
