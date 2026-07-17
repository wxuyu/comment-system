#!/usr/bin/env python3
"""修复 email.rs 中 4 处 "中文?  改为 "中文\""""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\email.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()
lines = content.split("\n")
# 修复 L32, L50, L74, L90 (索引 -1)
for line_no in [32, 50, 74, 90]:
    line = lines[line_no - 1]
    print(f"L{line_no} before: {line!r}")
    # 模式: 末尾是 ?  (没有 ")
    if line.rstrip().endswith('?'):
        # 末尾加 "
        new_line = line.rstrip() + '"'
        print(f"L{line_no} after:  {new_line!r}")
        lines[line_no - 1] = new_line
    else:
        print(f"L{line_no} skip (no ?)")
with open(src, "w", encoding="utf-8", newline="") as f:
    f.write("\n".join(lines))
print("Saved")
