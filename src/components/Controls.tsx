interface Props {
  canUndo: boolean;
  disabled: boolean;
  inPuzzle: boolean;
  onUndo: () => void;
  onHint: () => void;
  onResign: () => void;
  onNew: () => void;
}

export default function Controls({ canUndo, disabled, inPuzzle, onUndo, onHint, onResign, onNew }: Props) {
  return (
    <div className="controls">
      <button className="btn primary" onClick={onNew}>
        {inPuzzle ? "返回对局" : "新对局"}
      </button>
      <button className="btn" onClick={onUndo} disabled={!canUndo || disabled}>
        {inPuzzle ? "重做本题" : "悔棋"}
      </button>
      <button className="btn" onClick={onHint} disabled={disabled}>
        提示
      </button>
      {!inPuzzle && (
        <button className="btn danger" onClick={onResign} disabled={disabled}>
          认输
        </button>
      )}
    </div>
  );
}
