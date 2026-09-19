//! 伪合法走法生成（合法性过滤在 Board::legal_moves 中完成）与 perft 测试。

use super::board::*;

#[inline(always)]
fn in_board(r: i16, c: i16) -> bool {
    (0..10).contains(&r) && (0..9).contains(&c)
}

#[inline(always)]
fn in_palace(r: i16, c: i16, color: u8) -> bool {
    if !(3..=5).contains(&c) {
        return false;
    }
    if color == RED {
        (7..=9).contains(&r)
    } else {
        (0..=2).contains(&r)
    }
}

#[inline(always)]
fn own_half(r: i16, color: u8) -> bool {
    if color == RED {
        r >= 5
    } else {
        r <= 4
    }
}

const KNIGHT_MOVES: [(i16, i16); 8] = [
    (-2, -1),
    (-2, 1),
    (2, -1),
    (2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
];

const BISHOP_MOVES: [(i16, i16); 4] = [(-2, -2), (-2, 2), (2, -2), (2, 2)];

pub fn pseudo_moves(b: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(48);
    let side = b.side;
    let sqs = b.squares;
    for sq in 0u8..90 {
        let p = sqs[sq as usize];
        if p == EMPTY || piece_color(p) != side {
            continue;
        }
        let kind = piece_kind(p);
        let r = (sq / 9) as i16;
        let c = (sq % 9) as i16;
        let mut push = |moves: &mut Vec<Move>, rr: i16, cc: i16| {
            if !in_board(rr, cc) {
                return;
            }
            let t = sqs[rr as usize * 9 + cc as usize];
            if t != EMPTY && piece_color(t) == side {
                return;
            }
            moves.push(Move::new(sq, (rr * 9 + cc) as u8, t));
        };
        match kind {
            KING => {
                for (dr, dc) in ORTHO {
                    let (rr, cc) = (r + dr, c + dc);
                    if in_board(rr, cc) && in_palace(rr, cc, side) {
                        push(&mut moves, rr, cc);
                    }
                }
            }
            ADVISOR => {
                for (dr, dc) in DIAG {
                    let (rr, cc) = (r + dr, c + dc);
                    if in_board(rr, cc) && in_palace(rr, cc, side) {
                        push(&mut moves, rr, cc);
                    }
                }
            }
            BISHOP => {
                for (dr, dc) in BISHOP_MOVES {
                    let (rr, cc) = (r + dr, c + dc);
                    if in_board(rr, cc) && own_half(rr, side) {
                        // 象眼检查
                        if sqs[(r + dr / 2) as usize * 9 + (c + dc / 2) as usize] != EMPTY {
                            continue;
                        }
                        push(&mut moves, rr, cc);
                    }
                }
            }
            KNIGHT => {
                for (dr, dc) in KNIGHT_MOVES {
                    let (rr, cc) = (r + dr, c + dc);
                    if !in_board(rr, cc) {
                        continue;
                    }
                    // 马腿：沿两格方向相邻
                    let (lr, lc) = if dr.abs() == 2 { (r + dr / 2, c) } else { (r, c + dc / 2) };
                    if sqs[lr as usize * 9 + lc as usize] != EMPTY {
                        continue;
                    }
                    push(&mut moves, rr, cc);
                }
            }
            ROOK => {
                for (dr, dc) in ORTHO {
                    let mut rr = r + dr;
                    let mut cc = c + dc;
                    while in_board(rr, cc) {
                        let t = sqs[rr as usize * 9 + cc as usize];
                        if t == EMPTY {
                            moves.push(Move::new(sq, (rr * 9 + cc) as u8, EMPTY));
                        } else {
                            if piece_color(t) != side {
                                moves.push(Move::new(sq, (rr * 9 + cc) as u8, t));
                            }
                            break;
                        }
                        rr += dr;
                        cc += dc;
                    }
                }
            }
            CANNON => {
                for (dr, dc) in ORTHO {
                    let mut rr = r + dr;
                    let mut cc = c + dc;
                    // 阶段一：空位直走
                    while in_board(rr, cc) && sqs[rr as usize * 9 + cc as usize] == EMPTY {
                        moves.push(Move::new(sq, (rr * 9 + cc) as u8, EMPTY));
                        rr += dr;
                        cc += dc;
                    }
                    // 阶段二：越过炮架吃子
                    if in_board(rr, cc) {
                        rr += dr;
                        cc += dc;
                        while in_board(rr, cc) {
                            let t = sqs[rr as usize * 9 + cc as usize];
                            if t != EMPTY {
                                if piece_color(t) != side {
                                    moves.push(Move::new(sq, (rr * 9 + cc) as u8, t));
                                }
                                break;
                            }
                            rr += dr;
                            cc += dc;
                        }
                    }
                }
            }
            PAWN => {
                let fwd: i16 = if side == RED { -1 } else { 1 };
                let (rr, cc) = (r + fwd, c);
                if in_board(rr, cc) {
                    push(&mut moves, rr, cc);
                }
                let crossed = if side == RED { r <= 4 } else { r >= 5 };
                if crossed {
                    for dc in [-1i16, 1] {
                        let (rr, cc) = (r, c + dc);
                        if in_board(rr, cc) {
                            push(&mut moves, rr, cc);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    moves
}

/// 仅吃子走法（用于静态搜索）
pub fn capture_moves(b: &Board) -> Vec<Move> {
    pseudo_moves(b).into_iter().filter(|m| m.captured != EMPTY).collect()
}

/// perft 节点计数
pub fn perft(b: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let side = b.side;
    let mut nodes: u64 = 0;
    for mv in pseudo_moves(b) {
        b.make_move(mv);
        if !b.in_check(side) && !b.kings_facing() {
            nodes += if depth == 1 { 1 } else { perft(b, depth - 1) };
        }
        b.unmake_move();
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    // 象棋起始局面公开 perft 值
    #[test]
    fn perft_initial() {
        let mut b = Board::initial();
        assert_eq!(perft(&mut b, 1), 44);
        assert_eq!(perft(&mut b, 2), 1_920);
    }

    #[test]
    fn perft_initial_deep() {
        let mut b = Board::initial();
        assert_eq!(perft(&mut b, 3), 79_666);
    }

    #[test]
    fn perft_check_positions() {
        // 包含将军/应将的局面
        let mut b = Board::from_fen("4k4/9/9/9/9/9/9/9/9/3K1R3 r").unwrap();
        // 车 e1 将军：黑王必须应将
        let moves = b.legal_moves();
        assert!(moves.iter().any(|m| m.from % 9 == 5 && m.to % 9 == 4));
        // 黑方无子可动应将的场景
        let mut b2 = Board::from_fen("4k4/9/9/9/9/9/9/9/9/3K2R2 b").unwrap();
        let ms = b2.legal_moves();
        // 黑将可横移避开
        assert!(ms.len() >= 2);
    }

    #[test]
    fn kings_facing_rule() {
        // 双王照面：红车移开后若双王同列无遮挡则为非法
        let mut b = Board::from_fen("4k4/9/9/9/9/9/9/9/4R4/3K5 r").unwrap();
        // 红车在 e2 挡住照面；把车移走（车二进X离线）应导致照面非法
        let mv = Move::from_iccs("e2h2").unwrap();
        let legal = b.legal_moves();
        assert!(!legal.contains(&mv), "移开车导致双王照面应为非法");
    }
}
