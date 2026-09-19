import { LEVELS } from "../lib/types";
import type { Stats } from "../lib/storage";

interface Props {
  stats: Stats;
}

export default function StatsPanel({ stats }: Props) {
  const games = stats.games;
  const wins = games.filter((g) => g.result === "win").length;
  const losses = games.filter((g) => g.result === "loss").length;
  const draws = games.filter((g) => g.result === "draw").length;
  const solved = Object.values(stats.puzzleSolved).filter((p) => p.solved).length;

  // 等级分曲线
  const hist = stats.ratingHistory.length > 1 ? stats.ratingHistory : [stats.rating, stats.rating];
  const min = Math.min(...hist) - 10;
  const max = Math.max(...hist) + 10;
  const W = 260;
  const H = 70;
  const pts = hist
    .map((v, i) => {
      const x = (i / (hist.length - 1)) * W;
      const y = H - ((v - min) / Math.max(1, max - min)) * H;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  // 最近对局
  const recent = [...games].reverse().slice(0, 8);

  return (
    <div className="panel-col">
      <div className="panel-head">
        <span>我的棋力</span>
      </div>
      <div className="stat-hero">
        <div className="rating-big">{stats.rating}</div>
        <div className="rating-sub">等级分 · AI 对局实时结算</div>
      </div>
      <svg className="rating-chart" width={W} height={H}>
        <polyline points={pts} className="rating-line" />
        <circle
          cx={W}
          cy={H - ((hist[hist.length - 1] - min) / Math.max(1, max - min)) * H}
          r={3}
          className="rating-dot"
        />
      </svg>
      <div className="stat-grid">
        <div className="stat-cell">
          <b>{games.length}</b>
          <span>总对局</span>
        </div>
        <div className="stat-cell win">
          <b>{wins}</b>
          <span>胜</span>
        </div>
        <div className="stat-cell draw">
          <b>{draws}</b>
          <span>和</span>
        </div>
        <div className="stat-cell loss">
          <b>{losses}</b>
          <span>负</span>
        </div>
        <div className="stat-cell">
          <b>{solved}</b>
          <span>杀局通关</span>
        </div>
      </div>
      <div className="panel-head">
        <span>最近对局</span>
      </div>
      <div className="recent-list">
        {recent.length === 0 && <div className="rv-empty">还没有对局记录</div>}
        {recent.map((g, i) => (
          <div key={i} className={`recent-item ${g.result}`}>
            <span className={`dot ${g.result}`} />
            <span className="rg-name">{LEVELS[g.difficulty]?.name ?? "对局"}</span>
            <span className="rg-side">执{g.playerColor === "red" ? "红" : "黑"}</span>
            <span className="rg-result">{g.result === "win" ? "胜" : g.result === "draw" ? "和" : "负"}</span>
            <span className={`rg-delta ${g.result}`}>{g.ratingDelta !== undefined && g.ratingDelta !== 0 ? (g.ratingDelta > 0 ? `+${g.ratingDelta}` : `${g.ratingDelta}`) : ""}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
