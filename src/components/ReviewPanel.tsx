import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import { BADGE_COLOR, BADGE_LABEL, sqFromIccs, type ReviewItem, type Snapshot } from "../lib/types";

interface Props {
  snap: Snapshot;
  onReviewNav: (index: number | null) => void;
  onShowArrow: (from: number, to: number) => void;
  reviewItems: ReviewItem[];
  setReviewItems: (items: ReviewItem[]) => void;
}

const CLASS_ORDER = ["brilliant", "best", "good", "ok", "inaccuracy", "mistake", "blunder"];
const CLASS_NAME: Record<string, string> = {
  brilliant: "妙手",
  best: "最佳",
  good: "佳着",
  ok: "一般",
  inaccuracy: "欠准",
  mistake: "失误",
  blunder: "漏着",
};

export default function ReviewPanel({ snap, onReviewNav, onShowArrow, reviewItems, setReviewItems }: Props) {
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    const un = listen<{ done: number; total: number }>("review-progress", (e) => {
      setProgress(e.payload);
    });
    return () => {
      void un.then((f) => f());
    };
  }, []);

  const start = async () => {
    if (snap.records.length === 0) return;
    setRunning(true);
    setProgress({ done: 0, total: snap.records.length + 1 });
    try {
      const items = await api.analyzeGame(6, 500);
      setReviewItems(items);
    } catch {
      /* ignore */
    } finally {
      setRunning(false);
      setProgress(null);
    }
  };

  const accuracy = (color: "red" | "black") => {
    const mine = reviewItems.filter((i) => i.color === color);
    if (mine.length === 0) return 100;
    const sum = mine.reduce((acc, i) => {
      const l = Math.min(i.loss, 800);
      return acc + Math.max(0.1, 1 - 0.55 * Math.min(l / 300, 1) - 0.3 * Math.min(Math.max(l - 300, 0) / 500, 1));
    }, 0);
    return Math.round((sum / mine.length) * 100);
  };

  const sorted = [...reviewItems].sort((a, b) => {
    const ai = a.classification === "blunder" ? 6 : CLASS_ORDER.indexOf(a.classification);
    const bi = b.classification === "blunder" ? 6 : CLASS_ORDER.indexOf(b.classification);
    return bi - ai;
  });

  return (
    <div className="panel-col">
      <div className="panel-head">
        <span>复盘分析</span>
        {snap.records.length > 0 && (
          <button className="btn tiny primary" onClick={start} disabled={running}>
            {running ? `分析中 ${progress ? `${progress.done}/${progress.total}` : ""}` : "开始分析"}
          </button>
        )}
      </div>

      {reviewItems.length > 0 && (
        <>
          <div className="acc-row">
            <div className="acc red">
              <span>红方准确度</span>
              <b>{accuracy("red")}%</b>
            </div>
            <div className="acc black">
              <span>黑方准确度</span>
              <b>{accuracy("black")}%</b>
            </div>
          </div>
          <div className="review-list">
            {reviewItems.map((it, idx) => (
              <button
                key={idx}
                className={`rv-item ${it.classification}`}
                onClick={() => {
                  const [f, t] = sqFromIccs(it.iccs);
                  onShowArrow(f, t);
                  onReviewNav(it.ply + 1);
                }}
              >
                <span className="rv-no">{Math.floor(it.ply / 2) + 1}.</span>
                <span className={`rv-side ${it.color}`}>{it.color === "red" ? "红" : "黑"}</span>
                <span className="rv-move">{it.notation}</span>
                <span className="rv-cls" style={{ color: BADGE_COLOR[it.classification === "brilliant" ? "best" : it.classification] }}>
                  {CLASS_NAME[it.classification] ?? it.classification}
                </span>
                {it.bestNotation && it.bestNotation !== it.notation && (
                  <span className="rv-best" title={`引擎推荐：${it.bestNotation}`}>
                    → {it.bestNotation}
                  </span>
                )}
              </button>
            ))}
          </div>
          <div className="panel-head">
            <span>关键瞬间</span>
          </div>
          <div className="key-moments">
            {sorted.slice(0, 3).map((it, idx) => (
              <div key={idx} className="km-item">
                {it.color === "red" ? "红" : "黑"}方第 {Math.floor(it.ply / 2) + 1} 手 {it.notation}
                （{CLASS_NAME[it.classification]}，建议 {it.bestNotation ?? "—"}）
              </div>
            ))}
            {sorted.length === 0 && <div className="km-item">本局没有明显失误，打得漂亮！</div>}
          </div>
        </>
      )}

      {reviewItems.length === 0 && (
        <div className="review-empty">
          {snap.records.length === 0 ? "先下一盘棋，再来复盘吧。" : "点击“开始分析”，引擎将逐手评估本局，找出失误并给出改进建议。"}
          {running && progress && <div className="rv-progress"><div style={{ width: `${(progress.done / Math.max(1, progress.total)) * 100}%` }} /></div>}
        </div>
      )}
    </div>
  );
}
