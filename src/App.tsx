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

interface CalibrationStep {
  key: keyof Omit<CalibrationData, "resolution_width" | "resolution_height" | "skill_grid_spacing" | "paragon_node_spacing">;
  label: string;
  description: string;
}

const CALIBRATION_STEPS: CalibrationStep[] = [
  { key: "skill_allocate_button", label: "技能分配按钮 / Skill Allocate Button", description: "点击游戏中技能分配按钮的位置 / Click the skill allocate button position" },
  { key: "skill_panel_origin", label: "技能面板起点 / Skill Panel Origin", description: "点击第一个技能槽的左上角 / Click the top-left of the first skill slot" },
  { key: "paragon_center", label: "传奇天赋中心 / Paragon Board Center", description: "点击传奇天赋面板的中心点 / Click the center of the paragon board" },
  { key: "paragon_nav_next", label: "下一面板按钮 / Next Board Button", description: "点击切换到下一块传奇天赋面板的按钮 / Click the next paragon board navigation button" },
  { key: "paragon_nav_prev", label: "上一面板按钮 / Previous Board Button", description: "点击切换到上一块传奇天赋面板的按钮 / Click the previous paragon board navigation button" },
];

function CalibrationWizard({ onComplete, onCancel }: {
  onComplete: (data: CalibrationData) => void;
  onCancel: () => void;
}) {
  const [screenshot, setScreenshot] = useState<string | null>(null);
  const [gameWidth, setGameWidth] = useState(1920);
  const [gameHeight, setGameHeight] = useState(1080);
  const [currentStep, setCurrentStep] = useState(0);
  const [points, setPoints] = useState<Record<string, { x: number; y: number }>>({});
  const [captureError, setCaptureError] = useState<string | null>(null);

  const captureScreenshot = async () => {
    setCaptureError(null);
    try {
      const base64 = await invoke<string>("capture_game_screenshot");
      setScreenshot(`data:image/png;base64,${base64}`);
      // Get game resolution from get_game_state
      try {
        const state = await invoke<{ raw_width: number; raw_height: number }>("get_game_state");
        setGameWidth(state.raw_width);
        setGameHeight(state.raw_height);
      } catch {
        // Default 1920x1080 if game state unavailable
      }
    } catch (e) {
      setCaptureError(formatError(String(e)));
    }
  };

  const handleImageClick = (e: React.MouseEvent<HTMLImageElement>) => {
    const img = e.currentTarget;
    const rect = img.getBoundingClientRect();
    // Scale click coordinates from displayed image size to actual game resolution
    const scaleX = gameWidth / rect.width;
    const scaleY = gameHeight / rect.height;
    const actualX = Math.round((e.clientX - rect.left) * scaleX);
    const actualY = Math.round((e.clientY - rect.top) * scaleY);

    const step = CALIBRATION_STEPS[currentStep];
    const newPoints = { ...points, [step.key]: { x: actualX, y: actualY } };
    setPoints(newPoints);

    if (currentStep < CALIBRATION_STEPS.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      // All points captured — build CalibrationData and save
      const data: CalibrationData = {
        resolution_width: gameWidth,
        resolution_height: gameHeight,
        skill_allocate_button: newPoints.skill_allocate_button,
        skill_panel_origin: newPoints.skill_panel_origin,
        skill_grid_spacing: 80, // default, user can adjust later
        paragon_center: newPoints.paragon_center,
        paragon_node_spacing: 40, // default
        paragon_nav_next: newPoints.paragon_nav_next,
        paragon_nav_prev: newPoints.paragon_nav_prev,
      };
      onComplete(data);
    }
  };

  if (!screenshot) {
    return (
      <div className="calibration-wizard">
        <h2>校准向导 / Calibration Wizard</h2>
        <p className="calibration-desc">
          请确保暗黑破坏神IV正在运行，并打开技能树界面。
          <br />
          Make sure Diablo IV is running and the skill tree is open.
        </p>
        {captureError && <div className="error-text">{captureError}</div>}
        <div className="calibration-actions">
          <button className="btn-primary" onClick={captureScreenshot}>
            截取游戏画面 / Capture Screenshot
          </button>
          <button className="btn-secondary" onClick={onCancel}>
            取消 / Cancel
          </button>
        </div>
      </div>
    );
  }

  const step = CALIBRATION_STEPS[currentStep];
  return (
    <div className="calibration-wizard">
      <h2>校准向导 / Calibration Wizard</h2>
      <div className="calibration-step-indicator">
        步骤 {currentStep + 1}/{CALIBRATION_STEPS.length} — {step.label}
      </div>
      <p className="calibration-desc">{step.description}</p>
      <div className="calibration-image-container">
        <img
          src={screenshot}
          alt="Game screenshot"
          className="calibration-screenshot"
          onClick={handleImageClick}
        />
        {/* Show previously marked points as dots */}
        {Object.entries(points).map(([key, pt]) => {
          // Scale point back to display coordinates for the dot overlay
          const imgEl = document.querySelector(".calibration-screenshot") as HTMLImageElement | null;
          if (!imgEl) return null;
          const rect = imgEl.getBoundingClientRect();
          const dispX = (pt.x / gameWidth) * rect.width;
          const dispY = (pt.y / gameHeight) * rect.height;
          return (
            <div
              key={key}
              className="calibration-dot"
              style={{ left: `${dispX}px`, top: `${dispY}px` }}
            />
          );
        })}
      </div>
      <div className="calibration-actions">
        <button
          className="btn-secondary"
          onClick={() => currentStep > 0 && setCurrentStep(currentStep - 1)}
          disabled={currentStep === 0}
        >
          上一步 / Back
        </button>
        <button className="btn-secondary" onClick={onCancel}>
          取消 / Cancel
        </button>
      </div>
    </div>
  );
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

      {/* Calibration wizard */}
      {showCalibration && (
        <CalibrationWizard
          onComplete={async (data) => {
            try {
              await invoke("save_calibration", { data });
              setCalibrated(true);
              setShowCalibration(false);
            } catch (e) {
              setApplyError(formatError(String(e)));
            }
          }}
          onCancel={() => setShowCalibration(false)}
        />
      )}
    </div>
  );
}

export default App;
