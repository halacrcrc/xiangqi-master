//! 杀局解谜与残局训练库。
//! 杀局（mate_in）在测试中通过全宽搜索自动验证；残局训练由引擎防守，玩家实践胜法。

use super::board::Board;

pub enum PuzzleKind {
    /// 在 mate_in 回合内将杀对方
    Mate { mate_in: u8 },
    /// 残局实践：执红（或黑）战胜引擎防守
    Practice,
}

pub struct Puzzle {
    pub id: u32,
    pub name: &'static str,
    pub kind: PuzzleKind,
    pub difficulty: u8, // 1-5
    pub fen: &'static str,
    pub theme: &'static str,
    pub hint: &'static str,
    /// 主变（用于提示展示），ICCS 序列
    pub solution: &'static [&'static str],
}

pub static PUZZLES: &[Puzzle] = &[
    // 1. 马后炮：马 g6 跳到 e2，作炮架并封住将的横路，中路炮成杀
    Puzzle {
        id: 1,
        name: "马后炮",
        kind: PuzzleKind::Mate { mate_in: 1 },
        difficulty: 1,
        fen: "4k4/9/9/4C1N2/9/9/9/9/9/4K4 r",
        theme: "基本杀法",
        hint: "马跳到炮的正后方一格，用马作炮架，炮借马力将军。",
        solution: &["g6e7"],
    },
    // 2. 双车错：底线车横杀，另一车封住第二条横线
    Puzzle {
        id: 2,
        name: "双车错",
        kind: PuzzleKind::Mate { mate_in: 1 },
        difficulty: 1,
        fen: "4k4/R8/9/9/9/4P2R1/9/9/9/4K4 r",
        theme: "基本杀法",
        hint: "沉底车在底线横向将军，另一只车已经封住将的全部退路。",
        solution: &["h4h9"],
    },
    // 3. 重炮杀：前炮作架、后炮将军，将无法脱身
    Puzzle {
        id: 3,
        name: "重炮杀",
        kind: PuzzleKind::Mate { mate_in: 1 },
        difficulty: 2,
        fen: "4k4/9/2NP1PN2/C8/9/9/4C4/9/9/4K4 r",
        theme: "基本杀法",
        hint: "把炮调到中路，与另一门炮形成前后双重炮架。",
        solution: &["a6e6"],
    },
    // 4. 卧槽马：马跳卧槽位将军，车在卒林线封死逃路
    Puzzle {
        id: 4,
        name: "卧槽马",
        kind: PuzzleKind::Mate { mate_in: 1 },
        difficulty: 2,
        fen: "3k5/4R4/3AP4/9/3N5/9/9/9/9/4K4 r",
        theme: "基本杀法",
        hint: "马跳卧槽将军，车已经控制将的上下逃路。",
        solution: &["d5c7"],
    },
    // 5. 钓鱼马：马跳象肩控制花心，车在卒林封线
    Puzzle {
        id: 5,
        name: "钓鱼马",
        kind: PuzzleKind::Mate { mate_in: 1 },
        difficulty: 2,
        fen: "3aka3/R8/9/9/2N6/4P4/9/9/9/4K4 r",
        theme: "基本杀法",
        hint: "马跳到钓鱼位将军，车封住第二条横线，双士自堵将路。",
        solution: &["c5d7"],
    },
    // 6. 中线闷杀：炮借中兵作架沉杀，马兵封锁两翼
    Puzzle {
        id: 6,
        name: "中线闷杀",
        kind: PuzzleKind::Mate { mate_in: 1 },
        difficulty: 3,
        fen: "4k4/9/2NPPPN2/C8/9/9/9/1n7/9/4K4 r",
        theme: "中局杀法",
        hint: "炮调到中路，借中兵作炮架将军，马兵已封死将的逃路。",
        solution: &["a6e6"],
    },
    // —— 残局实战 ——
    Puzzle {
        id: 7,
        name: "单车胜单士",
        kind: PuzzleKind::Practice,
        difficulty: 2,
        fen: "4k4/4a4/9/3R5/9/9/9/9/9/3AK4 r",
        theme: "残局定式",
        hint: "用车控制中路，以帅助攻，捉士技巧：先控将门再捉士。",
        solution: &[],
    },
    Puzzle {
        id: 8,
        name: "单车胜单象",
        kind: PuzzleKind::Practice,
        difficulty: 2,
        fen: "4k4/9/2b6/9/9/3R5/9/9/4A4/3K5 r",
        theme: "残局定式",
        hint: "把将逼到与象同侧，车抢象眼。",
        solution: &[],
    },
    Puzzle {
        id: 9,
        name: "单车难胜士象全",
        kind: PuzzleKind::Practice,
        difficulty: 3,
        fen: "3aka3/9/2b3b2/9/9/2R6/9/9/4A4/3K5 b",
        theme: "残局防守",
        hint: "士象归位、将坐宫心，守住中路即是和棋。你执黑守和。",
        solution: &[],
    },
    Puzzle {
        id: 10,
        name: "马擒单士",
        kind: PuzzleKind::Practice,
        difficulty: 4,
        fen: "3k5/4a4/9/9/9/9/9/4N4/9/4K4 r",
        theme: "残局定式",
        hint: "马擒单士要诀：先捉士后取胜，用帅控制将门。",
        solution: &[],
    },
    Puzzle {
        id: 11,
        name: "炮仕胜单士",
        kind: PuzzleKind::Practice,
        difficulty: 3,
        fen: "3k5/4a4/9/9/9/9/9/4C4/4A4/4K4 r",
        theme: "残局定式",
        hint: "炮需仕作架，用帅与仕配合逐步捉士。",
        solution: &[],
    },
    Puzzle {
        id: 12,
        name: "双兵胜单士",
        kind: PuzzleKind::Practice,
        difficulty: 3,
        fen: "4k4/4a4/9/9/9/9/4P4/9/4P4/4K4 r",
        theme: "残局定式",
        hint: "双兵联手：一兵控将，一兵推进，稳步逼近九宫。",
        solution: &[],
    },
];

pub fn puzzle_by_id(id: u32) -> Option<&'static Puzzle> {
    PUZZLES.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::{Move, BLACK, RED};
    use crate::engine::search::{Searcher, MATE};

    /// 所有杀局谜题必须是强制杀，且不超回合数
    #[test]
    fn verify_mate_puzzles() {
        for p in PUZZLES {
            let PuzzleKind::Mate { mate_in } = p.kind else { continue };
            let b = Board::from_fen(p.fen).unwrap_or_else(|e| panic!("{} FEN 错误: {e}", p.name));
            assert_eq!(b.side, RED, "{} 应红先", p.name);
            assert!(!b.in_check(b.side), "{} 红方不应被将军", p.name);
            assert!(!b.in_check(b.side ^ 8), "{} 黑方在红走棋时不应被将军", p.name);
            assert!(!b.kings_facing(), "{} 双王照面", p.name);
            let mut s = Searcher::new();
            let plies = (mate_in as u8 * 2 - 1).min(10);
            assert!(
                s.forced_mate(&b, plies),
                "谜题【{}】无法在 {} 回合内强制将杀",
                p.name,
                mate_in
            );
            if mate_in >= 2 {
                assert!(!s.forced_mate(&b, 1), "谜题【{}】实为一步杀，与标注不符", p.name);
            }
            if let Some(first) = p.solution.first() {
                let mv = Move::from_iccs(first).unwrap_or_else(|| panic!("{} ICCS 非法", p.name));
                assert!(
                    b.legal_moves().iter().any(|m| m.from == mv.from && m.to == mv.to),
                    "{} 主变 {} 非法",
                    p.name,
                    first
                );
            }
        }
    }

    /// 教程图解中的额外杀法（一步杀，逐个引擎验证）
    #[test]
    fn verify_tutorial_mates() {
        let cases: &[(&str, &str, &str)] = &[
            ("二鬼拍门", "3k1a3/2P1P4/9/9/9/9/9/9/7R1/4K4 r", "h1d1"),
        ];
        for (name, fen, key) in cases {
            let b = Board::from_fen(fen).unwrap_or_else(|e| panic!("{name} FEN 错误: {e}"));
            assert_eq!(b.side, RED, "{name} 应红先");
            assert!(!b.in_check(BLACK), "{name}: 红走棋时黑方不应已被将军");
            assert!(!b.in_check(RED), "{name}: 红方不应被将军");
            assert!(!b.kings_facing(), "{name}: 双王照面");
            let mut s = Searcher::new();
            assert!(s.forced_mate(&b, 1), "{name}: 应为一步杀");
            let mv = Move::from_iccs(key).unwrap();
            assert!(
                b.legal_moves().iter().any(|m| m.from == mv.from && m.to == mv.to),
                "{name}: 主变 {key} 非法"
            );
        }
    }

    /// 残局训练局面应合法
    #[test]
    fn practice_positions_legal() {
        for p in PUZZLES {
            if matches!(p.kind, PuzzleKind::Mate { .. }) {
                continue;
            }
            let b = Board::from_fen(p.fen).unwrap_or_else(|e| panic!("{} FEN 错误: {e}", p.name));
            let opp = b.side ^ 8;
            assert!(!b.in_check(opp), "{}：不应轮到走棋方时对方被将军", p.name);
            assert!(!b.kings_facing(), "{}：双王照面", p.name);
        }
    }

    /// 引擎自弈验证：红方残局应保持优势（粗验证）
    #[test]
    fn practice_red_winnable() {
        for p in PUZZLES {
            if !matches!(p.kind, PuzzleKind::Practice) || p.fen.ends_with('b') {
                continue;
            }
            let mut s = Searcher::new();
            let b = Board::from_fen(p.fen).unwrap();
            let r = s.search(
                &b,
                crate::engine::search::SearchLimits {
                    max_depth: 8,
                    max_time_ms: 1500,
                    pool: 1,
                    window: 0,
                    pure: true,
                },
            );
            assert!(
                r.score > 30 || r.score >= MATE - 20,
                "残局【{}】引擎评估应为红优, 实际 {}",
                p.name,
                r.score
            );
        }
    }
}
