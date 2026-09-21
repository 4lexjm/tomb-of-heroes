//! Multi-ratio viewport module.
//!
//! Enforces `SPEC-REQ-VIEW-001` and `SPEC-REQ-VIEW-002`.

use bevy::ecs::system::Resource;

/// Logical virtual height (240 pixels).
pub const LOGICAL_HEIGHT: u32 = 240;
/// Logical virtual minimum width for 4:3 aspect ratio (320 pixels).
pub const LOGICAL_WIDTH_MIN: u32 = 320;
/// Logical virtual reference width for 16:9 aspect ratio (426 pixels).
pub const LOGICAL_WIDTH_REF: u32 = 426;
/// Logical virtual maximum width for 21:9 ultrawide aspect ratio (560 pixels).
pub const LOGICAL_WIDTH_MAX: u32 = 560;

/// Computes the integer pixel-perfect scaling factor.
///
/// Implements `SPEC-REQ-VIEW-002`:
/// $$S = \max\left(1, \, \min\left(\left\lfloor \frac{W_{\text{physique}}}{W_{\text{logic\_target}}} \right\rfloor, \, \left\lfloor \frac{H_{\text{physique}}}{240} \right\rfloor\right)\right)$$
#[must_use]
pub fn compute_pixel_perfect_scale(phys_w: u32, phys_h: u32, target_w: u32) -> u32 {
    if target_w == 0 || LOGICAL_HEIGHT == 0 {
        return 1;
    }
    let scale_w = phys_w / target_w;
    let scale_h = phys_h / LOGICAL_HEIGHT;
    scale_w.min(scale_h).max(1)
}

/// Viewport geometry containing scaling, logical, rendered, and letterbox metrics.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportGeometry {
    /// Integer scaling factor S.
    pub scale: u32,
    /// Logical virtual width in pixels.
    pub logical_width: u32,
    /// Logical virtual height in pixels (240 px).
    pub logical_height: u32,
    /// Rendered pixel width (`logical_width * scale`).
    pub rendered_width: u32,
    /// Rendered pixel height (`logical_height * scale`).
    pub rendered_height: u32,
    /// Symmetrical horizontal letterbox / pillarbox offset in physical pixels.
    pub letterbox_x: u32,
    /// Symmetrical vertical letterbox offset in physical pixels.
    pub letterbox_y: u32,
}

impl ViewportGeometry {
    /// Computes viewport geometry from physical window dimensions and logical target width.
    #[must_use]
    pub fn from_physical_dimensions(phys_w: u32, phys_h: u32, target_w: u32) -> Self {
        let scale = compute_pixel_perfect_scale(phys_w, phys_h, target_w);
        let logical_width = target_w;
        let logical_height = LOGICAL_HEIGHT;
        let rendered_width = logical_width.saturating_mul(scale);
        let rendered_height = logical_height.saturating_mul(scale);
        let letterbox_x = phys_w.saturating_sub(rendered_width) / 2;
        let letterbox_y = phys_h.saturating_sub(rendered_height) / 2;
        Self {
            scale,
            logical_width,
            logical_height,
            rendered_width,
            rendered_height,
            letterbox_x,
            letterbox_y,
        }
    }

    /// Convenience alias for `from_physical_dimensions`.
    #[inline]
    #[must_use]
    pub fn from_physical(phys_w: u32, phys_h: u32, target_w: u32) -> Self {
        Self::from_physical_dimensions(phys_w, phys_h, target_w)
    }

    /// Derives the safe zone for this viewport geometry.
    #[inline]
    #[must_use]
    pub fn safe_zone(&self) -> SafeZone {
        SafeZone::from_logical_width(self.logical_width)
    }
}

impl Default for ViewportGeometry {
    fn default() -> Self {
        Self::from_physical_dimensions(LOGICAL_WIDTH_REF, LOGICAL_HEIGHT, LOGICAL_WIDTH_REF)
    }
}

/// Safe Zone rectangle ensuring UI anchors and critical tiles are always visible.
///
/// Implements `SPEC-REQ-VIEW-001`.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeZone {
    /// Minimum horizontal coordinate in logical pixels.
    pub x_min: i32,
    /// Maximum horizontal coordinate in logical pixels.
    pub x_max: i32,
    /// Minimum vertical coordinate in logical pixels (0).
    pub y_min: i32,
    /// Maximum vertical coordinate in logical pixels (240).
    pub y_max: i32,
}

impl SafeZone {
    /// Constructs a centered safe zone for a given logical width.
    #[must_use]
    pub fn from_logical_width(logical_width: u32) -> Self {
        let safe_w = LOGICAL_WIDTH_MIN;
        let x_min = if logical_width >= safe_w {
            ((logical_width.saturating_sub(safe_w)) / 2) as i32
        } else {
            0
        };
        let x_max = x_min.saturating_add(safe_w as i32);
        let y_min = 0;
        let y_max = LOGICAL_HEIGHT as i32;
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Constructs a safe zone from logical width and height.
    #[must_use]
    pub fn from_logical_dimensions(logical_w: u32, logical_h: u32) -> Self {
        let safe_w = LOGICAL_WIDTH_MIN;
        let x_min = if logical_w >= safe_w {
            ((logical_w.saturating_sub(safe_w)) / 2) as i32
        } else {
            0
        };
        let x_max = x_min.saturating_add(safe_w as i32);
        let y_min = 0;
        let y_max = logical_h as i32;
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Checks if a logical coordinate point lies within the safe zone boundaries.
    #[inline]
    #[must_use]
    pub fn is_inside(&self, x: i32, y: i32) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }

    /// Clamps an arbitrary coordinate pair to the safe zone boundaries.
    #[inline]
    #[must_use]
    pub fn clamp(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x.clamp(self.x_min, self.x_max),
            y.clamp(self.y_min, self.y_max),
        )
    }
}

impl Default for SafeZone {
    fn default() -> Self {
        Self::from_logical_width(LOGICAL_WIDTH_REF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_geometry_defaults() {
        let default_geom = ViewportGeometry::default();
        assert_eq!(default_geom.logical_height, LOGICAL_HEIGHT);
        assert_eq!(default_geom.logical_width, LOGICAL_WIDTH_REF);
        assert_eq!(default_geom.scale, 1);

        let default_safe_zone = SafeZone::default();
        assert_eq!(default_safe_zone.x_min, 53);
        assert_eq!(default_safe_zone.x_max, 373);
        assert_eq!(default_safe_zone.y_min, 0);
        assert_eq!(default_safe_zone.y_max, 240);
    }

    #[test]
    fn test_viewport_geometry_from_physical_alias() {
        let geom1 = ViewportGeometry::from_physical_dimensions(1920, 1080, 426);
        let geom2 = ViewportGeometry::from_physical(1920, 1080, 426);
        assert_eq!(geom1, geom2);
    }

    #[test]
    fn test_safe_zone_from_logical_dimensions() {
        let sz = SafeZone::from_logical_dimensions(560, 240);
        // (560 - 320) / 2 = 120
        assert_eq!(sz.x_min, 120);
        assert_eq!(sz.x_max, 440);
        assert_eq!(sz.y_min, 0);
        assert_eq!(sz.y_max, 240);
    }

    #[test]
    fn test_safe_zone_under_safe_width() {
        let sz = SafeZone::from_logical_width(200);
        assert_eq!(sz.x_min, 0);
        assert_eq!(sz.x_max, 320);
    }
}
