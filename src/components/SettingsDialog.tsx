import type { Settings } from "../lib/storage";

interface Props {
  settings: Settings;
  onChange: (s: Settings) => void;
  onClose: () => void;
  onResetStats: () => void;
}

const THEMES = [
  { v: "dark", label: "墨夜" },
  { v: "wood", label: "原木" },
  { v: "jade", label: "青玉" },
] as const;

export default function SettingsDialog({ settings, onChange, onClose, onResetStats }: Props) {
  const toggle = (key: keyof Settings) => () => onChange({ ...settings, [key]: !settings[key] });

  return (
    <div className="dialog-mask" onClick={onClose}>
      <div className="dialog narrow" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">设置</div>

        <div className="field">
          <label>主题</label>
          <div className="seg">
            {THEMES.map((t) => (
              <button key={t.v} className={`seg-btn ${settings.theme === t.v ? "on" : ""}`} onClick={() => onChange({ ...settings, theme: t.v })}>
                {t.label}
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <label>开关</label>
          <div className="toggle-list">
            <button className="toggle-row" onClick={toggle("sound")}>
              <span>音效</span>
              <i className={`switch ${settings.sound ? "on" : ""}`} />
            </button>
            <button className="toggle-row" onClick={toggle("showLegal")}>
              <span>显示可走位置</span>
              <i className={`switch ${settings.showLegal ? "on" : ""}`} />
            </button>
            <button className="toggle-row" onClick={toggle("instantFeedback")}>
              <span>落子即时评价（AI 辅导）</span>
              <i className={`switch ${settings.instantFeedback ? "on" : ""}`} />
            </button>
            <button className="toggle-row" onClick={toggle("flipWithSide")}>
              <span>执黑时自动翻转棋盘</span>
              <i className={`switch ${settings.flipWithSide ? "on" : ""}`} />
            </button>
          </div>
        </div>

        <div className="dialog-actions space-between">
          <button className="btn danger-ghost" onClick={onResetStats}>
            清空棋力数据
          </button>
          <button className="btn primary" onClick={onClose}>
            完成
          </button>
        </div>
      </div>
    </div>
  );
}
