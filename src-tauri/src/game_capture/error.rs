use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CaptureError {
    #[error("Diablo IV window not found — is the game running?")]
    WindowNotFound,

    #[error("Failed to get window rect: {0}")]
    GetRectFailed(String),

    #[error("PrintWindow capture failed — game may be minimized")]
    PrintWindowFailed,

    #[error("Unsupported resolution: {0}x{1}")]
    UnsupportedResolution(u32, u32),

    #[error("Game is in exclusive fullscreen — switch to Borderless Windowed")]
    ExclusiveFullscreen,

    #[error("Win32 error: {0}")]
    Win32(String),
}
