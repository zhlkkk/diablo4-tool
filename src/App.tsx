import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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

function App() {
  const [url, setUrl] = useState("");
  const [buildPlan, setBuildPlan] = useState<BuildPlan | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validation, setValidation] = useState<string | null>(null);

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

  // Use first variant for preview (per CONTEXT.md decision)
  const variant = buildPlan?.variants[0] ?? null;

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
              <span>{skill.key}</span>
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
    </div>
  );
}

export default App;
