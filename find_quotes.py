#!/usr/bin/env python3
"""检查行 334 的实际内容"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
for line_no in [313, 318, 334, 342, 448, 462, 477, 488, 536, 557, 577, 632, 638, 640]:
    if line_no - 1 >= len(lines):
        continue
    line = lines[line_no - 1]
    text = line.decode("utf-8", errors="replace")
    print(f"L{line_no}: {text!r}")
    # 找 0x22 位置
    positions = [i for i, b in enumerate(line) if b == 0x22]
    print(f"  Quote positions: {positions}")
    print()
