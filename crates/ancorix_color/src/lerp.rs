use ancorix_math::Lerp;

use crate::Rgba;

impl Lerp for Rgba {
    fn lerp(self, rhs: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let channel =
            |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * t).round() as u8;

        Self::new(
            channel(self.r, rhs.r),
            channel(self.g, rhs.g),
            channel(self.b, rhs.b),
            channel(self.a, rhs.a),
        )
    }
}
