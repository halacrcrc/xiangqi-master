import { evalText } from "../lib/types";

interface Props {
  evalCp: number; // 红方视角
}

export default function EvalBar({ evalCp }: Props) {
  const clamped = Math.max(-1500, Math.min(1500, evalCp > 29000 ? 1500 : evalCp < -29000 ? -1500 : evalCp));
  const pct = 50 + 50 * (2 / (1 + Math.exp(-clamped / 320)) - 1);
  return (
    <div className="eval-bar" title={`形势评估：${evalText(evalCp)}（红方视角）`}>
      <div className="eval-black" style={{ height: `${100 - pct}%` }} />
      <div className="eval-red" style={{ height: `${pct}%` }} />
      <span className="eval-num">{evalText(evalCp)}</span>
    </div>
  );
}
