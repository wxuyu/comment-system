#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()
line = lines[283].rstrip("\n")
print("Before:", repr(line))
n = len(line) - 1
count = 0
while n >= 0 and line[n] == ")":
    count += 1
    n -= 1
print(f"Trailing ) count: {count}")
if count >= 3:
    # remove extra trailing )s, keep 2
    keep = 2
    new_line = line[: n + 1] + ")" * keep + "\n"
    print("After: ", repr(new_line))
    lines[283] = new_line
    with open(src, "w", encoding="utf-8", newline="\n") as f:
        f.writelines(lines)
    print("Fixed")
else:
    print(f"Skip, count={count}")
