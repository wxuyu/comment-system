#!/usr/bin/env python3
"""修复 sites.rs L79: 把 ?)))  改为 "")")))  - 加 关闭字符串的 " 和 1 个 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# 修复 L79 (index 78)
line = lines[78]
print(f"L79 before ({len(line)} bytes):")
print(line.decode("utf-8", errors="replace"))
# 找 ?  位置（ASCII ?  0x3F）
# 已知 L79 在 byte 102 是 ? 
# 替换 byte 102 的 ?  为 "，并在 byte 106 前插入 1 个 )
# 简化: 把 bytes 102-106 (?))) ; 5 bytes) 替换为 "")") ; 7 bytes)
old_bytes = line[102:107]  # ?))) ;
print(f"Bytes 102-106: {old_bytes!r}")
new_bytes = b'")))));'
new_line = line[:102] + new_bytes + line[107:]
print(f"L79 after ({len(new_line)} bytes):")
print(new_line.decode("utf-8", errors="replace"))
lines[78] = new_line
# 写回
with open(src, "wb") as f:
    f.write(b"\n".join(lines))
print("Saved")
