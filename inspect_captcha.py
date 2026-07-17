#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# 找所有可能的问题行
for i, line in enumerate(lines, 1):
    if 100 <= i <= 175:
        text = line.decode("utf-8", errors="replace")
        # 找末尾的 ? 或不正常字符
        stripped = text.rstrip()
        if stripped.endswith("?") and not stripped.endswith("?,") and not "TODO" in text:
            print(f"L{i}: {text!r}")
