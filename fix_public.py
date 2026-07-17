#!/usr/bin/env python3
"""修复 public.rs L83 (缩进)、L98 (?  -> ")、L123 (?  -> ")"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\public.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()
lines = content.split("\n")

# L83: 修复缩进
old_l83 = "if state.mailer.is_configured() {"
new_l83 = "        if state.mailer.is_configured() {"
if lines[82] == old_l83:
    lines[82] = new_l83
    print("L83: fixed indentation")
else:
    print(f"L83: skip, got: {lines[82]!r}")

# L98: ?  改为 "
old_l98 = lines[97]
if old_l98.rstrip().endswith('?'):
    lines[97] = old_l98.rstrip() + '"'
    print(f"L98: ?  -> \"  -> {lines[97]!r}")
else:
    print(f"L98: skip, {old_l98!r}")

# L123: ?  改为 "
old_l123 = lines[122]
if old_l123.rstrip().endswith('?,'):
    # ?  改为 "
    lines[122] = old_l123.rstrip()[:-1] + '",'  # 移除 ?  加 \",
    print(f"L123: ?, -> \",  -> {lines[122]!r}")
else:
    print(f"L123: skip, {old_l123!r}")

with open(src, "w", encoding="utf-8", newline="") as f:
    f.write("\n".join(lines))
print("Saved")
