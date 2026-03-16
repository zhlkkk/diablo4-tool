use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplyError {
    #[error("Safety check failed: {0}")]
    SafetyFailure(String),
    #[error("Input simulation failed: {0}")]
    InputFailed(String),
    #[error("Screenshot capture failed: {0}")]
    CaptureFailed(String),
    #[error("No build plan loaded")]
    NoBuildPlan,
    #[error("No game state available")]
    NoGameState,
    #[error("Unsupported resolution: {width}x{height}")]
    UnsupportedResolution { width: u32, height: u32 },
    #[error("Background task panicked: {0}")]
    TaskPanic(String),
    #[error("Automation cancelled")]
    Cancelled,
    #[error("No calibration data found — please calibrate coordinates first")]
    NoCalibration,
}
