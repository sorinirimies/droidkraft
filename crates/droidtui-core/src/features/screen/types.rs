//! Screen domain types.

/// A device screen resolution and density.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScreenResolution {
    pub width: u32,
    pub height: u32,
    pub density: u32,
}

impl ScreenResolution {
    /// Parse `wm size` and `wm density` output into a resolution.
    ///
    /// Example inputs:
    /// - `"Physical size: 1080x2340"`
    /// - `"Physical density: 440"`
    pub fn parse(size_output: &str, density_output: &str) -> Option<Self> {
        let (width, height) = size_output
            .lines()
            .find_map(|l| l.split_once(": ").map(|(_, v)| v))
            .and_then(|v| v.trim().split_once('x'))
            .and_then(|(w, h)| Some((w.trim().parse().ok()?, h.trim().parse().ok()?)))?;
        let density = density_output
            .lines()
            .find_map(|l| l.split_once(": ").map(|(_, v)| v))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0);
        Some(Self {
            width,
            height,
            density,
        })
    }

    /// Aspect ratio (width / height), or 0.0 when height is zero.
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_resolution() {
        let r = ScreenResolution::parse("Physical size: 1080x2340", "Physical density: 440")
            .unwrap();
        assert_eq!(r.width, 1080);
        assert_eq!(r.height, 2340);
        assert_eq!(r.density, 440);
    }

    #[test]
    fn parse_resolution_no_density() {
        let r = ScreenResolution::parse("Physical size: 720x1280", "garbage").unwrap();
        assert_eq!((r.width, r.height, r.density), (720, 1280, 0));
    }

    #[test]
    fn parse_resolution_invalid() {
        assert!(ScreenResolution::parse("nope", "nope").is_none());
    }

    #[test]
    fn aspect_ratio_computed() {
        let r = ScreenResolution {
            width: 100,
            height: 200,
            density: 0,
        };
        assert!((r.aspect_ratio() - 0.5).abs() < f32::EPSILON);
    }
}
