#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()
# Line 318 (index 317): remove 1 extra )
line = lines[317]
# 当前末尾: `被拦截"))))));`  (5 个 ))
# 期望: 4 个 )   -> `被拦截")))));` (4 个 ))
# 检查是否以 `)))));`  结尾
if line.rstrip().endswith(")))));"):
    new_line = line.replace(")))));", ")))));")  # 把 5 变 4
    # 上面 replace 不变。手动数
    stripped = line.rstrip("\n")
    # 数末尾的 )
    count = 0
    for c in reversed(stripped):
        if c == ")":
            count += 1
        else:
            break
    print(f"Trailing ): {count}")
    if count == 5:
        new_line = stripped[:-1] + "\n"
        lines[317] = new_line
        print(f"After: {new_line!r}")
        with open(src, "w", encoding="utf-8", newline="\n") as f:
            f.writelines(lines)
        print("Fixed")
else:
    print(f"Skip, end: {line.rstrip()[-20:]!r}")
