#!/usr/bin/env python3
"""查找所有 FFFD 字符 (0xEF 0xBF 0xBD) 在源码中"""
import os
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        with open(p, "rb") as f:
            content = f.read()
        # 找 FFFD
        fffd = b"\xef\xbf\xbd"
        positions = []
        i = 0
        while True:
            idx = content.find(fffd, i)
            if idx < 0:
                break
            positions.append(idx)
            i = idx + 1
        if positions:
            print(f"{n}: {len(positions)} FFFD characters")
            for pos in positions[:5]:
                # 显示前后 20 字节
                start = max(0, pos - 20)
                end = min(len(content), pos + 20)
                # 找到包含这个位置的行
                line_start = content.rfind(b"\n", 0, pos) + 1
                line_end = content.find(b"\n", pos)
                if line_end < 0:
                    line_end = len(content)
                line_no = content[:pos].count(b"\n") + 1
                line = content[line_start:line_end].decode("utf-8", errors="replace")
                print(f"  L{line_no} byte {pos}: ...{line[max(0, pos-line_start-15):pos-line_start+15]}...")
