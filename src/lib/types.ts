export type Side = "red" | "black";

export interface MoveRecord {
  ply: number;
  iccs: string;
  notation: string;
  isRed: boolean;
  capture: boolean;
  check: boolean;
  mate: boolean;
  badge?: string | null;
  bestIccs?: string | null;
  loss?: number | null;
  fenAfter: string;
}

export interface PuzzleView {
  id: number;
  name: string;
  theme: string;
  hint: string;
  difficulty: number;
  isMate: boolean;
  mateIn: number;
  solved: boolean;
  wrongCount: number;
  objective: string;
}

export interface Snapshot {
  fen: string;
  side: Side;
  records: MoveRecord[];
  lastMove: [number, number] | null;
  inCheck: boolean;
  status: "playing" | "red_win" | "black_win" | "draw";
  reason: string;
  evalCp: number;
  opening?: string | null;
  mode: "ai" | "pvp" | "puzzle" | "practice" | "study";
  difficulty: number;
  playerSide: Side | "both";
  canUndo: boolean;
  puzzle?: PuzzleView | null;
  startFen: string;
}

export interface ReviewItem {
  ply: number;
  iccs: string;
  notation: string;
  color: "red" | "black";
  scoreBefore: number;
  scoreAfter: number;
  loss: number;
  bestIccs?: string | null;
  bestNotation?: string | null;
  classification: string;
  isCheck: boolean;
  isCapture: boolean;
}

export interface PuzzleInfo {
  id: number;
  name: string;
  theme: string;
  difficulty: number;
  isMate: boolean;
  mateIn: number;
}

export interface PieceInfo {
  sq: number;
  kind: string; // k a b n r c p
  color: Side;
  char: string;
}

const CHAR_RED: Record<string, string> = { k: "帥", a: "仕", b: "相", n: "馬", r: "車", c: "炮", p: "兵" };
const CHAR_BLACK: Record<string, string> = { k: "將", a: "士", b: "象", n: "馬", r: "車", c: "砲", p: "卒" };

export function pieceChar(kind: string, color: Side): string {
  return color === "red" ? CHAR_RED[kind] : CHAR_BLACK[kind];
}

export function parseFen(fen: string): { pieces: PieceInfo[]; side: Side } {
  const pieces: PieceInfo[] = [];
  const [rows, sideField] = fen.split(" ");
  const rowList = rows.split("/");
  for (let r = 0; r < rowList.length && r < 10; r++) {
    let c = 0;
    for (const ch of rowList[r]) {
      if (/\d/.test(ch)) {
        c += parseInt(ch, 10);
      } else {
        const kind = ch.toLowerCase();
        const color: Side = ch === kind ? "black" : "red";
        pieces.push({ sq: r * 9 + c, kind, color, char: pieceChar(kind, color) });
        c += 1;
      }
    }
  }
  return { pieces, side: sideField === "b" ? "black" : "red" };
}

/** ICCS -> 棋盘索引 (row*9+col) */
export function sqFromIccs(iccs: string): [number, number] {
  const fromCol = iccs.charCodeAt(0) - 97;
  const fromRow = 9 - parseInt(iccs[1], 10);
  const toCol = iccs.charCodeAt(2) - 97;
  const toRow = 9 - parseInt(iccs[3], 10);
  return [fromRow * 9 + fromCol, toRow * 9 + toCol];
}

export const LEVELS = [
  { name: "入门", desc: "适合刚学会规则的朋友", elo: 800 },
  { name: "初级", desc: "偶尔会犯小错误", elo: 1100 },
  { name: "中级", desc: "思路比较稳健", elo: 1400 },
  { name: "高级", desc: "善于抓住机会", elo: 1700 },
  { name: "大师", desc: "计算深入，步步紧逼", elo: 2000 },
  { name: "特级大师", desc: "全力以赴，小心应对", elo: 2300 },
];

export const BADGE_LABEL: Record<string, string> = {
  best: "最佳",
  good: "佳着",
  ok: "一般",
  inaccuracy: "欠准",
  mistake: "失误",
  blunder: "漏着",
};

export const BADGE_COLOR: Record<string, string> = {
  best: "#4caf7d",
  good: "#8bc34a",
  ok: "#9aa4b5",
  inaccuracy: "#e6b455",
  mistake: "#e07f3f",
  blunder: "#e05656",
};

/** 红方视角分数 -> 显示文本 */
export function evalText(cp: number): string {
  if (cp > 29000) return "杀";
  if (cp < -29000) return "杀";
  const v = cp / 100;
  return (v > 0 ? "+" : "") + v.toFixed(1);
}
