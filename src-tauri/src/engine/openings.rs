//! 轻量开局库：按 ICCS 走法序列识别开局名称，并为 AI 提供开局变化。

pub struct OpeningLine {
    pub moves: &'static [&'static str],
    pub name: &'static str,
}

pub static OPENING_LINES: &[OpeningLine] = &[
    OpeningLine { moves: &["h2e2"], name: "中炮局" },
    OpeningLine { moves: &["b2e2"], name: "中炮局" },
    OpeningLine { moves: &["h2e2", "h7e7"], name: "顺手炮" },
    OpeningLine { moves: &["h2e2", "b7e7"], name: "列手炮" },
    OpeningLine { moves: &["h2e2", "h9g7"], name: "中炮对屏风马" },
    OpeningLine { moves: &["h2e2", "b9c7"], name: "中炮对进右马" },
    OpeningLine { moves: &["h2e2", "h9g7", "h0g2"], name: "中炮对屏风马" },
    OpeningLine { moves: &["h2e2", "h9g7", "h0g2", "b9c7"], name: "中炮直车对屏风马" },
    OpeningLine { moves: &["h2e2", "h9g7", "h0g2", "i0h0"], name: "中炮直车对屏风马" },
    OpeningLine { moves: &["h2e2", "h9g7", "h0g2", "b9c7", "i0h0"], name: "中炮直车对屏风马" },
    OpeningLine { moves: &["h2e2", "h9g7", "c3c4"], name: "中炮七兵对屏风马" },
    OpeningLine { moves: &["h2e2", "h9g7", "h0h1"], name: "中炮横车对屏风马" },
    OpeningLine { moves: &["h2e2", "h9g7", "i0h0", "i9h9"], name: "中炮直车对屏风马" },
    OpeningLine { moves: &["h2e2", "b7e7", "h0g2"], name: "列炮对屏风马" },
    OpeningLine { moves: &["g0e2"], name: "飞相局" },
    OpeningLine { moves: &["c0e2"], name: "飞相局" },
    OpeningLine { moves: &["g0e2", "c6c5"], name: "飞相局对卒底炮" },
    OpeningLine { moves: &["g0e2", "b7c7"], name: "飞相局对过宫炮" },
    OpeningLine { moves: &["g0e2", "h7e7"], name: "飞相局对中炮" },
    OpeningLine { moves: &["h0g2"], name: "起马局" },
    OpeningLine { moves: &["b0c2"], name: "起马局" },
    OpeningLine { moves: &["h0g2", "c6c5"], name: "起马局对卒底炮" },
    OpeningLine { moves: &["h0g2", "b7c7"], name: "起马局对过宫炮" },
    OpeningLine { moves: &["c3c4"], name: "仙人指路" },
    OpeningLine { moves: &["g3g4"], name: "仙人指路" },
    OpeningLine { moves: &["c3c4", "c6c5"], name: "对兵局" },
    OpeningLine { moves: &["c3c4", "b7c7"], name: "仙人指路对卒底炮" },
    OpeningLine { moves: &["c3c4", "h7e7"], name: "仙人指路对中炮" },
    OpeningLine { moves: &["c3c4", "h9g7"], name: "仙人指路对兵马局" },
    OpeningLine { moves: &["c3c4", "b2e2"], name: "卒底炮对中炮" },
    OpeningLine { moves: &["h2f2"], name: "士角炮" },
    OpeningLine { moves: &["b2f2"], name: "士角炮" },
    OpeningLine { moves: &["h2d2"], name: "过宫炮" },
    OpeningLine { moves: &["b2d2"], name: "过宫炮" },
    OpeningLine { moves: &["h2d2", "h9g7"], name: "过宫炮对屏风马" },
    OpeningLine { moves: &["h2d2", "c6c5"], name: "过宫炮对卒底炮" },
    OpeningLine { moves: &["e3e4"], name: "中兵局" },
];

/// 返回当前走法序列对应的开局名称（最长匹配）
pub fn identify(moves: &[String]) -> Option<&'static str> {
    let mut best: Option<(usize, &'static str)> = None;
    for line in OPENING_LINES {
        if line.moves.len() > moves.len() {
            continue;
        }
        if (0..line.moves.len()).all(|i| line.moves[i] == moves[i]) {
            match best {
                Some((l, _)) if l >= line.moves.len() => {}
                _ => best = Some((line.moves.len(), line.name)),
            }
        }
    }
    best.map(|(_, n)| n)
}

/// AI 开局应招：在库中寻找当前序列的下一手，返回候选列表
pub fn book_replies(moves: &[String]) -> Vec<&'static str> {
    let mut replies: Vec<&'static str> = Vec::new();
    for line in OPENING_LINES {
        if line.moves.len() > moves.len() {
            if (0..moves.len()).all(|i| line.moves[i] == moves[i]) {
                let next = line.moves[moves.len()];
                if !replies.contains(&next) {
                    replies.push(next);
                }
            }
        }
    }
    replies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_zhong_pao() {
        let seq: Vec<String> = vec!["h2e2".into()];
        assert_eq!(identify(&seq), Some("中炮局"));
        let seq: Vec<String> = vec!["h2e2".into(), "h9g7".into()];
        assert_eq!(identify(&seq), Some("中炮对屏风马"));
    }

    #[test]
    fn book_gives_replies() {
        let seq: Vec<String> = vec!["h2e2".into()];
        let r = book_replies(&seq);
        assert!(r.contains(&"h9g7"));
        assert!(r.contains(&"h7e7"));
    }
}
