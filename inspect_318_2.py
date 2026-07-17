#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
line = lines[317]
positions = [i for i, b in enumerate(line) if b == 0x22]
print(f"Positions of quote (0x22): {positions}")
qpos = [i for i, b in enumerate(line) if b == 0x3F]
print(f"Positions of ? (0x3F): {qpos}")
fw_comma = b"\xef\xbc\x8c"
fw_pos = []
i = 0
while True:
    idx = line.find(fw_comma, i)
    if idx < 0:
        break
    fw_pos.append(idx)
    i = idx + 1
print(f"Positions of fullwidth comma: {fw_pos}")
print(f"Line text: {line.decode('utf-8', errors='replace')}")
