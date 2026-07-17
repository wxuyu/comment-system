#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# 检查每行的引号平衡
for i, line in enumerate(lines, 1):
    if 1 <= i <= 106:
        # 简单计数 - 排除 // 注释
        stripped = line.lstrip()
        if stripped.startswith(b"//"):
            continue
        opens = line.count(b"(")
        closes = line.count(b")")
        quotes = line.count(b'"')
        unbalanced = quotes % 2 == 1
        if unbalanced:
            text = line.decode("utf-8", errors="replace")[:80]
            print(f"L{i}: q={quotes}  {text!r}")
