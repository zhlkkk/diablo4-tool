// Safety module -- detects game UI state via pixel sampling, gates automation on safe state

pub mod detector;
pub mod error;
pub mod hotkey;

pub use detector::{detect_safe_state, get_pixel, SamplePoint};
pub use error::SafetyError;

use crate::types::SafetyState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Events emitted by the safety module for user transparency (SAFE-05).
/// These are serialized and sent to the frontend via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SafetyEvent {
    /// Safety check passed -- automation may proceed
    CheckPassed {
        screen: String, // "SkillTree" or "ParagonBoard"
    },
    /// Safety check failed -- automation blocked
    CheckFailed { reason: String },
    /// Emergency stop triggered by user (F10)
    EmergencyStop,
    /// Automation started
    AutomationStarted,
    /// Automation halted due to safety failure
    AutomationAborted { reason: String },
}

/// Gate function: check if the game is in a safe state for automation.
/// Called before EVERY click step (SAFE-03 -- re-check before each click).
///
/// Takes pixel data directly (pure function pattern -- no Win32 dependency).
/// The caller (auto_applier) is responsible for capturing a fresh screenshot
/// and passing it here before each click.
///
/// Also checks the emergency stop flag (SAFE-04).
pub fn assert_safe_state(
    pixels: &[u8],
    width: u32,
    height: u32,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<SafetyState, SafetyError> {
    // Check emergency stop first (SAFE-04)
    if cancel_flag.load(Ordering::SeqCst) {
        return Err(SafetyError::EmergencyStop);
    }

    // Check game UI state (SAFE-01, SAFE-02)
    let state = detect_safe_state(pixels, width, height);
    match &state {
        SafetyState::Safe(_) => Ok(state),
        SafetyState::Unsafe { reason } => Err(SafetyError::UnsafeState {
            reason: reason.clone(),
        }),
    }
}

/// Create a SafetyEvent from the result of assert_safe_state.
/// Convenience function for the Tauri command layer to emit events (SAFE-05).
pub fn safety_result_to_event(result: &Result<SafetyState, SafetyError>) -> SafetyEvent {
    match result {
        Ok(SafetyState::Safe(screen)) => SafetyEvent::CheckPassed {
            screen: format!("{:?}", screen),
        },
        Ok(SafetyState::Unsafe { reason }) => SafetyEvent::CheckFailed {
            reason: reason.clone(),
        },
        Err(SafetyError::EmergencyStop) => SafetyEvent::EmergencyStop,
        Err(SafetyError::UnsafeState { reason }) => SafetyEvent::CheckFailed {
            reason: reason.clone(),
        },
        Err(SafetyError::WindowLost) => SafetyEvent::AutomationAborted {
            reason: "Game window lost".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DetectedScreen;

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
    fn test_gate_allows_safe_state() {
        let cancel = Arc::new(AtomicBool::new(false));
        let buf = make_buffer(1920, 1080, 25, 30, 35, 255);
        let result = assert_safe_state(&buf, 1920, 1080, &cancel);
        assert!(result.is_ok());
        match result.unwrap() {
            SafetyState::Safe(DetectedScreen::SkillTree) => {}
            other => panic!("Expected Safe(SkillTree), got {:?}", other),
        }
    }

    #[test]
    fn test_gate_blocks_unsafe_state() {
        let cancel = Arc::new(AtomicBool::new(false));
        let buf = make_buffer(1920, 1080, 255, 255, 255, 255); // all white -- unsafe
        let result = assert_safe_state(&buf, 1920, 1080, &cancel);
        assert!(result.is_err());
        match result.unwrap_err() {
            SafetyError::UnsafeState { reason } => {
                assert!(reason.contains("Pixel mismatch"));
            }
            other => panic!("Expected UnsafeState, got {:?}", other),
        }
    }

    #[test]
    fn test_gate_emergency_stop_takes_priority() {
        let cancel = Arc::new(AtomicBool::new(true)); // emergency stop active
        let buf = make_buffer(1920, 1080, 25, 30, 35, 255); // would be safe
        let result = assert_safe_state(&buf, 1920, 1080, &cancel);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SafetyError::EmergencyStop));
    }

    #[test]
    fn test_gate_recheck_pattern() {
        // Simulate re-check before each click (SAFE-03)
        let cancel = Arc::new(AtomicBool::new(false));
        let safe_buf = make_buffer(1920, 1080, 25, 30, 35, 255);
        let unsafe_buf = make_buffer(1920, 1080, 255, 255, 255, 255);

        // First check: safe
        assert!(assert_safe_state(&safe_buf, 1920, 1080, &cancel).is_ok());
        // Second check: still safe
        assert!(assert_safe_state(&safe_buf, 1920, 1080, &cancel).is_ok());
        // Third check: game state changed to unsafe
        assert!(assert_safe_state(&unsafe_buf, 1920, 1080, &cancel).is_err());
    }

    #[test]
    fn test_safety_event_from_ok_result() {
        let result: Result<SafetyState, SafetyError> =
            Ok(SafetyState::Safe(DetectedScreen::SkillTree));
        let event = safety_result_to_event(&result);
        match event {
            SafetyEvent::CheckPassed { screen } => assert_eq!(screen, "SkillTree"),
            other => panic!("Expected CheckPassed, got {:?}", other),
        }
    }

    #[test]
    fn test_safety_event_from_emergency_stop() {
        let result: Result<SafetyState, SafetyError> = Err(SafetyError::EmergencyStop);
        let event = safety_result_to_event(&result);
        assert!(matches!(event, SafetyEvent::EmergencyStop));
    }
}
