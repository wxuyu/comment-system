#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# Look at lines 84, 86, 118, 149
for line_no in [84, 86, 118, 149]:
    line = lines[line_no - 1]
    print(f"=== L{line_no} ({len(line)} bytes) ===")
    print(line.decode("utf-8", errors="replace"))
    positions = [i for i, b in enumerate(line) if b == 0x22]
    print(f"Quote positions: {positions}")
    print()
