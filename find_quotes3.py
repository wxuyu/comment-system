#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# L334 完整字节
for line_no in [313, 318, 334, 342, 448, 462, 477, 488, 536, 557, 577, 632, 638, 640]:
    line = lines[line_no - 1]
    print(f"=== L{line_no} (len={len(line)}) ===")
    print(line.decode("utf-8", errors="replace"))
    print()
