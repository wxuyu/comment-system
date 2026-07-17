#!/usr/bin/env python3
"""在所有使用 params! 的 .rs 文件中加 use libsql::params;"""
import os
import re

# 需要处理的文件
files = [
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\oauth.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\public.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\email.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\pages.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\admin.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs",
]

for path in files:
    with open(path, "r", encoding="utf-8", newline="") as f:
        content = f.read()
    
    # 检查是否已导入
    if "use libsql::params" in content or "use libsql::{params" in content or "use libsql::{" in content and "params" in content:
        # 看是否真的导入了 params
        if re.search(r"use libsql::\{[^}]*\bparams\b", content) or re.search(r"^use libsql::params;", content, re.MULTILINE):
            print(f"{os.path.basename(path)}: already has params import")
            continue
    
    # 添加 use libsql::params; 在第一个 use 语句之后
    # 找第一个 use ...; 行
    lines = content.split("\n")
    new_lines = []
    inserted = False
    for i, line in enumerate(lines):
        new_lines.append(line)
        if not inserted and line.startswith("use ") and ";" in line:
            # 检查是否已经有 libsql:: 导入
            if "libsql::" in line:
                # 在 libsql:: 那一行加 params
                # 例如: use libsql::Builder -> use libsql::{params, Builder}
                m = re.match(r"use libsql::\{?([^;]*?)\}?;", line)
                if m:
                    items = m.group(1).strip()
                    if "params" not in items:
                        if "{" in line:
                            new_lines[-1] = f"use libsql::{{{items}, params}};"
                        else:
                            new_lines[-1] = f"use libsql::{{{items}, params}};"
                        inserted = True
                        print(f"{os.path.basename(path)}: added params to existing libsql import")
            else:
                # 在下一行插入
                new_lines.append("use libsql::params;")
                inserted = True
                print(f"{os.path.basename(path)}: added separate use libsql::params")
    
    if inserted:
        with open(path, "w", encoding="utf-8", newline="") as f:
            f.write("\n".join(new_lines))
    else:
        print(f"{os.path.basename(path)}: no use statement found, skipping")
