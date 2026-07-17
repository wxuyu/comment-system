#!/usr/bin/env python3
"""修复 comment-system 中因编码问题被破坏为 U+FFFD 的字符（第三轮）。

直接从上下文生成替换表。
"""
import os
import sys
import io
import unicodedata

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

SRC_DIR = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# 替换规则：包含 \ufffd 的字符串 → 正确的中文
# (pattern_with_fffd, replacement)
FIXES = [
    # captcha.rs
    ("math captcha（数学题，如 `3 + 5 = ?`\ufffd//! 2. 自建 image captcha\ufffd 位数字）\n//! 3. Cloudflare Turnstile",
     "math captcha（数学题，如 `3 + 5 = ?`）\n//! 2. 自建 image captcha（6 位数字）\n//! 3. Cloudflare Turnstile"),
    ("math captcha（数学题，如 `3 + 5 = ?`\ufffd//! 2. 自建 image captcha\ufffd",
     "math captcha（数学题，如 `3 + 5 = ?`）\n//! 2. 自建 image captcha（"),
    ("//! 验证码模块 - 支持多种验证码类型\n//!\n//! 1. 自建 math captcha（数学题，如 `3 + 5 = ?`）\n//! 2. 自建 image captcha（6 位数字）\n//! 3. Cloudflare Turnstile",
     "//! 验证码模块 - 支持多种验证码类型\n//!\n//! 1. 自建 math captcha（数学题，如 `3 + 5 = ?`）\n//! 2. 自建 image captcha（6 位数字）\n//! 3. Cloudflare Turnstile"),
    # 一般修复
    ("Self::hash(&code_str, &id); // \ufffdid 作为 salt", "Self::hash(&code_str, &id); // 用 id 作为 salt"),
    ("保存验证码会话失\ufffd", "保存验证码会话失败"),
    ("也可解析为 datetime\ufffd", "也可解析为 datetime）"),
    ("TURNSTILE_SECRET_KEY 未配\ufffd", "TURNSTILE_SECRET_KEY 未配置"),
    # 注：上面有 ? 字符在 Rust 字符串里需要转义
    # 但我们是在做字符串替换，不是写 Rust 字符串

    # oauth.rs
    ("通用 OAuth2 客户\ufffd", "通用 OAuth2 客户端"),
    ("支持任意标准 OAuth2 provider\ufffd", "支持任意标准 OAuth2 provider："),
    ("OAuth Provider 配置（对\ufffdDB 表）", "OAuth Provider 配置（对应 DB 表）"),
    ("是否已存\ufffd", "是否已存在"),
    # 注：剩余的 1 个 FFFD 在 banner 框中，不重要

    # comments.rs
    ("查找或创建页\ufffd", "查找或创建页面"),
    ("需要指\ufffd page_id \ufffd page_url", "需要指定 page_id 或 page_url"),
    ("昵称长度需\ufffd 1-50 字符之间", "昵称长度需要在 1-50 字符之间"),
    ("评论内容需\ufffd 1-5000 字符之间", "评论内容需要在 1-5000 字符之间"),
    ("验证码错误或已过\ufffd", "验证码错误或已过期"),
    ("内容包含疑似垃圾信息，已被拦\ufffd", "内容包含疑似垃圾信息，已被拦截"),
    ("\"新评\ufffd\"", "\"新评论\""),
    ("通知订阅\ufffd", "通知订阅者"),
    ("\"订阅\ufffd\"", "\"订阅者\""),
    ("通知管理\ufffd", "通知管理员"),

    # email.rs
    ("已成功取消订\ufffd", "已成功取消订阅"),
    ("缺少 email \ufffd site 参数", "缺少 email 或 site 参数"),

    # pages.rs
    ("连接数据库失\ufffd", "连接数据库失败"),
    ("记录浏览量失\ufffd", "记录浏览量失败"),

    # public.rs
    ("验证\ufffd/ 通知 / 邮件订阅相关公开 API", "验证码 / 通知 / 邮件订阅相关公开 API"),
    ("异步发送验证邮\ufffd", "异步发送验证邮件"),
    ("已发送验证邮件，请查\ufffd", "已发送验证邮件，请查收"),
    ("这是一封测试邮件\ufffd", "这是一封测试邮件："),

    # sites.rs
    ("名称和域名不能为\ufffd", "名称和域名不能为空"),
    ("数据库连接失\ufffd", "数据库连接失败"),
]


def fix_file(path):
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    original = text
    applied = 0
    for old, new in FIXES:
        if old == new:
            continue
        if old in text:
            text = text.replace(old, new)
            applied += 1
    if text != original:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        remaining = text.count("\ufffd")
        print(f"  Fixed: {os.path.basename(path)} (applied {applied}, remaining FFFD: {remaining})")
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
