#!/usr/bin/env python3
"""修复缺 ) 的行: 行尾是 ? 的需要加 )."""
import os
import re
import sys
import io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# 已知需要修复的行
FIXES = {
    "db.rs": [
        (101, "    Ok(Some(row_str(row, idx)?))\n"),
        (196, "        Ok(Some(T::from_row(&row)?))\n"),
    ],
}


def fix_file(path):
    fname = os.path.basename(path)
    if fname not in FIXES:
        return False
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    changed = False
    for line_num, new_line in FIXES[fname]:
        if lines[line_num - 1] != new_line:
            print(f"  Line {line_num}: {lines[line_num-1]!r} -> {new_line!r}")
            lines[line_num - 1] = new_line
            changed = True
    if changed:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.writelines(lines)
        print(f"Fixed: {fname}")
    return changed


for root, _, files in os.walk(src):
    for n in files:
        if not n.endswith(".rs"):
            continue
        fix_file(os.path.join(root, n))
