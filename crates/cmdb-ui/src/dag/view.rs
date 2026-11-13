//! Screen-space camera for the DAG: fit, wheel zoom anchored under the
//! pointer, and drag panning. Layout coordinates stay in f32; only the
//! camera is f64 so large pan offsets do not jitter.

use makepad_widgets::{DVec2, Rect, dvec2};

/// Lossy `f64 -> f32` through the engine's own vector cast, which keeps the
/// workspace's `clippy::as_conversions = deny` gate satisfied.
pub fn f32_of(value: f64) -> f32 {
    DVec2 { x: value, y: 0.0 }.into_vec2().x
}

/// Smallest camera scale the wheel may reach.
pub const MIN_SCALE: f32 = 0.15;
/// Largest camera scale the wheel may reach.
pub const MAX_SCALE: f32 = 4.0;
/// Gap kept between the fitted content and the widget edge.
pub const FIT_MARGIN: f32 = 24.0;
/// Labels are omitted below this scale to keep the first paint cheap.
pub const LABEL_MIN_SCALE: f32 = 0.45;

/// Scale plus translation from layout coordinates to window coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub scale: f32,
    pub offset: DVec2,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: dvec2(0.0, 0.0),
        }
    }
}

impl ViewTransform {
    pub fn to_screen(self, rect: Rect, x: f32, y: f32) -> DVec2 {
        let scale = f64::from(self.scale);
        dvec2(
            rect.pos.x + self.offset.x + f64::from(x) * scale,
            rect.pos.y + self.offset.y + f64::from(y) * scale,
        )
    }

    pub fn to_world(self, rect: Rect, abs: DVec2) -> (f32, f32) {
        let scale = f64::from(self.scale);
        if scale <= f64::EPSILON {
            return (0.0, 0.0);
        }
        let local_x = abs.x - rect.pos.x - self.offset.x;
        let local_y = abs.y - rect.pos.y - self.offset.y;
        (f32_of(local_x / scale), f32_of(local_y / scale))
    }

    /// Centres content of `width` by `height` inside the widget with margins.
    pub fn fit(rect: Rect, width: f32, height: f32) -> Self {
        if width <= 0.0 || height <= 0.0 || rect.size.x <= 1.0 || rect.size.y <= 1.0 {
            return Self::default();
        }
        let margin = f64::from(FIT_MARGIN) * 2.0;
        let avail_w = (rect.size.x - margin).max(1.0);
        let avail_h = (rect.size.y - margin).max(1.0);
        let scale = (avail_w / f64::from(width))
            .min(avail_h / f64::from(height))
            .clamp(f64::from(MIN_SCALE), f64::from(MAX_SCALE));
        let content = dvec2(f64::from(width) * scale, f64::from(height) * scale);
        Self {
            scale: f32_of(scale),
            offset: dvec2(
                ((rect.size.x - content.x) * 0.5).max(0.0),
                ((rect.size.y - content.y) * 0.5).max(0.0),
            ),
        }
    }

    /// Scales around a window-space anchor, keeping the world point under it.
    pub fn zoomed(self, factor: f32, anchor: DVec2, rect: Rect) -> Self {
        let scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        let (world_x, world_y) = self.to_world(rect, anchor);
        let scale_64 = f64::from(scale);
        Self {
            scale,
            offset: dvec2(
                anchor.x - rect.pos.x - f64::from(world_x) * scale_64,
                anchor.y - rect.pos.y - f64::from(world_y) * scale_64,
            ),
        }
    }

    pub fn panned(self, delta: DVec2) -> Self {
        Self {
            scale: self.scale,
            offset: dvec2(self.offset.x + delta.x, self.offset.y + delta.y),
        }
    }
}
