mod types;
mod safety;
mod game_capture;
pub mod web_parser;
mod auto_applier;

pub use types::*;

use std::sync::Mutex;

/// Tauri command: Detect game window and return GameState.
/// Delegates to game_capture module functions (thin command pattern).
#[cfg(windows)]
#[tauri::command]
fn get_game_state(state: tauri::State<'_, Mutex<AppState>>) -> Result<GameState, String> {
    let hwnd = game_capture::window::find_diablo_window()
        .map_err(|e| e.to_string())?;

    let (width, height) = game_capture::dpi::get_game_resolution(hwnd)
        .map_err(|e| e.to_string())?;

    let dpi = game_capture::dpi::get_game_dpi(hwnd);
    let is_fullscreen = game_capture::window::is_exclusive_fullscreen(hwnd);

    let resolution = Resolution::from_dimensions(width, height);

    let game_state = GameState {
        window_found: true,
        resolution,
        raw_width: width,
        raw_height: height,
        dpi,
        is_exclusive_fullscreen: is_fullscreen,
    };

    // Store in app state
    let mut s = state.lock().unwrap();
    s.game_state = Some(game_state.clone());

    Ok(game_state)
}

/// Stub for non-Windows: returns an error since game capture requires Win32.
#[cfg(not(windows))]
#[tauri::command]
fn get_game_state(_state: tauri::State<'_, Mutex<AppState>>) -> Result<GameState, String> {
    Err("Game capture is only available on Windows".to_string())
}

/// Tauri command: Check if game is in a safe UI state for automation.
/// Captures a screenshot and runs safety detection.
#[cfg(windows)]
#[tauri::command]
fn check_safety(state: tauri::State<'_, Mutex<AppState>>) -> Result<String, String> {
    let s = state.lock().unwrap();
    let cancel_flag = s.cancel_flag.clone();
    drop(s); // Release lock before Win32 calls

    let hwnd = game_capture::window::find_diablo_window()
        .map_err(|e| e.to_string())?;
    let (width, height) = game_capture::dpi::get_game_resolution(hwnd)
        .map_err(|e| e.to_string())?;
    let pixels = game_capture::screenshot::capture_window(hwnd, width, height)
        .map_err(|e| e.to_string())?;

    let result = safety::assert_safe_state(&pixels, width, height, &cancel_flag);
    let _event = safety::safety_result_to_event(&result);

    match result {
        Ok(safe_state) => Ok(format!("{:?}", safe_state)),
        Err(e) => Err(e.to_string()),
    }
}

/// Stub for non-Windows: returns an error since safety check requires game capture.
#[cfg(not(windows))]
#[tauri::command]
fn check_safety(_state: tauri::State<'_, Mutex<AppState>>) -> Result<String, String> {
    Err("Safety check is only available on Windows".to_string())
}

/// Tauri command: Reset the emergency stop flag so automation can resume.
#[tauri::command]
fn reset_emergency_stop(state: tauri::State<'_, Mutex<AppState>>) {
    let s = state.lock().unwrap();
    s.cancel_flag.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Tauri command: Fetch and parse a d2core.com build link into a BuildPlan.
/// Accepts a full URL (https://d2core.com/d4/planner?bd=1QMw) or raw ID (1QMw).
/// Stores the result in AppState for use by the auto-applier.
#[tauri::command]
async fn parse_build_link(
    url: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<BuildPlan, String> {
    let client = web_parser::D2CoreClient::new();
    let build_plan = client.fetch_build(&url).await.map_err(|e| e.to_string())?;

    // Store in AppState
    let mut s = state.lock().unwrap();
    s.build_plan = Some(build_plan.clone());

    Ok(build_plan)
}

/// Tauri command: Start automation — applies the loaded BuildPlan to the game character.
/// Reads build_plan from AppState, then delegates to executor::run().
/// `variant_index` selects which variant to apply (0-based).
#[tauri::command]
async fn start_apply(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    variant_index: usize,
) -> Result<String, String> {
    let plan = {
        let s = state.lock().unwrap();
        s.build_plan
            .clone()
            .ok_or_else(|| "No build plan loaded".to_string())?
    };
    // state.inner() returns &Mutex<AppState> — matches run() signature directly
    auto_applier::executor::run(plan, variant_index, app, state.inner())
        .await
        .map(|_| "Apply complete".to_string())
        .map_err(|e| e.to_string())
}

/// Tauri command: Pause ongoing automation.
/// Sets cancel flag and saves current step index for resume.
#[tauri::command]
fn pause_apply(state: tauri::State<'_, Mutex<AppState>>) {
    auto_applier::executor::pause(state.inner());
}

/// Tauri command: Resume automation from the saved pause point.
/// Clears cancel flag and re-runs the executor from the saved step.
#[tauri::command]
async fn resume_apply(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    // resume() accepts &Mutex<AppState> via state.inner()
    auto_applier::executor::resume(app, state.inner())
        .await
        .map(|_| "Resume complete".to_string())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    let cancel_flag = app_state.cancel_flag.clone();

    tauri::Builder::default()
        .manage(Mutex::new(app_state))
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            check_safety,
            reset_emergency_stop,
            parse_build_link,
            start_apply,
            pause_apply,
            resume_apply,
        ])
        .setup(move |app| {
            // Register F10 emergency stop hotkey
            safety::hotkey::setup_emergency_hotkey(app.handle(), cancel_flag)
                .expect("Failed to register F10 emergency stop hotkey");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
