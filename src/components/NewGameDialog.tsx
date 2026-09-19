import { useState } from "react";
import { LEVELS } from "../lib/types";

interface Props {
  onClose: () => void;
  onStartAi: (difficulty: number, side: "red" | "black" | "random", minutes: number | null) => void;
  onStartPvp: (minutes: number | null) => void;
}

const CLOCKS = [
  { label: "不限时", minutes: null },
  { label: "10 分钟", minutes: 10 },
  { label: "5 分钟快棋", minutes: 5 },
];

export default function NewGameDialog({ onClose, onStartAi, onStartPvp }: Props) {
  const [difficulty, setDifficulty] = useState(2);
  const [side, setSide] = useState<"red" | "black" | "random">("red");
  const [clockIdx, setClockIdx] = useState(0);

  return (
    <div className="dialog-mask" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">开始新对局</div>

        <div className="field">
          <label>AI 难度</label>
          <div className="level-grid">
            {LEVELS.map((l, i) => (
              <button key={i} className={`level-card ${difficulty === i ? "on" : ""}`} onClick={() => setDifficulty(i)}>
                <b>{l.name}</b>
                <span>{l.desc}</span>
                <em>Elo ≈ {l.elo}</em>
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <label>你的执子</label>
          <div className="seg">
            {[
              { v: "red", t: "执红先行" },
              { v: "random", t: "随机" },
              { v: "black", t: "执黑后行" },
            ].map((o) => (
              <button key={o.v} className={`seg-btn ${side === o.v ? "on" : ""}`} onClick={() => setSide(o.v as typeof side)}>
                {o.t}
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <label>用时</label>
          <div className="seg">
            {CLOCKS.map((c, i) => (
              <button key={i} className={`seg-btn ${clockIdx === i ? "on" : ""}`} onClick={() => setClockIdx(i)}>
                {c.label}
              </button>
            ))}
          </div>
        </div>

        <div className="dialog-actions">
          <button className="btn" onClick={onClose}>
            取消
          </button>
          <button className="btn ghost" onClick={() => onStartPvp(CLOCKS[clockIdx].minutes)}>
            双人对战
          </button>
          <button className="btn primary" onClick={() => onStartAi(difficulty, side, CLOCKS[clockIdx].minutes)}>
            人机对局
          </button>
        </div>
      </div>
    </div>
  );
}
