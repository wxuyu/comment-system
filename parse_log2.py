#!/usr/bin/env python3
import re
with open(r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\build17.log", "rb") as f:
    content = f.read()
text = content.decode("utf-8", errors="replace")
# 找包含 318 |  的行
matches = re.findall(r"318 \|.*", text)
for m in matches:
    print(m)
print(f"Total: {len(matches)}")
