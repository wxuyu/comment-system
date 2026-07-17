#!/usr/bin/env python3
"""修复 db.rs 中 ?") 模式（多了一个 "）。"""
import os
import re
import sys
import io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"


def fix_file(path):
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    original = text
    # ?") 是错误模式：多了 " 和一个 )
    text = text.replace('?")', '?')
    # 同时可能多一个 )
    # 但要先检查是不是影响了 ) 计数
    if text != original:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        print(f"Fixed: {os.path.basename(path)}")
        return True
    return False


total = 0
for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        p = os.path.join(root, n)
        if fix_file(p):
            total += 1
print(f"\nTotal: {total}")
