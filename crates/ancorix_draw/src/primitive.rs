use crate::transform::Transform2D;
use ancorix_math::Vector2;

/// A rectangle defined by position and size.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "rectangle")]
#[doc(alias = "aabb")]
#[doc(alias = "bounds")]
#[doc(alias = "box")]
pub struct Rect {
    /// The position of the top-left corner of the rectangle.
    pub pos: Vector2,
    /// The size of the rectangle.
    pub size: Vector2,
}

impl Rect {
    /// Returns a new [`Rect`] with the given position and size.
    #[inline(always)]
    pub const fn new(pos: Vector2, size: Vector2) -> Self {
        Self { pos, size }
    }

    /// Returns a new [`Rect`] with the given center point and size.
    #[inline]
    pub fn from_center(center: Vector2, size: Vector2) -> Self {
        Self {
            pos: center - size * 0.5,
            size,
        }
    }

    /// Returns the [`Vector2`] center point of the rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let rect = Rect::new(v2!(10.0, 10.0), v2!(20.0, 20.0));
    ///
    /// assert!((rect.center().x - 20.0).abs() < 1e-5);
    /// assert!((rect.center().y - 20.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn center(self) -> Vector2 {
        self.pos + self.size * 0.5
    }

    /// Returns `true` if this rectangle overlaps `other` at all.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let a = Rect::new(v2!(0.0, 0.0), v2!(10.0, 10.0));
    /// let b = Rect::new(v2!(5.0, 5.0), v2!(10.0, 10.0));
    /// let c = Rect::new(v2!(20.0, 20.0), v2!(5.0, 5.0));
    ///
    /// assert!(a.intersects(b));
    /// assert!(!a.intersects(c));
    /// ```
    #[inline]
    #[must_use]
    pub fn intersects(self, other: Rect) -> bool {
        self.pos.x < other.pos.x + other.size.x
            && self.pos.x + self.size.x > other.pos.x
            && self.pos.y < other.pos.y + other.size.y
            && self.pos.y + self.size.y > other.pos.y
    }

    /// Returns the rectangle covered by both, or `None` if they don't
    /// overlap or only touch along an edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let a = Rect::new(v2!(0.0, 0.0), v2!(10.0, 10.0));
    /// let b = Rect::new(v2!(5.0, 5.0), v2!(10.0, 10.0));
    ///
    /// assert_eq!(a.intersection(b), Some(Rect::new(v2!(5.0, 5.0), v2!(5.0, 5.0))));
    /// assert_eq!(a.intersection(Rect::new(v2!(20.0, 0.0), v2!(5.0, 5.0))), None);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::intersects()`]
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Rect) -> Option<Rect> {
        let min = self.pos.max(other.pos);
        let max = (self.pos + self.size).min(other.pos + other.size);
        let size = max - min;

        (size.x > 0.0 && size.y > 0.0).then_some(Rect::new(min, size))
    }

    /// Returns the top `height` pixels of the rectangle and what is left
    /// below them.
    ///
    /// Laying a panel out is a chain of these: cut off a header, cut a
    /// sidebar out of the rest, keep going. Nothing is stored between calls,
    /// so there is no layout state to get out of sync.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let panel = Rect::new(v2!(0.0, 0.0), v2!(200.0, 100.0));
    /// let (header, body) = panel.cut_top(30.0);
    ///
    /// assert_eq!(header, Rect::new(v2!(0.0, 0.0), v2!(200.0, 30.0)));
    /// assert_eq!(body, Rect::new(v2!(0.0, 30.0), v2!(200.0, 70.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::cut_bottom()`]
    /// * [`Rect::cut_left()`]
    /// * [`Rect::cut_right()`]
    #[inline]
    #[must_use]
    pub fn cut_top(self, height: f32) -> (Rect, Rect) {
        let height = height.clamp(0.0, self.size.y);
        (
            Rect::new(self.pos, Vector2::new(self.size.x, height)),
            Rect::new(
                self.pos + Vector2::new(0.0, height),
                Vector2::new(self.size.x, self.size.y - height),
            ),
        )
    }

    /// Returns the bottom `height` pixels of the rectangle and what is left
    /// above them.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let panel = Rect::new(v2!(0.0, 0.0), v2!(200.0, 100.0));
    /// let (footer, body) = panel.cut_bottom(20.0);
    ///
    /// assert_eq!(footer, Rect::new(v2!(0.0, 80.0), v2!(200.0, 20.0)));
    /// assert_eq!(body, Rect::new(v2!(0.0, 0.0), v2!(200.0, 80.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::cut_top()`]
    #[inline]
    #[must_use]
    pub fn cut_bottom(self, height: f32) -> (Rect, Rect) {
        let height = height.clamp(0.0, self.size.y);
        let rest = self.size.y - height;
        (
            Rect::new(
                self.pos + Vector2::new(0.0, rest),
                Vector2::new(self.size.x, height),
            ),
            Rect::new(self.pos, Vector2::new(self.size.x, rest)),
        )
    }

    /// Returns the leftmost `width` pixels of the rectangle and what is left
    /// beside them.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let panel = Rect::new(v2!(0.0, 0.0), v2!(200.0, 100.0));
    /// let (sidebar, content) = panel.cut_left(60.0);
    ///
    /// assert_eq!(sidebar, Rect::new(v2!(0.0, 0.0), v2!(60.0, 100.0)));
    /// assert_eq!(content, Rect::new(v2!(60.0, 0.0), v2!(140.0, 100.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::cut_right()`]
    #[inline]
    #[must_use]
    pub fn cut_left(self, width: f32) -> (Rect, Rect) {
        let width = width.clamp(0.0, self.size.x);
        (
            Rect::new(self.pos, Vector2::new(width, self.size.y)),
            Rect::new(
                self.pos + Vector2::new(width, 0.0),
                Vector2::new(self.size.x - width, self.size.y),
            ),
        )
    }

    /// Returns the rightmost `width` pixels of the rectangle and what is left
    /// beside them.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let panel = Rect::new(v2!(0.0, 0.0), v2!(200.0, 100.0));
    /// let (scrollbar, content) = panel.cut_right(12.0);
    ///
    /// assert_eq!(scrollbar, Rect::new(v2!(188.0, 0.0), v2!(12.0, 100.0)));
    /// assert_eq!(content, Rect::new(v2!(0.0, 0.0), v2!(188.0, 100.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::cut_left()`]
    #[inline]
    #[must_use]
    pub fn cut_right(self, width: f32) -> (Rect, Rect) {
        let width = width.clamp(0.0, self.size.x);
        let rest = self.size.x - width;
        (
            Rect::new(
                self.pos + Vector2::new(rest, 0.0),
                Vector2::new(width, self.size.y),
            ),
            Rect::new(self.pos, Vector2::new(rest, self.size.y)),
        )
    }

    /// Returns the rectangle shrunk by `margin` pixels on every side, or an
    /// empty rectangle at its centre if there is nothing left.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let panel = Rect::new(v2!(0.0, 0.0), v2!(200.0, 100.0));
    ///
    /// assert_eq!(panel.inset(10.0), Rect::new(v2!(10.0, 10.0), v2!(180.0, 80.0)));
    /// assert_eq!(panel.inset(500.0).size, v2!(0.0, 0.0));
    /// ```
    #[inline]
    #[must_use]
    pub fn inset(self, margin: f32) -> Rect {
        let size = (self.size - Vector2::splat(margin * 2.0)).max(Vector2::ZERO);
        Rect::new(self.center() - size * 0.5, size)
    }

    /// Returns the rectangle moved by `delta`, keeping its size.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let square = Rect::new(v2!(10.0, 10.0), v2!(50.0, 50.0));
    ///
    /// assert_eq!(
    ///     square.offset(v2!(60.0, 0.0)),
    ///     Rect::new(v2!(70.0, 10.0), v2!(50.0, 50.0))
    /// );
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::inset()`]
    #[inline]
    #[must_use]
    pub const fn offset(self, delta: Vector2) -> Rect {
        Rect::new(self.pos.const_add(delta), self.size)
    }

    /// Returns `true` if `point` is inside the rectangle.
    ///
    /// Doesn't account for any [`Transform2D`] the rectangle was drawn
    /// with - use [`Rect::contains_ex()`] for that.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Rect;
    /// use ancorix_math::v2;
    ///
    /// let rect = Rect::new(v2!(10.0, 10.0), v2!(20.0, 20.0));
    ///
    /// assert!(rect.contains(v2!(15.0, 15.0)));
    /// assert!(!rect.contains(v2!(0.0, 0.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::contains_ex()`]
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vector2) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }

    /// Returns `true` if `point` is inside the rectangle after `transform`
    /// is applied - the same transform passed to [`crate::Draw::rect_ex()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Rect, Transform2D};
    /// use ancorix_math::v2;
    /// use std::f32::consts::PI;
    ///
    /// let rect = Rect::new(v2!(0.0, 0.0), v2!(20.0, 20.0));
    /// let flipped = Transform2D::rotated(PI); // 180 degrees around the center
    ///
    /// // still contains its own center regardless of rotation
    /// assert!(rect.contains_ex(v2!(10.0, 10.0), flipped));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rect::contains()`]
    /// * [`crate::Draw::rect_ex()`]
    #[inline]
    #[must_use]
    pub fn contains_ex(self, point: Vector2, transform: Transform2D) -> bool {
        self.contains(transform.invert(point, self.pos, self.size))
    }
}

/// A circle defined by center position and radius.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "disc")]
#[doc(alias = "round")]
pub struct Circle {
    /// The center point of the circle.
    pub pos: Vector2,
    /// The radius of the circle.
    pub radius: f32,
}

impl Circle {
    /// Returns a new [`Circle`] with the given position and radius.
    #[inline(always)]
    pub fn new(pos: Vector2, radius: f32) -> Self {
        Self { pos, radius }
    }

    /// Returns the circle moved by `delta`, keeping its radius.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Circle;
    /// use ancorix_math::v2;
    ///
    /// let dot = Circle::new(v2!(10.0, 10.0), 4.0);
    ///
    /// assert_eq!(dot.offset(v2!(0.0, 20.0)).pos, v2!(10.0, 30.0));
    /// ```
    #[inline]
    #[must_use]
    pub const fn offset(self, delta: Vector2) -> Circle {
        Circle {
            pos: self.pos.const_add(delta),
            radius: self.radius,
        }
    }

    /// Returns `true` if `point` is inside the circle.
    ///
    /// Doesn't account for any [`Transform2D`] the circle was drawn with -
    /// use [`Circle::contains_ex()`] for that.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Circle;
    /// use ancorix_math::v2;
    ///
    /// let circle = Circle::new(v2!(10.0, 10.0), 5.0);
    ///
    /// assert!(circle.contains(v2!(10.0, 10.0)));
    /// assert!(!circle.contains(v2!(20.0, 20.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Circle::contains_ex()`]
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vector2) -> bool {
        point.distance_squared(self.pos) <= self.radius * self.radius
    }

    /// Returns `true` if `point` is inside the circle after `transform` is
    /// applied - the same transform passed to [`crate::Draw::circle_ex()`].
    ///
    /// A non-uniform [`Transform2D::scale`] turns the circle into an
    /// ellipse for this check, matching how it's actually drawn.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Circle, Transform2D};
    /// use ancorix_math::v2;
    ///
    /// let circle = Circle::new(v2!(0.0, 0.0), 5.0);
    /// let scaled = Transform2D::scaled(2.0);
    ///
    /// assert!(circle.contains_ex(v2!(9.0, 0.0), scaled));
    /// assert!(!circle.contains_ex(v2!(9.0, 0.0), Transform2D::IDENTITY));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Circle::contains()`]
    #[inline]
    #[must_use]
    pub fn contains_ex(self, point: Vector2, transform: Transform2D) -> bool {
        let bounds_min = self.pos - Vector2::splat(self.radius);
        let bounds_size = Vector2::splat(self.radius * 2.0);
        self.contains(transform.invert(point, bounds_min, bounds_size))
    }
}

/// A rectangle with uniformly rounded corners.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "rounded")]
#[doc(alias = "radius")]
#[doc(alias = "card")]
pub struct RoundedRect {
    /// The position of the top-left corner of the rectangle.
    pub pos: Vector2,
    /// The size of the rectangle.
    pub size: Vector2,
    /// The corner radius, in pixels, applied to all four corners equally.
    pub radius: f32,
}

impl RoundedRect {
    /// Returns a new [`RoundedRect`] with the given position, size, and
    /// corner radius.
    #[inline(always)]
    pub const fn new(pos: Vector2, size: Vector2, radius: f32) -> Self {
        Self { pos, size, radius }
    }

    /// Returns the rectangle moved by `delta`, keeping its size and radius.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::RoundedRect;
    /// use ancorix_math::v2;
    ///
    /// let card = RoundedRect::new(v2!(0.0, 0.0), v2!(80.0, 40.0), 6.0);
    ///
    /// assert_eq!(card.offset(v2!(90.0, 0.0)).pos, v2!(90.0, 0.0));
    /// ```
    #[inline]
    #[must_use]
    pub const fn offset(self, delta: Vector2) -> RoundedRect {
        RoundedRect {
            pos: self.pos.const_add(delta),
            size: self.size,
            radius: self.radius,
        }
    }

    /// Returns a new [`RoundedRect`] with the given center point, size, and
    /// corner radius.
    #[inline]
    pub fn from_center(center: Vector2, size: Vector2, radius: f32) -> Self {
        Self {
            pos: center - size * 0.5,
            size,
            radius,
        }
    }

    /// Returns the [`Vector2`] center point of the rectangle.
    #[inline]
    pub fn center(self) -> Vector2 {
        self.pos + self.size * 0.5
    }

    /// Returns `true` if `point` is inside the rounded rectangle.
    ///
    /// Uses the exact rounded-box distance field (same formula the GPU
    /// renders with - see `ancorix_render`), not just the outer bounding
    /// box, so it's accurate right up to the rounded corners. Doesn't
    /// account for any [`Transform2D`] the shape was drawn with - use
    /// [`RoundedRect::contains_ex()`] for that.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::RoundedRect;
    /// use ancorix_math::v2;
    ///
    /// let shape = RoundedRect::new(v2!(0.0, 0.0), v2!(20.0, 20.0), 5.0);
    ///
    /// assert!(shape.contains(v2!(10.0, 10.0))); // center
    /// assert!(!shape.contains(v2!(1.0, 1.0))); // clipped by the rounded corner
    /// ```
    ///
    /// # See also
    ///
    /// * [`RoundedRect::contains_ex()`]
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vector2) -> bool {
        let half = self.size * 0.5;
        let p = (point - self.center()).abs();
        let q = (p - half + Vector2::splat(self.radius)).max(Vector2::ZERO);
        q.length() - self.radius <= 0.0
    }

    /// Returns `true` if `point` is inside the rounded rectangle after
    /// `transform` is applied - the same transform passed to
    /// [`crate::Draw::rounded_rect_ex()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{RoundedRect, Transform2D};
    /// use ancorix_math::v2;
    /// use std::f32::consts::PI;
    ///
    /// let shape = RoundedRect::new(v2!(0.0, 0.0), v2!(20.0, 20.0), 5.0);
    /// let flipped = Transform2D::rotated(PI); // 180 degrees around the center
    ///
    /// // still contains its own center regardless of rotation
    /// assert!(shape.contains_ex(v2!(10.0, 10.0), flipped));
    /// ```
    ///
    /// # See also
    ///
    /// * [`RoundedRect::contains()`]
    /// * [`crate::Draw::rounded_rect_ex()`]
    #[inline]
    #[must_use]
    pub fn contains_ex(self, point: Vector2, transform: Transform2D) -> bool {
        self.contains(transform.invert(point, self.pos, self.size))
    }
}

/// A line segment between two points.
#[derive(Debug, Clone, Copy, PartialEq)]
#[doc(alias = "segment")]
pub struct Line {
    /// The starting point of the line segment.
    pub from: Vector2,
    /// The ending point of the line segment.
    pub to: Vector2,
    /// The thickness of the line segment.
    pub thickness: f32,
}

impl Line {
    /// Returns a new [`Line`] with the given start and end points.
    #[inline(always)]
    pub fn new(from: Vector2, to: Vector2, thickness: f32) -> Self {
        Self {
            from,
            to,
            thickness,
        }
    }

    /// Returns the segment moved by `delta`, both ends together.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Line;
    /// use ancorix_math::v2;
    ///
    /// let rule = Line::new(v2!(0.0, 0.0), v2!(100.0, 0.0), 1.0);
    /// let below = rule.offset(v2!(0.0, 20.0));
    ///
    /// assert_eq!((below.from, below.to), (v2!(0.0, 20.0), v2!(100.0, 20.0)));
    /// ```
    #[inline]
    #[must_use]
    pub const fn offset(self, delta: Vector2) -> Line {
        Line {
            from: self.from.const_add(delta),
            to: self.to.const_add(delta),
            thickness: self.thickness,
        }
    }

    /// Returns `true` if `point` is within [`Line::thickness`] / 2 of the
    /// segment.
    ///
    /// Lines have no [`Transform2D`] variant to draw with, so there is no
    /// `contains_ex` either.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Line;
    /// use ancorix_math::v2;
    ///
    /// let line = Line::new(v2!(0.0, 0.0), v2!(10.0, 0.0), 4.0);
    ///
    /// assert!(line.contains(v2!(5.0, 1.0))); // near the middle of the segment
    /// assert!(!line.contains(v2!(5.0, 10.0))); // too far away
    /// assert!(!line.contains(v2!(15.0, 0.0))); // past the end of the segment
    /// ```
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vector2) -> bool {
        let dir = self.to - self.from;
        let closest = match dir.try_normalize() {
            Some(unit) => {
                let t = (point - self.from).dot(unit).clamp(0.0, dir.length());
                self.from + unit * t
            }
            None => self.from,
        };

        point.distance_squared(closest) <= (self.thickness * 0.5) * (self.thickness * 0.5)
    }
}

impl Default for Line {
    #[inline]
    fn default() -> Self {
        Self {
            from: Vector2::ZERO,
            to: Vector2::ZERO,
            thickness: 1.0,
        }
    }
}

/// A triangle defined by three vertices.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "tri")]
#[doc(alias = "polygon")]
pub struct Triangle {
    pub a: Vector2,
    pub b: Vector2,
    pub c: Vector2,
}

impl Triangle {
    /// Returns a new [`Triangle`] with the given vertices.
    #[inline(always)]
    pub fn new(a: Vector2, b: Vector2, c: Vector2) -> Self {
        Self { a, b, c }
    }

    /// Returns the triangle moved by `delta`, all three corners together.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Triangle;
    /// use ancorix_math::v2;
    ///
    /// let arrow = Triangle::new(v2!(0.0, 0.0), v2!(10.0, 0.0), v2!(5.0, 8.0));
    ///
    /// assert_eq!(arrow.offset(v2!(3.0, 0.0)).a, v2!(3.0, 0.0));
    /// ```
    #[inline]
    #[must_use]
    pub const fn offset(self, delta: Vector2) -> Triangle {
        Triangle {
            a: self.a.const_add(delta),
            b: self.b.const_add(delta),
            c: self.c.const_add(delta),
        }
    }

    /// Returns `true` if `point` is inside the triangle.
    ///
    /// Works regardless of the vertices' winding order. Doesn't account
    /// for any [`Transform2D`] the triangle was drawn with - use
    /// [`Triangle::contains_ex()`] for that.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Triangle;
    /// use ancorix_math::v2;
    ///
    /// let triangle = Triangle::new(v2!(0.0, 0.0), v2!(10.0, 0.0), v2!(5.0, 10.0));
    ///
    /// assert!(triangle.contains(v2!(5.0, 3.0)));
    /// assert!(!triangle.contains(v2!(0.0, 10.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Triangle::contains_ex()`]
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vector2) -> bool {
        let d1 = edge_sign(point, self.a, self.b);
        let d2 = edge_sign(point, self.b, self.c);
        let d3 = edge_sign(point, self.c, self.a);

        let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;

        !(has_neg && has_pos)
    }

    /// Returns `true` if `point` is inside the triangle after `transform`
    /// is applied - the same transform passed to [`crate::Draw::triangle_ex()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Transform2D, Triangle};
    /// use ancorix_math::v2;
    /// use std::f32::consts::PI;
    ///
    /// let triangle = Triangle::new(v2!(0.0, 0.0), v2!(10.0, 0.0), v2!(5.0, 10.0));
    /// let flipped = Transform2D::rotated(PI); // 180 degrees around the bounding-box center
    ///
    /// // still contains the point it was rotated around
    /// assert!(triangle.contains_ex(v2!(5.0, 5.0), flipped));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Triangle::contains()`]
    #[inline]
    #[must_use]
    pub fn contains_ex(self, point: Vector2, transform: Transform2D) -> bool {
        let bounds_min = self.a.min(self.b).min(self.c);
        let bounds_max = self.a.max(self.b).max(self.c);
        let bounds_size = bounds_max - bounds_min;
        self.contains(transform.invert(point, bounds_min, bounds_size))
    }
}

/// Returns a value whose sign indicates which side of the line through `a`
/// and `b` the point `p` is on - the building block for [`Triangle::contains()`].
#[inline]
fn edge_sign(p: Vector2, a: Vector2, b: Vector2) -> f32 {
    (p - a).cross(b - a)
}

/// A sprite's on-screen position and size, and which part of its texture it
/// samples.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "image")]
#[doc(alias = "quad")]
#[doc(alias = "billboard")]
pub struct Sprite {
    /// The position of the top-left corner of the sprite.
    pub pos: Vector2,
    /// The size of the sprite, in pixels.
    pub size: Vector2,
    /// The sub-rectangle of the texture to sample, in texture pixels.
    /// `None` samples the whole texture.
    pub source: Option<Rect>,
}

impl Sprite {
    /// Returns a new [`Sprite`] with the given position and size, sampling
    /// the whole texture.
    #[inline(always)]
    pub const fn new(pos: Vector2, size: Vector2) -> Self {
        Self {
            pos,
            size,
            source: None,
        }
    }

    /// Returns the sprite moved by `delta`, keeping its size and the part of
    /// the texture it samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Sprite;
    /// use ancorix_math::v2;
    ///
    /// let tile = Sprite::new(v2!(0.0, 0.0), v2!(16.0, 16.0));
    ///
    /// assert_eq!(tile.offset(v2!(16.0, 0.0)).pos, v2!(16.0, 0.0));
    /// ```
    #[inline]
    #[must_use]
    pub const fn offset(self, delta: Vector2) -> Sprite {
        Sprite {
            pos: self.pos.const_add(delta),
            size: self.size,
            source: self.source,
        }
    }

    /// Returns the same sprite sampling only `source` - a sub-rectangle of
    /// the texture, in texture pixels - instead of the whole texture.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Rect, Sprite};
    /// use ancorix_math::v2;
    ///
    /// // draw the second 32x32 frame of a sheet at twice its authored size
    /// let sprite = Sprite::new(v2!(0.0, 0.0), v2!(64.0, 64.0))
    ///     .with_source(Rect::new(v2!(32.0, 0.0), v2!(32.0, 32.0)));
    ///
    /// assert_eq!(sprite.source, Some(Rect::new(v2!(32.0, 0.0), v2!(32.0, 32.0))));
    /// ```
    ///
    /// # See also
    ///
    /// * [`crate::SpriteSheet`]
    #[inline]
    pub const fn with_source(self, source: Rect) -> Self {
        Self {
            pos: self.pos,
            size: self.size,
            source: Some(source),
        }
    }

    /// Returns a new [`Sprite`] with the given center point and size.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Sprite;
    /// use ancorix_math::v2;
    ///
    /// let sprite = Sprite::from_center(v2!(20.0, 20.0), v2!(20.0, 20.0));
    ///
    /// assert_eq!(sprite.pos, v2!(10.0, 10.0));
    /// ```
    #[inline]
    pub fn from_center(center: Vector2, size: Vector2) -> Self {
        Self::new(center - size * 0.5, size)
    }

    /// Returns the [`Vector2`] center point of the sprite.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Sprite;
    /// use ancorix_math::v2;
    ///
    /// let sprite = Sprite::new(v2!(10.0, 10.0), v2!(20.0, 20.0));
    ///
    /// assert!((sprite.center().x - 20.0).abs() < 1e-5);
    /// assert!((sprite.center().y - 20.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn center(self) -> Vector2 {
        self.pos + self.size * 0.5
    }

    /// Returns `true` if `point` is inside the sprite's drawn area.
    ///
    /// Doesn't account for any [`Transform2D`] the sprite was drawn with -
    /// use [`Sprite::contains_ex()`] for that.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::Sprite;
    /// use ancorix_math::v2;
    ///
    /// let sprite = Sprite::new(v2!(10.0, 10.0), v2!(20.0, 20.0));
    ///
    /// assert!(sprite.contains(v2!(15.0, 15.0)));
    /// assert!(!sprite.contains(v2!(0.0, 0.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Sprite::contains_ex()`]
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vector2) -> bool {
        self.bounds().contains(point)
    }

    /// Returns `true` if `point` is inside the sprite's drawn area after
    /// `transform` is applied - the same transform passed to
    /// [`crate::Draw::sprite_ex()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Sprite, Transform2D};
    /// use ancorix_math::v2;
    /// use std::f32::consts::PI;
    ///
    /// let sprite = Sprite::new(v2!(0.0, 0.0), v2!(20.0, 20.0));
    /// let flipped = Transform2D::rotated(PI); // 180 degrees around the center
    ///
    /// // still contains its own center regardless of rotation
    /// assert!(sprite.contains_ex(v2!(10.0, 10.0), flipped));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Sprite::contains()`]
    /// * [`crate::Draw::sprite_ex()`]
    #[inline]
    #[must_use]
    pub fn contains_ex(self, point: Vector2, transform: Transform2D) -> bool {
        self.bounds().contains_ex(point, transform)
    }

    #[inline(always)]
    fn bounds(self) -> Rect {
        Rect::new(self.pos, self.size)
    }
}
