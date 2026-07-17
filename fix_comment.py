#!/usr/bin/env python3
"""修复 line 116: 把 ?\n 替换回 \n (让 // 注释在该行结束)"""
import os
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# 找所有 "// 中文?let ..." 形式的行
import re
fixed = 0
for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        with open(p, "r", encoding="utf-8") as f:
            content = f.read()
        # 模式: // [中文]?空格[代码]
        # 替换 ?[space]+[code] 为 \n[code]
        new_content = re.sub(
            r'(//[^\n]*?)\?(\s+)([a-zA-Z])',
            lambda m: m.group(1) + '\n' + ' ' * (len(m.group(1)) - 1) + m.group(3) if False else m.group(1) + '\n' + m.group(3),
            content
        )
        # 上面 regex 不好，简化
        # 找 // ...? ... 这种，保留 // 注释，把 ? 替换为 \n
        new_content = re.sub(
            r'//([^\n]*)\?\s*(\S)',
            lambda m: f'//{m.group(1)}\n{m.group(2)}',
            content
        )
        if new_content != content:
            with open(p, "w", encoding="utf-8", newline="\n") as f:
                f.write(new_content)
            print(f"Fixed: {n}")
            fixed += 1
print(f"\nTotal: {fixed}")
