use rand::Rng;

/// Add random jitter of 2-5 pixels magnitude to coordinates.
/// Per user decision: +/-2-5px jitter range (minimum 2px offset).
/// Clamps to minimum 0 to prevent underflow.
pub fn jitter_coord(x: u32, y: u32) -> (u32, u32) {
    let mut rng = rand::thread_rng();
    // Generate magnitude in [2, 5] then random sign
    let mag_x: i32 = rng.gen_range(2..=5);
    let mag_y: i32 = rng.gen_range(2..=5);
    let sign_x: i32 = if rng.gen_bool(0.5) { 1 } else { -1 };
    let sign_y: i32 = if rng.gen_bool(0.5) { 1 } else { -1 };
    let jx = mag_x * sign_x;
    let jy = mag_y * sign_y;
    ((x as i32 + jx).max(0) as u32, (y as i32 + jy).max(0) as u32)
}

/// Generate a random delay between click actions.
/// Per user decision: 50-200ms range mimics human reaction time.
pub fn random_delay_ms() -> u64 {
    rand::thread_rng().gen_range(50..=200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_within_bounds() {
        for _ in 0..100 {
            let (rx, ry) = jitter_coord(500, 500);
            assert!(
                rx >= 495 && rx <= 505,
                "jitter_coord x out of bounds: got {}",
                rx
            );
            assert!(
                ry >= 495 && ry <= 505,
                "jitter_coord y out of bounds: got {}",
                ry
            );
        }
    }

    #[test]
    fn test_jitter_magnitude_at_least_2() {
        for _ in 0..100 {
            let (rx, ry) = jitter_coord(500, 500);
            let diff_x = (rx as i32 - 500).abs();
            let diff_y = (ry as i32 - 500).abs();
            assert!(
                diff_x >= 2,
                "jitter magnitude x must be >= 2, got {}",
                diff_x
            );
            assert!(
                diff_y >= 2,
                "jitter magnitude y must be >= 2, got {}",
                diff_y
            );
        }
    }

    #[test]
    fn test_jitter_clamps_at_zero() {
        for _ in 0..100 {
            let (rx, ry) = jitter_coord(0, 0);
            assert!(rx <= 5, "jitter x from origin must be <= 5, got {}", rx);
            assert!(ry <= 5, "jitter y from origin must be <= 5, got {}", ry);
        }
    }

    #[test]
    fn test_jitter_near_zero() {
        for _ in 0..100 {
            let (rx, ry) = jitter_coord(2, 2);
            assert!(rx <= 7, "jitter x from 2 must be <= 7, got {}", rx);
            assert!(ry <= 7, "jitter y from 2 must be <= 7, got {}", ry);
        }
    }

    #[test]
    fn test_delay_in_range() {
        for _ in 0..100 {
            let d = random_delay_ms();
            assert!(
                d >= 50 && d <= 200,
                "delay out of range [50, 200]: got {}",
                d
            );
        }
    }

    #[test]
    fn test_jitter_has_variance() {
        use std::collections::HashSet;
        let results: HashSet<(u32, u32)> = (0..20)
            .map(|_| jitter_coord(500, 500))
            .collect();
        assert!(
            results.len() > 1,
            "jitter_coord should not always return the same value"
        );
    }
}
