use ancorix_math::Vector2;

/// 2D transformation applied on top of a primitive's position.
#[derive(Debug, Clone, Copy, PartialEq)]
#[doc(alias = "transform")]
#[doc(alias = "rotation")]
#[doc(alias = "scale")]
#[doc(alias = "pivot")]
pub struct Transform2D {
    /// Rotation in radians, counter-clockwise.
    pub rotation: f32,
    /// Pivot point for rotation and scale, in normalized coordinates.
    ///
    /// `(0.0, 0.0)` = top-left corner of the primitive's bounding box.
    /// `(0.5, 0.5)` = center (most common for rotation).
    /// `(1.0, 1.0)` = bottom-right corner.
    pub origin: Vector2,
    /// Per-axis scale factor. [`Vector2::ONE`] means no scaling.
    pub scale: Vector2,
}

impl Transform2D {
    /// No rotation, top-left origin, no scaling.
    pub const IDENTITY: Self = Self {
        rotation: 0.0,
        origin: Vector2::ZERO,
        scale: Vector2::ONE,
    };

    /// Returns a [`Transform2D`] with rotation around the center of the primitive.
    #[inline]
    pub fn rotated(radians: f32) -> Self {
        Self {
            rotation: radians,
            origin: Vector2::splat(0.5),
            scale: Vector2::ONE,
        }
    }

    /// Returns a [`Transform2D`] with rotation around a custom origin.
    #[inline]
    pub fn rotated_around(radians: f32, origin: Vector2) -> Self {
        Self {
            rotation: radians,
            origin,
            scale: Vector2::ONE,
        }
    }

    /// Returns a [`Transform2D`] with uniform scale.
    #[inline]
    pub fn scaled(factor: f32) -> Self {
        Self {
            rotation: 0.0,
            origin: Vector2::splat(0.5),
            scale: Vector2::splat(factor),
        }
    }

    /// Applies the transform to `p`, pivoting around [`Transform2D::origin`]
    /// (normalized within the box `[bounds_min, bounds_min + bounds_size]`).
    ///
    /// This is what happens to every vertex of a primitive drawn with a
    /// `_ex` method - `bounds_min`/`bounds_size` are the primitive's own
    /// untransformed bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Transform2D;
    /// use ancorix_math::v2;
    /// use std::f32::consts::FRAC_PI_2;
    ///
    /// let t = Transform2D::rotated(FRAC_PI_2);
    /// let p = t.apply(v2!(10.0, 0.0), v2!(0.0, 0.0), v2!(20.0, 20.0));
    ///
    /// assert!((p.x - 20.0).abs() < 1e-5);
    /// assert!((p.y - 10.0).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Transform2D::invert()`]
    #[inline]
    pub fn apply(self, p: Vector2, bounds_min: Vector2, bounds_size: Vector2) -> Vector2 {
        // Untransformed shapes - every plain `rect`/`circle`/`sprite` - pass
        // `IDENTITY`, and this runs once per vertex, four times per quad.
        // Without this the sin/cos in `rotated` is paid to compute `p` back.
        // Exact equality is right here: `IDENTITY` is built from literal
        // `0.0` and `1.0`, so no epsilon is involved.
        if self.is_identity() {
            return p;
        }

        let pivot = bounds_min + self.origin.component_mul(bounds_size);
        let local = (p - pivot).component_mul(self.scale).rotated(self.rotation);
        pivot + local
    }

    /// Returns `true` if this transform maps every point to itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Transform2D;
    ///
    /// assert!(Transform2D::IDENTITY.is_identity());
    /// assert!(!Transform2D::rotated(0.5).is_identity());
    /// ```
    #[inline(always)]
    pub fn is_identity(self) -> bool {
        // `origin` is deliberately not checked: with no rotation and unit
        // scale the pivot cancels out of both `apply` and `invert`, so any
        // origin still maps a point to itself.
        self.rotation == 0.0 && self.scale.x == 1.0 && self.scale.y == 1.0
    }

    /// Returns the point that [`Transform2D::apply()`] would map to `p` -
    /// the inverse transform.
    ///
    /// Useful for hit-testing a transformed primitive: apply the inverse to
    /// a point (e.g. the mouse position) to bring it into the primitive's
    /// own untransformed space, then check it against the untransformed
    /// shape (see `Rect::contains_ex()` and friends in `ancorix_draw`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Transform2D;
    /// use ancorix_math::v2;
    ///
    /// let t = Transform2D {
    ///     rotation: 0.7,
    ///     origin: v2!(0.3, 0.8),
    ///     scale: v2!(2.0, 0.5),
    /// };
    /// let bounds_min = v2!(-5.0, -5.0);
    /// let bounds_size = v2!(10.0, 10.0);
    ///
    /// let p = v2!(2.0, 4.0);
    /// let transformed = t.apply(p, bounds_min, bounds_size);
    /// let restored = t.invert(transformed, bounds_min, bounds_size);
    ///
    /// assert!((restored.x - p.x).abs() < 1e-4);
    /// assert!((restored.y - p.y).abs() < 1e-4);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Transform2D::apply()`]
    #[inline]
    pub fn invert(self, p: Vector2, bounds_min: Vector2, bounds_size: Vector2) -> Vector2 {
        // same short-circuit as `apply`, for the same reason
        if self.is_identity() {
            return p;
        }

        let pivot = bounds_min + self.origin.component_mul(bounds_size);
        let local = (p - pivot)
            .rotated(-self.rotation)
            .component_div(self.scale);
        pivot + local
    }
}

impl Default for Transform2D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}
