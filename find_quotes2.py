#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# L313 字节详情
for line_no in [313, 318]:
    line = lines[line_no - 1]
    print(f"=== L{line_no} ===")
    print(f"Length: {len(line)}")
    # 找所有 0x22 位置
    positions = [i for i, b in enumerate(line) if b == 0x22]
    print(f"Quote positions: {positions}")
    # 找所有 0x3F
    qpos = [i for i, b in enumerate(line) if b == 0x3F]
    print(f"? positions: {qpos}")
    # 找所有全角字符位置 (3-byte UTF-8 sequences starting with non-ASCII)
    print("Bytes 70-end:")
    for i in range(70, len(line)):
        b = line[i]
        c = chr(b) if 0x20 <= b < 0x7F else f"\\x{b:02x}"
        print(f"  {i:3}: 0x{b:02x} ({c})")
    print()
