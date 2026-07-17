#!/usr/bin/env python3
"""批量修复剩余编译错误"""
import re
import os

# ============================================================
# 1. 修复 db::fetch_optional::<R> -> db::fetch_optional::<R, _>
# ============================================================
files_with_generics = [
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\admin.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\pages.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\public.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\oauth.rs",
    r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs",
]
for path in files_with_generics:
    with open(path, "r", encoding="utf-8", newline="") as f:
        content = f.read()
    # 修复 fetch_optional::<R> -> fetch_optional::<R, _>
    new_content = re.sub(r'(db::fetch_optional::<)([A-Za-z0-9_]+)>', r'\1\2, _>', content)
    new_content = re.sub(r'(db::fetch_one::<)([A-Za-z0-9_]+)>', r'\1\2, _>', new_content)
    new_content = re.sub(r'(db::fetch_all::<)([A-Za-z0-9_]+)>', r'\1\2, _>', new_content)
    if new_content != content:
        with open(path, "w", encoding="utf-8", newline="") as f:
            f.write(new_content)
        print(f"{os.path.basename(path)}: fixed generics")

# ============================================================
# 2. 替换 &db::values_of(&[...])  -> params![...] 模式
# ============================================================
def replace_values_of(content, file_path):
    # 模式: &db::values_of(&[\n            &a, &b, &c,\n        ])
    # 替换为: params![a.clone(), b.clone(), c.clone()]
    # 但 a/b/c 可能是 String 或 i64 等
    
    pattern = re.compile(r'&db::values_of\(\&\[(.*?)\]\)', re.DOTALL)
    
    def replacer(m):
        items_str = m.group(1)
        # 解析 items (each starts with &)
        # 简单处理: 移除每个 & 前缀
        items = []
        for line in items_str.split('\n'):
            line = line.strip().rstrip(',').strip()
            if not line:
                continue
            if line.startswith('&'):
                line = line[1:].strip()
            # 添加 .clone() 但要小心 i64 等
            # 对所有项加 .clone() - 大部分类型实现了 Clone
            items.append(f"{line}.clone()")
        return f"params![{', '.join(items)}]"
    
    new_content = pattern.sub(replacer, content)
    if new_content != content:
        with open(file_path, "w", encoding="utf-8", newline="") as f:
            f.write(new_content)
        print(f"{os.path.basename(file_path)}: replaced values_of")

for path in files_with_generics:
    if not os.path.exists(path):
        continue
    with open(path, "r", encoding="utf-8", newline="") as f:
        content = f.read()
    if '&db::values_of(&[' in content:
        replace_values_of(content, path)

print("Done")
