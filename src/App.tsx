import { useMemo, useState } from "react";
import Board from "./components/Board";
import Controls from "./components/Controls";
import EvalBar from "./components/EvalBar";
import MoveList from "./components/MoveList";
import NewGameDialog from "./components/NewGameDialog";
import PlayerCard from "./components/PlayerCards";
import PuzzlePanel from "./components/PuzzlePanel";
import ReviewPanel from "./components/ReviewPanel";
import SettingsDialog from "./components/SettingsDialog";
import StatsPanel from "./components/StatsPanel";
import { useGame } from "./hooks/useGame";
import { BADGE_LABEL, parseFen, type ReviewItem } from "./lib/types";
import { defaultSettings, saveStats, type Stats } from "./lib/storage";

type Tab = "moves" | "puzzle" | "review" | "stats";

export default function App() {
  const g = useGame();
  const [tab, setTab] = useState<Tab>("moves");
  const [showNew, setShowNew] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [reviewItems, setReviewItems] = useState<ReviewItem[]>([]);

  const { snap } = g;

  // 将军格
  const checkSquare = useMemo(() => {
    if (!snap?.inCheck) return null;
    const { pieces, side } = parseFen(snap.fen);
    const king = pieces.find((p) => p.kind === "k" && p.color === side);
    return king ? king.sq : null;
  }, [snap]);

  // 浏览历史局面
  const viewFen = useMemo(() => {
    if (!snap) return "";
    if (g.viewIndex === null) return snap.fen;
    if (g.viewIndex === 0) return snap.startFen;
    return snap.records[g.viewIndex - 1].fenAfter;
  }, [snap, g.viewIndex]);

  const viewing = g.viewIndex !== null;
  const viewLast: [number, number] | null = useMemo(() => {
    if (!snap) return null;
    if (g.viewIndex === null) return snap.lastMove;
    if (g.viewIndex === 0) return null;
    const rec = snap.records[g.viewIndex - 1];
    const f = (9 - parseInt(rec.iccs[1], 10)) * 9 + (rec.iccs.charCodeAt(0) - 97);
    const t = (9 - parseInt(rec.iccs[3], 10)) * 9 + (rec.iccs.charCodeAt(2) - 97);
    return [f, t];
  }, [snap, g.viewIndex]);

  if (!snap) {
    return <div className="loading">正在加载…</div>;
  }

  const inPuzzle = snap.mode === "puzzle" || snap.mode === "practice";
  const interactive = snap.status === "playing" && !viewing && !g.thinking;
  const lastBadge = snap.records.length > 0 ? snap.records[snap.records.length - 1].badge : null;
  const lastIsMine = snap.records.length > 0 && (snap.mode === "pvp" || (snap.records[snap.records.length - 1].isRed ? "red" : "black") === snap.playerSide);

  const statusText = () => {
    if (snap.status === "playing") return null;
    const win = snap.status === "red_win" ? "红方获胜" : snap.status === "black_win" ? "黑方获胜" : "和棋";
    return `${win} · ${snap.reason}`;
  };

  return (
    <div className={`app theme-${g.settings.theme}`}>
      {/* 顶栏 */}
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark">象</span>
          <span className="brand-name">象棋大师</span>
          {snap.opening && <span className="opening-tag">{snap.opening}</span>}
        </div>
        <div className="top-status">
          {g.thinking && <span className="thinking-tag">AI 思考中…</span>}
          {statusText() && <span className="result-tag">{statusText()}</span>}
        </div>
        <div className="top-actions">
          <button className="icon-btn" title="新对局" onClick={() => setShowNew(true)}>
            ＋
          </button>
          <button className="icon-btn" title="设置" onClick={() => setShowSettings(true)}>
            ⚙
          </button>
        </div>
      </header>

      <main className="main">
        {/* 左：评估条 + 棋盘 */}
        <section className="board-area">
          {!inPuzzle && <EvalBar evalCp={viewing ? 0 : snap.evalCp} />}
          <div className="board-stack">
            <PlayerCard snap={snap} side={snap.mode === "pvp" || snap.playerSide === "red" ? "black" : "red"} thinking={g.thinking} clockMs={g.clock ? g.clock.black : null} />
            <Board
              fen={viewFen}
              lastMove={viewLast}
              selected={viewing ? null : g.selected}
              legalTargets={g.legalTargets}
              checkSquare={checkSquare}
              hint={g.hintMove}
              bestArrow={g.reviewArrows}
              flip={g.flipBoard}
              interactive={interactive}
              showLegal={g.settings.showLegal}
              onSquareClick={(sq) => {
                if (viewing) return;
                const { pieces, side } = parseFen(snap.fen);
                const p = pieces.find((x) => x.sq === sq);
                if (g.selected !== null && (!p || p.color !== side)) {
                  void g.tryMove(g.selected, sq);
                } else {
                  void g.select(sq);
                }
              }}
            />
            <PlayerCard snap={snap} side={snap.mode === "pvp" ? "red" : (snap.playerSide as "red" | "black")} thinking={g.thinking} clockMs={g.clock ? g.clock.red : null} />
          </div>
          {/* 落子评价浮标 */}
          {lastBadge && !inPuzzle && (
            <div className={`fb-chip fb-${lastBadge} ${lastIsMine ? "" : "opp"}`} key={snap.records.length}>
              {lastIsMine ? "你的落子" : "对方落子"} · {BADGE_LABEL[lastBadge] ?? lastBadge}
            </div>
          )}
        </section>

        {/* 右：侧栏 */}
        <aside className="sidebar">
          <Controls
            canUndo={snap.canUndo}
            disabled={viewing || g.thinking}
            inPuzzle={inPuzzle}
            onUndo={() => void g.undoMove()}
            onHint={() => void g.requestHint()}
            onResign={() => void g.resignGame()}
            onNew={() => setShowNew(true)}
          />
          <nav className="tabs">
            {[
              { k: "moves", t: "棋谱" },
              { k: "puzzle", t: "解谜" },
              { k: "review", t: "复盘" },
              { k: "stats", t: "棋力" },
            ].map((x) => (
              <button key={x.k} className={`tab ${tab === x.k ? "on" : ""}`} onClick={() => setTab(x.k as Tab)}>
                {x.t}
                {x.k === "moves" && snap.records.length > 0 && <i className="tab-badge">{Math.ceil(snap.records.length / 2)}</i>}
              </button>
            ))}
          </nav>
          <div className="tab-body">
            {tab === "moves" && (
              <>
                <MoveList
                  records={snap.records}
                  viewIndex={g.viewIndex}
                  onSelect={(i) => {
                    g.setViewIndex(i);
                    setReviewItems([]);
                    g.setReviewArrows(null);
                  }}
                />
                {viewing && (
                  <div className="view-banner">
                    正在浏览第 {g.viewIndex} 手
                    <button className="btn tiny" onClick={() => g.setViewIndex(null)}>
                      回到当前
                    </button>
                  </div>
                )}
              </>
            )}
            {tab === "puzzle" && (
              <PuzzlePanel
                snap={snap}
                banner={g.puzzleBanner}
                onStart={(id) => void g.startPuzzle(id)}
                onClose={() => {
                  if (snap.startFen === defaultStartFen) void g.newPvpGame(null);
                  else void g.newGame(snap.difficulty, snap.playerSide as "red" | "black", null);
                }}
              />
            )}
            {tab === "review" && (
              <ReviewPanel
                snap={snap}
                reviewItems={reviewItems}
                setReviewItems={setReviewItems}
                onReviewNav={(i) => {
                  g.setViewIndex(i);
                }}
                onShowArrow={(f, t) => g.setReviewArrows({ from: f, to: t })}
              />
            )}
            {tab === "stats" && <StatsPanel stats={g.stats} />}
          </div>
        </aside>
      </main>

      {/* Toast */}
      {g.toast && <div className={`toast ${g.toast.kind}`}>{g.toast.text}</div>}

      {/* 对话框 */}
      {showNew && (
        <NewGameDialog
          onClose={() => setShowNew(false)}
          onStartAi={(d, side, minutes) => {
            setShowNew(false);
            setReviewItems([]);
            void g.newGame(d, side, minutes);
          }}
          onStartPvp={(minutes) => {
            setShowNew(false);
            setReviewItems([]);
            void g.newPvpGame(minutes);
          }}
        />
      )}
      {showSettings && (
        <SettingsDialog
          settings={g.settings}
          onChange={g.setSettings}
          onClose={() => setShowSettings(false)}
          onResetStats={() => {
            const blank: Stats = { rating: 1200, ratingHistory: [1200], games: [], puzzleSolved: {} };
            g.setStats(blank);
            saveStats(blank);
          }}
        />
      )}
    </div>
  );
}

const defaultStartFen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r";
