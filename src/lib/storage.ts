/** localStorage 持久化：设置、棋力统计、解谜进度 */

export interface Settings {
  theme: "dark" | "wood" | "jade" | "ink" | "dusk" | "ocean" | "amber";
  sound: boolean;
  showLegal: boolean;
  instantFeedback: boolean;
  flipWithSide: boolean;
}

export interface GameRecord {
  date: string;
  mode: "ai" | "pvp";
  difficulty: number;
  result: "win" | "loss" | "draw";
  playerColor: "red" | "black";
  moves: number;
  ratingDelta?: number;
}

export interface Stats {
  rating: number;
  ratingHistory: number[];
  games: GameRecord[];
  puzzleSolved: Record<number, { solved: boolean; wrong: number; date: string }>;
}

const SETTINGS_KEY = "xq_settings_v1";
const STATS_KEY = "xq_stats_v1";

export const defaultSettings: Settings = {
  theme: "dark",
  sound: true,
  showLegal: true,
  instantFeedback: true,
  flipWithSide: true,
};

export function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (raw) return { ...defaultSettings, ...JSON.parse(raw) };
  } catch {
    /* ignore */
  }
  return { ...defaultSettings };
}

export function saveSettings(s: Settings) {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(s));
  } catch {
    /* ignore */
  }
}

export function loadStats(): Stats {
  try {
    const raw = localStorage.getItem(STATS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return { rating: 1200, ratingHistory: [1200], games: [], puzzleSolved: {}, ...parsed };
    }
  } catch {
    /* ignore */
  }
  return { rating: 1200, ratingHistory: [1200], games: [], puzzleSolved: {} };
}

export function saveStats(s: Stats) {
  try {
    localStorage.setItem(STATS_KEY, JSON.stringify(s));
  } catch {
    /* ignore */
  }
}

/** Elo 更新，返回 (newRating, delta) */
export function updateRating(rating: number, opponent: number, score: number, k = 24): { rating: number; delta: number } {
  const expected = 1 / (1 + Math.pow(10, (opponent - rating) / 400));
  const delta = Math.round(k * (score - expected));
  return { rating: rating + delta, delta };
}
