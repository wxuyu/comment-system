#!/usr/bin/env python3
"""修复 line 156 和 165 - 加回缺的 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"

with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()

# Line 155: 已经有 ))  OK
# Line 156: 需要 ));  (从 ); 改为 ));)
# Line 164: 已经有 ))  OK
# Line 165: 需要 ));  (从 ); 改为 ));)

lines[155] = lines[155].replace('                        );\n', '                        ));\n')
lines[164] = lines[164].replace('        );\n', '        ));\n')

with open(src, "w", encoding="utf-8", newline="\n") as f:
    f.writelines(lines)
print("Fixed")
