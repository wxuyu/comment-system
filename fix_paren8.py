#!/usr/bin/env python3
"""修复 line 284: 移除多余的 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"

with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()

# Line 284 (index 283): Json(ApiResponse::error(404, "评论不存在"))),
# 当前是 3 个 ) + 逗号，需要 2 个 ) + 逗号
line = lines[283]
print(f"Before: {line!r}")
# 找 "评论不存在"))),  -> "评论不存在")),
# 因为中文被 replace 后的 ? 占位了，但实际行就是 3 个 ) 模式
# 找 ?)))),  替换为 ?)),
# 假设有 ? 标记（来自损坏），但本行没有，应该是 3 个 ) 后面跟逗号
# 我用字符检查
# 末尾是 ))),  -> )), 
if line.rstrip().endswith('))),'):
    new_line = line.rstrip()[:-1] + '\n'  # 去掉最后一个 )
    lines[283] = new_line
    print(f"After:  {new_line!r}")
    with open(src, "w", encoding="utf-8", newline="\n") as f:
        f.writelines(lines)
    print("Fixed")
else:
    print("Pattern not matched, current end:", repr(line[-20:]))
