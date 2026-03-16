use crate::types::{DetectedScreen, Resolution, SafetyState};

/// A pixel coordinate + expected color range for safe-state detection.
/// Buffer format is BGRA (from PrintWindow).
#[derive(Debug, Clone)]
pub struct SamplePoint {
    pub x: u32,
    pub y: u32,
    pub expected_r: (u8, u8), // (min, max) inclusive range
    pub expected_g: (u8, u8),
    pub expected_b: (u8, u8),
    pub label: &'static str, // human-readable description for debug
}

impl SamplePoint {
    pub fn matches(&self, pixel: [u8; 4]) -> bool {
        // Buffer is BGRA format from PrintWindow
        let (b, g, r) = (pixel[0], pixel[1], pixel[2]);
        r >= self.expected_r.0
            && r <= self.expected_r.1
            && g >= self.expected_g.0
            && g <= self.expected_g.1
            && b >= self.expected_b.0
            && b <= self.expected_b.1
    }
}

/// Extract a single pixel (BGRA) from a flat pixel buffer at (x, y).
pub fn get_pixel(buffer: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * width + x) * 4) as usize;
    [
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ]
}

/// Get sample points for detecting the skill tree screen at a given resolution.
/// PLACEHOLDER coordinates -- must be calibrated with actual game screenshots.
/// Each resolution has its own set of coordinates because UI scales with resolution.
pub fn get_skill_tree_points(resolution: &Resolution) -> Vec<SamplePoint> {
    match resolution {
        // TODO: Calibrate with actual 1080p skill tree screenshot
        // These are placeholder coordinates targeting the skill tree panel border/chrome
        Resolution::Res1080p => vec![
            SamplePoint {
                x: 960,
                y: 100,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_header_bg",
            },
            SamplePoint {
                x: 960,
                y: 540,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_center_bg",
            },
            SamplePoint {
                x: 200,
                y: 300,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_left_panel",
            },
        ],
        Resolution::Res1440p => vec![
            SamplePoint {
                x: 1280,
                y: 133,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_header_bg",
            },
            SamplePoint {
                x: 1280,
                y: 720,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_center_bg",
            },
            SamplePoint {
                x: 267,
                y: 400,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_left_panel",
            },
        ],
        Resolution::Res4K => vec![
            SamplePoint {
                x: 1920,
                y: 200,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_header_bg",
            },
            SamplePoint {
                x: 1920,
                y: 1080,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_center_bg",
            },
            SamplePoint {
                x: 400,
                y: 600,
                expected_r: (20, 50),
                expected_g: (15, 45),
                expected_b: (10, 40),
                label: "skill_tree_left_panel",
            },
        ],
    }
}

/// Get sample points for detecting the paragon board screen at a given resolution.
/// PLACEHOLDER coordinates -- must be calibrated with actual game screenshots.
pub fn get_paragon_board_points(resolution: &Resolution) -> Vec<SamplePoint> {
    match resolution {
        Resolution::Res1080p => vec![
            SamplePoint {
                x: 960,
                y: 80,
                expected_r: (30, 70),
                expected_g: (20, 50),
                expected_b: (15, 45),
                label: "paragon_header_bg",
            },
            SamplePoint {
                x: 960,
                y: 540,
                expected_r: (10, 30),
                expected_g: (10, 30),
                expected_b: (20, 50),
                label: "paragon_board_bg",
            },
            SamplePoint {
                x: 150,
                y: 540,
                expected_r: (10, 30),
                expected_g: (10, 30),
                expected_b: (20, 50),
                label: "paragon_left_panel",
            },
        ],
        Resolution::Res1440p => vec![
            SamplePoint {
                x: 1280,
                y: 107,
                expected_r: (30, 70),
                expected_g: (20, 50),
                expected_b: (15, 45),
                label: "paragon_header_bg",
            },
            SamplePoint {
                x: 1280,
                y: 720,
                expected_r: (10, 30),
                expected_g: (10, 30),
                expected_b: (20, 50),
                label: "paragon_board_bg",
            },
            SamplePoint {
                x: 200,
                y: 720,
                expected_r: (10, 30),
                expected_g: (10, 30),
                expected_b: (20, 50),
                label: "paragon_left_panel",
            },
        ],
        Resolution::Res4K => vec![
            SamplePoint {
                x: 1920,
                y: 160,
                expected_r: (30, 70),
                expected_g: (20, 50),
                expected_b: (15, 45),
                label: "paragon_header_bg",
            },
            SamplePoint {
                x: 1920,
                y: 1080,
                expected_r: (10, 30),
                expected_g: (10, 30),
                expected_b: (20, 50),
                label: "paragon_board_bg",
            },
            SamplePoint {
                x: 300,
                y: 1080,
                expected_r: (10, 30),
                expected_g: (10, 30),
                expected_b: (20, 50),
                label: "paragon_left_panel",
            },
        ],
    }
}

/// Detect whether the current screenshot shows a safe game UI state.
/// Takes raw BGRA pixel buffer and dimensions. Pure function -- no Win32 dependency.
pub fn detect_safe_state(pixels: &[u8], width: u32, height: u32) -> SafetyState {
    let resolution = match Resolution::from_dimensions(width, height) {
        Some(r) => r,
        None => {
            return SafetyState::Unsafe {
                reason: format!("Unsupported resolution: {}x{}", width, height),
            }
        }
    };

    // Try skill tree first
    let skill_points = get_skill_tree_points(&resolution);
    let skill_match = skill_points.iter().all(|point| {
        let pixel = get_pixel(pixels, width, point.x, point.y);
        point.matches(pixel)
    });
    if skill_match {
        return SafetyState::Safe(DetectedScreen::SkillTree);
    }

    // Try paragon board
    let paragon_points = get_paragon_board_points(&resolution);
    let paragon_match = paragon_points.iter().all(|point| {
        let pixel = get_pixel(pixels, width, point.x, point.y);
        point.matches(pixel)
    });
    if paragon_match {
        return SafetyState::Safe(DetectedScreen::ParagonBoard);
    }

    // Neither matched -- find the first failing point for the error message
    let first_fail = skill_points.iter().find(|point| {
        let pixel = get_pixel(pixels, width, point.x, point.y);
        !point.matches(pixel)
    });
    let reason = match first_fail {
        Some(point) => {
            let pixel = get_pixel(pixels, width, point.x, point.y);
            format!(
                "Pixel mismatch at ({}, {}) [{}]: got RGB({}, {}, {}), expected R({}-{}), G({}-{}), B({}-{})",
                point.x, point.y, point.label,
                pixel[2], pixel[1], pixel[0], // BGRA -> RGB for display
                point.expected_r.0, point.expected_r.1,
                point.expected_g.0, point.expected_g.1,
                point.expected_b.0, point.expected_b.1,
            )
        }
        None => "Unknown unsafe state".to_string(),
    };

    SafetyState::Unsafe { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a synthetic pixel buffer filled with a single BGRA color
    fn make_buffer(width: u32, height: u32, b: u8, g: u8, r: u8, a: u8) -> Vec<u8> {
        let size = (width * height * 4) as usize;
        let mut buf = vec![0u8; size];
        for i in (0..size).step_by(4) {
            buf[i] = b;
            buf[i + 1] = g;
            buf[i + 2] = r;
            buf[i + 3] = a;
        }
        buf
    }

    #[test]
    fn test_get_pixel_extracts_correct_values() {
        let mut buf = vec![0u8; 4 * 4 * 4]; // 4x4 image
        // Set pixel at (2, 1) to BGRA = (10, 20, 30, 255)
        let offset = ((1 * 4 + 2) * 4) as usize;
        buf[offset] = 10;
        buf[offset + 1] = 20;
        buf[offset + 2] = 30;
        buf[offset + 3] = 255;
        let pixel = get_pixel(&buf, 4, 2, 1);
        assert_eq!(pixel, [10, 20, 30, 255]);
    }

    #[test]
    fn test_sample_point_matches_within_range() {
        let point = SamplePoint {
            x: 0,
            y: 0,
            expected_r: (20, 50),
            expected_g: (15, 45),
            expected_b: (10, 40),
            label: "test",
        };
        // BGRA format: B=25, G=30, R=35, A=255
        assert!(point.matches([25, 30, 35, 255]));
    }

    #[test]
    fn test_sample_point_rejects_out_of_range() {
        let point = SamplePoint {
            x: 0,
            y: 0,
            expected_r: (20, 50),
            expected_g: (15, 45),
            expected_b: (10, 40),
            label: "test",
        };
        // R=200 is way outside (20, 50) range. BGRA: B=25, G=30, R=200, A=255
        assert!(!point.matches([25, 30, 200, 255]));
    }

    #[test]
    fn test_detect_safe_state_skill_tree() {
        // Create a 1920x1080 buffer with pixels matching skill tree sample points
        let buf = make_buffer(1920, 1080, 25, 30, 35, 255);
        let result = detect_safe_state(&buf, 1920, 1080);
        assert!(matches!(
            result,
            SafetyState::Safe(DetectedScreen::SkillTree)
        ));
    }

    #[test]
    fn test_detect_safe_state_paragon_board() {
        // Create a buffer that does NOT match skill tree but DOES match paragon board.
        // Paragon board points expect: header R(30-70) G(20-50) B(15-45),
        //   board/left R(10-30) G(10-30) B(20-50).
        // Base fill: BGRA = (35, 25, 25, 255) => R=25, G=25, B=35
        // This matches paragon board bg (R 10-30, G 10-30, B 20-50) and
        // paragon header (R 30-70? No, R=25 < 30).
        // We need targeted pixels: paragon header gets R=40, G=30, B=25 (BGRA: 25,30,40,255)
        // and rest stays as base fill for board bg points.
        let mut buf = make_buffer(1920, 1080, 35, 25, 25, 255); // BGRA: B=35, G=25, R=25

        // First, corrupt ALL skill tree sample points so skill tree check fails
        let skill_points = get_skill_tree_points(&Resolution::Res1080p);
        for point in &skill_points {
            let offset = ((point.y * 1920 + point.x) * 4) as usize;
            buf[offset] = 35; // B
            buf[offset + 1] = 25; // G
            buf[offset + 2] = 200; // R -- fails skill tree (outside 20-50)
            buf[offset + 3] = 255;
        }

        // Now, set paragon board sample points to values that match paragon expectations.
        // This must happen AFTER skill tree corruption, to restore any shared coordinates.
        let paragon_points = get_paragon_board_points(&Resolution::Res1080p);
        for point in &paragon_points {
            let offset = ((point.y * 1920 + point.x) * 4) as usize;
            if point.label == "paragon_header_bg" {
                // Header expects R(30-70), G(20-50), B(15-45)
                buf[offset] = 25; // B=25
                buf[offset + 1] = 30; // G=30
                buf[offset + 2] = 40; // R=40
                buf[offset + 3] = 255;
            } else {
                // Board/left expects R(10-30), G(10-30), B(20-50)
                buf[offset] = 35; // B=35
                buf[offset + 1] = 25; // G=25
                buf[offset + 2] = 25; // R=25
                buf[offset + 3] = 255;
            }
        }

        let result = detect_safe_state(&buf, 1920, 1080);
        assert!(matches!(
            result,
            SafetyState::Safe(DetectedScreen::ParagonBoard)
        ));
    }

    #[test]
    fn test_detect_safe_state_unsafe() {
        // All white pixels -- should not match any safe state
        let buf = make_buffer(1920, 1080, 255, 255, 255, 255);
        let result = detect_safe_state(&buf, 1920, 1080);
        assert!(matches!(result, SafetyState::Unsafe { .. }));
    }

    #[test]
    fn test_detect_safe_state_unsupported_resolution() {
        let buf = make_buffer(1280, 720, 25, 30, 35, 255);
        let result = detect_safe_state(&buf, 1280, 720);
        match result {
            SafetyState::Unsafe { reason } => {
                assert!(reason.contains("Unsupported resolution"));
                assert!(reason.contains("1280x720"));
            }
            _ => panic!("Expected Unsafe for unsupported resolution"),
        }
    }
}
