#!/usr/bin/env python3
"""修复 comment-system 中因编码问题被破坏为 U+FFFD 的字符。

策略：直接处理 UTF-8 字节序列 EF BF BD，对常见的 FFFD 模式做上下文替换。
"""
import os
import sys
import re

SRC_DIR = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# (字节序列包含 FFFD 模式的字符串, 替换为)
# 使用 Python 原生字符串便于处理 UTF-8
FIXES = [
    # 通用词根替换（注意顺序：从长到短）
    # "邮?件" = "邮件"（F+1个字节）
    ("邮\ufffd件", "邮件"),
    # "邮?箱" = "邮箱"（F+箱）
    ("邮\ufffd箱", "邮箱"),
    ("邮\ufffd编", "邮编"),
    ("邮\ufffd戳", "邮戳"),
    ("邮\ufffd局", "邮局"),
    ("邮\ufffd费", "邮费"),
    ("邮\ufffd政", "邮政"),
    # 单词
    ("管理员路\ufffd", "管理员路由"),
    ("验证码模\ufffd", "验证码模块"),
    ("验证码配\ufffd", "验证码配置"),
    ("验证码状\ufffd", "验证码状态"),
    ("验证码图\ufffd", "验证码图片"),
    ("验证码生\ufffd", "验证码生成"),
    ("验证码校\ufffd", "验证码校验"),
    ("验证码验\ufffd", "验证码验证"),
    ("验证码失\ufffd", "验证码失败"),
    ("验证码错\ufffd", "验证码错误"),
    ("验证码正\ufffd", "验证码正确"),
    ("验证码已使\ufffd", "验证码已使用"),
    ("验证码已过\ufffd", "验证码已过期"),
    ("验证码无\ufffd", "验证码无效"),
    ("验证码过\ufffd", "验证码过期"),
    ("验证码不存\ufffd", "验证码不存在"),
    ("验证码不支\ufffd", "验证码不支持"),
    ("验证码类\ufffd", "验证码类型"),
    ("验证码使\ufffd", "验证码使用"),
    ("验证码创\ufffd", "验证码创建"),
    ("验证码获取", "验证码获取"),
    ("验证码数据", "验证码数据"),
    ("OAuth 认证模\ufffd", "OAuth 认证模块"),
    ("OAuth 登录路\ufffd", "OAuth 登录路由"),
    ("OAuth 认证失\ufffd", "OAuth 认证失败"),
    ("OAuth 状态失\ufffd", "OAuth 状态失败"),
    ("OAuth 回调失\ufffd", "OAuth 回调失败"),
    ("OAuth 提供商未配\ufffd", "OAuth 提供商未配置"),
    ("OAuth 授权码交换失\ufffd", "OAuth 授权码交换失败"),
    ("OAuth 访问令牌获取失\ufffd", "OAuth 访问令牌获取失败"),
    ("OAuth 用户信息获取失\ufffd", "OAuth 用户信息获取失败"),
    ("OAuth 状态已失\ufffd", "OAuth 状态已失效"),
    ("OAuth 状态验证失\ufffd", "OAuth 状态验证失败"),
    ("OAuth 提供商不支\ufffd", "OAuth 提供商不支持"),
    ("OAuth 重定\ufffd", "OAuth 重定向"),
    ("OAuth 登录失\ufffd", "OAuth 登录失败"),
    ("OAuth 登出失\ufffd", "OAuth 登出失败"),
    ("OAuth 绑定失\ufffd", "OAuth 绑定失败"),
    ("OAuth 解绑失\ufffd", "OAuth 解绑失败"),
    ("OAuth 状态无\ufffd", "OAuth 状态无效"),
    ("OAuth 状态已使\ufffd", "OAuth 状态已使用"),
    ("OAuth 状态已过\ufffd", "OAuth 状态已过期"),
    ("OAuth 状\ufffd", "OAuth 状态"),
    ("OAuth 配\ufffd", "OAuth 配置"),
    ("OAuth 模\ufffd", "OAuth 模块"),
    ("OAuth 不存\ufffd", "OAuth 不存在"),
    ("OAuth 已存\ufffd", "OAuth 已存在"),
    ("OAuth 未启\ufffd", "OAuth 未启用"),
    ("OAuth 已启\ufffd", "OAuth 已启用"),
    ("页面管理路\ufffd", "页面管理路由"),
    ("站点管理路\ufffd", "站点管理路由"),
    ("评论模块 - 包含公开 API 和管理 API", "评论模块 - 包含公开 API 和管理 API"),
    ("邮件订阅模\ufffd", "邮件订阅模块"),
    ("公开 API：站点查询、评论列表、验证码、邮\ufffd", "公开 API：站点查询、评论列表、验证码、邮件"),
    ("上传文件到 Vercel Blob", "上传文件到 Vercel Blob"),
    # 通用错误消息
    ("需要指\ufffd page_id \ufffd page_url", "需要指定 page_id 或 page_url"),
    ("评论不存\ufffd", "评论不存在"),
    ("数据库错\ufffd", "数据库错误"),
    ("查询失\ufffd", "查询失败"),
    ("操作失\ufffd", "操作失败"),
    ("无效请\ufffd", "无效请求"),
    ("需要登\ufffd", "需要登录"),
    ("未授\ufffd", "未授权"),
    ("服务器错\ufffd", "服务器错误"),
    ("用户不存在或密码错\ufffd", "用户不存在或密码错误"),
    ("用户已被封\ufffd", "用户已被封禁"),
    ("用户已注\ufffd", "用户已注册"),
    ("注册失\ufffd", "注册失败"),
    ("验证码错\ufffd", "验证码错误"),
    ("验证码已过\ufffd", "验证码已过期"),
    ("评论发布失\ufffd", "评论发布失败"),
    ("评论更新失\ufffd", "评论更新失败"),
    ("评论删除失\ufffd", "评论删除失败"),
    ("评论已存\ufffd", "评论已存在"),
    ("评论审核失\ufffd", "评论审核失败"),
    ("页面创建失\ufffd", "页面创建失败"),
    ("页面更新失\ufffd", "页面更新失败"),
    ("页面删除失\ufffd", "页面删除失败"),
    ("站点创建失\ufffd", "站点创建失败"),
    ("站点更新失\ufffd", "站点更新失败"),
    ("站点删除失\ufffd", "站点删除失败"),
    ("参数解\ufffd", "参数解析"),
    ("请求体解\ufffd", "请求体解析"),
    ("查询参数解\ufffd", "查询参数解析"),
    ("未授权访\ufffd", "未授权访问"),
    ("禁止访\ufffd", "禁止访问"),
    ("未找\ufffd", "未找到"),
    ("用户已存\ufffd", "用户已存在"),
    ("用户未注\ufffd", "用户未注册"),
    ("未审\ufffd", "未审核"),
    ("已审\ufffd", "已审核"),
    ("待审\ufffd", "待审核"),
    ("审核中", "审核中"),
    ("已通过", "已通过"),
    ("未通过", "未通过"),
    ("通过", "通过"),
    ("失败", "失败"),
    ("审核失\ufffd", "审核失败"),
    ("无效的审核状\ufffd", "无效的审核状态"),
    # 邮箱相关
    ("邮箱已被使用", "邮箱已被使用"),
    ("邮箱格式不合法", "邮箱格式不合法"),
    ("邮箱已存\ufffd", "邮箱已存在"),
    ("邮箱重复", "邮箱重复"),
    ("邮箱不存\ufffd", "邮箱不存在"),
    ("邮箱无\ufffd", "邮箱无效"),
    ("邮箱已验\ufffd", "邮箱已验证"),
    ("邮箱未验\ufffd", "邮箱未验证"),
    ("邮箱验\ufffd", "邮箱验证"),
    # 通用
    ("昵称已被使用", "昵称已被使用"),
    ("密码错误", "密码错误"),
    ("用户创建失败", "用户创建失败"),
    ("用户更新失败", "用户更新失败"),
    ("用户删除失败", "用户删除失败"),
    ("请稍后重试", "请稍后重试"),
    ("无效的排序字段", "无效的排序字段"),
    ("无效的排序方式", "无效的排序方式"),
    ("无效的评论ID", "无效的评论ID"),
    ("无效的页面ID", "无效的页面ID"),
    ("无效的站点ID", "无效的站点ID"),
    ("无效的状态", "无效的状态"),
    ("无效的操作", "无效的操作"),
    ("无效的请求", "无效的请求"),
    ("已删除", "已删除"),
    ("已存在", "已存在"),
    ("已禁用", "已禁用"),
    ("已启用", "已启用"),
    ("已批准", "已批准"),
    ("已拒绝", "已拒绝"),
    ("未批准", "未批准"),
    ("未拒绝", "未拒绝"),
    ("未删除", "未删除"),
    ("未禁用", "未禁用"),
    ("未启用", "未启用"),
]


def fix_file(path):
    with open(path, "rb") as f:
        raw = f.read()
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as e:
        print(f"  Cannot decode {path}: {e}")
        return False
    original = text
    for old, new in FIXES:
        text = text.replace(old, new)
    # 参数调用修复
    text = text.replace("&[]", "params![]")
    text = text.replace("&params![", "params![")
    if text != original:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        # 统计剩余 FFFD
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
