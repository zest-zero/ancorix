use ancorix::prelude::*;

struct Demo {
    rect: Rect,
    drag: bool,
    off: Vector2,
    font: Font,
}

impl App for Demo {
    fn init(ctx: &mut Ctx) -> Self {
        Self {
            rect: Rect::from_center(v2!(220.0), v2!(100.0)),
            drag: false,
            off: v2!(0.0),
            font: ctx.assets.builtin_font(2),
        }
    }
    fn frame(&mut self, ctx: &mut Ctx) {
        let m = ctx.input.mouse_pos();

        if ctx.input.is_mouse_just_pressed(MouseButton::Left) && self.rect.contains(m) {
            self.drag = true;
            self.off = self.rect.pos - m;
        }
        if !ctx.input.is_mouse_pressed(MouseButton::Left) {
            self.drag = false;
        }
        if self.drag {
            self.rect.pos = m + self.off;
        }

        let color = rgba!("#22d3ee");
        let color = if self.drag { color.darken(0.2) } else { color };

        ctx.draw.clear(Rgba::WHITE);

        ctx.draw.rect(self.rect, color);

        let text = "drag me";
        let text_size = self.font.measure(text);
        let text_pos = self.rect.center() - text_size / 2.0;

        ctx.draw.text(&self.font, text, text_pos, Rgba::BLACK);
    }
}
fn main() {
    Window::new("Ancorix: drag", 1280, 720).run::<Demo>();
}
