// Game capture module — finds D4 window, detects resolution, handles DPI, captures screenshots

pub mod error;
pub mod window;
pub mod dpi;

pub use error::CaptureError;
