#!/usr/bin/env python3
"""直接修复 sites.rs L79 - 用 Python 字符串操作"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()
lines = content.split("\n")
# 修复 L79 (index 78)
line = lines[78]
print(f"L79 before: {line!r}")
# 现在 line 末尾是 ??????\")  + ;  (5 个 )) + ")
# 实际: ?????)))\";  错误。当前 line 显示 "")))))；
# 期望: "")))))；  (4 个 )) + ")
# 移除倒数第 2 个 )
# 找到倒数第 2 个 )  在 ;  之前
# 简单方法: 数末尾的 )，如果是 5，删除 1 个
stripped = line
count = 0
for c in reversed(stripped):
    if c == ")":
        count += 1
    else:
        break
print(f"Trailing ): {count}")
if count == 5:
    # 找倒数第 2 个 )  位置
    # 跳过 ;  和最后 1 个 )  后再跳过 1 个 )
    idx = len(stripped) - 2  # ;  之前
    idx = stripped.rfind(")", 0, idx)  # 倒数第 2 个 )
    print(f"Removing ) at index {idx}")
    new_line = stripped[:idx] + stripped[idx+1:]
    print(f"L79 after: {new_line!r}")
    lines[78] = new_line
    with open(src, "w", encoding="utf-8", newline="") as f:
        f.write("\n".join(lines))
    print("Saved")
