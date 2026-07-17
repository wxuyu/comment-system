#!/usr/bin/env python3
"""修复 L313: 把 过期))))) 改为 过期"))))) - 实际是 过期))))) 应该是 过期")))))  (但我们想 过期")))))  + 4 个 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
line_313 = lines[312]
print(f"L313 bytes ({len(line_313)}):")
print(line_313.decode("utf-8", errors="replace"))
# 找过期 的位置
idx = line_313.find("过期".encode("utf-8"))
print(f"过期 at byte: {idx}")
# 过期 后面是 5 个 ) 加 ; (bytes 106-111)
# 我们要: 过期 + " + 4 个 ) + ;
# 所以在 过期 后面（idx+6）插入 " 并删除 1 个 )
# 即 bytes [106, 110] (5 个 )) 中保留 4 个，删除 1 个，并插入 "
# 简化: 把 byte 106 替换为 "（这样把第一个 ) 变成 "）
# 然后 bytes 107-110 是 4 个 ) ， 111 是 ;
# 但是这样 106 是 "  107-110 是 4 个 ) + 111 是 ; - 正确!
new_line = bytes(line_313[:106]) + b'"' + bytes(line_313[107:])
lines[312] = new_line
print(f"New L313:")
print(new_line.decode("utf-8", errors="replace"))
# 写回
with open(src, "wb") as f:
    f.write(b"\n".join(lines))
print("Saved")
