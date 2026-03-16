use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// === Resolution and Display ===

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Resolution {
    Res1080p,
    Res1440p,
    Res4K,
}

impl Resolution {
    pub fn from_dimensions(w: u32, h: u32) -> Option<Self> {
        match (w, h) {
            (1920, 1080) => Some(Self::Res1080p),
            (2560, 1440) => Some(Self::Res1440p),
            (3840, 2160) => Some(Self::Res4K),
            _ => None,
        }
    }
}

// === Safety ===

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyState {
    Safe(DetectedScreen),
    Unsafe { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetectedScreen {
    SkillTree,
    ParagonBoard,
}

// === Game State ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub window_found: bool,
    pub resolution: Option<Resolution>,
    pub raw_width: u32,
    pub raw_height: u32,
    pub dpi: u32,
    pub is_exclusive_fullscreen: bool,
}

// === Build Plan (from web_parser design spec) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPlan {
    pub id: String,
    pub char_class: String,
    pub title: String,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub skill: HashMap<u32, u32>,
    pub skill_order: Vec<u32>,
    pub equip_skills: Vec<EquipSkill>,
    pub paragon: Vec<ParagonBoard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipSkill {
    pub key: String,
    pub mods: Vec<String>,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagonBoard {
    pub name: String,
    pub index: u32,
    pub rotate: u32,
    #[serde(rename = "data")]
    pub nodes: Vec<String>,
    pub glyph: Option<String>,
}

// === Apply Phase ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplyPhase {
    Idle,
    Running { step: usize, total: usize },
    Paused { step: usize, total: usize },
    Complete,
    Aborted { reason: String },
}

// === App State (not Serialize — contains Arc) ===

pub struct AppState {
    pub game_state: Option<GameState>,
    pub build_plan: Option<BuildPlan>,
    pub apply_phase: ApplyPhase,
    pub cancel_flag: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            game_state: None,
            build_plan: None,
            apply_phase: ApplyPhase::Idle,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

// === Calibration Data ===

/// A serde-compatible 2D point for calibration coordinates.
/// Separate from auto_applier::coords::Point2D (which is Copy-only, not Serialize).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationPoint {
    pub x: u32,
    pub y: u32,
}

/// Calibration data saved to appDataDir/calibration.json.
/// Coordinates are captured at the resolution specified by resolution_width/height.
/// scale_factor from coords.rs is applied at runtime for other resolutions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub skill_allocate_button: CalibrationPoint,
    pub skill_panel_origin: CalibrationPoint,
    pub skill_grid_spacing: u32,
    pub paragon_center: CalibrationPoint,
    pub paragon_node_spacing: u32,
    pub paragon_nav_next: CalibrationPoint,
    pub paragon_nav_prev: CalibrationPoint,
}
