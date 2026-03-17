#[cfg(windows)]
use windows::core::{BOOL, PCWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetWindowLongW, GetWindowRect, GetWindowTextW, IsWindow, GWL_STYLE,
};

#[cfg(windows)]
use super::error::CaptureError;

/// Pure logic for checking whether window style bits indicate exclusive fullscreen.
/// Extracted so it can be unit-tested without a real HWND.
pub fn check_fullscreen_style(style: u32, win_rect: [i32; 4], monitor_rect: [i32; 4]) -> bool {
    let has_popup = (style & 0x8000_0000) != 0; // WS_POPUP
    let has_frame = (style & 0x0004_0000) != 0; // WS_THICKFRAME
    let has_caption = (style & 0x00C0_0000) != 0; // WS_CAPTION

    if !has_popup || has_frame || has_caption {
        return false;
    }

    // Check if window covers entire monitor
    win_rect[0] == monitor_rect[0]
        && win_rect[1] == monitor_rect[1]
        && win_rect[2] == monitor_rect[2]
        && win_rect[3] == monitor_rect[3]
}

/// Known window class names for Diablo IV.
const D4_CLASS_NAMES: &[&str] = &["D3 Main Window Class"];

/// Known window title substrings for Diablo IV (English and Chinese).
const D4_TITLE_PATTERNS: &[&str] = &["Diablo IV", "暗黑破坏神"];

/// Check if a window title contains any known Diablo IV title pattern.
pub fn title_matches_diablo(title: &str) -> bool {
    D4_TITLE_PATTERNS.iter().any(|pattern| title.contains(pattern))
}

/// Find the Diablo IV window by class name, falling back to title enumeration.
#[cfg(windows)]
pub fn find_diablo_window() -> Result<HWND, CaptureError> {
    // Try FindWindowW with known class names first
    for class_name in D4_CLASS_NAMES {
        let encoded: Vec<u16> = format!("{}\0", class_name).encode_utf16().collect();
        if let Ok(hwnd) = unsafe { FindWindowW(PCWSTR(encoded.as_ptr()), PCWSTR::null()) } {
            return Ok(hwnd);
        }
    }

    // Fallback: enumerate windows and match by title substring
    find_by_title_substring()
}

/// Enumerate all top-level windows and find one whose title contains a known Diablo IV pattern.
#[cfg(windows)]
fn find_by_title_substring() -> Result<HWND, CaptureError> {
    let mut ctx = EnumContext { found: None };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut ctx as *mut EnumContext as isize),
        );
    }

    ctx.found.ok_or(CaptureError::WindowNotFound)
}

#[cfg(windows)]
struct EnumContext {
    found: Option<HWND>,
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut EnumContext);
    let mut title_buf = [0u16; 256];
    let len = GetWindowTextW(hwnd, &mut title_buf);
    if len > 0 {
        let title = String::from_utf16_lossy(&title_buf[..len as usize]);
        if title_matches_diablo(&title) {
            ctx.found = Some(hwnd);
            return BOOL(0); // Stop enumeration
        }
    }
    BOOL(1) // Continue
}

/// Check if the given window is in exclusive fullscreen mode.
/// Returns true if WS_POPUP is set without WS_THICKFRAME or WS_CAPTION,
/// AND the window rect covers the entire monitor.
#[cfg(windows)]
pub fn is_exclusive_fullscreen(hwnd: HWND) -> bool {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;

        let mut win_rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut win_rect);
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut mi);

        check_fullscreen_style(
            style,
            [win_rect.left, win_rect.top, win_rect.right, win_rect.bottom],
            [
                mi.rcMonitor.left,
                mi.rcMonitor.top,
                mi.rcMonitor.right,
                mi.rcMonitor.bottom,
            ],
        )
    }
}

/// Check if a window handle is still valid (not stale after game restart).
#[cfg(windows)]
#[allow(dead_code)]
pub fn is_window_valid(hwnd: HWND) -> bool {
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_exclusive_fullscreen_popup_covers_monitor() {
        // WS_POPUP = 0x80000000, no WS_THICKFRAME or WS_CAPTION
        let style = 0x8000_0000u32;
        let win_rect = [0, 0, 1920, 1080];
        let monitor_rect = [0, 0, 1920, 1080];
        assert!(check_fullscreen_style(style, win_rect, monitor_rect));
    }

    #[test]
    fn test_is_not_fullscreen_has_caption() {
        // WS_POPUP | WS_CAPTION
        let style = 0x8000_0000u32 | 0x00C0_0000u32;
        let win_rect = [0, 0, 1920, 1080];
        let monitor_rect = [0, 0, 1920, 1080];
        assert!(!check_fullscreen_style(style, win_rect, monitor_rect));
    }

    #[test]
    fn test_is_not_fullscreen_has_thickframe() {
        // WS_POPUP | WS_THICKFRAME
        let style = 0x8000_0000u32 | 0x0004_0000u32;
        let win_rect = [0, 0, 1920, 1080];
        let monitor_rect = [0, 0, 1920, 1080];
        assert!(!check_fullscreen_style(style, win_rect, monitor_rect));
    }

    #[test]
    fn test_is_not_fullscreen_no_popup() {
        // Regular windowed: no WS_POPUP
        let style = 0x00C0_0000u32; // WS_CAPTION only
        let win_rect = [0, 0, 1920, 1080];
        let monitor_rect = [0, 0, 1920, 1080];
        assert!(!check_fullscreen_style(style, win_rect, monitor_rect));
    }

    #[test]
    fn test_is_not_fullscreen_window_smaller_than_monitor() {
        // WS_POPUP but doesn't cover monitor
        let style = 0x8000_0000u32;
        let win_rect = [100, 100, 1820, 980];
        let monitor_rect = [0, 0, 1920, 1080];
        assert!(!check_fullscreen_style(style, win_rect, monitor_rect));
    }

    #[test]
    fn test_fullscreen_multi_monitor_offset() {
        // WS_POPUP, covers second monitor at offset
        let style = 0x8000_0000u32;
        let win_rect = [1920, 0, 3840, 1080];
        let monitor_rect = [1920, 0, 3840, 1080];
        assert!(check_fullscreen_style(style, win_rect, monitor_rect));
    }

    // --- title_matches_diablo tests ---

    #[test]
    fn test_title_matches_english() {
        assert!(title_matches_diablo("Diablo IV"));
    }

    #[test]
    fn test_title_matches_english_with_suffix() {
        assert!(title_matches_diablo("Diablo IV (v1.2.3)"));
    }

    #[test]
    fn test_title_matches_chinese() {
        assert!(title_matches_diablo("暗黑破坏神IV"));
    }

    #[test]
    fn test_title_matches_chinese_with_suffix() {
        assert!(title_matches_diablo("暗黑破坏神 IV"));
    }

    #[test]
    fn test_title_no_match() {
        assert!(!title_matches_diablo("Diablo III"));
        assert!(!title_matches_diablo("Notepad"));
        assert!(!title_matches_diablo(""));
    }
}
