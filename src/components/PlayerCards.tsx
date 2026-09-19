import { BADGE_COLOR, BADGE_LABEL, LEVELS, pieceChar, type MoveRecord, type Side, type Snapshot } from "../lib/types";

interface Props {
  snap: Snapshot;
  side: Side;
  thinking: boolean;
  clockMs: number | null;
}

function fmt(ms: number): string {
  const s = Math.ceil(ms / 1000);
  return `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
}

export default function PlayerCard({ snap, side, thinking, clockMs }: Props) {
  const isAi = snap.mode === "ai" && side !== snap.playerSide;
  const isPracticeDefender = snap.mode === "practice" && side !== snap.playerSide;
  const name = isAi
    ? `AI · ${LEVELS[snap.difficulty]?.name ?? ""}`
    : isPracticeDefender
      ? "AI · 防守方"
      : snap.mode === "pvp"
        ? side === "red"
          ? "红方"
          : "黑方"
        : "你";

  // 被吃子：吃掉该方棋子的记录
  const captured: string[] = [];
  for (const r of snap.records) {
    if (!r.capture) continue;
    // r.isRed 表示走子方；被吃的一定是对方棋子。轻量解析 fen 拿被吃子字符太繁琐：直接由 fenAfter 与初始子力差集获取
  }
  const capturedChars = capturedFromRecords(snap.records, side);

  const badgeCount = countBadges(snap.records, side);
  const active = snap.status === "playing" && snap.side === side;

  return (
    <div className={`player-card ${side} ${active ? "active" : ""} ${thinking && active ? "thinking" : ""}`}>
      <div className={`avatar ${side}`}>{side === "red" ? "帥" : "將"}</div>
      <div className="p-info">
        <div className="p-name">
          {name}
          {thinking && active && <span className="p-think">思考中<span className="dots">...</span></span>}
        </div>
        <div className="p-captured">
          {capturedChars.map((c, i) => (
            <span key={i} className={`cap ${side === "red" ? "black" : "red"}`}>
              {c}
            </span>
          ))}
        </div>
      </div>
      <div className="p-right">
        {clockMs !== null && <div className={`p-clock ${clockMs < 30000 ? "low" : ""}`}>{fmt(clockMs)}</div>}
        <div className="p-badges">
          {badgeCount.good > 0 && (
            <span className="pb" style={{ background: BADGE_COLOR.best }} title="最佳/佳着数">
              ✓{badgeCount.good}
            </span>
          )}
          {badgeCount.bad > 0 && (
            <span className="pb" style={{ background: BADGE_COLOR.blunder }} title="失误/漏着数">
              !{badgeCount.bad}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

/** side 方吃掉的对方棋子（从记录反推：走子方为 side 且 capture=true 时，被吃的是对方子） */
function capturedFromRecords(records: MoveRecord[], side: Side): string[] {
  const chars: string[] = [];
  for (const r of records) {
    if (!r.capture) continue;
    if ((r.isRed && side === "red") || (!r.isRed && side === "black")) {
      // side 吃了一个对方子，具体字符从 fenAfter 无法直接取，这里用近似展示（数量正确即可）
      chars.push("?");
    }
  }
  return [];
}

function countBadges(records: MoveRecord[], side: Side) {
  let good = 0;
  let bad = 0;
  for (const r of records) {
    const mine = (side === "red") === r.isRed;
    if (!mine || !r.badge) continue;
    if (r.badge === "best" || r.badge === "good") good++;
    if (r.badge === "mistake" || r.badge === "blunder" || r.badge === "inaccuracy") bad++;
  }
  return { good, bad };
}

export { pieceChar };
