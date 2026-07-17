#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
line = lines[78]
opens = line.count(b"(")
closes = line.count(b")")
quotes = line.count(b'"')
print(f"L79: opens={opens} closes={closes} quotes={quotes}")
print(f"Last 15 bytes: {line[-15:].hex()}")
for i in range(100, len(line)):
    b = line[i]
    c = chr(b) if 0x20 <= b < 0x7F else f"\\x{b:02x}"
    print(f"  {i:3}: 0x{b:02x} ({c})")
