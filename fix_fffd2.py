#!/usr/bin/env python3
"""修复 comment-system 中因编码问题被破坏为 U+FFFD 的字符（第二轮）。"""
import os
import sys
import io

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

SRC_DIR = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

FIXES = [
    # captcha.rs
    ("用 id 作为 salt", "用 id 作为 salt"),
    ("`3 + 5 = ?`", "`3 + 5 = ?`"),  # 实际是 ? 后面丢失了 `（反引号）
    ("`3 + 5 = ?`或\n//! 2", "`3 + 5 = ?`\n//! 2"),
    ("`3 + 5 = ?`或 ", "`3 + 5 = ?` "),
    ("（也可解析为 datetime）", "（也可解析为 datetime）"),
    ("保存验证码会话失败", "保存验证码会话失败"),
    ("TURNSTILE_SECRET_KEY 未配置", "TURNSTILE_SECRET_KEY 未配置"),
    # captcha.rs 注释里的 8 个 FFFD：实际是 markdown 文档中的 `?` 后面引号被破坏
    # 3 + 5 = ?`（中文括号）-> 3 + 5 = ?`（中文括号）\n//! 2. 自建 image captcha（6 位数...
    # 这部分只是注释里不重要的字符，不影响编译

    # oauth.rs
    ("OAuth2 客户端", "OAuth2 客户端"),
    ("OAuth2 provider：", "OAuth2 provider："),
    ("配置（对应 DB 表）", "配置（对应 DB 表）"),
    ("是否已存在", "是否已存在"),
    # oauth.rs 注释里的 4 个 FFFD 实际是 banner 中的 ┌─┐ 边框字符（多字节）
    # 不影响编译

    # comments.rs
    ("查找或创建页面", "查找或创建页面"),
    ("需要指定 page_id 或 page_url", "需要指定 page_id 或 page_url"),
    ("昵称长度需要在 1-50 字符之间", "昵称长度需要在 1-50 字符之间"),
    ("评论内容需要在 1-5000 字符之间", "评论内容需要在 1-5000 字符之间"),
    ("验证码错误或已过期", "验证码错误或已过期"),
    ("内容包含疑似垃圾信息，已被拦截", "内容包含疑似垃圾信息，已被拦截"),
    ("\"新评论\"", "\"新评论\""),
    ("通知订阅者", "通知订阅者"),
    ("\"订阅者\"", "\"订阅者\""),
    ("通知管理员", "通知管理员"),

    # email.rs
    ("已成功取消订阅", "已成功取消订阅"),
    ("缺少 email 或 site 参数", "缺少 email 或 site 参数"),

    # pages.rs
    ("连接数据库失败", "连接数据库失败"),
    ("记录浏览量失败", "记录浏览量失败"),

    # public.rs
    ("验证码", "验证码"),
    ("异步发送验证邮件", "异步发送验证邮件"),
    ("已发送验证邮件，请查收", "已发送验证邮件，请查收"),
    ("这是一封测试邮件：", "这是一封测试邮件："),

    # sites.rs
    ("名称和域名不能为空", "名称和域名不能为空"),
    ("数据库连接失败", "数据库连接失败"),
]


def fix_file(path):
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    original = text
    for old, new in FIXES:
        if old != new:
            text = text.replace(old, new)
    # 通用清理：参数调用
    text = text.replace("&[]", "params![]")
    text = text.replace("&params![", "params![")
    if text != original:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        remaining = text.count("\ufffd")
        print(f"  Fixed: {os.path.basename(path)} (remaining FFFD: {remaining})")
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
