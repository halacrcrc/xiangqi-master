//! 中文纵线记谱法：如 炮二平五 / 马8进7 / 前车退二

use super::board::*;

const NAME_RED: [&str; 8] = ["", "帅", "仕", "相", "马", "车", "炮", "兵"];
const NAME_BLACK: [&str; 8] = ["", "将", "士", "象", "马", "车", "炮", "卒"];
const NUM_RED: [&str; 9] = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];

fn file_name(color: u8, col: u8) -> String {
    if color == RED {
        NUM_RED[(8 - col) as usize].to_string()
    } else {
        char::from_digit((col + 1) as u32, 10).unwrap_or('?').to_string()
    }
}

fn step_name(color: u8, n: u8) -> String {
    let s = n.to_string();
    if color == RED {
        // 红方用中文数字
        if n >= 1 && n <= 9 {
            NUM_RED[(n - 1) as usize].to_string()
        } else {
            s
        }
    } else {
        s
    }
}

/// 生成走法的中文记谱。board 应为走子前的局面。
pub fn move_notation(b: &Board, mv: Move) -> String {
    let p = b.piece_at(mv.from);
    if p == EMPTY {
        return mv.iccs();
    }
    let color = piece_color(p);
    let kind = piece_kind(p);
    let from_row = mv.from / 9;
    let from_col = mv.from % 9;
    let to_row = mv.to / 9;
    let to_col = mv.to % 9;

    // 同纵线上是否有同种同色棋子（前/后/中 消歧）
    let mut same_file: Vec<u8> = Vec::new();
    for r in 0u8..10 {
        let q = b.piece_at(r * 9 + from_col);
        if q != EMPTY && piece_kind(q) == kind && piece_color(q) == color {
            same_file.push(r);
        }
    }

    let prefix = if same_file.len() >= 2 {
        // 前方定义：红方行进方向为 row 减小
        let mut rows = same_file.clone();
        if color == RED {
            rows.sort(); // row 小在前
        } else {
            rows.sort_by(|a, b| b.cmp(a)); // row 大在前
        }
        let idx = rows.iter().position(|&r| r == from_row).unwrap_or(0);
        let tag = match rows.len() {
            2 => {
                if idx == 0 {
                    "前"
                } else {
                    "后"
                }
            }
            3 => match idx {
                0 => "前",
                2 => "后",
                _ => "中",
            },
            _ => {
                if idx == 0 {
                    "前"
                } else if idx == rows.len() - 1 {
                    "后"
                } else {
                    "中"
                }
            }
        };
        let name = if color == RED { NAME_RED[kind as usize] } else { NAME_BLACK[kind as usize] };
        format!("{}{}", tag, name)
    } else {
        let name = if color == RED { NAME_RED[kind as usize] } else { NAME_BLACK[kind as usize] };
        format!("{}{}", name, file_name(color, from_col))
    };

    let action = if to_row == from_row {
        "平".to_string()
    } else {
        let forward = if color == RED { to_row < from_row } else { to_row > from_row };
        if forward {
            "进".to_string()
        } else {
            "退".to_string()
        }
    };

    let tail = if to_row == from_row {
        file_name(color, to_col)
    } else if kind == KNIGHT || kind == ADVISOR || kind == BISHOP {
        file_name(color, to_col)
    } else {
        step_name(color, (to_row as i16 - from_row as i16).unsigned_abs() as u8)
    };

    format!("{}{}{}", prefix, action, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_opening_notations() {
        let b = Board::initial();
        // 炮二平五：h2e2
        let mv = Move::from_iccs("h2e2").unwrap();
        assert_eq!(move_notation(&b, mv), "炮二平五");
        // 马二进三：h0g2
        let mv = Move::from_iccs("h0g2").unwrap();
        assert_eq!(move_notation(&b, mv), "马二进三");
        // 兵三进一：g3g4
        let mv = Move::from_iccs("g3g4").unwrap();
        assert_eq!(move_notation(&b, mv), "兵三进一");
        // 车一平二：i0h0
        let mv = Move::from_iccs("i0h0").unwrap();
        assert_eq!(move_notation(&b, mv), "车一平二");
    }

    #[test]
    fn black_notations() {
        let b = Board::initial();
        // 黑方：炮8平5 = h7e7
        let mv = Move::from_iccs("h7e7").unwrap();
        assert_eq!(move_notation(&b, mv), "炮8平5");
        // 黑方：马8进7 = h9g7
        let mv = Move::from_iccs("h9g7").unwrap();
        assert_eq!(move_notation(&b, mv), "马8进7");
        // 黑方：卒3进1 = c6c5
        let mv = Move::from_iccs("c6c5").unwrap();
        assert_eq!(move_notation(&b, mv), "卒3进1");
    }

    #[test]
    fn front_back_disambiguation() {
        // 两红炮同线（row6、row8）
        let b = Board::from_fen("4k4/9/9/9/9/9/4C4/9/4C4/4K4 r").unwrap();
        let mv = Move::from_iccs("e3e6").unwrap(); // 前炮（row6 更靠前）
        assert!(move_notation(&b, mv).starts_with("前炮"), "实际: {}", move_notation(&b, mv));
        let mv = Move::from_iccs("e1e6").unwrap(); // 后炮
        assert!(move_notation(&b, mv).starts_with("后炮"), "实际: {}", move_notation(&b, mv));
    }
}
