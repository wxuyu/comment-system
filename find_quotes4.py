#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# Print all bytes for L334
line = lines[333]
print(f"=== L334 bytes ({len(line)}) ===")
for i, b in enumerate(line):
    c = chr(b) if 0x20 <= b < 0x7F else f"\\x{b:02x}"
    print(f"  {i:3}: 0x{b:02x} ({c})")
