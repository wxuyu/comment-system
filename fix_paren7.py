#!/usr/bin/env python3
"""修复 line 284: 移除多余的 )"""
import re
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"

with open(src, "r", encoding="utf-8") as f:
    content = f.read()

# 找 "评论不存在" 后面跟着 ))) 的位置
m = re.search(r'(不存在"))\)\)\),', content)
print("Match:", m)
if m:
    # 把 3 个 ) 替换为 2 个 )
    new = m.group(1) + ")),"
    content = content[:m.start()] + new + content[m.end():]
    print("Replaced")

with open(src, "w", encoding="utf-8", newline="\n") as f:
    f.write(content)
