//! Emergency stop hotkey setup (F10) using tauri-plugin-global-shortcut.
//! Sets the cancel_flag AtomicBool and emits an emergency_stop Tauri event.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

/// Register the F10 global hotkey for emergency stop.
/// When pressed, sets cancel_flag to true and emits "emergency_stop" event.
///
/// Called during Tauri `setup` phase. The plugin must be registered before
/// the shortcut can be bound, so this function both registers the plugin
/// and binds the F10 key.
pub fn setup_emergency_hotkey(
    app: &AppHandle,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.clone();
    let flag = cancel_flag;

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                if shortcut == &Shortcut::new(None, Code::F10)
                    && event.state() == ShortcutState::Pressed
                {
                    flag.store(true, Ordering::SeqCst);
                    let _ = app_handle.emit("emergency_stop", ());
                }
            })
            .build(),
    )?;

    let f10 = Shortcut::new(None, Code::F10);
    app.global_shortcut().register(f10)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f10_shortcut_creation() {
        // Verify we can construct the F10 shortcut without panic
        let f10 = Shortcut::new(None, Code::F10);
        // Shortcut should be valid — if this compiles and runs, the types are correct
        assert_eq!(f10, Shortcut::new(None, Code::F10));
    }
}
