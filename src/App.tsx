import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import "./App.css";

interface EquipSkill {
  key: string;
  mods: string[];
  rank: number;
}

interface ParagonBoard {
  name: string;
  index: number;
  rotate: number;
  nodes: string[];
  glyph: string | null;
}

interface Variant {
  name: string;
  skill: Record<string, number>;
  skill_order: number[];
  equip_skills: EquipSkill[];
  paragon: ParagonBoard[];
}

interface BuildPlan {
  id: string;
  char_class: string;
  title: string;
  variants: Variant[];
}

type ApplyPhaseState = "Idle" | "Running" | "Paused" | "Complete" | "Aborted";

interface ProgressInfo {
  step: number;
  total: number;
  label: string;
}

interface CalibrationData {
  resolution_width: number;
  resolution_height: number;
  skill_allocate_button: { x: number; y: number };
  skill_panel_origin: { x: number; y: number };
  skill_grid_spacing: number;
  paragon_center: { x: number; y: number };
  paragon_node_spacing: number;
  paragon_nav_next: { x: number; y: number };
  paragon_nav_prev: { x: number; y: number };
}

const SKILL_NAMES: Record<string, string> = {
  "Basic_Lunging_Strike": "冲刺打击",
  "Core_Whirlwind": "旋风斩",
  "Defensive_Iron_Skin": "铁甲",
  "Core_Hammer_of_the_Ancients": "先祖之锤",
  "Ultimate_Call_of_the_Ancients": "先祖召唤",
  "Ultimate_Wrath_of_the_Berserker": "狂战士之怒",
};

function displaySkillName(key: string): string {
  return SKILL_NAMES[key] ?? key;
}

const ERROR_MESSAGES: Record<string, string> = {
  "no build plan loaded": "未加载构建 / No build loaded",
  "game window not found": "游戏未找到 / Game not found",
  "game not found": "游戏未找到 / Game not found",
  "unsafe state": "不安全状态 / Unsafe game state",
  "emergency stop": "紧急停止已触发 / Emergency stop triggered",
  "automation aborted": "自动化中止 / Automation aborted",
  "unsupported resolution": "不支持的分辨率 / Unsupported resolution",
  "exclusive fullscreen": "请切换为无边框窗口 / Please switch to borderless windowed",
};

function formatError(raw: string): string {
  const lower = raw.toLowerCase();
  for (const [key, msg] of Object.entries(ERROR_MESSAGES)) {
    if (lower.includes(key)) return msg;
  }
  return raw;
}

function App() {
  const [url, setUrl] = useState("");
  const [buildPlan, setBuildPlan] = useState<BuildPlan | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validation, setValidation] = useState<string | null>(null);

  const [selectedVariant, setSelectedVariant] = useState(0);
  const [applyPhase, setApplyPhase] = useState<ApplyPhaseState>("Idle");
  const [progress, setProgress] = useState<ProgressInfo | null>(null);
  const [applyError, setApplyError] = useState<string | null>(null);
  const [calibrated, setCalibrated] = useState<boolean | null>(null); // null = checking
  const [showCalibration, setShowCalibration] = useState(false);

  useEffect(() => {
    invoke<CalibrationData | null>("load_calibration").then((data) => {
      setCalibrated(data !== null);
    }).catch(() => {
      setCalibrated(false);
    });
  }, []);

  useEffect(() => {
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenSafety: UnlistenFn | undefined;
    let unlistenComplete: UnlistenFn | undefined;

    (async () => {
      unlistenProgress = await listen<ProgressInfo>("apply_progress", (event) => {
        setProgress(event.payload);
        setApplyPhase("Running");
      });

      unlistenSafety = await listen<{ type: string; reason?: string }>("safety_event", (event) => {
        const { type, reason } = event.payload;
        if (type === "AutomationAborted") {
          setApplyPhase("Aborted");
          setApplyError(reason ?? "不安全状态 / Unsafe game state");
        } else if (type === "EmergencyStop") {
          setApplyPhase("Aborted");
          setApplyError("紧急停止已触发 / Emergency stop triggered");
        } else if (type === "AutomationStarted") {
          setApplyPhase("Running");
          setApplyError(null);
        }
      });

      unlistenComplete = await listen("apply_complete", () => {
        setApplyPhase("Complete");
        setProgress(null);
      });
    })();

    return () => {
      unlistenProgress?.();
      unlistenSafety?.();
      unlistenComplete?.();
    };
  }, []);

  const handleParse = async () => {
    // Clear previous state
    setError(null);
    setValidation(null);

    // Client-side validation: must contain d2core.com and bd=
    if (!url.trim()) {
      setValidation("请粘贴构建链接");
      return;
    }
    if (!url.includes("d2core.com") || !url.includes("bd=")) {
      // Allow raw IDs (2-10 alphanumeric chars) too
      if (!/^[A-Za-z0-9]{2,10}$/.test(url.trim())) {
        setValidation("链接无效，请粘贴完整的 d2core.com 构建链接");
        return;
      }
    }

    setLoading(true);
    try {
      const result = await invoke<BuildPlan>("parse_build_link", { url: url.trim() });
      setBuildPlan(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleParse();
  };

  const handleStart = async () => {
    setApplyError(null);
    setProgress(null);
    try {
      if (applyPhase === "Paused") {
        await invoke("resume_apply");
      } else {
        setApplyPhase("Running");
        await invoke("start_apply", { variantIndex: selectedVariant });
      }
    } catch (e) {
      setApplyPhase("Aborted");
      setApplyError(formatError(String(e)));
    }
  };

  const handlePause = async () => {
    try {
      await invoke("pause_apply");
      setApplyPhase("Paused");
    } catch (e) {
      setApplyError(formatError(String(e)));
    }
  };

  const handleStop = async () => {
    try {
      await invoke("pause_apply");
      setApplyPhase("Idle");
      setProgress(null);
      setApplyError(null);
    } catch (e) {
      setApplyError(formatError(String(e)));
    }
  };

  // Use selected variant for preview
  const variant = buildPlan?.variants[selectedVariant] ?? buildPlan?.variants[0] ?? null;

  return (
    <div className="app">
      <h1 className="app-title">Diablo4 Build Applier</h1>

      {/* Link Input */}
      <div className="input-group">
        <input
          className="link-input"
          type="text"
          placeholder="粘贴 d2core.com 构建链接"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <button
          className="parse-button"
          onClick={handleParse}
          disabled={loading}
        >
          解析构建
        </button>
      </div>

      {/* Validation error */}
      {validation && <div className="validation-text">{validation}</div>}

      {/* Loading */}
      {loading && <div className="status-text">正在解析...</div>}

      {/* API error */}
      {error && !loading && <div className="error-text">{error}</div>}

      {/* Empty state — shown only when no build and not loading and no error */}
      {!buildPlan && !loading && !error && (
        <div className="empty-state">
          <h3>尚无构建</h3>
          <p>粘贴 d2core.com 构建链接后，技能和传奇天赋预览将显示在此处。</p>
        </div>
      )}

      {/* Variant selector — only when multiple variants */}
      {buildPlan && buildPlan.variants.length > 1 && (
        <select
          className="variant-select"
          value={selectedVariant}
          onChange={(e) => setSelectedVariant(Number(e.target.value))}
        >
          {buildPlan.variants.map((v, i) => (
            <option key={i} value={i}>
              {v.name || `变体 ${i + 1} / Variant ${i + 1}`}
            </option>
          ))}
        </select>
      )}

      {/* Build Preview Card */}
      {buildPlan && variant && !loading && (
        <div className="build-card">
          <div className="build-title">{buildPlan.title || "未命名构建"}</div>
          <span className="build-class">职业</span>
          <span className="build-class-value">{buildPlan.char_class}</span>

          <hr className="divider" />

          {/* Skills section */}
          <div className="section-header">技能</div>
          {variant.equip_skills.map((skill, i) => (
            <div key={i} className="skill-row">
              <span>{displaySkillName(skill.key)}</span>
              <span className="skill-rank">{skill.rank} pts</span>
            </div>
          ))}

          <hr className="divider" />

          {/* Paragon section */}
          <div className="section-header">传奇天赋</div>
          {variant.paragon.map((board, i) => (
            <div key={i} className="paragon-row">
              <span>{board.name}</span>
              {board.glyph && <span className="paragon-glyph">{board.glyph}</span>}
            </div>
          ))}
        </div>
      )}

      {/* Calibration warning */}
      {calibrated === false && (
        <div className="calibration-warning">
          请先校准坐标 / Please calibrate coordinates first
          <button className="btn-calibrate" onClick={() => setShowCalibration(true)}>
            校准 / Calibrate
          </button>
        </div>
      )}

      {/* Apply controls row — only when build is loaded */}
      {buildPlan && (
        <div className="controls-row">
          <button
            className="btn-primary"
            onClick={handleStart}
            disabled={!buildPlan || !calibrated || applyPhase === "Running"}
          >
            {applyPhase === "Paused" ? "继续 / Resume" : "开始 / Start"}
          </button>
          <button
            className="btn-secondary"
            onClick={handlePause}
            disabled={applyPhase !== "Running"}
          >
            暂停 / Pause
          </button>
          <button
            className="btn-secondary"
            onClick={handleStop}
            disabled={applyPhase === "Idle" || applyPhase === "Complete"}
          >
            停止 / Stop
          </button>
        </div>
      )}

      {/* Progress bar */}
      {progress && (
        <div className="progress-container">
          <div className="progress-track">
            <div
              className="progress-bar"
              style={{ width: `${(progress.step / progress.total) * 100}%` }}
            />
          </div>
          <div className="progress-label">
            {`应用中 ${progress.step}/${progress.total} / Applying ${progress.step} of ${progress.total}`}
          </div>
          <div className="progress-step">{progress.label}</div>
        </div>
      )}

      {/* Apply error banner */}
      {applyError && (
        <div className="error-text apply-error">{applyError}</div>
      )}

      {/* Calibration modal placeholder — showCalibration state reserved for Plan 03 */}
      {showCalibration && (
        <div className="calibration-overlay" onClick={() => setShowCalibration(false)}>
          <div className="calibration-modal" onClick={(e) => e.stopPropagation()}>
            <div className="calibration-modal-title">校准 / Calibrate</div>
            <p>校准工具将在下一版本中实现。/ Calibration tool coming in next version.</p>
            <button className="btn-primary" onClick={() => setShowCalibration(false)}>
              关闭 / Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
