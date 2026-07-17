#!/usr/bin/env python3
"""修复 sites.rs L79: 移除 1 个多余的 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
line = lines[78]
print(f"L79 before ({len(line)} bytes):")
print(line.decode("utf-8", errors="replace"))
# 当前末尾: ?)));  - 我之前加错了，现在是 "")))))；
# 期望: "")")))；  (即 "" + 4 个 ) + ;)
# 找到 "")))))；  替换为 "")")))；
stripped = line.rstrip(b"\n")
# 末尾是 "")")))；  (7 bytes) - 1 """ + 5 ") + 1 ";
# 改为 "")))))；  ?  5 末尾 应该是 "" + 4 ) + ;
# 区别: 1 个 )
# 找第一个 ?  之后 数 末尾的 ) 数量
# 但我们已知: stripped 末尾是 "")")))；  (5 个 ))"
# 直接 strip 末尾 1 个 )
# stripped[:-1] 是 7 字符末尾去掉 ;
# 上面去 ;  后 6 字符: "")")))  (5 个 ))
# 去掉 1 个 )  -> "")))))  (4 个 ))
# 加上 ;  -> "")")))；  (6 字符)
# 整个 stripped 是 108 字节，末尾 7 字节
# 改为 末尾 6 字节
new_stripped = stripped[:-7] + b'")))));'  # 1 " + 4 ) + ;
new_line = new_stripped + b"\n"
print(f"New end: {new_stripped[-15:]!r}")
print(f"L79 after ({len(new_line)} bytes):")
print(new_line.decode("utf-8", errors="replace"))
lines[78] = new_line
with open(src, "wb") as f:
    f.write(b"\n".join(lines))
print("Saved")
