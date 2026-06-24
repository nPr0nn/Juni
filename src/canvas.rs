//! A `Canvas` is handed to `Game::draw`. Its methods push geometry into the
//! current frame's batch; all shapes are decomposed into triangles. Coordinates
//! are in virtual-canvas pixels (origin top-left, +Y down).

use crate::color::Color;
use crate::math::{Rect, Vec2D};
use crate::renderer::Batch;
use crate::Shader;

pub struct Canvas<'a> {
    batch: &'a mut Batch,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(batch: &'a mut Batch) -> Self {
        Self { batch }
    }

    /// Set the background (render-texture clear) color for this frame.
    pub fn clear_background(&mut self, color: Color) {
        self.batch.clear_color = color;
    }

    /// Draw subsequent shapes with a custom [`Shader`] until
    /// [`end_shader_mode`](Self::end_shader_mode). Raylib-style: build the shader
    /// once (e.g. in `Game::init` via
    /// [`Context::load_shader_from_memory`](crate::Context::load_shader_from_memory))
    /// and wrap the draw calls you want it applied to.
    pub fn begin_shader_mode(&mut self, shader: &Shader) {
        self.batch.set_pipeline(Some(shader.pipeline.clone()));
    }

    /// Revert to the default shape shader. Pairs with
    /// [`begin_shader_mode`](Self::begin_shader_mode).
    pub fn end_shader_mode(&mut self) {
        self.batch.set_pipeline(None);
    }

    /// Draw a filled, axis-aligned rectangle.
    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let tl = [x, y];
        let tr = [x + width, y];
        let br = [x + width, y + height];
        let bl = [x, y + height];
        self.batch.push_triangle(tl, tr, br, color);
        self.batch.push_triangle(tl, br, bl, color);
    }

    /// Draw a filled rectangle from a [`Rect`].
    pub fn rectangle_from_rect(&mut self, rec: Rect, color: Color) {
        self.rectangle(rec.x, rec.y, rec.width, rec.height, color);
    }

    /// Draw a filled triangle. Any winding works (culling is disabled).
    pub fn triangle(&mut self, v1: Vec2D, v2: Vec2D, v3: Vec2D, color: Color) {
        self.batch
            .push_triangle(v1.to_array(), v2.to_array(), v3.to_array(), color);
    }

    /// Draw a filled quad from four corners (in order, e.g. clockwise).
    pub fn quad(&mut self, v1: Vec2D, v2: Vec2D, v3: Vec2D, v4: Vec2D, color: Color) {
        self.quad_gradient(v1, v2, v3, v4, color, color, color, color);
    }

    /// Draw a quad with one color per corner (in the same order as the
    /// vertices). The colors interpolate smoothly across the surface — give the
    /// four corners distinct hues for a rainbow fill.
    #[allow(clippy::too_many_arguments)]
    pub fn quad_gradient(
        &mut self,
        v1: Vec2D,
        v2: Vec2D,
        v3: Vec2D,
        v4: Vec2D,
        c1: Color,
        c2: Color,
        c3: Color,
        c4: Color,
    ) {
        self.batch
            .push_triangle_gradient(v1.to_array(), v2.to_array(), v3.to_array(), c1, c2, c3);
        self.batch
            .push_triangle_gradient(v1.to_array(), v3.to_array(), v4.to_array(), c1, c3, c4);
    }

    /// Draw a line segment of the given thickness (a rotated rectangle). A
    /// zero-length segment draws nothing.
    pub fn line(&mut self, start: Vec2D, end: Vec2D, thickness: f32, color: Color) {
        let dir = end - start;
        let len = dir.length();
        if len <= f32::EPSILON {
            return;
        }
        // Perpendicular offset of half the thickness on each side.
        let normal = Vec2D::new(-dir.y, dir.x) / len * (thickness * 0.5);
        self.quad(start + normal, end + normal, end - normal, start - normal, color);
    }

    /// Draw a filled regular polygon with `sides` sides (>= 3), `rotation` in
    /// degrees. Built as a fan of triangles from the center.
    pub fn regular_polygon(
        &mut self,
        center: Vec2D,
        sides: u32,
        radius: f32,
        rotation: f32,
        color: Color,
    ) {
        if sides < 3 {
            return;
        }
        let step = std::f32::consts::TAU / sides as f32;
        let start = rotation.to_radians();
        let point = |i: u32| {
            let a = start + step * i as f32;
            center + Vec2D::new(a.cos(), a.sin()) * radius
        };
        for i in 0..sides {
            self.triangle(center, point(i), point(i + 1), color);
        }
    }

    /// Draw a filled circle. The number of segments is chosen from the radius;
    /// it is a [`regular_polygon`](Self::regular_polygon) with enough sides to
    /// look round.
    pub fn circle(&mut self, center: Vec2D, radius: f32, color: Color) {
        let segments = ((radius * 0.6) as u32).clamp(12, 64);
        self.regular_polygon(center, segments, radius, 0.0, color);
    }
}
