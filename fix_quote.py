#!/usr/bin/env python3
"""修复 \"...)? 模式（应该是 \"...\"）。"""
import os
import re
import sys
import io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# 修复模式: "中文?))  → "中文"))  和 "中文?)))  → "中文")))
FIXES = [
    # comments.rs
    ('"数据库错误?)))', '"数据库错误"))'),
    ('"评论不存在?)),', '"评论不存在")),'),
    ('"验证码错误或已过期?)));', '"验证码错误或已过期")));'),
    ('"内容包含疑似垃圾信息，已被拦截?)));', '"内容包含疑似垃圾信息，已被拦截")));'),
    # pages.rs
    ('"服务器错误?)))', '"服务器错误"))'),
    # sites.rs
    ('"名称和域名不能为空?)));', '"名称和域名不能为空")));'),
    # 通用模式 - 处理任何 "...)? → "..." 
    (re.compile(r'"([^"]*\?)\)(\))'), r'"\1")\2'),
]


def fix_file(path):
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    original = text
    for old, new in FIXES:
        if isinstance(old, re.Pattern):
            text = old.sub(new, text)
        else:
            text = text.replace(old, new)
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
