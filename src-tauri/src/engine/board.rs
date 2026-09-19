//! 棋盘表示、FEN、Zobrist 哈希、走子/撤销与攻击检测。
//! 棋盘为 9 列 x 10 行，sq = row * 9 + col。row 0 为黑方底线(上方)，row 9 为红方底线(下方)。

use std::sync::OnceLock;

pub const EMPTY: u8 = 0;
pub const KING: u8 = 1;
pub const ADVISOR: u8 = 2;
pub const BISHOP: u8 = 3;
pub const KNIGHT: u8 = 4;
pub const ROOK: u8 = 5;
pub const CANNON: u8 = 6;
pub const PAWN: u8 = 7;

pub const RED: u8 = 0; // 红方颜色位
pub const BLACK: u8 = 8; // 黑方颜色位

#[inline(always)]
pub fn piece_color(p: u8) -> u8 {
    p & 8
}
#[inline(always)]
pub fn piece_kind(p: u8) -> u8 {
    p & 7
}
#[inline(always)]
pub fn make_piece(color: u8, kind: u8) -> u8 {
    color | kind
}
#[inline(always)]
pub fn is_red(p: u8) -> bool {
    p != EMPTY && (p & 8) == 0
}

/// 走法。captured 为目标格上的棋子(走子前)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub captured: u8,
}

impl Move {
    pub const fn null() -> Self {
        Move { from: 0, to: 0, captured: 0 }
    }
    pub const fn new(from: u8, to: u8, captured: u8) -> Self {
        Move { from, to, captured }
    }
    pub fn is_null(&self) -> bool {
        self.from == 0 && self.to == 0
    }
    /// ICCS 文本，如 "h2e2"
    pub fn iccs(&self) -> String {
        let f = (b'a' + self.from % 9) as char;
        let t = (b'a' + self.to % 9) as char;
        format!("{}{}{}{}", f, 9 - self.from / 9, t, 9 - self.to / 9)
    }
    pub fn from_iccs(s: &str) -> Option<Move> {
        let b = s.as_bytes();
        if b.len() != 4 {
            return None;
        }
        let fc = (b[0] as char).to_ascii_lowercase();
        let tc = (b[2] as char).to_ascii_lowercase();
        if !('a'..='i').contains(&fc) || !('a'..='i').contains(&tc) {
            return None;
        }
        let (fr, tr) = match ((b[1] as char).to_digit(10), (b[3] as char).to_digit(10)) {
            (Some(a), Some(c)) => (a, c),
            _ => return None,
        };
        let from = ((9 - fr) * 9 + (fc as u8 - b'a') as u32) as u8;
        let to = ((9 - tr) * 9 + (tc as u8 - b'a') as u32) as u8;
        if from > 89 || to > 89 {
            return None;
        }
        Some(Move { from, to, captured: EMPTY })
    }
}

struct Zobrist {
    piece: Vec<u64>, // [color_idx(0/1)][kind 0..=7][sq 0..90] -> 展平为 idx
    side: u64,
}

fn zobrist() -> &'static Zobrist {
    static Z: OnceLock<Zobrist> = OnceLock::new();
    Z.get_or_init(|| {
        let mut seed: u64 = 0x9E3779B97F4A7C15;
        let mut next = || {
            seed = seed.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        };
        let mut piece = vec![0u64; 2 * 8 * 90];
        for v in piece.iter_mut() {
            *v = next();
        }
        Zobrist { piece, side: next() }
    })
}

#[inline]
fn z_piece(p: u8, sq: u8) -> u64 {
    let c = (piece_color(p) >> 3) as usize;
    zobrist().piece[(c * 8 + piece_kind(p) as usize) * 90 + sq as usize]
}
#[inline]
fn z_side() -> u64 {
    zobrist().side
}

#[derive(Clone, Copy, Debug)]
struct Undo {
    mv: Move,
    prev_hash: u64,
    null_move: bool,
}

#[derive(Clone, Debug)]
pub struct Board {
    pub squares: [u8; 90],
    /// 当前行棋方：RED 或 BLACK
    pub side: u8,
    hash: u64,
    king_sq: [u8; 2],
    undo_stack: Vec<Undo>,
    /// 含开局在内的全部局面哈希（含空着标记），用于重复检测
    pub history_hashes: Vec<u64>,
}

impl Default for Board {
    fn default() -> Self {
        Board::initial()
    }
}

impl Board {
    pub fn initial() -> Self {
        Board::from_fen("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r")
            .expect("标准开局 FEN 不会出错")
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut squares = [EMPTY; 90];
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.is_empty() {
            return Err("FEN 为空".into());
        }
        let rows: Vec<&str> = parts[0].split('/').collect();
        if rows.len() != 10 {
            return Err(format!("FEN 应有 10 行，实际 {}", rows.len()));
        }
        for (r, row) in rows.iter().enumerate() {
            let mut c = 0usize;
            for ch in row.chars() {
                if let Some(d) = ch.to_digit(10) {
                    c += d as usize;
                } else {
                    let (color, kind) = match ch {
                        'K' => (RED, KING),
                        'A' => (RED, ADVISOR),
                        'B' => (RED, BISHOP),
                        'N' => (RED, KNIGHT),
                        'R' => (RED, ROOK),
                        'C' => (RED, CANNON),
                        'P' => (RED, PAWN),
                        'k' => (BLACK, KING),
                        'a' => (BLACK, ADVISOR),
                        'b' => (BLACK, BISHOP),
                        'n' => (BLACK, KNIGHT),
                        'r' => (BLACK, ROOK),
                        'c' => (BLACK, CANNON),
                        'p' => (BLACK, PAWN),
                        _ => return Err(format!("非法 FEN 字符: {ch}")),
                    };
                    if c > 8 {
                        return Err("FEN 行超长".into());
                    }
                    squares[r * 9 + c] = make_piece(color, kind);
                    c += 1;
                }
            }
            if c != 9 {
                return Err(format!("FEN 第 {r} 行长度错误"));
            }
        }
        let side = match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("r") | Some("w") | None => RED,
            Some("b") => BLACK,
            Some(other) => return Err(format!("非法行棋方: {other}")),
        };
        let board = Board {
            squares,
            side,
            hash: 0,
            king_sq: [255; 2],
            undo_stack: Vec::new(),
            history_hashes: Vec::new(),
        };
        let mut b = board;
        b.recompute_derived();
        Ok(b)
    }

    fn recompute_derived(&mut self) {
        let mut h = 0u64;
        self.king_sq = [255; 2];
        for sq in 0..90u8 {
            let p = self.squares[sq as usize];
            if p == EMPTY {
                continue;
            }
            h ^= z_piece(p, sq);
            if piece_kind(p) == KING {
                self.king_sq[(piece_color(p) >> 3) as usize] = sq;
            }
        }
        if self.side == BLACK {
            h ^= z_side();
        }
        self.hash = h;
        self.history_hashes.clear();
        self.history_hashes.push(h);
    }

    pub fn to_fen(&self) -> String {
        let mut rows = Vec::new();
        for r in 0..10 {
            let mut row = String::new();
            let mut empty = 0;
            for c in 0..9 {
                let p = self.squares[r * 9 + c];
                if p == EMPTY {
                    empty += 1;
                } else {
                    if empty > 0 {
                        row.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let kind = piece_kind(p);
                    let ch = match kind {
                        KING => 'k',
                        ADVISOR => 'a',
                        BISHOP => 'b',
                        KNIGHT => 'n',
                        ROOK => 'r',
                        CANNON => 'c',
                        _ => 'p',
                    };
                    row.push(if is_red(p) { ch.to_ascii_uppercase() } else { ch });
                }
            }
            if empty > 0 {
                row.push_str(&empty.to_string());
            }
            rows.push(row);
        }
        format!("{} {}", rows.join("/"), if self.side == RED { "r" } else { "b" })
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: u8) -> u8 {
        self.squares[sq as usize]
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    pub fn king_square(&self, color: u8) -> Option<u8> {
        let s = self.king_sq[(color >> 3) as usize];
        if s == 255 {
            None
        } else {
            Some(s)
        }
    }

    pub fn make_move(&mut self, mv: Move) {
        let piece = self.squares[mv.from as usize];
        let captured = self.squares[mv.to as usize];
        debug_assert!(piece != EMPTY);
        let mut h = self.hash ^ z_side();
        h ^= z_piece(piece, mv.from) ^ z_piece(piece, mv.to);
        if captured != EMPTY {
            h ^= z_piece(captured, mv.to);
        }
        self.squares[mv.from as usize] = EMPTY;
        self.squares[mv.to as usize] = piece;
        if piece_kind(piece) == KING {
            self.king_sq[(piece_color(piece) >> 3) as usize] = mv.to;
        }
        let prev = self.hash;
        self.hash = h;
        self.undo_stack.push(Undo { mv: Move::new(mv.from, mv.to, captured), prev_hash: prev, null_move: false });
        self.history_hashes.push(h);
        self.side ^= 8;
    }

    pub fn unmake_move(&mut self) {
        let undo = self.undo_stack.pop().expect("撤销栈为空");
        if undo.null_move {
            self.hash = undo.prev_hash;
            self.history_hashes.pop();
            self.side ^= 8;
            return;
        }
        let piece = self.squares[undo.mv.to as usize];
        self.squares[undo.mv.from as usize] = piece;
        self.squares[undo.mv.to as usize] = undo.mv.captured;
        if piece_kind(piece) == KING {
            self.king_sq[(piece_color(piece) >> 3) as usize] = undo.mv.from;
        }
        self.hash = undo.prev_hash;
        self.history_hashes.pop();
        self.side ^= 8;
    }

    /// 空着（仅用于搜索中的 null-move 剪枝）
    pub fn make_null(&mut self) {
        let prev = self.hash;
        self.hash ^= z_side();
        self.undo_stack.push(Undo { mv: Move::null(), prev_hash: prev, null_move: true });
        self.history_hashes.push(self.hash);
        self.side ^= 8;
    }

    pub fn unmake_null(&mut self) {
        self.unmake_move();
    }

    /// 撤销到第 n 步之后（undo_stack 长度为 n）
    pub fn undo_to(&mut self, n: usize) {
        while self.undo_stack.len() > n {
            let was_null = self.undo_stack.last().unwrap().null_move;
            if was_null {
                self.unmake_null();
            } else {
                self.unmake_move();
            }
        }
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// 全局重复计数（当前哈希在历史中出现的次数）
    pub fn repetition_count(&self) -> usize {
        self.history_hashes.iter().filter(|&&h| h == self.hash).count()
    }

    /// color 方是否被将军（含双王照面）
    pub fn in_check(&self, color: u8) -> bool {
        match self.king_square(color) {
            Some(k) => self.is_attacked(k, color ^ 8),
            None => false,
        }
    }

    /// 双王是否直接照面
    pub fn kings_facing(&self) -> bool {
        match self.king_square(RED) {
            Some(rk) => match self.king_square(BLACK) {
                Some(bk) => {
                    let (rr, rc) = (rk / 9, rk % 9);
                    let (br, bc) = (bk / 9, bk % 9);
                    if rc != bc {
                        return false;
                    }
                    let (lo, hi) = if rr < br { (rr, br) } else { (br, rr) };
                    for r in (lo + 1)..hi {
                        if self.squares[r as usize * 9 + rc as usize] != EMPTY {
                            return false;
                        }
                    }
                    true
                }
                None => false,
            },
            None => false,
        }
    }

    /// target 格是否被 by 颜色攻击（仅用于王的安全判断，不包含象的攻击——象永远够不到九宫内的王）
    pub fn is_attacked(&self, target: u8, by: u8) -> bool {
        let r = (target / 9) as i16;
        let c = (target % 9) as i16;
        let sq = self.squares.as_slice();

        // 直线方向：车 / 炮(隔一子) / 邻格王 / 邻格兵
        for d in 0..4 {
            let (dr, dc) = ORTHO[d];
            let mut rr = r + dr;
            let mut cc = c + dc;
            let mut screen = false;
            while (0..10).contains(&rr) && (0..9).contains(&cc) {
                let p = sq[rr as usize * 9 + cc as usize];
                if p != EMPTY {
                    let friendly = piece_color(p) == by;
                    let kind = piece_kind(p);
                    if !screen {
                        // 第一个子：车直击、邻格王、兵的攻击；任意子都可作其后炮的炮架
                        if friendly {
                            if kind == ROOK {
                                return true;
                            }
                            if kind == KING && (rr - r).abs() + (cc - c).abs() == 1 {
                                return true;
                            }
                            if kind == PAWN {
                                if by == RED {
                                    if rr == r + 1 && cc == c {
                                        return true;
                                    }
                                    if rr == r && (cc == c - 1 || cc == c + 1) && rr <= 4 {
                                        return true;
                                    }
                                } else {
                                    if rr == r - 1 && cc == c {
                                        return true;
                                    }
                                    if rr == r && (cc == c - 1 || cc == c + 1) && rr >= 5 {
                                        return true;
                                    }
                                }
                            }
                        }
                        screen = true;
                    } else {
                        // 第二个子：若为对方炮，则隔炮架攻击目标
                        if friendly && kind == CANNON {
                            return true;
                        }
                        break;
                    }
                }
                rr += dr;
                cc += dc;
            }
        }

        // 马：反向马位 + 马腿
        for (dr, dc, lr, lc) in KNIGHT_ATTACK {
            let kr = r + dr;
            let kc = c + dc;
            if !(0..10).contains(&kr) || !(0..9).contains(&kc) {
                continue;
            }
            let p = sq[kr as usize * 9 + kc as usize];
            if p != EMPTY && piece_color(p) == by && piece_kind(p) == KNIGHT {
                let ley = r + lr;
                let lex = c + lc;
                if sq[ley as usize * 9 + lex as usize] == EMPTY {
                    return true;
                }
            }
        }

        // 士：斜邻格
        for (dr, dc) in DIAG {
            let ar = r + dr;
            let ac = c + dc;
            if !(0..10).contains(&ar) || !(0..9).contains(&ac) {
                continue;
            }
            let p = sq[ar as usize * 9 + ac as usize];
            if p != EMPTY && piece_color(p) == by && piece_kind(p) == ADVISOR {
                return true;
            }
        }

        false
    }

    /// 生成伪合法走法后逐个过滤：走完后己方不被将军且双王不照面
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut legal = Vec::new();
        let side = self.side;
        let mut b = self.clone();
        for mv in movegen::pseudo_moves(&b) {
            b.make_move(mv);
            if !b.in_check(side) && !b.kings_facing() {
                legal.push(mv);
            }
            b.unmake_move();
        }
        legal
    }

    /// 当前行棋方是否无合法走法（被将死或困毙——象棋中均为输棋）
    pub fn no_moves(&self) -> bool {
        self.legal_moves().is_empty()
    }

    /// 双方是否都只剩“无法将死对方”的子力（自动和棋参考）
    pub fn insufficient_material(&self) -> bool {
        let mut heavy = [false; 2];
        for sq in 0..90u8 {
            let p = self.piece_at(sq);
            if p == EMPTY {
                continue;
            }
            match piece_kind(p) {
                KING | ADVISOR | BISHOP => {}
                ROOK | CANNON => heavy[(piece_color(p) >> 3) as usize] = true,
                KNIGHT | PAWN => heavy[(piece_color(p) >> 3) as usize] = true,
                _ => {}
            }
        }
        // 只剩双王 / 单边仅士象且另一边无进攻子
        !heavy[0] && !heavy[1]
    }
}

pub const ORTHO: [(i16, i16); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
pub const DIAG: [(i16, i16); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

/// (马相对目标的偏移, 马腿相对目标的偏移)
pub const KNIGHT_ATTACK: [(i16, i16, i16, i16); 8] = [
    (-2, -1, -1, -1),
    (-2, 1, -1, 1),
    (2, -1, 1, -1),
    (2, 1, 1, 1),
    (-1, -2, -1, -1),
    (-1, 2, -1, 1),
    (1, -2, 1, -1),
    (1, 2, 1, 1),
];

use crate::engine::movegen;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_roundtrip() {
        let fens = [
            "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r",
            "2b1k4/9/2b6/p1N6/9/9/4C4/9/9/4K4 r",
            "4k4/4a4/9/4R4/9/9/9/9/9/3AK4 b",
        ];
        for f in fens {
            let b = Board::from_fen(f).unwrap();
            assert_eq!(b.to_fen(), f, "FEN 往返不一致: {f}");
        }
    }

    #[test]
    fn iccs_roundtrip() {
        let mv = Move::from_iccs("h2e2").unwrap();
        assert_eq!((mv.from / 9, mv.from % 9), (7, 7)); // 红炮 h2
        assert_eq!((mv.to / 9, mv.to % 9), (7, 4));
        assert_eq!(mv.iccs(), "h2e2");
    }

    #[test]
    fn make_unmake_restores() {
        let mut b = Board::initial();
        let moves = b.legal_moves();
        let hash0 = b.hash();
        let fen0 = b.to_fen();
        for mv in moves.iter().take(10) {
            b.make_move(*mv);
            b.unmake_move();
        }
        assert_eq!(b.hash(), hash0);
        assert_eq!(b.to_fen(), fen0);
    }

    #[test]
    fn initial_legal_moves_44() {
        let b = Board::initial();
        assert_eq!(b.legal_moves().len(), 44);
    }
}
