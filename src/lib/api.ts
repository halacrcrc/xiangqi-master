import { invoke } from "@tauri-apps/api/core";
import type { PuzzleInfo, ReviewItem, Snapshot } from "./types";

export interface HintResult {
  iccs: string;
  notation: string;
  from: number;
  to: number;
  score: number;
  scoreLabel: string;
}

export interface PuzzleMoveResult {
  snapshot: Snapshot;
  accepted: boolean;
  feedback: string;
  solved: boolean;
  reply?: string | null;
}

export interface PuzzleHintResult {
  kind: "move" | "text";
  iccs?: string | null;
  notation?: string | null;
  text: string;
}

export const api = {
  gameNew: (difficulty: number, playerSide: "red" | "black", instantFeedback: boolean) =>
    invoke<Snapshot>("game_new", { difficulty, playerSide, instantFeedback }),
  gameNewPvp: (instantFeedback: boolean) => invoke<Snapshot>("game_new_pvp", { instantFeedback }),
  studyStart: (fen: string) => invoke<Snapshot>("game_study_start", { fen }),
  legalMoves: (from: number) => invoke<number[]>("legal_moves", { from }),
  playerMove: (from: number, to: number) => invoke<Snapshot>("player_move", { from, to }),
  aiMove: () => invoke<Snapshot>("ai_move"),
  undo: () => invoke<Snapshot>("undo"),
  hint: () => invoke<HintResult>("hint"),
  resign: () => invoke<Snapshot>("resign"),
  timeLoss: (side: "red" | "black") => invoke<Snapshot>("time_loss", { side }),
  analyzeGame: (depth: number, timeMs: number) => invoke<ReviewItem[]>("analyze_game", { depth, timeMs }),
  puzzlesList: () => invoke<PuzzleInfo[]>("puzzles_list"),
  puzzleStart: (id: number) => invoke<Snapshot>("puzzle_start", { id }),
  puzzleMove: (from: number, to: number) => invoke<PuzzleMoveResult>("puzzle_move", { from, to }),
  puzzleHint: () => invoke<PuzzleHintResult>("puzzle_hint"),
  sessionSnapshot: () => invoke<Snapshot>("session_snapshot"),
};
