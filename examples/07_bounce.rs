use ancorix::prelude::*;

const SPEED: f32 = 260.0;
const SIZE: f32 = 200.0;
const TINTS: [&str; 4] = ["#22d3ee", "#a855f7", "#f59e0b", "#34d399"];

struct Demo {
    logo: Handle<Texture>,
    pos: Vector2,
    vel: Vector2,
    tint: usize,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        let bytes = include_bytes!("../assets/images/logo.png");

        Self {
            logo: ctx.assets.texture(bytes),
            pos: v2!(240.0, 180.0),
            vel: v2!(SPEED, SPEED * 0.72),
            tint: 0,
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let bounds = ctx.window.size();
        let half = SIZE / 2.0;

        self.pos += self.vel * ctx.time.dt();

        let mut bounced = false;
        if self.pos.x - half <= 0.0 || self.pos.x + half >= bounds.x {
            self.vel.x = -self.vel.x;
            self.pos.x = self.pos.x.clamp(half, bounds.x - half);
            bounced = true;
        }
        if self.pos.y - half <= 0.0 || self.pos.y + half >= bounds.y {
            self.vel.y = -self.vel.y;
            self.pos.y = self.pos.y.clamp(half, bounds.y - half);
            bounced = true;
        }
        if bounced {
            self.tint = (self.tint + 1) % TINTS.len();
        }

        ctx.draw.clear(Rgba::WHITE);

        let shape = Sprite::from_center(self.pos, v2!(SIZE));
        ctx.draw.sprite_ex(
            self.logo,
            shape,
            Transform2D::IDENTITY,
            Rgba::from_hex(TINTS[self.tint]),
        );
    }
}

fn main() {
    Window::new("Ancorix: bouncing logo", 1280, 720).run::<Demo>();
}
