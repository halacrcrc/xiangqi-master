//! 局面评估：子力价值 + 位置价值表(红方视角, 黑方镜像) + 结构小项。
//! 返回值为红方视角厘兵单位。

use super::board::*;

pub const VAL_KING: i32 = 100_000;
pub const VAL_ROOK: i32 = 1_000;
pub const VAL_CANNON: i32 = 470;
pub const VAL_KNIGHT: i32 = 460;
pub const VAL_PAWN: i32 = 100;
pub const VAL_ADVISOR: i32 = 120;
pub const VAL_BISHOP: i32 = 120;

#[inline]
pub fn kind_value(kind: u8) -> i32 {
    match kind {
        KING => VAL_KING,
        ROOK => VAL_ROOK,
        CANNON => VAL_CANNON,
        KNIGHT => VAL_KNIGHT,
        PAWN => VAL_PAWN,
        ADVISOR => VAL_ADVISOR,
        BISHOP => VAL_BISHOP,
        _ => 0,
    }
}

/// 兵（红方视角，row 0 为敌方底线）
#[rustfmt::skip]
const PST_PAWN: [i32; 90] = [
    10, 15, 20, 30, 35, 30, 20, 15, 10,
    15, 20, 30, 45, 55, 45, 30, 20, 15,
    20, 30, 40, 55, 65, 55, 40, 30, 20,
    20, 30, 40, 50, 60, 50, 40, 30, 20,
    15, 25, 35, 45, 50, 45, 35, 25, 15,
    10, 15, 20, 25, 30, 25, 20, 15, 10,
    4, 0, 4, 0, 6, 0, 4, 0, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[rustfmt::skip]
const PST_KNIGHT: [i32; 90] = [
    0, 10, 14, 18, 22, 18, 14, 10, 0,
    8, 16, 24, 28, 30, 28, 24, 16, 8,
    10, 20, 28, 34, 36, 34, 28, 20, 10,
    10, 20, 30, 36, 40, 36, 30, 20, 10,
    10, 20, 30, 38, 42, 38, 30, 20, 10,
    6, 16, 24, 32, 34, 32, 24, 16, 6,
    4, 12, 18, 24, 28, 24, 18, 12, 4,
    0, 8, 12, 16, 18, 16, 12, 8, 0,
    -4, 0, 6, 8, 10, 8, 6, 0, -4,
    -8, -4, 0, 2, 4, 2, 0, -4, -8,
];

#[rustfmt::skip]
const PST_ROOK: [i32; 90] = [
    20, 24, 26, 30, 34, 30, 26, 24, 20,
    18, 22, 26, 30, 32, 30, 26, 22, 18,
    14, 20, 24, 28, 28, 28, 24, 20, 14,
    12, 18, 22, 24, 26, 24, 22, 18, 12,
    10, 16, 20, 24, 24, 24, 20, 16, 10,
    8, 14, 16, 20, 20, 20, 16, 14, 8,
    6, 12, 14, 18, 18, 18, 14, 12, 6,
    4, 10, 12, 14, 16, 14, 12, 10, 4,
    2, 8, 10, 12, 12, 12, 10, 8, 2,
    0, 4, 6, 8, 10, 8, 6, 4, 0,
];

#[rustfmt::skip]
const PST_CANNON: [i32; 90] = [
    10, 12, 12, 16, 18, 16, 12, 12, 10,
    10, 14, 14, 18, 20, 18, 14, 14, 10,
    10, 14, 16, 20, 22, 20, 16, 14, 10,
    8, 12, 14, 18, 20, 18, 14, 12, 8,
    8, 12, 14, 18, 20, 18, 14, 12, 8,
    6, 10, 12, 14, 16, 14, 12, 10, 6,
    4, 8, 10, 12, 12, 12, 10, 8, 4,
    2, 6, 8, 10, 10, 10, 8, 6, 2,
    0, 4, 6, 8, 8, 8, 6, 4, 0,
    0, 2, 4, 6, 6, 6, 4, 2, 0,
];

/// 士象帅的位置微调（红方视角）
#[inline]
fn pst_defense(kind: u8, sq: u8) -> i32 {
    let r = sq / 9;
    let c = sq % 9;
    match kind {
        ADVISOR => match (r, c) {
            (9, 3) | (9, 5) => 5,
            (8, 4) => 12,
            (7, 3) | (7, 5) => 10,
            _ => 0,
        },
        BISHOP => match (r, c) {
            (9, 2) | (9, 6) => 5,
            (7, 0) | (7, 8) => 8,
            (7, 4) => 12,
            (5, 2) | (5, 6) => 10,
            _ => 0,
        },
        KING => match (r, c) {
            (9, 4) => 0,
            (9, 3) | (9, 5) => -6,
            (8, 4) => -12,
            _ => -20,
        },
        _ => 0,
    }
}

#[inline]
fn flip(sq: u8) -> u8 {
    (9 - sq / 9) * 9 + sq % 9
}

/// 红方视角评估
pub fn evaluate(b: &Board) -> i32 {
    let mut score: i32 = 0;
    let mut advisors = [0i32; 2];
    let mut bishops = [0i32; 2];
    for sq in 0u8..90 {
        let p = b.piece_at(sq);
        if p == EMPTY {
            continue;
        }
        let kind = piece_kind(p);
        let red = is_red(p);
        // 位置表均为红方视角，黑方需镜像索引
        let idx = if red { sq } else { flip(sq) };
        let pst = match kind {
            PAWN => PST_PAWN[idx as usize],
            KNIGHT => PST_KNIGHT[idx as usize],
            ROOK => PST_ROOK[idx as usize],
            CANNON => PST_CANNON[idx as usize],
            _ => pst_defense(kind, idx),
        };
        let v = kind_value(kind) + pst;
        if red {
            score += v;
            if kind == ADVISOR {
                advisors[0] += 1;
            }
            if kind == BISHOP {
                bishops[0] += 1;
            }
        } else {
            score -= v;
            if kind == ADVISOR {
                advisors[1] += 1;
            }
            if kind == BISHOP {
                bishops[1] += 1;
            }
        }
    }
    // 士象联防小奖励
    if advisors[0] == 2 {
        score += 8;
    }
    if bishops[0] == 2 {
        score += 8;
    }
    if advisors[1] == 2 {
        score -= 8;
    }
    if bishops[1] == 2 {
        score -= 8;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_eval_near_zero() {
        let b = Board::initial();
        let e = evaluate(&b);
        assert!(e.abs() < 60, "开局评估应接近均衡，实际 {e}");
    }

    #[test]
    fn material_dominance() {
        // 红多一车
        let b = Board::from_fen("4k4/9/9/9/9/9/9/9/9/R3K4 r").unwrap();
        assert!(evaluate(&b) > 900);
    }
}
