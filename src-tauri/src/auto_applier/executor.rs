use crate::auto_applier::error::ApplyError;
use crate::types::{ApplyPhase, BuildPlan, CalibrationData, Resolution, Variant};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use tauri::Emitter;

/// A single mouse click step in the automation sequence.
#[derive(Debug, Clone)]
pub struct ClickStep {
    pub x: u32,        // 1080p reference x
    pub y: u32,        // 1080p reference y
    pub label: String, // Human-readable description for progress events
}

/// Load CalibrationData from appDataDir/calibration.json.
/// Returns ApplyError::NoCalibration if file does not exist.
fn load_calibration_from_disk(app: &tauri::AppHandle) -> Result<CalibrationData, ApplyError> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
    let path = dir.join("calibration.json");
    if !path.exists() {
        return Err(ApplyError::NoCalibration);
    }
    let contents =
        std::fs::read_to_string(&path).map_err(|e| ApplyError::InputFailed(e.to_string()))?;
    let data: CalibrationData =
        serde_json::from_str(&contents).map_err(|e| ApplyError::InputFailed(e.to_string()))?;
    Ok(data)
}

/// Build an ordered sequence of ClickSteps from a Variant at 1080p reference coordinates.
/// Order: skills (per skill_order or skill.keys()), then equip_skills, then paragon boards.
///
/// Coordinates are 1080p reference values; scale_coord() is applied later in run().
pub fn build_step_sequence(variant: &Variant, _res: &Resolution, cal: &CalibrationData) -> Vec<ClickStep> {
    let mut steps: Vec<ClickStep> = Vec::new();

    // Phase 1: Skills — iterate in skill_order, or fall back to sorted skill.keys()
    let order: Vec<u32> = if !variant.skill_order.is_empty() {
        variant.skill_order.clone()
    } else {
        let mut keys: Vec<u32> = variant.skill.keys().cloned().collect();
        keys.sort();
        keys
    };

    for (idx, &skill_id) in order.iter().enumerate() {
        let rank = match variant.skill.get(&skill_id) {
            Some(&r) if r > 0 => r,
            _ => continue,
        };
        for n in 1..=rank {
            let x = cal.skill_allocate_button.x
                + (idx as u32) * cal.skill_grid_spacing;
            let y = cal.skill_allocate_button.y;
            steps.push(ClickStep {
                x,
                y,
                label: format!("Skill {} point {}/{}", skill_id, n, rank),
            });
        }
    }

    // Phase 2: Equip skills
    for equip in &variant.equip_skills {
        steps.push(ClickStep {
            x: cal.skill_allocate_button.x,
            y: cal.skill_allocate_button.y,
            label: format!("Equip skill {}", equip.key),
        });
    }

    // Phase 3: Paragon boards — sorted by index
    let mut boards = variant.paragon.clone();
    boards.sort_by_key(|b| b.index);

    for board in &boards {
        // Navigate to this board (nav click for boards after the first)
        if board.index > 0 {
            steps.push(ClickStep {
                x: cal.paragon_nav_next.x,
                y: cal.paragon_nav_next.y,
                label: format!("Navigate to paragon board {}", board.name),
            });
        }
        // One click per node
        for node_id in &board.nodes {
            steps.push(ClickStep {
                x: cal.paragon_center.x,
                y: cal.paragon_center.y,
                label: format!("Paragon {} node {}", board.name, node_id),
            });
        }
    }

    steps
}

/// Simulate a left mouse click at absolute screen coordinates.
/// Wrapped in spawn_blocking so async callers don't block the tokio runtime.
#[cfg(windows)]
pub async fn click_at(x: u32, y: u32) -> Result<(), ApplyError> {
    tokio::task::spawn_blocking(move || {
        use enigo::{Button, Coordinate, Direction::Click, Enigo, Mouse, Settings};
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        enigo
            .move_mouse(x as i32, y as i32, Coordinate::Abs)
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        enigo
            .button(Button::Left, Click)
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        Ok::<_, ApplyError>(())
    })
    .await
    .map_err(|e| ApplyError::TaskPanic(e.to_string()))?
}

#[cfg(not(windows))]
pub async fn click_at(_x: u32, _y: u32) -> Result<(), ApplyError> {
    Err(ApplyError::InputFailed(
        "Mouse input only available on Windows".to_string(),
    ))
}

/// Bring the game window to the foreground before automation starts.
#[cfg(windows)]
pub fn bring_window_foreground(hwnd: windows::Win32::Foundation::HWND) -> Result<(), ApplyError> {
    use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
    unsafe {
        SetForegroundWindow(hwnd);
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn bring_window_foreground(_hwnd: usize) -> Result<(), ApplyError> {
    Ok(())
}

/// Run the full automation sequence for the given BuildPlan.
///
/// Accepts `&Mutex<AppState>` (not Arc-wrapped) — matches `state.inner()` from Tauri.
/// Locks state briefly at start to extract config, then releases before any async work.
/// `variant_index` selects which variant from the plan to apply (0-based).
pub async fn run(
    plan: BuildPlan,
    variant_index: usize,
    app: tauri::AppHandle,
    state: &Mutex<crate::types::AppState>,
) -> Result<(), ApplyError> {
    // Extract game state and cancel flag under a brief lock
    let (resolution, cancel_flag, resume_step) = {
        let s = state.lock().unwrap();
        let game_state = s.game_state.as_ref().ok_or(ApplyError::NoGameState)?;
        let resolution = game_state
            .resolution
            .clone()
            .ok_or(ApplyError::UnsupportedResolution {
                width: game_state.raw_width,
                height: game_state.raw_height,
            })?;
        let cancel_flag = s.cancel_flag.clone();
        // Check if we're resuming from a paused step
        let resume_step = match s.apply_phase {
            ApplyPhase::Paused { step, .. } => step,
            _ => 0,
        };
        (resolution, cancel_flag, resume_step)
    };

    // Bring game window to foreground (Windows-only)
    #[cfg(windows)]
    {
        let hwnd = crate::game_capture::window::find_diablo_window()
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        bring_window_foreground(hwnd)?;
    }

    // Load calibration data
    let calibration = load_calibration_from_disk(&app)?;

    // Build click step sequence from the specified variant
    let variant = plan
        .variants
        .get(variant_index)
        .ok_or(ApplyError::NoBuildPlan)?;
    let steps = build_step_sequence(variant, &resolution, &calibration);
    let total = steps.len();

    // Set apply_phase to Running
    {
        let mut s = state.lock().unwrap();
        s.apply_phase = ApplyPhase::Running {
            step: resume_step,
            total,
        };
    }

    // Emit automation started event
    let _ = app.emit(
        "safety_event",
        crate::safety::SafetyEvent::AutomationStarted,
    );

    // Execute each step starting from resume_step
    for (i, step) in steps.iter().enumerate().skip(resume_step) {
        // Check cancel flag before each click
        if cancel_flag.load(Ordering::SeqCst) {
            // Check if it's a pause (apply_phase will be Paused) or full cancel
            let is_paused = {
                let s = state.lock().unwrap();
                matches!(s.apply_phase, ApplyPhase::Paused { .. })
            };
            if is_paused {
                return Ok(());
            } else {
                return Err(ApplyError::Cancelled);
            }
        }

        // Capture screenshot and run safety check (Windows-only path)
        #[cfg(windows)]
        {
            let hwnd = crate::game_capture::window::find_diablo_window()
                .map_err(|e| ApplyError::CaptureFailed(e.to_string()))?;
            let (width, height) = crate::game_capture::dpi::get_game_resolution(hwnd)
                .map_err(|e| ApplyError::CaptureFailed(e.to_string()))?;
            let pixels = crate::game_capture::screenshot::capture_window(hwnd, width, height)
                .map_err(|e| ApplyError::CaptureFailed(e.to_string()))?;

            match crate::safety::assert_safe_state(&pixels, width, height, &cancel_flag) {
                Ok(_) => {}
                Err(e) => {
                    let reason = e.to_string();
                    let _ = app.emit(
                        "safety_event",
                        crate::safety::SafetyEvent::AutomationAborted {
                            reason: reason.clone(),
                        },
                    );
                    let mut s = state.lock().unwrap();
                    s.apply_phase = ApplyPhase::Aborted {
                        reason: reason.clone(),
                    };
                    return Err(ApplyError::SafetyFailure(reason));
                }
            }
        }

        // Scale coordinates to target resolution
        let (sx, sy) = crate::auto_applier::coords::scale_from_calibration(
            step.x,
            step.y,
            calibration.resolution_width,
            &resolution,
        );

        // Apply jitter for humanization
        let (jx, jy) = crate::auto_applier::humanize::jitter_coord(sx, sy);

        // Perform the click
        click_at(jx, jy).await?;

        // Update apply_phase progress under brief lock
        {
            let mut s = state.lock().unwrap();
            s.apply_phase = ApplyPhase::Running {
                step: i + 1,
                total,
            };
        }

        // Emit progress event
        let _ = app.emit(
            "apply_progress",
            serde_json::json!({
                "step": i + 1,
                "total": total,
                "label": step.label
            }),
        );

        // Random delay between clicks
        let delay_ms = crate::auto_applier::humanize::random_delay_ms();
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }

    // Sequence complete
    {
        let mut s = state.lock().unwrap();
        s.apply_phase = ApplyPhase::Complete;
    }
    let _ = app.emit("apply_complete", serde_json::json!({"status": "complete"}));

    Ok(())
}

/// Pause automation: set the cancel flag and update apply_phase to Paused.
/// The run() loop checks this flag before each click and returns Ok() when paused.
pub fn pause(state: &Mutex<crate::types::AppState>) {
    let mut s = state.lock().unwrap();
    s.cancel_flag.store(true, Ordering::SeqCst);
    if let ApplyPhase::Running { step, total } = s.apply_phase {
        s.apply_phase = ApplyPhase::Paused { step, total };
    }
}

/// Resume automation from the saved pause point.
/// Clears cancel flag, reads build_plan, and re-runs the executor (which detects Paused state).
pub async fn resume(
    app: tauri::AppHandle,
    state: &Mutex<crate::types::AppState>,
) -> Result<(), ApplyError> {
    // Clear cancel flag and clone build plan under brief lock
    let plan = {
        let s = state.lock().unwrap();
        s.cancel_flag.store(false, Ordering::SeqCst);
        s.build_plan.clone().ok_or(ApplyError::NoBuildPlan)?
    };

    // Resume always uses variant 0 for v1 — user cannot change variant mid-apply
    run(plan, 0, app, state).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EquipSkill, ParagonBoard, Resolution, Variant};
    use std::collections::HashMap;

    fn empty_variant() -> Variant {
        Variant {
            name: "test".to_string(),
            skill: HashMap::new(),
            skill_order: vec![],
            equip_skills: vec![],
            paragon: vec![],
        }
    }

    #[test]
    fn test_step_sequence_skills_before_paragon() {
        // 2 skills (rank 1 each) + 1 paragon board (1 node)
        let mut variant = empty_variant();
        variant.skill.insert(1, 1);
        variant.skill.insert(2, 1);
        variant.paragon.push(ParagonBoard {
            name: "Board1".to_string(),
            index: 0,
            rotate: 0,
            nodes: vec!["node_a".to_string()],
            glyph: None,
        });

        let steps = build_step_sequence(&variant, &Resolution::Res1080p);

        // Should have 2 skill steps + 1 paragon step = 3 total
        assert_eq!(steps.len(), 3, "Expected 3 steps, got {}", steps.len());

        // Verify skill steps come before paragon steps
        let skill_indices: Vec<usize> = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.label.starts_with("Skill "))
            .map(|(i, _)| i)
            .collect();
        let paragon_indices: Vec<usize> = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.label.starts_with("Paragon "))
            .map(|(i, _)| i)
            .collect();

        assert!(!skill_indices.is_empty(), "No skill steps found");
        assert!(!paragon_indices.is_empty(), "No paragon steps found");
        assert!(
            skill_indices
                .iter()
                .all(|&si| paragon_indices.iter().all(|&pi| si < pi)),
            "Skill steps must all come before paragon steps"
        );
    }

    #[test]
    fn test_step_sequence_respects_skill_order() {
        // skill_order=[2, 1] means skill 2 should produce steps before skill 1
        let mut variant = empty_variant();
        variant.skill.insert(1, 1);
        variant.skill.insert(2, 1);
        variant.skill_order = vec![2, 1];

        let steps = build_step_sequence(&variant, &Resolution::Res1080p);

        assert_eq!(steps.len(), 2, "Expected 2 steps");
        assert!(
            steps[0].label.contains("Skill 2"),
            "First step label should reference Skill 2, got: {}",
            steps[0].label
        );
        assert!(
            steps[1].label.contains("Skill 1"),
            "Second step label should reference Skill 1, got: {}",
            steps[1].label
        );
    }

    #[test]
    fn test_step_sequence_empty_variant() {
        let variant = empty_variant();
        let steps = build_step_sequence(&variant, &Resolution::Res1080p);
        assert!(steps.is_empty(), "Empty variant should produce no steps");
    }

    #[test]
    fn test_step_sequence_uses_specified_variant() {
        let mut v0 = empty_variant();
        v0.skill.insert(1, 1);

        let mut v1 = empty_variant();
        v1.skill.insert(99, 2);

        // Build steps for variant index 1 (v1 with skill 99, rank 2)
        let steps = build_step_sequence(&v1, &Resolution::Res1080p);
        assert_eq!(steps.len(), 2, "Variant 1 should produce 2 steps for skill 99 rank 2");
        assert!(steps[0].label.contains("Skill 99"), "Steps should reference skill 99, got: {}", steps[0].label);
    }

    #[test]
    fn test_step_sequence_equip_after_skill() {
        // 1 skill + 1 equip_skill — equip step must come after skill step
        let mut variant = empty_variant();
        variant.skill.insert(5, 1);
        variant.equip_skills.push(EquipSkill {
            key: "slot_1".to_string(),
            mods: vec![],
            rank: 0,
        });

        let steps = build_step_sequence(&variant, &Resolution::Res1080p);

        assert_eq!(steps.len(), 2, "Expected 2 steps");
        assert!(
            steps[0].label.starts_with("Skill "),
            "First step should be a skill step, got: {}",
            steps[0].label
        );
        assert!(
            steps[1].label.starts_with("Equip skill "),
            "Second step should be an equip step, got: {}",
            steps[1].label
        );
    }
}
