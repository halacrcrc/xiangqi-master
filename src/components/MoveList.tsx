import { useEffect, useRef } from "react";
import { BADGE_COLOR, BADGE_LABEL, type MoveRecord } from "../lib/types";

interface Props {
  records: MoveRecord[];
  viewIndex: number | null;
  onSelect: (index: number | null) => void;
}

export default function MoveList({ records, viewIndex, onSelect }: Props) {
  const listRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (viewIndex === null && listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, [records.length, viewIndex]);

  const rows: React.ReactElement[] = [];
  for (let i = 0; i < records.length; i += 2) {
    const red = records[i];
    const black = records[i + 1];
    rows.push(
      <div className="mv-row" key={i}>
        <span className="mv-no">{i / 2 + 1}.</span>
        <button
          className={`mv-cell red ${viewIndex === i + 1 ? "active" : ""}`}
          onClick={() => onSelect(viewIndex === i + 1 ? null : i + 1)}
          title={red.badge ? `评价：${BADGE_LABEL[red.badge] ?? ""}${red.bestIccs ? " · 推荐 " + red.bestIccs : ""}` : undefined}
        >
          {red.notation}
          {red.badge && <i className="mv-badge" style={{ background: BADGE_COLOR[red.badge] }} title={BADGE_LABEL[red.badge]} />}
          {red.mate && <b className="mv-mate">杀</b>}
        </button>
        {black ? (
          <button
            className={`mv-cell black ${viewIndex === i + 2 ? "active" : ""}`}
            onClick={() => onSelect(viewIndex === i + 2 ? null : i + 2)}
            title={black.badge ? `评价：${BADGE_LABEL[black.badge] ?? ""}${black.bestIccs ? " · 推荐 " + black.bestIccs : ""}` : undefined}
          >
            {black.notation}
            {black.badge && <i className="mv-badge" style={{ background: BADGE_COLOR[black.badge] }} title={BADGE_LABEL[black.badge]} />}
            {black.mate && <b className="mv-mate">杀</b>}
          </button>
        ) : (
          <span className="mv-cell empty" />
        )}
      </div>
    );
  }

  return (
    <div className="mv-list" ref={listRef}>
      {rows.length === 0 && <div className="mv-empty">尚无着法 · 红先行</div>}
      {rows}
    </div>
  );
}
