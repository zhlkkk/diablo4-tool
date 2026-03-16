# Phase 5: GUI + Integration - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete frontend UI connecting all backend modules into a single end-to-end user flow: paste link → preview build → select variant → start automation → monitor progress → pause/stop if needed. Includes a coordinate calibration tool to replace PLACEHOLDER values from Phase 4, and bilingual (Chinese primary / English subtitle) error and status messages.

</domain>

<decisions>
## Implementation Decisions

### Apply controls & progress display
- Start/Pause/Stop buttons in a single row below the build preview card
- Start button is primary (gold accent), Pause and Stop are secondary (outlined)
- Progress bar showing automation progress (e.g., `应用中 3/15 技能点 / Applying skill 3 of 15`)
- Real-time status text updated per step event showing current action
- Listen to Tauri `apply_progress` and `safety_event` events via `@tauri-apps/api/event`
- Buttons disabled appropriately: Start disabled when no build loaded or during apply; Pause/Stop disabled when idle
- ApplyPhase state machine drives button visibility (Idle → Running → Paused → Complete/Aborted)

### Variant selector
- Dropdown selector appears above the build preview ONLY when `variants.length > 1`
- Single-variant builds skip dropdown entirely (current Phase 3 behavior preserved)
- Selecting a different variant updates the preview card immediately
- Selected variant index passed to `start_apply` command

### Language & error display
- Bilingual display: Chinese primary text + English subtitle for all alerts, errors, and status messages
- Pattern: `"链接无效 / Invalid link"`, `"游戏未找到 / Game not found"`, `"不安全状态 / Unsafe game state"`
- Error messages specific per failure type (GUI-05): game not found, invalid link, unsafe state, automation failed, emergency stop triggered
- Error banner inline below controls, not a modal — consistent with existing Phase 3 error display pattern
- Validation errors remain inline below input (existing pattern)

### Coordinate calibration tool
- Calibration mode accessible via a settings/gear icon button
- Guided wizard flow: capture game screenshot → overlay screenshot in app → user clicks to mark UI positions → save to JSON config
- Steps: "Click the Skill Tree button" → "Click the first skill slot" → "Click the Paragon Board button" → etc.
- Coordinates saved to a `calibration.json` file in app data directory (Tauri `appDataDir`)
- On app start, check for `calibration.json` — if present, override PLACEHOLDER constants in coords.rs via Tauri command
- If no calibration exists, show warning: `"请先校准坐标 / Please calibrate coordinates first"` and disable Start button
- Calibration wizard captures coordinates at current game resolution, scale_factor applies for other resolutions

### Skill name display
- Phase 3 deferred skill name mapping to Phase 5 — implement a simple lookup table (API key → Chinese name)
- If no mapping found, display raw API key as fallback (existing behavior)

### Claude's Discretion
- Exact progress bar CSS styling (should match dark theme + gold accent)
- Component file organization (single App.tsx vs extracted components)
- Calibration screenshot capture mechanism (Tauri screenshot command vs game_capture reuse)
- Exact calibration wizard step sequence and number of points to capture
- Whether to use React context or prop drilling for app state
- Tauri event listener setup and cleanup patterns

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing frontend
- `src/App.tsx` — Current React app with link input, build preview, 5-state machine (empty/loading/error/validation/preview)
- `src/App.css` — Dark theme CSS, Diablo gold `#d4af37`, 600px max-width single column layout

### Backend APIs (Tauri commands)
- `src-tauri/src/lib.rs` — All registered commands: get_game_state, check_safety, reset_emergency_stop, parse_build_link, start_apply, pause_apply, resume_apply
- `src-tauri/src/types.rs` — BuildPlan, Variant, EquipSkill, ParagonBoard, ApplyPhase (Idle/Running/Paused/Complete/Aborted), AppState, GameState, Resolution
- `src-tauri/src/auto_applier/executor.rs` — run(), pause(), resume(), ClickStep, build_step_sequence
- `src-tauri/src/auto_applier/coords.rs` — SkillTreeCoords, ParagonBoardCoords with PLACEHOLDER values, scale_coord, scale_factor

### Safety & events
- `src-tauri/src/safety/mod.rs` — SafetyEvent enum (CheckPassed, CheckFailed, EmergencyStop, AutomationStarted, AutomationAborted)

### Requirements
- `.planning/REQUIREMENTS.md` — GUI-01 through GUI-06 (all Phase 5 requirements)

### Prior decisions
- `.planning/phases/03-web-parser/03-CONTEXT.md` — Chinese-only errors (now upgraded to bilingual), first variant auto-select, raw skill API keys
- `.planning/phases/04-auto-applier/04-CONTEXT.md` — ApplyPhase state machine, progress events, safety integration, PLACEHOLDER coordinates

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App.tsx`: Link input + build preview card already working — extend, don't rewrite
- `App.css`: Complete dark theme with Diablo gold accent — extend with new component styles
- TypeScript interfaces for BuildPlan/Variant/EquipSkill/ParagonBoard already defined in App.tsx
- `@tauri-apps/api/core` already imported for `invoke`

### Established Patterns
- Single-file React app (App.tsx) — may need to extract components for Phase 5 complexity
- Chinese-only text with inline strings (no i18n framework) — bilingual can follow same pattern with `"中文 / English"` format
- State managed via useState hooks — no global state library
- Tauri invoke for commands, will need `@tauri-apps/api/event` for event listeners
- Dark theme: `#1a1a1a` background, `#262626` card background, `#404040` borders, `#d4af37` gold accent, `#f5f5f5` text

### Integration Points
- `App.tsx`: Add variant selector, apply controls, progress bar, calibration button
- `App.css`: Add styles for new components (progress bar, controls, calibration wizard)
- Tauri events: `listen("apply_progress", ...)` and `listen("safety_event", ...)` for real-time updates
- Tauri commands: `invoke("start_apply", ...)`, `invoke("pause_apply")`, `invoke("resume_apply")`
- New Tauri command needed: calibration data load/save (read/write calibration.json)

</code_context>

<specifics>
## Specific Ideas

- User explicitly requested: calibration tool for PLACEHOLDER coordinates, bilingual EN/CN alerts, progress bar, variant selector dropdown
- Calibration solves the Phase 4 known risk: "Exact paragon board UI pixel coordinates across resolutions are unknown"
- Bilingual format `"中文 / English"` is simple to implement without i18n framework — just string literals
- Progress bar should show both step count and description of current action
- Variant dropdown only appears when needed (multi-variant builds) — no UI noise for single-variant builds

</specifics>

<deferred>
## Deferred Ideas

- Full i18n framework with language toggle — v2 (bilingual inline strings sufficient for v1)
- Auto-detect build link from clipboard (QOL-V2-01)
- Build comparison (current vs target) (QOL-V2-02)
- Seasonal build template library (QOL-V2-03)
- Skill ID → human-readable name mapping beyond simple lookup table — v2

</deferred>

---

*Phase: 05-gui-integration*
*Context gathered: 2026-03-16*
