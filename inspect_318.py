#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
line = lines[317]  # line 318
print(f"Line 318 total bytes: {len(line)}")
positions = [i for i, b in enumerate(line) if b == 0x22]
print(f"Positions of quote (0x22): {positions}")
qpos = [i for i, b in enumerate(line) if b == 0x3F]
print(f"Positions of ? (0x3F): {qpos}")
# Print the bytes around quote
for p in positions:
    print(f"  Around byte {p}: {line[max(0,p-5):p+5].hex()}")
for p in qpos:
    print(f"  Around ? at byte {p}: {line[max(0,p-5):p+5].hex()}")
