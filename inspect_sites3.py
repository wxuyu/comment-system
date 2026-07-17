#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# Check L149 with all bytes
for line_no in [86, 118, 149]:
    line = lines[line_no - 1]
    print(f"=== L{line_no} ({len(line)} bytes) ===")
    for i, b in enumerate(line):
        c = chr(b) if 0x20 <= b < 0x7F else f"\\x{b:02x}"
        if 30 <= i <= 70:
            print(f"  {i:3}: 0x{b:02x} ({c})")
    print()
