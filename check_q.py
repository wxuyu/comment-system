#!/usr/bin/env python3
"""查找行尾是 ? 的行（可能缺 )）。"""
import os
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        with open(p, "r", encoding="utf-8") as f:
            lines = f.readlines()
        for i, line in enumerate(lines, 1):
            stripped = line.rstrip()
            if stripped.endswith("?)") and not stripped.endswith("??)"):
                # 检查这行是否有不匹配的括号
                opens = line.count("(")
                closes = line.count(")")
                if opens > closes:
                    print(f"{p}:{i} (o={opens} c={closes}): {stripped}")
