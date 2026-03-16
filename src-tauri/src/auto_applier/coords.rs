use crate::types::Resolution;

#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: u32,
    pub y: u32,
}

/// Returns the linear scale factor for a given resolution relative to 1080p baseline.
pub fn scale_factor(res: &Resolution) -> f64 {
    match res {
        Resolution::Res1080p => 1.0,
        Resolution::Res1440p => 2560.0 / 1920.0,
        Resolution::Res4K => 3840.0 / 1920.0,
    }
}

/// Scale a 1080p reference coordinate to the target resolution.
pub fn scale_coord(x: u32, y: u32, res: &Resolution) -> (u32, u32) {
    let factor = scale_factor(res);
    ((x as f64 * factor).round() as u32, (y as f64 * factor).round() as u32)
}

/// Skill tree UI coordinate constants (1080p reference, requires empirical measurement).
pub struct SkillTreeCoords;

impl SkillTreeCoords {
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const ALLOCATE_BUTTON: Point2D = Point2D { x: 960, y: 800 };
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const SKILL_PANEL_ORIGIN: Point2D = Point2D { x: 400, y: 200 };
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const SKILL_GRID_SPACING: u32 = 80;
}

/// Scale a coordinate captured at calibration resolution to the target game resolution.
/// calibration_width is the resolution_width from CalibrationData.
/// target_res is the current game Resolution.
pub fn scale_from_calibration(x: u32, y: u32, calibration_width: u32, target_res: &Resolution) -> (u32, u32) {
    // First normalize to 1080p (calibration may have been done at any resolution)
    let calib_to_1080p = 1920.0 / calibration_width as f64;
    let x_1080p = (x as f64 * calib_to_1080p).round() as u32;
    let y_1080p = (y as f64 * calib_to_1080p).round() as u32;
    // Then scale from 1080p to target
    scale_coord(x_1080p, y_1080p, target_res)
}

/// Paragon board UI coordinate constants (1080p reference, requires empirical measurement).
pub struct ParagonBoardCoords;

impl ParagonBoardCoords {
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const CENTER: Point2D = Point2D { x: 960, y: 540 };
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const NODE_SPACING: u32 = 40;
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const BOARD_NAV_NEXT: Point2D = Point2D { x: 1700, y: 540 };
    /// PLACEHOLDER: requires empirical measurement at 1080p
    pub const BOARD_NAV_PREV: Point2D = Point2D { x: 220, y: 540 };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Resolution;

    #[test]
    fn test_scale_coord_1080p_identity() {
        assert_eq!(scale_coord(960, 540, &Resolution::Res1080p), (960, 540));
    }

    #[test]
    fn test_scale_coord_1440p() {
        assert_eq!(scale_coord(960, 540, &Resolution::Res1440p), (1280, 720));
    }

    #[test]
    fn test_scale_coord_4k() {
        assert_eq!(scale_coord(960, 540, &Resolution::Res4K), (1920, 1080));
    }

    #[test]
    fn test_scale_coord_origin_stays_origin() {
        assert_eq!(scale_coord(0, 0, &Resolution::Res4K), (0, 0));
    }

    #[test]
    fn test_scale_coord_boundary_1080p() {
        assert_eq!(scale_coord(1920, 1080, &Resolution::Res1080p), (1920, 1080));
    }

    #[test]
    fn test_scale_factor_1080p() {
        assert_eq!(scale_factor(&Resolution::Res1080p), 1.0);
    }

    #[test]
    fn test_scale_factor_1440p() {
        let f = scale_factor(&Resolution::Res1440p);
        // 2560/1920 = 1.333...
        assert!((f - 1.333).abs() < 0.001, "Expected ~1.333, got {}", f);
    }

    #[test]
    fn test_scale_factor_4k() {
        assert_eq!(scale_factor(&Resolution::Res4K), 2.0);
    }

    #[test]
    fn test_skill_tree_allocate_button_nonzero() {
        let pt = SkillTreeCoords::ALLOCATE_BUTTON;
        assert!(pt.x > 0 && pt.y > 0, "ALLOCATE_BUTTON should have non-zero coords");
    }

    #[test]
    fn test_paragon_center_nonzero() {
        let pt = ParagonBoardCoords::CENTER;
        assert!(pt.x > 0 && pt.y > 0, "PARAGON CENTER should have non-zero coords");
    }

    #[test]
    fn test_scale_from_calibration_same_resolution() {
        // Calibrated at 1080p, target 1080p — should be identity
        assert_eq!(scale_from_calibration(960, 540, 1920, &Resolution::Res1080p), (960, 540));
    }

    #[test]
    fn test_scale_from_calibration_1080p_to_1440p() {
        // Calibrated at 1080p, target 1440p — should scale up
        let (x, y) = scale_from_calibration(960, 540, 1920, &Resolution::Res1440p);
        assert_eq!((x, y), (1280, 720));
    }

    #[test]
    fn test_scale_from_calibration_1440p_to_1080p() {
        // Calibrated at 1440p (2560 wide), target 1080p
        // 1280 at 2560 -> normalize to 1080p: 1280 * (1920/2560) = 960
        let (x, y) = scale_from_calibration(1280, 720, 2560, &Resolution::Res1080p);
        assert_eq!((x, y), (960, 540));
    }
}
