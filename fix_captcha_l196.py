#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()
lines = content.split("\n")
line = lines[195]
print(f"L196 before: {line!r}")
# ?);" -> ");
if line.rstrip().endswith('?);"'):
    new_line = line.rstrip()[:-4] + '");'
    print(f"L196 after:  {new_line!r}")
    lines[195] = new_line
with open(src, "w", encoding="utf-8", newline="") as f:
    f.write("\n".join(lines))
print("Saved")
