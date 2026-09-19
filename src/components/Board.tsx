import { useEffect, useRef, useState } from "react";
import { parseFen } from "../lib/types";

export const CELL = 66;
export const MARGIN = 46;
export const BW = CELL * 8 + MARGIN * 2;
export const BH = CELL * 9 + MARGIN * 2;
const R = 29;

export interface BoardProps {
  fen: string;
  lastMove: [number, number] | null;
  selected: number | null;
  legalTargets: number[];
  checkSquare: number | null;
  hint: { from: number; to: number } | null;
  bestArrow: { from: number; to: number } | null;
  flip: boolean;
  interactive: boolean;
  showLegal: boolean;
  onSquareClick: (sq: number) => void;
}

interface LivePiece {
  id: number;
  sq: number;
  kind: string;
  color: "red" | "black";
  char: string;
}

let pieceSeq = 1;

function xyOf(sq: number, flip: boolean): { x: number; y: number } {
  const row = Math.floor(sq / 9);
  const col = sq % 9;
  const r = flip ? 9 - row : row;
  const c = flip ? 8 - col : col;
  return { x: MARGIN + c * CELL, y: MARGIN + r * CELL };
}

const NUM_RED = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];

export default function Board(props: BoardProps) {
  const { fen, lastMove, selected, legalTargets, checkSquare, hint, bestArrow, flip, interactive, showLegal, onSquareClick } = props;

  const [pieces, setPieces] = useState<LivePiece[]>([]);
  const [anim, setAnim] = useState<{ id: number; from: number } | null>(null);
  const [animDone, setAnimDone] = useState(true);
  const prevRef = useRef<LivePiece[]>([]);

  useEffect(() => {
    const { pieces: parsed } = parseFen(fen);
    const prev = prevRef.current;
    const prevBySq = new Map<number, LivePiece>();
    prev.forEach((p) => prevBySq.set(p.sq, p));
    const usedIds = new Set<number>();
    let animInfo: { id: number; from: number } | null = null;
    const next: LivePiece[] = parsed.map((p) => {
      // 走子棋子：沿用旧 id 以获得动画
      if (lastMove && p.sq === lastMove[1]) {
        const old = prevBySq.get(lastMove[0]);
        if (old && old.kind === p.kind && old.color === p.color && !usedIds.has(old.id)) {
          usedIds.add(old.id);
          animInfo = { id: old.id, from: lastMove[0] };
          return { ...p, id: old.id };
        }
      }
      const samePos = prevBySq.get(p.sq);
      if (samePos && samePos.kind === p.kind && samePos.color === p.color && !usedIds.has(samePos.id)) {
        usedIds.add(samePos.id);
        return { ...p, id: samePos.id };
      }
      return { ...p, id: pieceSeq++ };
    });
    prevRef.current = next;
    setPieces(next);
    if (animInfo) {
      setAnim(animInfo);
      setAnimDone(false);
    } else {
      setAnim(null);
      setAnimDone(true);
    }
  }, [fen, lastMove]);

  useEffect(() => {
    if (anim && !animDone) {
      let raf2 = 0;
      const raf1 = requestAnimationFrame(() => {
        raf2 = requestAnimationFrame(() => setAnimDone(true));
      });
      return () => {
        cancelAnimationFrame(raf1);
        cancelAnimationFrame(raf2);
      };
    }
  }, [anim, animDone]);

  // 棋盘线条
  const lines: React.ReactElement[] = [];
  for (let r = 0; r < 10; r++) {
    const y1 = MARGIN + (flip ? 9 - r : r) * CELL;
    lines.push(<line key={`h${r}`} x1={MARGIN} y1={y1} x2={BW - MARGIN} y2={y1} className="bd-line" />);
  }
  for (let c = 0; c < 9; c++) {
    const x = MARGIN + (flip ? 8 - c : c) * CELL;
    if (c === 0 || c === 8) {
      lines.push(<line key={`v${c}`} x1={x} y1={MARGIN} x2={x} y2={BH - MARGIN} className="bd-line" />);
    } else {
      lines.push(<line key={`va${c}`} x1={x} y1={MARGIN} x2={x} y2={MARGIN + 4 * CELL} className="bd-line" />);
      lines.push(<line key={`vb${c}`} x1={x} y1={MARGIN + 5 * CELL} x2={x} y2={BH - MARGIN} className="bd-line" />);
    }
  }
  // 九宫斜线
  const palace: [number, number, number, number][] = [
    [0, 3, 2, 5],
    [0, 5, 2, 3],
    [7, 3, 9, 5],
    [7, 5, 9, 3],
  ];
  palace.forEach(([r1, c1, r2, c2], i) => {
    const p1 = xyOf(r1 * 9 + c1, flip);
    const p2 = xyOf(r2 * 9 + c2, flip);
    lines.push(<line key={`p${i}`} x1={p1.x} y1={p1.y} x2={p2.x} y2={p2.y} className="bd-line thin" />);
  });

  // 炮兵位标记
  const marks: React.ReactElement[] = [];
  const markPts: [number, number][] = [
    [2, 1],
    [2, 7],
    [7, 1],
    [7, 7],
    [3, 0],
    [3, 2],
    [3, 4],
    [3, 6],
    [3, 8],
    [6, 0],
    [6, 2],
    [6, 4],
    [6, 6],
    [6, 8],
  ];
  markPts.forEach(([r, c], idx) => {
    const { x, y } = xyOf(r * 9 + c, flip);
    const d = 5;
    const g = 3;
    [[-1, -1], [1, -1], [-1, 1], [1, 1]].forEach(([sx, sy], j) => {
      if (c === 0 && sx < 0) return;
      if (c === 8 && sx > 0) return;
      const px = x + sx * g;
      const py = y + sy * g;
      marks.push(
        <path
          key={`m${idx}_${j}`}
          d={`M ${px + sx * d} ${py} L ${px} ${py} L ${px} ${py + sy * d}`}
          className="bd-mark"
        />
      );
    });
  });

  // 坐标
  const coords: React.ReactElement[] = [];
  for (let i = 0; i < 9; i++) {
    const actualCol = flip ? 8 - i : i;
    const x = MARGIN + i * CELL;
    const redFile = NUM_RED[8 - actualCol];
    const blackFile = String(actualCol + 1);
    coords.push(
      <text key={`cb${i}`} x={x} y={BH - 14} className="bd-coord" textAnchor="middle">
        {flip ? blackFile : redFile}
      </text>
    );
    coords.push(
      <text key={`ct${i}`} x={x} y={22} className="bd-coord" textAnchor="middle">
        {flip ? redFile : blackFile}
      </text>
    );
  }

  const hintPts = hint ? { f: xyOf(hint.from, flip), t: xyOf(hint.to, flip) } : null;
  const bestPts = bestArrow ? { f: xyOf(bestArrow.from, flip), t: xyOf(bestArrow.to, flip) } : null;

  return (
    <div className="board-wrap" style={{ width: BW, height: BH }}>
      <svg className="board-svg" width={BW} height={BH}>
        <defs>
          <radialGradient id="bdBg" cx="50%" cy="38%" r="80%">
            <stop offset="0%" stopColor="var(--board-bg-hi)" />
            <stop offset="100%" stopColor="var(--board-bg)" />
          </radialGradient>
        </defs>
        <rect x={4} y={4} width={BW - 8} height={BH - 8} rx={18} fill="url(#bdBg)" className="bd-frame" />
        {lines}
        {marks}
        {coords}
        <text x={MARGIN + 2 * CELL} y={MARGIN + 4.5 * CELL} className="bd-river" textAnchor="middle">
          楚 河
        </text>
        <text x={MARGIN + 6 * CELL} y={MARGIN + 4.5 * CELL} className="bd-river" textAnchor="middle">
          汉 界
        </text>
        {/* 上一步高亮 */}
        {lastMove &&
          lastMove.map((sq, i) => {
            const { x, y } = xyOf(sq, flip);
            return <rect key={`lm${i}`} x={x - CELL / 2 + 4} y={y - CELL / 2 + 4} width={CELL - 8} height={CELL - 8} rx={8} className="bd-last" />;
          })}
        {/* 选中 */}
        {selected !== null && (() => {
          const { x, y } = xyOf(selected, flip);
          return <circle cx={x} cy={y} r={R + 3} className="bd-selected" />;
        })()}
        {/* 被将军 */}
        {checkSquare !== null && (() => {
          const { x, y } = xyOf(checkSquare, flip);
          return <circle cx={x} cy={y} r={R + 6} className="bd-check" />;
        })()}
        {/* 箭头 */}
        {hintPts && <Arrow f={hintPts.f} t={hintPts.t} cls="bd-arrow-hint" />}
        {bestPts && <Arrow f={bestPts.f} t={bestPts.t} cls="bd-arrow-best" />}
      </svg>

      {/* 棋子层 */}
      <div className="pieces-layer">
        {pieces.map((p) => {
          const live = xyOf(p.sq, flip);
          const startPos = anim && anim.id === p.id && !animDone ? xyOf(anim.from, flip) : live;
          const isMoving = anim && anim.id === p.id && !animDone;
          return (
            <div
              key={p.id}
              className={`piece ${p.color} ${isMoving ? "moving" : ""}`}
              style={{
                width: R * 2,
                height: R * 2,
                transform: `translate3d(${startPos.x - R}px, ${startPos.y - R}px, 0)`,
              }}
            >
              <span className="piece-char">{p.char}</span>
            </div>
          );
        })}
      </div>

      {/* 交互层 */}
      <svg className="hit-layer" width={BW} height={BH} style={{ pointerEvents: interactive ? "auto" : "none" }}>
        {Array.from({ length: 90 }, (_, sq) => {
          const { x, y } = xyOf(sq, flip);
          const isLegal = showLegal && legalTargets.includes(sq);
          return (
            <g key={sq} onClick={() => onSquareClick(sq)} style={{ cursor: interactive ? "pointer" : "default" }}>
              <circle cx={x} cy={y} r={CELL / 2} fill="transparent" />
              {isLegal &&
                (legalHasPiece(pieces, sq) ? (
                  <circle cx={x} cy={y} r={R + 2} className="bd-cap" />
                ) : (
                  <circle cx={x} cy={y} r={7} className="bd-dot" />
                ))}
            </g>
          );
        })}
      </svg>
    </div>
  );
}

function legalHasPiece(pieces: LivePiece[], sq: number): boolean {
  return pieces.some((p) => p.sq === sq);
}

function Arrow({ f, t, cls }: { f: { x: number; y: number }; t: { x: number; y: number }; cls: string }) {
  const dx = t.x - f.x;
  const dy = t.y - f.y;
  const len = Math.sqrt(dx * dx + dy * dy);
  const ux = dx / len;
  const uy = dy / len;
  const start = { x: f.x + ux * 26, y: f.y + uy * 26 };
  const end = { x: t.x - ux * 34, y: t.y - uy * 34 };
  const head = 12;
  const a1 = { x: end.x - ux * head - uy * head * 0.6, y: end.y - uy * head + ux * head * 0.6 };
  const a2 = { x: end.x - ux * head + uy * head * 0.6, y: end.y - uy * head - ux * head * 0.6 };
  return (
    <g className={cls}>
      <line x1={start.x} y1={start.y} x2={end.x} y2={end.y} />
      <polygon points={`${end.x},${end.y} ${a1.x},${a1.y} ${a2.x},${a2.y}`} />
    </g>
  );
}
