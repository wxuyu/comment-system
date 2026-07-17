#!/usr/bin/env python3
import os
import io
import sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

SRC = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# 修复 comments.rs 最后 2 个 FFFD
p = os.path.join(SRC, "routes", "comments.rs")
with open(p, "r", encoding="utf-8") as f:
    t = f.read()

# 原始: 新评\ufffd?.to_string() → 新评论\".to_string()
t = t.replace("新评\ufffd?.to_string()", "新评论\".to_string()")
# 原始: 订阅\ufffd?, → 订阅者\",
t = t.replace("订阅\ufffd?,", "订阅者\",")

with open(p, "w", encoding="utf-8", newline="\n") as f:
    f.write(t)

print("Done. Remaining FFFD:", t.count("\ufffd"))
