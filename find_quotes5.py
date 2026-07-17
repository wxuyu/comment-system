#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# Print all bytes for L632, 638, 640
for line_no in [632, 638, 640, 641, 642, 643]:
    if line_no - 1 >= len(lines):
        break
    line = lines[line_no - 1]
    print(f"=== L{line_no} bytes ({len(line)}) ===")
    for i, b in enumerate(line):
        c = chr(b) if 0x20 <= b < 0x7F else f"\\x{b:02x}"
        print(f"  {i:3}: 0x{b:02x} ({c})")
    print()
