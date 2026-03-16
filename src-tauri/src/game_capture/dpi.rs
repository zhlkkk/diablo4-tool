#[cfg(windows)]
use windows::Win32::Foundation::{HWND, RECT};
#[cfg(windows)]
use windows::Win32::UI::HiDpi::GetDpiForWindow;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

#[cfg(windows)]
use super::error::CaptureError;

/// Get client area dimensions of the game window.
#[cfg(windows)]
pub fn get_game_resolution(hwnd: HWND) -> Result<(u32, u32), CaptureError> {
    let mut rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rect)
            .map_err(|e| CaptureError::GetRectFailed(e.to_string()))?;
    }
    Ok(((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32))
}

/// Get the DPI scale factor for the game window.
#[cfg(windows)]
pub fn get_game_dpi(hwnd: HWND) -> u32 {
    unsafe { GetDpiForWindow(hwnd) }
}

/// Normalize a logical coordinate to physical pixels for the game window's DPI.
/// Formula: physical_px = logical_px * dpi / 96
pub fn normalize_coord(logical: u32, game_dpi: u32) -> u32 {
    (logical as f64 * game_dpi as f64 / 96.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Resolution;

    // DPI normalization tests
    #[test]
    fn test_normalize_100_percent() {
        assert_eq!(normalize_coord(1920, 96), 1920);
    }

    #[test]
    fn test_normalize_125_percent() {
        assert_eq!(normalize_coord(1920, 120), 2400);
    }

    #[test]
    fn test_normalize_150_percent() {
        assert_eq!(normalize_coord(1920, 144), 2880);
    }

    #[test]
    fn test_normalize_200_percent() {
        assert_eq!(normalize_coord(100, 192), 200);
    }

    #[test]
    fn test_normalize_small_value() {
        assert_eq!(normalize_coord(100, 96), 100);
    }

    // Resolution detection tests (tests the Resolution type from types.rs)
    #[test]
    fn test_resolution_1080p() {
        assert_eq!(
            Resolution::from_dimensions(1920, 1080),
            Some(Resolution::Res1080p)
        );
    }

    #[test]
    fn test_resolution_1440p() {
        assert_eq!(
            Resolution::from_dimensions(2560, 1440),
            Some(Resolution::Res1440p)
        );
    }

    #[test]
    fn test_resolution_4k() {
        assert_eq!(
            Resolution::from_dimensions(3840, 2160),
            Some(Resolution::Res4K)
        );
    }

    #[test]
    fn test_resolution_unsupported() {
        assert_eq!(Resolution::from_dimensions(1280, 720), None);
    }
}
