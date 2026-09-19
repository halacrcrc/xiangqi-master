//! Tauri 应用层：对局会话、AI 对弈、解谜、复盘分析等命令。

use engine::analysis::{self, ReviewItem};
use engine::board::*;
use engine::search::{Searcher, LEVELS, MATE};
use engine::{notation, openings, puzzles};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, State};

pub mod engine;

pub struct AppState(pub Mutex<Session>);

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Ai,
    Pvp,
    Puzzle,
    Practice,
    Study,
}

#[derive(Clone, Copy, PartialEq)]
enum GameStatus {
    Playing,
    RedWin,
    BlackWin,
    Draw,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MoveRecord {
    ply: usize,
    iccs: String,
    notation: String,
    is_red: bool,
    capture: bool,
    check: bool,
    mate: bool,
    badge: Option<String>,
    best_iccs: Option<String>,
    loss: Option<i32>,
    fen_after: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PuzzleView {
    id: u32,
    name: String,
    theme: String,
    hint: String,
    difficulty: u8,
    is_mate: bool,
    mate_in: u8,
    solved: bool,
    wrong_count: u32,
    objective: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    fen: String,
    side: String,
    records: Vec<MoveRecord>,
    last_move: Option<(u8, u8)>,
    in_check: bool,
    status: String,
    reason: String,
    eval_cp: i32,
    opening: Option<String>,
    mode: String,
    difficulty: usize,
    player_side: String,
    can_undo: bool,
    puzzle: Option<PuzzleView>,
    start_fen: String,
}

struct PuzzleState {
    id: u32,
    mate: bool,
    mate_in: u8,
    solved: bool,
    wrong_count: u32,
}

struct Session {
    board: Board,
    mode: Mode,
    difficulty: usize,
    player_side: u8,
    records: Vec<MoveRecord>,
    status: GameStatus,
    reason: String,
    start_fen: String,
    eval_cp: i32,
    instant_feedback: bool,
    serial: u64,
    puzzle: Option<PuzzleState>,
}

impl Session {
    fn new() -> Self {
        Session {
            board: Board::initial(),
            mode: Mode::Ai,
            difficulty: 2,
            player_side: RED,
            records: Vec::new(),
            status: GameStatus::Playing,
            reason: String::new(),
            start_fen: Board::initial().to_fen(),
            eval_cp: 0,
            instant_feedback: true,
            serial: 0,
            puzzle: None,
        }
    }

    fn bump(&mut self) {
        self.serial += 1;
    }

    /// 落子并记录；返回是否结束。假设 mv 已验证合法。
    fn apply_move(&mut self, mv: Move) {
        let nota = notation::move_notation(&self.board, mv);
        let capture = mv.captured != EMPTY;
        let mover = self.board.side;
        let mover_red = mover == RED;
        self.board.make_move(mv);
        let opp = self.board.side;
        let check = self.board.in_check(opp);
        let no_moves = self.board.no_moves();
        let mate = no_moves; // 无子可动即终局（将死或困毙）
        let fen_after = self.board.to_fen();

        if no_moves {
            self.status = if mover_red { GameStatus::RedWin } else { GameStatus::BlackWin };
            self.reason = if check { "绝杀".to_string() } else { "困毙".to_string() };
        } else if self.board.repetition_count() >= 3 {
            self.status = GameStatus::Draw;
            self.reason = "三次重复局面".to_string();
        } else if self.board.insufficient_material() {
            self.status = GameStatus::Draw;
            self.reason = "双方子力不足".to_string();
        }

        self.records.push(MoveRecord {
            ply: self.records.len(),
            iccs: mv.iccs(),
            notation: nota,
            is_red: mover_red,
            capture,
            check,
            mate,
            badge: None,
            best_iccs: None,
            loss: None,
            fen_after,
        });
        self.bump();
    }

    fn move_seq(&self) -> Vec<String> {
        self.records.iter().map(|r| r.iccs.clone()).collect()
    }

    fn is_player_turn(&self) -> bool {
        match self.mode {
            Mode::Ai | Mode::Puzzle | Mode::Practice => self.board.side == self.player_side,
            Mode::Pvp | Mode::Study => true,
        }
    }
}

fn side_str(c: u8) -> &'static str {
    if c == RED {
        "red"
    } else {
        "black"
    }
}

fn snapshot(s: &Session) -> Snapshot {
    let last_move = s.records.last().and_then(|r| Move::from_iccs(&r.iccs)).map(|m| (m.from, m.to));
    let status = match s.status {
        GameStatus::Playing => "playing",
        GameStatus::RedWin => "red_win",
        GameStatus::BlackWin => "black_win",
        GameStatus::Draw => "draw",
    };
    let mode = match s.mode {
        Mode::Ai => "ai",
        Mode::Pvp => "pvp",
        Mode::Puzzle => "puzzle",
        Mode::Practice => "practice",
        Mode::Study => "study",
    };
    let opening = if s.start_fen == Board::initial().to_fen() {
        openings::identify(&s.move_seq()).map(|n| n.to_string())
    } else {
        None
    };
    let puzzle = s.puzzle.as_ref().and_then(|p| puzzles::puzzle_by_id(p.id)).map(|def| {
        let st = s.puzzle.as_ref().unwrap();
        let objective = match def.kind {
            puzzles::PuzzleKind::Mate { mate_in } => format!("{mate_in} 回合内将死黑方"),
            puzzles::PuzzleKind::Practice => {
                if s.player_side == RED {
                    "执红取胜（对方认负即达成）".to_string()
                } else {
                    "执黑守和即为成功".to_string()
                }
            }
        };
        PuzzleView {
            id: def.id,
            name: def.name.to_string(),
            theme: def.theme.to_string(),
            hint: def.hint.to_string(),
            difficulty: def.difficulty,
            is_mate: st.mate,
            mate_in: st.mate_in,
            solved: st.solved,
            wrong_count: st.wrong_count,
            objective,
        }
    });
    Snapshot {
        fen: s.board.to_fen(),
        side: side_str(s.board.side).to_string(),
        records: s.records.clone(),
        last_move,
        in_check: s.board.in_check(s.board.side),
        status: status.to_string(),
        reason: s.reason.clone(),
        eval_cp: s.eval_cp,
        opening,
        mode: mode.to_string(),
        difficulty: s.difficulty,
        player_side: match s.mode {
            Mode::Pvp | Mode::Study => "both".to_string(),
            _ => side_str(s.player_side).to_string(),
        },
        can_undo: !s.records.is_empty() && s.status == GameStatus::Playing,
        puzzle,
        start_fen: s.start_fen.clone(),
    }
}

fn quick_eval_red(board: &Board, depth: u8) -> i32 {
    let mut s = Searcher::new();
    let r = s.search(board, engine::search::SearchLimits::quick(depth));
    if board.side == RED {
        r.score
    } else {
        -r.score
    }
}

#[tauri::command]
fn game_new(state: State<'_, AppState>, difficulty: usize, player_side: String, instant_feedback: bool) -> Snapshot {
    let mut s = state.0.lock().unwrap();
    let diff = difficulty.min(LEVELS.len() - 1);
    let side = if player_side == "black" { BLACK } else { RED };
    let board = Board::initial();
    s.start_fen = board.to_fen();
    s.board = board;
    s.mode = Mode::Ai;
    s.difficulty = diff;
    s.player_side = side;
    s.instant_feedback = instant_feedback;
    s.records.clear();
    s.status = GameStatus::Playing;
    s.reason.clear();
    s.puzzle = None;
    s.eval_cp = 0;
    s.bump();
    snapshot(&s)
}

/// 研究模式：从任意 FEN 开始，双方棋子均由玩家自由移动（用于教程演局）
#[tauri::command]
fn game_study_start(state: State<'_, AppState>, fen: String) -> Result<Snapshot, String> {
    let mut s = state.0.lock().unwrap();
    let board = Board::from_fen(&fen).map_err(|e| format!("局面错误: {e}"))?;
    s.start_fen = board.to_fen();
    s.board = board;
    s.mode = Mode::Study;
    s.player_side = RED;
    s.records.clear();
    s.status = GameStatus::Playing;
    s.reason.clear();
    s.puzzle = None;
    s.eval_cp = quick_eval_red(&s.board, 3);
    s.bump();
    Ok(snapshot(&s))
}

#[tauri::command]
fn game_new_pvp(state: State<'_, AppState>, instant_feedback: bool) -> Snapshot {    let mut s = state.0.lock().unwrap();
    let board = Board::initial();
    s.start_fen = board.to_fen();
    s.board = board;
    s.mode = Mode::Pvp;
    s.player_side = RED;
    s.instant_feedback = instant_feedback;
    s.records.clear();
    s.status = GameStatus::Playing;
    s.reason.clear();
    s.puzzle = None;
    s.eval_cp = 0;
    s.bump();
    snapshot(&s)
}

#[tauri::command]
fn legal_moves(state: State<'_, AppState>, from: u8) -> Vec<u8> {
    let s = state.0.lock().unwrap();
    if from > 89 {
        return vec![];
    }
    s.board
        .legal_moves()
        .into_iter()
        .filter(|m| m.from == from)
        .map(|m| m.to)
        .collect()
}

/// 玩家落子（人机/双人/残局训练通用）。异步执行以免阻塞主线程。
#[tauri::command]
async fn player_move(state: State<'_, AppState>, from: u8, to: u8) -> Result<Snapshot, String> {
    // 1) 验证并落子
    let (applied, serial, judge_ctx) = {
        let mut s = state.0.lock().unwrap();
        if s.status != GameStatus::Playing {
            return Err("对局已结束".into());
        }
        if !s.is_player_turn() {
            return Err("还没轮到你走棋".into());
        }
        if from > 89 || to > 89 {
            return Err("坐标非法".into());
        }
        let mv = Move::new(from, to, s.board.piece_at(to));
        let legal = s.board.legal_moves();
        if !legal.iter().any(|m| m.from == from && m.to == to) {
            return Err("该走法不合法".into());
        }
        let board_before = s.board.clone();
        let want_judge = s.instant_feedback && (s.mode == Mode::Pvp || (s.mode == Mode::Ai && s.board.side == s.player_side));
        s.apply_move(mv);
        // 更新评估条
        if s.status == GameStatus::Playing {
            let b = s.board.clone();
            let depth = 3u8;
            s.eval_cp = quick_eval_red(&b, depth);
        } else {
            s.eval_cp = if matches!(s.status, GameStatus::RedWin) { MATE } else if matches!(s.status, GameStatus::BlackWin) { -MATE } else { 0 };
        }
        let ctx = if want_judge && s.status == GameStatus::Playing {
            Some((board_before, mv))
        } else {
            None
        };
        (true, s.serial, ctx)
    };
    let _ = applied;

    // 2) 即时评价（后台线程搜索）
    if let Some((board_before, mv)) = judge_ctx {
        let judge = tauri::async_runtime::spawn_blocking(move || analysis::judge_move(&board_before, mv, 4, 260)).await.map_err(|e| e.to_string())?;
        let mut s = state.0.lock().unwrap();
        if s.serial == serial {
            if let Some(last) = s.records.last_mut() {
                last.badge = Some(judge.classification.clone());
                last.best_iccs = judge.best_iccs.clone();
                last.loss = Some(judge.loss);
            }
        }
        return Ok(snapshot(&s));
    }
    let s = state.0.lock().unwrap();
    Ok(snapshot(&s))
}

/// AI 走棋（人机模式的对手 / 残局训练的防守方）
#[tauri::command]
async fn ai_move(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let (board, serial, difficulty, mode, start_fen, seq, puzzle_mate) = {
        let s = state.0.lock().unwrap();
        if s.status != GameStatus::Playing {
            return Ok(snapshot(&s));
        }
        let difficulty = if s.mode == Mode::Practice { 3usize } else { s.difficulty };
        (
            s.board.clone(),
            s.serial,
            difficulty,
            s.mode,
            s.start_fen.clone(),
            s.move_seq(),
            s.puzzle.as_ref().map(|p| (p.mate, p.mate_in)),
        )
    };

    let is_initial = start_fen == Board::initial().to_fen() && !matches!(mode, Mode::Puzzle | Mode::Practice);
    let book = if is_initial && seq.len() < 8 {
        openings::book_replies(&seq)
    } else {
        vec![]
    };

    let chosen: Option<Move> = if let Some(next) = pick_book(&book, board.hash()) {
        Move::from_iccs(next)
    } else {
        let limits = LEVELS[difficulty.min(LEVELS.len() - 1)].limits;
        let think = tauri::async_runtime::spawn_blocking(move || {
            let mut s = Searcher::new();
            s.search(&board, limits).best
        })
        .await
        .map_err(|e| e.to_string())?;
        think
    };

    let mut s = state.0.lock().unwrap();
    if s.serial != serial || s.status != GameStatus::Playing {
        return Ok(snapshot(&s));
    }
    if let Some(mv) = chosen {
        if s.board.legal_moves().iter().any(|m| m.from == mv.from && m.to == mv.to) {
            // 记录评估：AI 思考分数没有返回时用快速评估
            s.apply_move(mv);
            if s.status == GameStatus::Playing {
                if let Some((is_mate, _)) = puzzle_mate {
                    let _ = is_mate;
                }
                let b = s.board.clone();
                s.eval_cp = quick_eval_red(&b, 3);
            } else {
                s.eval_cp = if matches!(s.status, GameStatus::RedWin) { MATE } else if matches!(s.status, GameStatus::BlackWin) { -MATE } else { 0 };
            }
        }
    }
    Ok(snapshot(&s))
}

fn pick_book<'a>(book: &[&'a str], seed: u64) -> Option<&'a str> {
    if book.is_empty() {
        return None;
    }
    let mut x = seed.wrapping_mul(0x2545F4914F6CDD1D).wrapping_add(0x9E3779B9);
    x ^= x >> 33;
    let idx = (x as usize) % book.len();
    Some(book[idx])
}

#[tauri::command]
fn undo(state: State<'_, AppState>) -> Snapshot {
    let mut s = state.0.lock().unwrap();
    if s.records.is_empty() {
        return snapshot(&s);
    }
    if s.status != GameStatus::Playing {
        // 终局后悔棋：回到最后一手前
        s.status = GameStatus::Playing;
        s.reason.clear();
    }
    match s.mode {
        Mode::Pvp | Mode::Study => {
            let n = s.board.undo_len().saturating_sub(1);
            s.board.undo_to(n);
            s.records.pop();
        }
        _ => {
            let n = s.board.undo_len().saturating_sub(2);
            s.board.undo_to(n);
            s.records.pop();
            s.records.pop();
        }
    }
    // 若人机模式玩家执黑且此刻轮红（AI 先手被撤销过多），再回退一手由玩家先走的情况无需处理
    let last_rec = s.records.last().cloned();
    s.eval_cp = quick_eval_red(&s.board, 3);
    s.bump();
    let _ = last_rec;
    snapshot(&s)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HintResult {
    iccs: String,
    notation: String,
    from: u8,
    to: u8,
    score: i32,
    score_label: String,
}

#[tauri::command]
async fn hint(state: State<'_, AppState>) -> Result<HintResult, String> {
    let (board, in_puzzle) = {
        let s = state.0.lock().unwrap();
        if s.status != GameStatus::Playing {
            return Err("对局已结束".into());
        }
        (s.board.clone(), s.puzzle.is_some())
    };
    let limits = if in_puzzle {
        engine::search::SearchLimits { max_depth: 8, max_time_ms: 1500, pool: 1, window: 0, pure: true }
    } else {
        engine::search::SearchLimits { max_depth: 7, max_time_ms: 1200, pool: 1, window: 0, pure: false }
    };
    let r = tauri::async_runtime::spawn_blocking(move || {
        let mut s = Searcher::new();
        s.search(&board, limits)
    })
    .await
    .map_err(|e| e.to_string())?;
    let mv = r.best.ok_or("无棋可走")?;
    let nota = {
        let s = state.0.lock().unwrap();
        notation::move_notation(&s.board, mv)
    };
    let score_label = if r.score > MATE - 100 {
        "红方绝杀在即".to_string()
    } else if r.score < -(MATE - 100) {
        "黑方绝杀在即".to_string()
    } else {
        let cp = r.score;
        if cp > 300 {
            "红方大优".to_string()
        } else if cp > 60 {
            "红方略优".to_string()
        } else if cp > -60 {
            "局面均衡".to_string()
        } else if cp > -300 {
            "黑方略优".to_string()
        } else {
            "黑方大优".to_string()
        }
    };
    Ok(HintResult { iccs: mv.iccs(), notation: nota, from: mv.from, to: mv.to, score: r.score, score_label })
}

#[tauri::command]
fn resign(state: State<'_, AppState>) -> Snapshot {
    let mut s = state.0.lock().unwrap();
    if s.status == GameStatus::Playing {
        let loser = match s.mode {
            Mode::Pvp => s.board.side,
            _ => s.player_side,
        };
        s.status = if loser == RED { GameStatus::BlackWin } else { GameStatus::RedWin };
        s.reason = "认输".to_string();
        s.eval_cp = if matches!(s.status, GameStatus::RedWin) { MATE } else { -MATE };
        s.bump();
    }
    snapshot(&s)
}

#[tauri::command]
fn time_loss(state: State<'_, AppState>, side: String) -> Snapshot {
    let mut s = state.0.lock().unwrap();
    if s.status == GameStatus::Playing {
        let loser = if side == "red" { RED } else { BLACK };
        s.status = if loser == RED { GameStatus::BlackWin } else { GameStatus::RedWin };
        s.reason = "超时".to_string();
        s.eval_cp = if matches!(s.status, GameStatus::RedWin) { MATE } else { -MATE };
        s.bump();
    }
    snapshot(&s)
}

#[tauri::command]
async fn analyze_game(app: tauri::AppHandle, state: State<'_, AppState>, depth: u8, time_ms: u64) -> Result<Vec<ReviewItem>, String> {
    let (start_fen, meta): (String, Vec<(String, String, bool, bool)>) = {
        let s = state.0.lock().unwrap();
        (
            s.start_fen.clone(),
            s.records.iter().map(|r| (r.iccs.clone(), r.notation.clone(), r.check, r.capture)).collect(),
        )
    };
    let app2 = app.clone();
    let items = tauri::async_runtime::spawn_blocking(move || {
        analysis::review_game(&start_fen, &meta, depth, time_ms, |done, total| {
            let _ = app2.emit("review-progress", serde_json::json!({ "done": done, "total": total }));
        })
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(items)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PuzzleInfo {
    id: u32,
    name: String,
    theme: String,
    difficulty: u8,
    is_mate: bool,
    mate_in: u8,
}

#[tauri::command]
fn puzzles_list() -> Vec<PuzzleInfo> {
    puzzles::PUZZLES
        .iter()
        .map(|p| {
            let is_mate = matches!(p.kind, puzzles::PuzzleKind::Mate { .. });
            let mate_in = match p.kind {
                puzzles::PuzzleKind::Mate { mate_in } => mate_in,
                _ => 0,
            };
            PuzzleInfo { id: p.id, name: p.name.to_string(), theme: p.theme.to_string(), difficulty: p.difficulty, is_mate, mate_in }
        })
        .collect()
}

#[tauri::command]
fn puzzle_start(state: State<'_, AppState>, id: u32) -> Result<Snapshot, String> {
    let p = puzzles::puzzle_by_id(id).ok_or("谜题不存在")?;
    let mut s = state.0.lock().unwrap();
    let board = Board::from_fen(p.fen).map_err(|e| format!("谜题局面错误: {e}"))?;
    let (mate, mate_in) = match p.kind {
        puzzles::PuzzleKind::Mate { mate_in } => (true, mate_in),
        puzzles::PuzzleKind::Practice => (false, 0),
    };
    s.start_fen = board.to_fen();
    s.board = board;
    s.mode = if mate { Mode::Puzzle } else { Mode::Practice };
    s.player_side = s.board.side; // 谜题由局面指定先手
    s.records.clear();
    s.status = GameStatus::Playing;
    s.reason.clear();
    s.puzzle = Some(PuzzleState { id, mate, mate_in, solved: false, wrong_count: 0 });
    s.eval_cp = quick_eval_red(&s.board, 4);
    s.bump();
    Ok(snapshot(&s))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PuzzleMoveResult {
    snapshot: Snapshot,
    accepted: bool,
    feedback: String,
    solved: bool,
    /// 引擎防守应着（ICCS），用于展示
    reply: Option<String>,
}

#[tauri::command]
async fn puzzle_move(state: State<'_, AppState>, from: u8, to: u8) -> Result<PuzzleMoveResult, String> {
    // 验证合法走法
    let (board_before, mv, mate, mate_in, plies_done, solved, wrong) = {
        let s = state.0.lock().unwrap();
        if s.status != GameStatus::Playing {
            return Err("本题已结束，请开始下一题".into());
        }
        let pz = s.puzzle.as_ref().ok_or("当前不在解谜模式")?;
        if pz.solved {
            return Err("已解答".into());
        }
        if from > 89 || to > 89 {
            return Err("坐标非法".into());
        }
        let mv = Move::new(from, to, s.board.piece_at(to));
        if !s.board.legal_moves().iter().any(|m| m.from == from && m.to == to) {
            return Err("该走法不合法".into());
        }
        (s.board.clone(), mv, pz.mate, pz.mate_in, s.records.len() as u32, pz.solved, pz.wrong_count)
    };
    let _ = solved;

    let mut work = board_before.clone();
    work.make_move(mv);
    let black_dead = work.no_moves();

    if !mate {
        // 残局训练：直接接受走法，前端再触发 ai_move 防守
        let mut s = state.0.lock().unwrap();
        s.apply_move(mv);
        if s.status == GameStatus::Playing {
            let b = s.board.clone();
            s.eval_cp = quick_eval_red(&b, 3);
        } else {
            // 玩家将死对方 → 完成
            let player_won = (s.status == GameStatus::RedWin) == (s.player_side == RED);
            if player_won {
                if let Some(pz) = s.puzzle.as_mut() {
                    pz.solved = true;
                }
            }
        }
        let solved_now = s.puzzle.as_ref().map(|p| p.solved).unwrap_or(false);
        return Ok(PuzzleMoveResult { snapshot: snapshot(&s), accepted: true, feedback: String::new(), solved: solved_now, reply: None });
    }

    // 杀局解谜：验证仍然保持强制杀
    let player_moves_used = plies_done / 2 + 1;
    let remaining_red_moves = (mate_in as i32 - player_moves_used as i32).max(0);
    let verify_plies = (remaining_red_moves * 2).min(10) as u8;

    let (still_mating, defense) = if black_dead {
        (true, None)
    } else {
        let b = work.clone();
        let v = verify_plies;
        let res = tauri::async_runtime::spawn_blocking(move || {
            let mut s = Searcher::new();
            let limits = engine::search::SearchLimits { max_depth: v.max(2), max_time_ms: 2500, pool: 1, window: 0, pure: true };
            let r = s.search(&b, limits);
            let still = r.score <= -(MATE - v as i32 - 2);
            let best = r.best;
            (still, best)
        })
        .await
        .map_err(|e| e.to_string())?;
        (res.0, res.1)
    };

    if !still_mating {
        let mut s = state.0.lock().unwrap();
        if let Some(pz) = s.puzzle.as_mut() {
            pz.wrong_count += 1;
        }
        let feedback = format!("走错啦！这步之后无法在 {} 回合内成杀，再想想。", mate_in);
        return Ok(PuzzleMoveResult { snapshot: snapshot(&s), accepted: false, feedback, solved: false, reply: None });
    }

    // 接受走法；若未终局，引擎给出最强防守
    let mut s = state.0.lock().unwrap();
    s.apply_move(mv);
    if black_dead {
        if let Some(pz) = s.puzzle.as_mut() {
            pz.solved = true;
        }
        s.eval_cp = MATE;
        let feedback = "漂亮！绝杀达成！".to_string();
        return Ok(PuzzleMoveResult { snapshot: snapshot(&s), accepted: true, feedback, solved: true, reply: None });
    }
    let reply_iccs = defense.map(|d| d.iccs());
    if let Some(d) = defense {
        if s.board.legal_moves().iter().any(|m| m.from == d.from && m.to == d.to) {
            s.apply_move(d);
        }
    }
    if s.status == GameStatus::Playing {
        let b = s.board.clone();
        s.eval_cp = quick_eval_red(&b, 3);
    }
    let feedback = "好棋！继续保持强制杀。".to_string();
    Ok(PuzzleMoveResult { snapshot: snapshot(&s), accepted: true, feedback, solved: false, reply: reply_iccs })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PuzzleHintResult {
    kind: String, // "move" | "text"
    iccs: Option<String>,
    notation: Option<String>,
    text: String,
}

#[tauri::command]
async fn puzzle_hint(state: State<'_, AppState>) -> Result<PuzzleHintResult, String> {
    let (board, puzzle_id, plies_done, text_hint) = {
        let s = state.0.lock().unwrap();
        let pz = s.puzzle.as_ref().ok_or("当前不在解谜模式")?;
        let p = puzzles::puzzle_by_id(pz.id).ok_or("谜题不存在")?;
        (s.board.clone(), pz.id, s.records.len(), p.hint.to_string())
    };
    let p = puzzles::puzzle_by_id(puzzle_id).unwrap();
    // 主变还在时优先给主变下一步
    let solution_next = p.solution.get(plies_done).map(|s| s.to_string());
    let best = {
        let b = board.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let mut s = Searcher::new();
            let limits = engine::search::SearchLimits { max_depth: 6, max_time_ms: 1200, pool: 1, window: 0, pure: true };
            s.search(&b, limits)
        })
        .await
        .map_err(|e| e.to_string())?
    };

    if let Some(mv) = best.best {
        let nota = notation::move_notation(&board, mv);
        Ok(PuzzleHintResult {
            kind: "move".into(),
            iccs: Some(solution_next.unwrap_or_else(|| mv.iccs())),
            notation: Some(nota),
            text: text_hint,
        })
    } else {
        Ok(PuzzleHintResult { kind: "text".into(), iccs: None, notation: None, text: text_hint })
    }
}

#[tauri::command]
fn session_snapshot(state: State<'_, AppState>) -> Snapshot {
    let s = state.0.lock().unwrap();
    snapshot(&s)
}

#[tauri::command]
fn levels_info() -> serde_json::Value {
    serde_json::json!(LEVELS
        .iter()
        .enumerate()
        .map(|(i, l)| serde_json::json!({
            "index": i,
            "name": l.name,
            "desc": l.desc,
            "elo": l.elo,
        }))
        .collect::<Vec<_>>())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(Session::new())))
        .invoke_handler(tauri::generate_handler![
            game_new,
            game_new_pvp,
            game_study_start,
            legal_moves,
            player_move,
            ai_move,
            undo,
            hint,
            resign,
            time_loss,
            analyze_game,
            puzzles_list,
            puzzle_start,
            puzzle_move,
            puzzle_hint,
            session_snapshot,
            levels_info,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
