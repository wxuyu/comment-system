#!/usr/bin/env python3
"""修复 sites.rs L79: 移除 1 个多余的 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
line = lines[78]
print(f"L79 before ({len(line)} bytes):")
print(line.decode("utf-8", errors="replace"))
# 找 ?)));  模式 (应该已被修改为 "")"))); )
# 当前是 ""))))) ;  -> "")))));  (移除 1 个 ))
# 末尾 bytes: ?)));  -> 已改为 "")")));  (5 )))
# 简化：找到 末尾的 ")))))  替换为 ")))) 
stripped = line.rstrip(b"\n")
if stripped.endswith(b'")))));'):
    new_stripped = stripped[:-2] + b');'  # 移除 1 个 )
    # 上面去掉了最后 2 字符  + 加 2 字符  不变
    # 实际：stripped 是 109 字节，末尾 7 字节是 "")))))；
    # 改为 6 字节 "")"));  ：即 5 字节（"） + 4 字节（)） + 1 字节（;）
    # 应该是  "")))))；  -> "")))))；
    # 6 bytes: ") + 4) + ;
    new_stripped = stripped[:-1]  # 移除 ;
    new_stripped = new_stripped[:-1]  # 移除 1 个 )
    new_stripped += b');'  # 加 );  -> 1 个 ) 和 1 个 ;
    print(f"New end: {new_stripped[-20:]!r}")
    new_line = new_stripped + b"\n"
    lines[78] = new_line
    print(f"L79 after ({len(new_line)} bytes):")
    print(new_line.decode("utf-8", errors="replace"))
    with open(src, "wb") as f:
        f.write(b"\n".join(lines))
    print("Saved")
else:
    print(f"Skip, end: {stripped[-15:]!r}")
