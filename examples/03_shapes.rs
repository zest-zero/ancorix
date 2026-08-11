use ancorix::prelude::*;

const STEP: f32 = 210.0;
const SIZE: f32 = 124.0;
const ROW: f32 = 115.0;

struct Demo {
    first: Vector2,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        let center = ctx.window.size() / 2.0;

        Self {
            first: v2!(center.x - STEP * 2.0, center.y),
        }
    }

    fn frame(&mut self, ctx: &mut Ctx) {
        let spin = Transform2D::rotated(ctx.time.elapsed());
        let up = self.first - v2!(0.0, ROW);
        let down = self.first + v2!(0.0, ROW);

        ctx.draw.clear(rgba!("#1e1e1e"));

        // rect
        let square = Rect::from_center(up, v2!(SIZE));
        ctx.draw.rect(square, rgba!("#22d3ee"));

        // rounded rect
        let card = RoundedRect::from_center(up + v2!(STEP, 0.0), v2!(SIZE), 34.0);
        ctx.draw.rounded_rect(card, rgba!("#a855f7"));

        // circle
        let dot = Circle::new(up + v2!(STEP * 2.0, 0.0), SIZE / 2.0);
        ctx.draw.circle(dot, rgba!("#f59e0b"));

        // triangle
        let tip = up + v2!(STEP * 3.0, 0.0);
        let arrow = Triangle::new(
            tip - v2!(0.0, 66.0),
            tip + v2!(62.0, 52.0),
            tip - v2!(62.0, -52.0),
        );
        ctx.draw.triangle(arrow, rgba!("#34d399"));

        // line
        let end = up + v2!(STEP * 4.0, 0.0);
        let stroke = Line::new(end - v2!(58.0), end + v2!(58.0), 14.0);
        ctx.draw.line(stroke, rgba!("#f43f5e"));

        // spinning rect
        let square = Rect::from_center(down, v2!(SIZE));
        ctx.draw.rect_ex(square, spin, rgba!("#22d3ee"));

        // spinning rounded rect
        let card = RoundedRect::from_center(down + v2!(STEP, 0.0), v2!(SIZE), 34.0);
        ctx.draw.rounded_rect_ex(card, spin, rgba!("#a855f7"));

        // spinning ellipse
        let dot = Circle::new(down + v2!(STEP * 2.0, 0.0), SIZE / 2.0);
        let squash = Transform2D {
            rotation: spin.rotation,
            origin: v2!(0.5),
            scale: v2!(1.5, 0.7),
        };
        ctx.draw.circle_ex(dot, squash, rgba!("#f59e0b"));

        // spinning triangle
        let tip = down + v2!(STEP * 3.0, 0.0);
        let arrow = Triangle::new(
            tip - v2!(0.0, 66.0),
            tip + v2!(62.0, 52.0),
            tip - v2!(62.0, -52.0),
        );
        ctx.draw.triangle_ex(arrow, spin, rgba!("#34d399"));
    }
}

fn main() {
    Window::new("Ancorix: shapes", 1280, 720).run::<Demo>();
}
