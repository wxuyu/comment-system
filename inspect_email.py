#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\email.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
for line_no in [32, 43, 50, 74, 81, 90]:
    line = lines[line_no - 1]
    print(f"=== L{line_no} ({len(line)} bytes) ===")
    print(repr(line.decode("utf-8", errors="replace")))
    print()
