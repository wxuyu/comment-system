#!/usr/bin/env python3
"""直接修复 sites.rs L79 - 数所有 () 然后找不匹配的位置"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\sites.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()
lines = content.split("\n")
# 修复 L79 (index 78)
line = lines[78]
print(f"L79 before: {line!r}")
# 数 ( 和 )
opens = line.count("(")
closes = line.count(")")
print(f"opens={opens} closes={closes}")
# 期望: opens=closes
# 如果 closes=opens+1, 移除最后 1 个 )
if closes == opens + 1:
    # 移除最后 1 个 )
    idx = line.rfind(")")
    new_line = line[:idx] + line[idx+1:]
    print(f"L79 after: {new_line!r}")
    lines[78] = new_line
    with open(src, "w", encoding="utf-8", newline="") as f:
        f.write("\n".join(lines))
    print("Saved")
