#!/usr/bin/env python3
"""修复 line 156 和 165 的多余 ) 。"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"

with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()

# Line 155: ?))) -> ?))  (remove one close)
# Line 156: ));  -> );   (remove one close)
# Line 164: same as 155
# Line 165: same as 156

# 当前 155: ...  "创建页面失败"))), -> ... "创建页面失败"))  + 保持逗号
lines[154] = lines[154].replace('"))),\n', '"))\n')
# 当前 156: )); -> );
lines[155] = lines[155].replace('));', ');')
# 当前 164: ?"))),  →  ?"))
lines[163] = lines[163].replace('"))),\n', '"))\n')
# 当前 165: )); -> );
lines[164] = lines[164].replace('));', ');')

with open(src, "w", encoding="utf-8", newline="\n") as f:
    f.writelines(lines)
print("Fixed")
