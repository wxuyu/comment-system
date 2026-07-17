#!/usr/bin/env python3
"""修复 sites.rs 和其他文件中的中文? (0x3F) 替换为 中文： (U+FF1A) 或其他合适字符"""
import os
src_dir = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
# 中文字符串中不应该有 ASCII ?  (那是 ? 操作符)
# 我们的 0x3F 在中文文本里通常是损坏的全角字符
# 常见损坏: ： (全角冒号) -> ?, ， (全角逗号) -> ?, 。 (全角句号) -> ? 等
# 修复策略: 在中文字符串中将 0x3F 替换为 ，
target = "，".encode("utf-8")
for root, _, files in os.walk(src_dir):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        with open(p, "rb") as f:
            content = f.read()
        original = content
        # 找所有 0x3F 位置
        positions = []
        i = 0
        while True:
            idx = content.find(b"\x3f", i)
            if idx < 0:
                break
            positions.append(idx)
            i = idx + 1
        if not positions:
            continue
        print(f"\n{n}: {len(positions)} 0x3F chars")
        for pos in positions:
            # 找上下文
            start = max(0, pos - 15)
            end = min(len(content), pos + 15)
            line_start = content.rfind(b"\n", 0, pos) + 1
            line_end = content.find(b"\n", pos)
            if line_end < 0:
                line_end = len(content)
            line_no = content[:pos].count(b"\n") + 1
            line = content[line_start:line_end].decode("utf-8", errors="replace")
            col = pos - line_start + 1
            # 检查前后是否是非ASCII（中文）
            before = content[max(0, pos-3):pos]
            after = content[pos+1:min(len(content), pos+4)]
            is_cjk = any(b > 0x7F for b in before) and any(b > 0x7F for b in after)
            print(f"  L{line_no} col {col}: CJK={is_cjk}  | ...{line[max(0,col-15):col+15]}...")
            if is_cjk:
                # 替换为 ，
                content = content[:pos] + target + content[pos+1:]
                print(f"    -> replaced with ，")
        if content != original:
            with open(p, "wb") as f:
                f.write(content)
            print(f"  Saved")
