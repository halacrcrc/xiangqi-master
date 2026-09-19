# -*- coding: utf-8 -*-
"""校验 tutorials.ts 中全部 FEN：行结构、双王、九宫、士象落点、双王照面。"""
import re
import sys
from pathlib import Path

ADVISOR_PTS = {(0, 3), (0, 5), (1, 4), (2, 3), (2, 5), (7, 3), (7, 5), (8, 4), (9, 3), (9, 5)}
ELEPHANT_PTS = {(r, c) for r in (0, 2, 4, 5, 7, 9) for c in (0, 2, 4, 6, 8)}

def parse_board(fen):
    rows = fen.split()[0].split("/")
    assert len(rows) == 10, f"行数 {len(rows)} != 10"
    board = {}
    for r, row in enumerate(rows):
        c = 0
        for ch in row:
            if ch.isdigit():
                c += int(ch)
            else:
                assert 0 <= c <= 8, f"第{r}行列溢出"
                board[(r, c)] = ch
                c += 1
        assert c == 9, f"第{r}行长度 {c} != 9"
    return board

def check(fen):
    board = parse_board(fen)
    kings = [sq for sq, p in board.items() if p.upper() == "K"]
    assert len(kings) == 2, "王的数量不对"
    for (r, c) in kings:
        assert 3 <= c <= 5, f"王 {r},{c} 不在九宫横线"
        assert r <= 2 or r >= 7, f"王 {r},{c} 不在九宫"
    for (r, c), p in board.items():
        if p.upper() == "A":
            assert (r, c) in ADVISOR_PTS, f"士落点非法 {r},{c}"
        if p.upper() == "B":
            assert (r, c) in ELEPHANT_PTS, f"象落点非法 {r},{c}"
            if p == "B":
                assert r >= 5, f"红象过河 {r},{c}"
            else:
                assert r <= 4, f"黑象过河 {r},{c}"
        if p.upper() == "K":
            pass
    # 双王照面
    (r1, c1), (r2, c2) = sorted(kings)
    if c1 == c2:
        blockers = [(r, c1) for r in range(r1 + 1, r2) if (r, c1) in board]
        assert blockers, "双王照面!"
    return True

def main():
    src = Path(__file__).parent.parent / "src" / "data" / "tutorials.ts"
    text = src.read_text(encoding="utf-8")
    fens = re.findall(r'"((?:[rnbakcpRNBAKCP1-9]+/){9}[rnbakcpRNBAKCP1-9]+ [rb])"', text)
    ok = 0
    for fen in fens:
        try:
            check(fen)
            ok += 1
        except AssertionError as e:
            print(f"FAIL  {fen}  -> {e}")
            sys.exit(1)
    print(f"ALL {ok} FENS OK")

if __name__ == "__main__":
    main()
