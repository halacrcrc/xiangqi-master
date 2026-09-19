import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { loadStats, type Stats } from "../lib/storage";
import type { PuzzleInfo, Snapshot } from "../lib/types";
interface Props {
  snap: Snapshot;
  onStart: (id: number) => void;
  onClose: () => void;
  banner: string | null;
}

export default function PuzzlePanel({ snap, onStart, onClose, banner }: Props) {
  const [list, setList] = useState<PuzzleInfo[]>([]);
  const [stats, setStats] = useState<Stats>(() => loadStats());
  const [showHint, setShowHint] = useState<string | null>(null);

  useEffect(() => {
    api.puzzlesList().then(setList).catch(() => {});
  }, []);

  useEffect(() => {
    setStats(loadStats());
    setShowHint(null);
  }, [snap.puzzle?.id, snap.puzzle?.solved]);

  const inPuzzle = snap.mode === "puzzle" || snap.mode === "practice";

  return (
    <div className="panel-col">
      {inPuzzle ? (
        <>
          <div className="panel-head">
            <span>解谜模式</span>
            <button className="btn tiny" onClick={onClose}>
              退出
            </button>
          </div>
          <div className="puzzle-active">
            <div className="pz-name">
              #{snap.puzzle!.id} {snap.puzzle!.name}
              <span className="pz-diff">{"★".repeat(snap.puzzle!.difficulty)}</span>
            </div>
            <div className="pz-meta">
              {snap.puzzle!.theme} · {snap.puzzle!.objective}
            </div>
            {snap.puzzle!.wrongCount > 0 && <div className="pz-wrong">已尝试失败 {snap.puzzle!.wrongCount} 次</div>}
            {banner && <div className={`pz-banner ${snap.puzzle!.solved ? "ok" : ""}`}>{banner}</div>}
            {snap.puzzle!.solved && (
              <div className="pz-done">
                恭喜通关！
                <button className="btn tiny" onClick={() => {
                  const next = list.find((p) => p.id === snap.puzzle!.id + 1);
                  if (next) onStart(next.id);
                }}>
                  下一题
                </button>
              </div>
            )}
            <div className="pz-actions">
              <button
                className="btn"
                onClick={async () => {
                  try {
                    const { invoke } = await import("@tauri-apps/api/core");
                    const h = await invoke<{ kind: string; iccs?: string; notation?: string; text: string }>("puzzle_hint");
                    if (h.kind === "move" && h.notation) setShowHint(`推荐着法：${h.notation}`);
                    else setShowHint(h.text);
                  } catch {
                    setShowHint("暂无提示");
                  }
                }}
              >
                看提示
              </button>
            </div>
            {showHint && <div className="pz-hint">{showHint}</div>}
          </div>
        </>
      ) : (
        <>
          <div className="panel-head">
            <span>杀局 & 残局训练</span>
            <span className="panel-sub">
              已通关 {Object.values(stats.puzzleSolved).filter((p) => p.solved).length}/{list.length}
            </span>
          </div>
          <div className="puzzle-list">
            {list.map((p) => {
              const rec = stats.puzzleSolved[p.id];
              return (
                <button key={p.id} className={`pz-card ${rec?.solved ? "done" : ""}`} onClick={() => onStart(p.id)}>
                  <div className="pz-card-head">
                    <b>
                      #{p.id} {p.name}
                    </b>
                    {rec?.solved && <i className="pz-check">✓</i>}
                  </div>
                  <div className="pz-card-meta">
                    {p.isMate ? `${p.mateIn} 回合杀` : "残局实战"} · {p.theme} · {"★".repeat(p.difficulty)}
                    {rec && !rec.solved && rec.wrong > 0 && <em>· 试过 {rec.wrong} 次</em>}
                  </div>
                </button>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
