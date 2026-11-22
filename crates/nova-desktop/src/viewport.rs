//! Limit screenshot dimensions to bound CPU cost and frame memory.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub const DEFAULT: Self = Self { width: 1200, height: 740 };

    pub fn from_window(width: f32, height: f32) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() || width < 1.0 || height < 1.0 {
            return None;
        }
        let width = width.round().clamp(64.0, 3840.0) as u32;
        let mut height = height.round().clamp(64.0, 2160.0) as u32;
        const MAX_AREA: u32 = 8_000_000;
        if width.saturating_mul(height) > MAX_AREA {
            height = MAX_AREA / width;
        }
        Some(Self { width, height })
    }

    pub fn css_size(self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }

    pub fn point(self, x: f32, y: f32) -> Option<(f32, f32)> {
        if !x.is_finite() || !y.is_finite() || !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            return None;
        }
        Some((x * self.width as f32, y * self.height as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_normalized_pointer_to_viewport() {
        let vp = Viewport::from_window(800.0, 600.0).unwrap();
        assert_eq!(vp.point(0.5, 0.25), Some((400.0, 150.0)));
        assert!(vp.point(-0.1, 0.5).is_none());
    }

    #[test]
    fn ignores_invalid_size_and_caps_memory() {
        assert!(Viewport::from_window(0.0, 800.0).is_none());
        assert!(Viewport::from_window(f32::INFINITY, 800.0).is_none());
        let vp = Viewport::from_window(10000.0, 10000.0).unwrap();
        assert!(vp.width * vp.height <= 8_000_000);
    }
}
