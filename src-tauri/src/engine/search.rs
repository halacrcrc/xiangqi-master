//! Alpha-Beta 搜索：迭代加深、置换表、杀手/历史启发、空着剪枝、静态搜索。
//! 分级难度通过深度/时限/候选走法池实现。

use super::board::*;
use super::eval::{evaluate, kind_value};
use std::time::Instant;

pub const MATE: i32 = 30_000;
pub const MATE_IN_MAX_PLY: i32 = MATE - 256;

#[derive(Clone, Copy, Debug)]
pub struct SearchLimits {
    pub max_depth: u8,
    pub max_time_ms: u64,
    /// 候选池大小：>1 时在前 pool 个近似最佳走法中加权随机
    pub pool: usize,
    /// 候选分数窗口（厘兵）
    pub window: i32,
    /// 纯净模式：禁用空着剪枝（用于杀局验证等需要精确证明的场合）
    pub pure: bool,
}

impl SearchLimits {
    pub fn quick(depth: u8) -> Self {
        SearchLimits { max_depth: depth, max_time_ms: 400, pool: 1, window: 0, pure: false }
    }
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub best: Option<Move>,
    pub score: i32,
    pub pv: Vec<Move>,
    /// 根节点各走法与分数（完成迭代的最后一层）
    pub root_scores: Vec<(Move, i32)>,
    pub depth_reached: u8,
    pub nodes: u64,
}

#[derive(Clone, Copy)]
struct TTEntry {
    key: u64,
    mv: u16,
    score: i16,
    depth: u8,
    bound: u8, // 0 精确 1 下界 2 上界
}

const TT_SIZE: usize = 1 << 21;
const MAX_PLY: usize = 80;

#[inline]
fn mv_to_u16(mv: Move) -> u16 {
    (mv.from as u16) | ((mv.to as u16) << 7)
}
#[inline]
fn u16_to_mv(v: u16) -> Move {
    Move::new((v & 0x7f) as u8, ((v >> 7) & 0x7f) as u8, EMPTY)
}

pub struct Searcher {
    tt: Vec<TTEntry>,
    killers: [[Move; 2]; MAX_PLY],
    history: [[[i32; 90]; 90]; 2],
    nodes: u64,
    deadline: Option<Instant>,
    stopped: bool,
    use_null: bool,
    game_hashes: Vec<u64>,
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Searcher {
    pub fn new() -> Self {
        Searcher {
            tt: vec![TTEntry { key: 0, mv: 0, score: 0, depth: 0, bound: 0 }; TT_SIZE],
            killers: [[Move::null(); 2]; MAX_PLY],
            history: [[[0; 90]; 90]; 2],
            nodes: 0,
            deadline: None,
            stopped: false,
            use_null: true,
            game_hashes: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tt.iter_mut().for_each(|e| *e = TTEntry { key: 0, mv: 0, score: 0, depth: 0, bound: 0 });
        self.killers = [[Move::null(); 2]; MAX_PLY];
        self.history = [[[0; 90]; 90]; 2];
        self.nodes = 0;
    }

    fn check_time(&mut self) {
        if let Some(d) = self.deadline {
            if (self.nodes & 2047) == 0 && Instant::now() >= d {
                self.stopped = true;
            }
        }
    }

    fn side_rel_eval(b: &Board) -> i32 {
        let e = evaluate(b);
        if b.side == RED {
            e
        } else {
            -e
        }
    }

    fn order_moves(&self, b: &Board, moves: &mut [(Move, i32)], tt_mv: Option<Move>, ply: usize) {
        let side_idx = (b.side >> 3) as usize;
        for (mv, sc) in moves.iter_mut() {
            *sc = 0;
            if let Some(t) = tt_mv {
                if mv.from == t.from && mv.to == t.to {
                    *sc = 1_000_000;
                    continue;
                }
            }
            if mv.captured != EMPTY {
                *sc = 100_000 + kind_value(piece_kind(mv.captured)) * 16 - kind_value(piece_kind(b.piece_at(mv.from)));
            } else if self.killers[ply][0].from == mv.from && self.killers[ply][0].to == mv.to {
                *sc = 90_000;
            } else if self.killers[ply][1].from == mv.from && self.killers[ply][1].to == mv.to {
                *sc = 80_000;
            } else {
                *sc = self.history[side_idx][mv.from as usize][mv.to as usize].min(70_000);
            }
        }
        moves.sort_by(|a, b2| b2.1.cmp(&a.1));
    }

    fn tt_probe(&self, hash: u64, depth: u8, alpha: i32, beta: i32, ply: usize) -> Option<(i32, Option<Move>)> {
        let e = self.tt[(hash as usize) & (TT_SIZE - 1)];
        if e.key != hash || e.depth < depth {
            return None;
        }
        let mv = if e.mv == 0 { None } else { Some(u16_to_mv(e.mv)) };
        let score = e.score as i32;
        if score.abs() > MATE_IN_MAX_PLY {
            return None; // 不使用/存储带杀分的条目，避免污染
        }
        match e.bound {
            0 => Some((score, mv)),
            1 if score >= beta => Some((score, mv)),
            2 if score <= alpha => Some((score, mv)),
            _ => None,
        }
    }

    fn tt_store(&mut self, hash: u64, mv: Move, score: i32, depth: u8, bound: u8) {
        if score.abs() > MATE_IN_MAX_PLY {
            return;
        }
        let idx = (hash as usize) & (TT_SIZE - 1);
        self.tt[idx] = TTEntry { key: hash, mv: mv_to_u16(mv), score: score as i16, depth, bound };
    }

    fn is_draw_by_repetition(&self, b: &Board, ply: usize) -> bool {
        if ply == 0 {
            return false;
        }
        let h = b.hash();
        // 搜索路径内一次重复即按和棋处理，避免长打循环
        let n = b.history_hashes.len();
        if n >= 2 {
            for i in (0..n.saturating_sub(1)).rev() {
                if b.history_hashes[i] == h {
                    return true;
                }
            }
        }
        let _ = self.game_hashes;
        false
    }

    fn has_non_king_material(b: &Board) -> bool {
        for sq in 0u8..90 {
            let p = b.piece_at(sq);
            if p != EMPTY && piece_color(p) == b.side && piece_kind(p) != KING {
                return true;
            }
        }
        false
    }

    pub fn negamax(&mut self, b: &mut Board, depth: i32, mut alpha: i32, beta: i32, ply: usize) -> i32 {
        self.nodes += 1;
        self.check_time();
        if self.stopped {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return Self::side_rel_eval(b);
        }
        if self.is_draw_by_repetition(b, ply) {
            return 0;
        }
        let in_check = b.in_check(b.side);
        let mut depth = depth;
        if in_check {
            depth += 1; // 将军延伸
        }
        if depth <= 0 {
            return self.quiescence(b, alpha, beta, ply);
        }

        if let Some((score, _)) = self.tt_probe(b.hash(), depth as u8, alpha, beta, ply) {
            return score;
        }

        // 空着剪枝
        if self.use_null && !in_check && depth >= 3 && beta < MATE_IN_MAX_PLY && Self::has_non_king_material(b) {
            b.make_null();
            let score = -self.negamax(b, depth - 3, -beta, -beta + 1, ply + 1);
            b.unmake_null();
            if self.stopped {
                return 0;
            }
            if score >= beta {
                return beta;
            }
        }

        let mut moves: Vec<(Move, i32)> = super::movegen::pseudo_moves(b).into_iter().map(|m| (m, 0)).collect();
        self.order_moves(b, &mut moves, None, ply);
        let mut best_score = -MATE * 2;
        let mut best_move = Move::null();
        let mut legal_count = 0;
        let orig_alpha = alpha;
        let side = b.side;

        for (mv, _) in moves {
            b.make_move(mv);
            if b.in_check(side) || b.kings_facing() {
                b.unmake_move();
                continue;
            }
            legal_count += 1;
            let score = -self.negamax(b, depth - 1, -beta, -alpha, ply + 1);
            b.unmake_move();
            if self.stopped {
                return 0;
            }
            if score > best_score {
                best_score = score;
                best_move = mv;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                if mv.captured == EMPTY {
                    let k = &mut self.killers[ply];
                    if !(k[0].from == mv.from && k[0].to == mv.to) {
                        k[1] = k[0];
                        k[0] = mv;
                    }
                    let si = (side >> 3) as usize;
                    self.history[si][mv.from as usize][mv.to as usize] += depth * depth;
                }
                break;
            }
        }

        if legal_count == 0 {
            // 象棋：无子可动（被将死或困毙）均为负
            return -MATE + ply as i32;
        }

        let bound = if best_score <= orig_alpha { 2 } else if best_score >= beta { 1 } else { 0 };
        self.tt_store(b.hash(), best_move, best_score, depth as u8, bound);
        best_score
    }

    fn quiescence(&mut self, b: &mut Board, mut alpha: i32, beta: i32, ply: usize) -> i32 {
        self.nodes += 1;
        self.check_time();
        if self.stopped {
            return 0;
        }
        let in_check = b.in_check(b.side);
        if !in_check {
            let stand = Self::side_rel_eval(b);
            if stand >= beta {
                return stand;
            }
            if stand > alpha {
                alpha = stand;
            }
            if ply >= MAX_PLY - 1 {
                return stand;
            }
        } else if ply >= MAX_PLY - 1 {
            return Self::side_rel_eval(b);
        }

        let side = b.side;
        let mut moves: Vec<(Move, i32)> = if in_check {
            super::movegen::pseudo_moves(b).into_iter().map(|m| (m, 0)).collect()
        } else {
            super::movegen::capture_moves(b).into_iter().map(|m| (m, 0)).collect()
        };
        self.order_moves(b, &mut moves, None, ply.min(MAX_PLY - 1));

        let mut best = if in_check { -MATE * 2 } else { alpha };
        let mut legal = 0;
        for (mv, _) in moves {
            b.make_move(mv);
            if b.in_check(side) || b.kings_facing() {
                b.unmake_move();
                continue;
            }
            legal += 1;
            let score = -self.quiescence(b, -beta, -alpha, ply + 1);
            b.unmake_move();
            if self.stopped {
                return 0;
            }
            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }
        if in_check && legal == 0 {
            return -MATE + ply as i32;
        }
        best
    }

    pub fn search(&mut self, b: &Board, limits: SearchLimits) -> SearchResult {
        self.stopped = false;
        self.nodes = 0;
        self.deadline = if limits.max_time_ms > 0 { Some(Instant::now() + std::time::Duration::from_millis(limits.max_time_ms)) } else { None };
        self.use_null = !limits.pure;
        self.game_hashes = b.history_hashes.clone();

        let mut root_moves: Vec<Move> = b.legal_moves();
        if root_moves.is_empty() {
            let score = if b.in_check(b.side) { -MATE } else { -MATE };
            return SearchResult { best: None, score, pv: vec![], root_scores: vec![], depth_reached: 0, nodes: 0 };
        }

        let mut last_scores: Vec<(Move, i32)> = root_moves.iter().map(|m| (*m, 0)).collect();
        let mut best_move = root_moves[0];
        let mut best_score = 0;
        let mut depth_reached: u8 = 0;

        for depth in 1..=limits.max_depth {
            let mut ordered: Vec<(Move, i32)> = last_scores.clone();
            self.order_moves(b, &mut ordered, Some(best_move), 0);
            let mut iteration: Vec<(Move, i32)> = Vec::new();
            let mut alpha = -MATE * 2;
            let mut iteration_best = Move::null();
            let mut work = b.clone();

            for (mv, _) in ordered {
                work.make_move(mv);
                let score = -self.negamax(&mut work, depth as i32 - 1, -MATE * 2, -alpha, 1);
                work.unmake_move();
                if self.stopped {
                    break;
                }
                iteration.push((mv, score));
                if score > alpha {
                    alpha = score;
                    iteration_best = mv;
                }
            }

            if self.stopped && depth > 1 {
                break;
            }
            if !iteration.is_empty() {
                iteration.sort_by(|a, b2| b2.1.cmp(&a.1));
                last_scores = iteration;
                best_move = last_scores[0].0;
                best_score = last_scores[0].1;
                depth_reached = depth;
            }
            if best_score > MATE_IN_MAX_PLY {
                break; // 已找到杀棋
            }
            if let Some(d) = self.deadline {
                if Instant::now() >= d {
                    break;
                }
            }
        }

        // 低难度：在近似最佳走法中加权随机，制造“人味”波动
        if limits.pool > 1 && !last_scores.is_empty() {
            let top: Vec<(Move, i32)> = last_scores
                .iter()
                .filter(|(_, s)| last_scores[0].1 - s <= limits.window)
                .take(limits.pool)
                .cloned()
                .collect();
            if !top.is_empty() {
                let pick = weighted_pick(&top, b.hash());
                best_move = pick.0;
                best_score = pick.1;
            }
        }

        let pv = self.extract_pv(b, best_move, depth_reached);
        SearchResult { best: Some(best_move), score: best_score, pv, root_scores: last_scores, depth_reached, nodes: self.nodes }
    }

    fn extract_pv(&self, b: &Board, first: Move, depth: u8) -> Vec<Move> {
        let mut pv = vec![first];
        let mut work = b.clone();
        work.make_move(first);
        for _ in 0..depth.min(24) {
            let e = self.tt[(work.hash() as usize) & (TT_SIZE - 1)];
            if e.key != work.hash() || e.mv == 0 {
                break;
            }
            let mv = u16_to_mv(e.mv);
            let legal = work.legal_moves();
            if !legal.iter().any(|m| m.from == mv.from && m.to == mv.to) {
                break;
            }
            work.make_move(mv);
            pv.push(mv);
        }
        pv
    }

    /// 判断当前行棋方是否能在 max_plies 半回合内强制将杀
    pub fn forced_mate(&mut self, b: &Board, max_plies: u8) -> bool {
        let limits = SearchLimits {
            max_depth: max_plies,
            max_time_ms: 5_000,
            pool: 1,
            window: 0,
            pure: true,
        };
        let r = self.search(b, limits);
        r.score >= MATE - max_plies as i32 - 1
    }
}

fn weighted_pick(top: &[(Move, i32)], seed: u64) -> (Move, i32) {
    let n = top.len();
    let weights: Vec<u32> = (0..n).map(|i| (n as u32 - i as u32).max(1) * 2).collect();
    let total: u32 = weights.iter().sum();
    let mut x = splitmix(seed ^ std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos() as u64);
    let r = (x >> 33) as u32 % total;
    let mut acc = 0;
    for (i, w) in weights.iter().enumerate() {
        acc += w;
        if r < acc {
            return top[i];
        }
    }
    top[0]
}

fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// AI 难度等级
pub struct Level {
    pub name: &'static str,
    pub desc: &'static str,
    pub elo: u32,
    pub limits: SearchLimits,
}

pub const LEVELS: [Level; 6] = [
    Level { name: "入门", desc: "适合刚学会规则的朋友", elo: 800, limits: SearchLimits { max_depth: 2, max_time_ms: 200, pool: 6, window: 260, pure: false } },
    Level { name: "初级", desc: "偶尔会犯小错误", elo: 1100, limits: SearchLimits { max_depth: 3, max_time_ms: 400, pool: 4, window: 120, pure: false } },
    Level { name: "中级", desc: "思路比较稳健", elo: 1400, limits: SearchLimits { max_depth: 4, max_time_ms: 800, pool: 3, window: 45, pure: false } },
    Level { name: "高级", desc: "善于抓住机会", elo: 1700, limits: SearchLimits { max_depth: 5, max_time_ms: 1500, pool: 2, window: 18, pure: false } },
    Level { name: "大师", desc: "计算深入，步步紧逼", elo: 2000, limits: SearchLimits { max_depth: 7, max_time_ms: 3000, pool: 1, window: 0, pure: false } },
    Level { name: "特级大师", desc: "全力以赴，小心应对", elo: 2300, limits: SearchLimits { max_depth: 9, max_time_ms: 6000, pool: 1, window: 0, pure: false } },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_mate_in_one() {
        // 高钓马杀：车五进四吃士沉底即杀（f4f9）
        let b = Board::from_fen("2bak4/4a4/2b3N2/9/9/r4R3/9/9/9/4K4 r").unwrap();
        let mut s = Searcher::new();
        let r = s.search(&b, SearchLimits { max_depth: 4, max_time_ms: 2000, pool: 1, window: 0, pure: true });
        assert!(r.score >= MATE - 4, "应发现杀棋, score={}", r.score);
        let mv = r.best.unwrap();
        assert_eq!(mv.iccs(), "f4f9", "应走车沉底, 实际 {}", mv.iccs());
    }

    #[test]
    fn finds_recapture() {
        // 黑车压中路，红车应回吃
        let b = Board::from_fen("4k4/9/9/9/4r4/9/9/9/9/3KR4 r").unwrap();
        let mut s = Searcher::new();
        let r = s.search(&b, SearchLimits::quick(4));
        let mv = r.best.unwrap();
        assert_eq!((mv.to / 9, mv.to % 9), (4, 4), "应吃黑车, 实际 {}", mv.iccs());
        assert!(r.score > 300, "吃车后应大优, score={}", r.score);
    }
}
