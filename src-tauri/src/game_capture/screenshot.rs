#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::*;
#[cfg(windows)]
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};

#[cfg(windows)]
use super::error::CaptureError;

#[cfg(windows)]
const PW_RENDERFULLCONTENT: u32 = 0x00000002;

/// Capture a screenshot of the game window as a BGRA pixel buffer.
/// Returns a Vec<u8> of length width * height * 4 in BGRA format.
#[cfg(windows)]
pub fn capture_window(hwnd: HWND, width: u32, height: u32) -> Result<Vec<u8>, CaptureError> {
    unsafe {
        let hdc_window = GetDC(Some(hwnd));
        let hdc_mem = CreateCompatibleDC(Some(hdc_window));
        let hbm = CreateCompatibleBitmap(hdc_window, width as i32, height as i32);
        let old_bm = SelectObject(hdc_mem, hbm.into());

        let success = PrintWindow(hwnd, hdc_mem, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT));

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let buf_size = (width * height * 4) as usize;
        let mut buffer = vec![0u8; buf_size];
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height,
            Some(buffer.as_mut_ptr().cast()),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI objects
        SelectObject(hdc_mem, old_bm);
        let _ = DeleteObject(hbm.into());
        DeleteDC(hdc_mem);
        ReleaseDC(Some(hwnd), hdc_window);

        if success.as_bool() {
            Ok(buffer)
        } else {
            Err(CaptureError::PrintWindowFailed)
        }
    }
}
