import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../lib/api";
import { sfx, setSoundEnabled } from "../lib/sound";
import { loadSettings, loadStats, saveSettings, saveStats, updateRating, type Settings, type Stats } from "../lib/storage";
import { LEVELS, type Snapshot } from "../lib/types";

export interface Toast {
  text: string;
  kind: "info" | "success" | "error";
  id: number;
}

export interface Clock {
  total: number; // 初始毫秒
  red: number;
  black: number;
}

let toastSeq = 1;

export function useGame() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [thinking, setThinking] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);
  const [legalTargets, setLegalTargets] = useState<number[]>([]);
  const [viewIndex, setViewIndex] = useState<number | null>(null); // null=实时
  const [hintMove, setHintMove] = useState<{ from: number; to: number; notation: string } | null>(null);
  const [toast, setToast] = useState<Toast | null>(null);
  const [settings, setSettings] = useState<Settings>(() => loadSettings());
  const [stats, setStats] = useState<Stats>(() => loadStats());
  const [clock, setClock] = useState<Clock | null>(null);
  const [reviewArrows, setReviewArrows] = useState<{ from: number; to: number } | null>(null);
  const [gameOverSeen, setGameOverSeen] = useState<string | null>(null);
  const [puzzleBanner, setPuzzleBanner] = useState<string | null>(null);

  const prevRecordCount = useRef(0);
  const prevSnapRef = useRef<Snapshot | null>(null);
  const ratingApplied = useRef<string>("");

  useEffect(() => {
    setSoundEnabled(settings.sound);
    saveSettings(settings);
  }, [settings]);

  useEffect(() => {
    // 启动时取当前会话
    api.sessionSnapshot().then(setSnap).catch(() => {});
  }, []);

  const showToast = useCallback((text: string, kind: Toast["kind"] = "info") => {
    setToast({ text, kind, id: toastSeq++ });
    window.setTimeout(() => {
      setToast((t) => (t && t.id === toastSeq - 1 ? null : t));
    }, 2600);
  }, []);

  /** 应用新快照：音效 + 终局处理 */
  const applySnap = useCallback(
    (next: Snapshot, opts?: { puzzleFeedback?: string; accepted?: boolean }) => {
      const prev = prevSnapRef.current;
      // 音效
      if (next.records.length > prevRecordCount.current) {
        const last = next.records[next.records.length - 1];
        if (last.mate) {
          sfx.capture();
          sfx.win();
        } else if (last.check) {
          sfx.check();
        } else if (last.capture) {
          sfx.capture();
        } else {
          sfx.move();
        }
      }
      prevRecordCount.current = next.records.length;
      prevSnapRef.current = next;

      if (opts?.puzzleFeedback !== undefined) {
        setPuzzleBanner(opts.puzzleFeedback || null);
        if (opts.accepted === false) sfx.puzzleBad();
        else if (opts.puzzleFeedback) sfx.puzzleOk();
      }

      setSnap(next);
      setSelected(null);
      setLegalTargets([]);
      setHintMove(null);
      setViewIndex(null);
      setReviewArrows(null);

      // 终局：音效 + 棋力结算
      if (next.status !== "playing" && next.mode !== "puzzle") {
        const key = `${next.startFen}|${next.records.length}|${next.status}`;
        if (gameOverSeen !== key) {
          setGameOverSeen(key);
          if (next.status === "draw") sfx.draw();
          else if (next.mode === "ai") {
            const playerWon = (next.status === "red_win" && next.playerSide === "red") || (next.status === "black_win" && next.playerSide === "black");
            if (playerWon) sfx.win();
            else sfx.lose();
          }
          // AI 对局结算等级分
          if (next.mode === "ai" && ratingApplied.current !== key) {
            ratingApplied.current = key;
            const playerWon = (next.status === "red_win" && next.playerSide === "red") || (next.status === "black_win" && next.playerSide === "black");
            const draw = next.status === "draw";
            const score = draw ? 0.5 : playerWon ? 1 : 0;
            const aiElo = LEVELS[next.difficulty]?.elo ?? 1200;
            setStats((st) => {
              const { rating, delta } = updateRating(st.rating, aiElo, score);
              const nextStats: Stats = {
                ...st,
                rating,
                ratingHistory: [...st.ratingHistory, rating].slice(-60),
                games: [
                  ...st.games,
                  {
                    date: new Date().toISOString(),
                    mode: "ai" as const,
                    difficulty: next.difficulty,
                    result: draw ? ("draw" as const) : playerWon ? ("win" as const) : ("loss" as const),
                    playerColor: next.playerSide as "red" | "black",
                    moves: next.records.length,
                    ratingDelta: delta,
                  },
                ].slice(-200),
              };
              saveStats(nextStats);
              return nextStats;
            });
          }
        }
      }

      // 解谜完成记录
      if (next.puzzle?.solved) {
        setStats((st) => {
          const id = next.puzzle!.id;
          const current = st.puzzleSolved[id];
          if (current?.solved) return st;
          const nextStats: Stats = {
            ...st,
            puzzleSolved: { ...st.puzzleSolved, [id]: { solved: true, wrong: next.puzzle!.wrongCount, date: new Date().toISOString() } },
          };
          saveStats(nextStats);
          return nextStats;
        });
      }
    },
    [gameOverSeen]
  );

  // 首次快照记录
  useEffect(() => {
    if (snap && !prevSnapRef.current) {
      prevSnapRef.current = snap;
      prevRecordCount.current = snap.records.length;
    }
  }, [snap]);

  /** 落子（点击目标格） */
  const tryMove = useCallback(
    async (from: number, to: number) => {
      if (!snap || snap.status !== "playing") return;
      if (viewIndex !== null) return;
      const isPlayerTurn =
        snap.mode === "pvp" ? true : snap.side === snap.playerSide;
      if (!isPlayerTurn) return;
      setSelected(null);
      setLegalTargets([]);
      try {
        if (snap.mode === "puzzle") {
          const r = await api.puzzleMove(from, to);
          applySnap(r.snapshot, { puzzleFeedback: r.feedback, accepted: r.accepted });
        } else {
          const next = await api.playerMove(from, to);
          applySnap(next);
        }
      } catch (e) {
        sfx.illegal();
        showToast(String(e), "error");
      }
    },
    [snap, viewIndex, applySnap, showToast]
  );

  /** 选择棋子：加载合法目标 */
  const select = useCallback(
    async (sq: number) => {
      if (!snap || snap.status !== "playing" || viewIndex !== null) return;
      if (snap.mode !== "pvp" && snap.side !== snap.playerSide) return;
      if (selected === sq) {
        setSelected(null);
        setLegalTargets([]);
        return;
      }
      const piece = snap.fen;
      // 解析 fen 检查该格是否为己方棋子
      const { pieces, side } = parseQuick(piece);
      const p = pieces.find((x) => x.sq === sq);
      if (!p || p.color !== side) {
        // 点击非当前方棋子：可能是吃子目标
        if (selected !== null && legalTargets.includes(sq)) {
          void tryMove(selected, sq);
        } else {
          setSelected(null);
          setLegalTargets([]);
        }
        return;
      }
      setSelected(sq);
      setHintMove(null);
      sfx.select();
      try {
        const targets = await api.legalMoves(sq);
        setLegalTargets(targets);
      } catch {
        setLegalTargets([]);
      }
    },
    [snap, selected, legalTargets, viewIndex, tryMove]
  );

  const newGame = useCallback(
    async (difficulty: number, side: "red" | "black" | "random", clockMinutes: number | null) => {
      const playerSide: "red" | "black" =
        side === "random" ? (Math.random() < 0.5 ? "red" : "black") : side;
      const s = await api.gameNew(difficulty, playerSide, settings.instantFeedback);
      setGameOverSeen(null);
      ratingApplied.current = "";
      setPuzzleBanner(null);
      setClock(clockMinutes ? { total: clockMinutes * 60000, red: clockMinutes * 60000, black: clockMinutes * 60000 } : null);
      applySnap(s);
      showToast(`人机对局开始 · ${LEVELS[difficulty].name} · 你执${playerSide === "red" ? "红" : "黑"}`, "info");
    },
    [settings.instantFeedback, applySnap, showToast]
  );

  const newPvpGame = useCallback(
    async (clockMinutes: number | null) => {
      const s = await api.gameNewPvp(settings.instantFeedback);
      setGameOverSeen(null);
      ratingApplied.current = "";
      setPuzzleBanner(null);
      setClock(clockMinutes ? { total: clockMinutes * 60000, red: clockMinutes * 60000, black: clockMinutes * 60000 } : null);
      applySnap(s);
      showToast("双人对战开始 · 红先黑后", "info");
    },
    [settings.instantFeedback, applySnap, showToast]
  );

  const startPuzzle = useCallback(
    async (id: number) => {
      try {
        const s = await api.puzzleStart(id);
        setGameOverSeen(null);
        setPuzzleBanner(null);
        setClock(null);
        applySnap(s);
      } catch (e) {
        showToast(String(e), "error");
      }
    },
    [applySnap, showToast]
  );

  const undoMove = useCallback(async () => {
    if (!snap || !snap.canUndo) return;
    if (snap.mode === "puzzle") {
      // 杀局重来
      await startPuzzle(snap.puzzle!.id);
      showToast("本题重新开始", "info");
      return;
    }
    const s = await api.undo();
    setGameOverSeen(null);
    applySnap(s);
  }, [snap, applySnap, showToast, startPuzzle]);

  const requestHint = useCallback(async () => {
    if (!snap || snap.status !== "playing") return;
    try {
      if (snap.mode === "puzzle") {
        const h = await api.puzzleHint();
        if (h.kind === "move" && h.iccs && h.notation) {
          const [f, t] = iccsToSqs(h.iccs);
          setHintMove({ from: f, to: t, notation: h.notation });
          showToast(`提示：${h.notation}（${h.text}）`, "info");
        } else {
          showToast(`提示：${h.text}`, "info");
        }
      } else {
        const h = await api.hint();
        setHintMove({ from: h.from, to: h.to, notation: h.notation });
        showToast(`建议：${h.notation} · ${h.scoreLabel}`, "info");
      }
    } catch (e) {
      showToast(String(e), "error");
    }
  }, [snap, showToast]);

  const resignGame = useCallback(async () => {
    const s = await api.resign();
    applySnap(s);
  }, [applySnap]);

  // AI 行棋触发（人机 / 残局训练防守方）
  const aiBusy = useRef(false);
  useEffect(() => {
    if (!snap || snap.status !== "playing" || viewIndex !== null) return;
    if (snap.mode !== "ai" && snap.mode !== "practice") return;
    if (snap.side === snap.playerSide) return;
    if (aiBusy.current) return;
    aiBusy.current = true;
    setThinking(true);
    api
      .aiMove()
      .then((s) => applySnap(s))
      .catch(() => {})
      .finally(() => {
        aiBusy.current = false;
        setThinking(false);
      });
  }, [snap, viewIndex, applySnap]);

  // 计时器
  useEffect(() => {
    if (!clock || !snap || snap.status !== "playing") return;
    const timer = window.setInterval(() => {
      setClock((c) => {
        if (!c) return c;
        const key = snap.side === "red" ? "red" : "black";
        const nv = { ...c, [key]: Math.max(0, c[key] - 200) };
        return nv;
      });
    }, 200);
    return () => window.clearInterval(timer);
  }, [clock, snap]);

  useEffect(() => {
    if (clock && snap && snap.status === "playing") {
      if (clock.red <= 0) {
        api.timeLoss("red").then((s) => applySnap(s));
      } else if (clock.black <= 0) {
        api.timeLoss("black").then((s) => applySnap(s));
      }
    }
  }, [clock, snap, applySnap]);

  const flipBoard = snap ? snap.mode === "pvp" ? false : snap.playerSide === "black" && settings.flipWithSide ? true : false : false;

  return {
    snap,
    thinking,
    selected,
    legalTargets,
    viewIndex,
    setViewIndex,
    hintMove,
    setHintMove,
    reviewArrows,
    setReviewArrows,
    toast,
    showToast,
    settings,
    setSettings,
    stats,
    setStats,
    clock,
    select,
    tryMove,
    newGame,
    newPvpGame,
    startPuzzle,
    undoMove,
    requestHint,
    resignGame,
    applySnap,
    flipBoard,
    puzzleBanner,
    setPuzzleBanner,
  };
}

function parseQuick(fen: string) {
  // 轻量解析：仅取 side
  const side = fen.split(" ")[1] === "b" ? "black" : "red";
  const pieces: { sq: number; color: "red" | "black" }[] = [];
  const rows = fen.split(" ")[0].split("/");
  for (let r = 0; r < rows.length; r++) {
    let c = 0;
    for (const ch of rows[r]) {
      if (/\d/.test(ch)) c += parseInt(ch, 10);
      else {
        pieces.push({ sq: r * 9 + c, color: ch === ch.toLowerCase() ? "black" : "red" });
        c++;
      }
    }
  }
  return { pieces, side };
}

function iccsToSqs(iccs: string): [number, number] {
  const f = (9 - parseInt(iccs[1], 10)) * 9 + (iccs.charCodeAt(0) - 97);
  const t = (9 - parseInt(iccs[3], 10)) * 9 + (iccs.charCodeAt(2) - 97);
  return [f, t];
}
