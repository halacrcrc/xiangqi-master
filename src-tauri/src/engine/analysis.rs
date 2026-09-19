//! 复盘分析：对整局每步进行引擎评估，标注 妙手/佳着/一般/欠准/失误/漏着。

use super::board::{Board, Move, RED};
use super::search::{SearchLimits, Searcher, MATE};
use serde::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct ReviewItem {
    pub ply: usize,
    pub iccs: String,
    pub notation: String,
    pub color: String, // "red" | "black"
    /// 走此步前，行棋方最优分（厘兵，行棋方视角）
    pub score_before: i32,
    /// 走此步后，行棋方实际分（行棋方视角）
    pub score_after: i32,
    /// score_before - score_after，越大越差
    pub loss: i32,
    /// 引擎推荐的最佳着法（ICCS）
    pub best_iccs: Option<String>,
    pub best_notation: Option<String>,
    pub classification: String, // brilliant/best/good/ok/inaccuracy/mistake/blunder
    pub is_check: bool,
    pub is_capture: bool,
}

/// 分级阈值（厘兵损失）
fn classify(loss: i32, is_best: bool, was_in_check: bool) -> &'static str {
    if loss <= 5 {
        if is_best {
            "best"
        } else {
            "best"
        }
    } else if loss <= 25 {
        "good"
    } else if loss <= 70 {
        if was_in_check {
            "good" // 应将时的保守着法宽容一些
        } else {
            "ok"
        }
    } else if loss <= 160 {
        "inaccuracy"
    } else if loss <= 380 {
        "mistake"
    } else {
        "blunder"
    }
}

/// 分析一整局。moves 为 ICCS 走法序列，progress 回调 (已完成, 总数)。
pub fn review_game(
    start_fen: &str,
    moves: &[(String, String, bool, bool)], // (iccs, notation, is_check, is_capture)
    depth: u8,
    time_ms: u64,
    mut progress: impl FnMut(usize, usize),
) -> Vec<ReviewItem> {
    let mut searcher = Searcher::new();
    let mut items = Vec::new();
    let n = moves.len();
    if n == 0 {
        return items;
    }

    // 对每个局面（含最终局面）做一次搜索，得到“即将行棋方最优分”
    let mut scores: Vec<i32> = Vec::with_capacity(n + 1); // mover 视角
    let mut bests: Vec<Option<Move>> = Vec::with_capacity(n + 1);
    let mut b = Board::from_fen(start_fen).expect("复盘起始局面");
    let limits = SearchLimits { max_depth: depth, max_time_ms: time_ms, pool: 1, window: 0, pure: false };

    for i in 0..=n {
        if i < n {
            // 逐局面推进
        }
        let r = searcher.search(&b, limits);
        scores.push(r.score);
        bests.push(r.best);
        if i < n {
            let mv = Move::from_iccs(&moves[i].0).expect("复盘走法");
            b.make_move(mv);
        }
        progress(i + 1, n + 1);
    }

    let mut b = Board::from_fen(start_fen).expect("复盘起始局面");
    for i in 0..n {
        let mv = Move::from_iccs(&moves[i].0).expect("复盘走法");
        let mover_color = b.side;
        let was_in_check = b.in_check(mover_color);
        let played = mv;
        b.make_move(played);

        // score_after：下一局面分数取反（对手视角 -> 行棋方视角）
        let score_after = -scores[i + 1];
        let score_before = scores[i];
        let best = bests[i];
        let is_best = match best {
            Some(bb) => bb.from == played.from && bb.to == played.to,
            None => true,
        };
        let loss = (score_before - score_after).max(0).min(MATE as i32);

        let (best_iccs, best_notation) = {
            let mut b2 = Board::from_fen(start_fen).expect("");
            for j in 0..i {
                b2.make_move(Move::from_iccs(&moves[j].0).expect(""));
            }
            match best {
                Some(bb) => {
                    let nota = super::notation::move_notation(&b2, bb);
                    (Some(bb.iccs()), Some(nota))
                }
                None => (None, None),
            }
        };

        let classification = if score_before <= -(MATE - 300) || score_after <= -(MATE - 300) {
            // 已处于败势或刚被将杀时，降低惩罚敏感度
            classify(loss / 2, is_best, was_in_check)
        } else {
            classify(loss, is_best, was_in_check)
        };

        items.push(ReviewItem {
            ply: i,
            iccs: moves[i].0.clone(),
            notation: moves[i].1.clone(),
            color: if mover_color == RED { "red".into() } else { "black".into() },
            score_before,
            score_after,
            loss,
            best_iccs,
            best_notation,
            classification: classification.to_string(),
            is_check: moves[i].2,
            is_capture: moves[i].3,
        });
    }
    items
}

/// 单步走法即时评价（用于对局中提示“好棋/失误”）
pub struct MoveFeedback {
    pub classification: String,
    pub loss: i32,
    pub best_iccs: Option<String>,
    pub best_notation: Option<String>,
}

pub fn judge_move(start_board: &Board, played: Move, depth: u8, time_ms: u64) -> MoveFeedback {
    let mut s = Searcher::new();
    let limits = SearchLimits { max_depth: depth, max_time_ms: time_ms, pool: 1, window: 0, pure: false };
    let r_before = s.search(start_board, limits);
    let score_before = r_before.score;
    let best = r_before.best;

    let mut after = start_board.clone();
    after.make_move(played);
    let r_after = s.search(&after, limits);
    let score_after = -r_after.score;

    let is_best = match best {
        Some(bb) => bb.from == played.from && bb.to == played.to,
        None => true,
    };
    let loss = (score_before - score_after).max(0);
    let classification = classify(loss, is_best, start_board.in_check(start_board.side));
    MoveFeedback {
        classification: classification.to_string(),
        loss,
        best_iccs: best.map(|m| m.iccs()),
        best_notation: best.map(|m| super::notation::move_notation(start_board, m)),
    }
}

/// 分类中文标签
pub fn classification_label(c: &str) -> &'static str {
    match c {
        "best" => "最佳",
        "good" => "佳着",
        "ok" => "一般",
        "inaccuracy" => "欠准",
        "mistake" => "失误",
        "blunder" => "漏着",
        _ => "—",
    }
}

/// 棋手准确度（0-100）：基于每步损失
pub fn accuracy(items: &[ReviewItem], color: &str) -> f64 {
    let mine: Vec<&ReviewItem> = items.iter().filter(|i| i.color == color).collect();
    if mine.is_empty() {
        return 100.0;
    }
    // 每步得分：loss 0 -> 1.0，loss 300+ -> 0.35，loss 600+ -> 0.1
    let sum: f64 = mine
        .iter()
        .map(|i| {
            let l = i.loss.min(800) as f64;
            (1.0 - 0.55 * (l / 300.0).min(1.0) - 0.3 * ((l - 300.0).max(0.0) / 500.0).min(1.0)).max(0.1)
        })
        .sum();
    (sum / mine.len() as f64) * 100.0
}
