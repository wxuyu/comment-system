#!/usr/bin/env python3
"""修复 lines 313, 318: ?))); -> ))));  (把孤立的 ? 替换为 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()

# Line 313 (index 312): 验证码错误或已过期?))); -> 验证码错误或已过期"))))；
# Line 318 (index 317): 已被拦截?))); -> 已被拦截"))));

# 改用字符串模式: 找 ?))); 模式 (3 个 ) 加 ; 在中文之后)
import re
for i, line in enumerate(lines, 1):
    if i in (313, 318):
        old = line
        # 替换 ?)));  -> "))));  
        # 因为前面是 中文(过期) 或 中文(拦截)，后面应该先是 " 关闭字符串
        # 修复: ? 变 )  即可：?)));  -> ))))；
        new = line.replace("?)));", ")))));")
        if new != old:
            print(f"L{i}: {old.rstrip()!r} -> {new.rstrip()!r}")
            lines[i - 1] = new

with open(src, "w", encoding="utf-8", newline="\n") as f:
    f.writelines(lines)
print("Done")
