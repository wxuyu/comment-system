#!/usr/bin/env python3
"""查找文件中损坏的 ?)) 和 ?" 模式。"""
import os
import sys
import io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        with open(p, "r", encoding="utf-8") as fp:
            for i, line in enumerate(fp, 1):
                if "?" in line and ("))" in line or '",' in line):
                    # 检查是否是损坏模式
                    if "?))" in line or '?",' in line or '?)' in line and '?\\' not in line:
                        print(f"{p}:{i}: {line.rstrip()}")
