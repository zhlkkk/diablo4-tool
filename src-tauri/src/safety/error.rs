use thiserror::Error;

#[derive(Debug, Error)]
pub enum SafetyError {
    #[error("Game is not in a safe UI state: {reason}")]
    UnsafeState { reason: String },

    #[error("Emergency stop triggered by user")]
    EmergencyStop,

    #[error("Game window lost during safety check")]
    WindowLost,
}
