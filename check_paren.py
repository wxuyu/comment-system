#!/usr/bin/env python3
"""查找 ApiResponse::error 后少 ) 的行。"""
import os
import re

src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
issues = []
for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        with open(p, "r", encoding="utf-8") as f:
            for i, line in enumerate(f, 1):
                m = re.search(r'ApiResponse::error\(\d+,\s*"[^"]*"\)\s*', line)
                if m:
                    after = line[m.end():]
                    close_count = 0
                    for c in after:
                        if c == ')':
                            close_count += 1
                        else:
                            break
                    if close_count < 2:
                        issues.append((p, i, close_count, line.rstrip()))
for p, i, n, l in issues:
    print(f"{p}:{i} closes={n}: {l}")
print(f"\nTotal: {len(issues)}")
