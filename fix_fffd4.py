#!/usr/bin/env python3
"""修复 comment-system 中 FFFD+? 的复合损坏模式。"""
import os
import sys
import io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

SRC_DIR = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# FFFD + ? 模式的修复
# (pattern_包含_FFFD_和_?_字符, 正确字符串)
FIXES = [
    # comments.rs
    ("需要指\ufffd?page_id \ufffd?page_url", "需要指定 page_id 或 page_url"),
    ("昵称长度需\ufffd?1-50 字符之间", "昵称长度需要在 1-50 字符之间"),
    ("评论内容需\ufffd?1-5000 字符之间", "评论内容需要在 1-5000 字符之间"),
    ("(\"新评\ufffd?\"", "(\"新评论\""),
    ("\"订阅\ufffd?\", &nickname", "\"订阅者\", &nickname"),
    ("\"订阅\ufffd?\",", "\"订阅者\","),
    ("对\ufffd?DB 表", "对应 DB 表"),
    # email.rs
    ("缺少 email \ufffd?site 参数", "缺少 email 或 site 参数"),
    # public.rs
    ("//! 验证\ufffd?/ 通知", "//! 验证码/ 通知"),
    # captcha.rs - 注释里的数学题示例
    ("`3 + 5 = ?`\ufffd?//! 2. 自建 image captcha\ufffd? 6 位数字", "`3 + 5 = ?`）\n//! 2. 自建 image captcha（6 位数字"),
    ("`3 + 5 = ?`\ufffd?//! 2. 自建 image captcha\ufffd?", "`3 + 5 = ?`）\n//! 2. 自建 image captcha（"),
    ("`3 + 5 = ?`\ufffd?", "`3 + 5 = ?`）"),
    ("image captcha\ufffd? 6 位数字", "image captcha（6 位数字"),
    ("image captcha\ufffd?", "image captcha（"),
    # captcha.rs - 用 id 作为 salt
    ("// \ufffd?id 作为 salt", "// 用 id 作为 salt"),
    ("// \ufffd?id 作为 salt", "// 用 id 作为 salt"),
    # oauth.rs
    ("配置（对\ufffd?DB 表）", "配置（对应 DB 表）"),
    ("对\ufffd?DB 表", "对应 DB 表"),
]


def fix_file(path):
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    original = text
    applied = []
    for old, new in FIXES:
        if old in text:
            text = text.replace(old, new)
            applied.append(old[:30] + "...")
    if text != original:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        remaining = text.count("\ufffd")
        print(f"  Fixed: {os.path.basename(path)} (applied {len(applied)}, remaining FFFD: {remaining})")
        return True
    return False


def main():
    if not os.path.isdir(SRC_DIR):
        print(f"Source dir not found: {SRC_DIR}")
        sys.exit(1)
    total = 0
    for root, _, files in os.walk(SRC_DIR):
        for name in files:
            if not name.endswith(".rs"):
                continue
            path = os.path.join(root, name)
            if fix_file(path):
                total += 1
    print(f"\nTotal fixed: {total}")


if __name__ == "__main__":
    main()
