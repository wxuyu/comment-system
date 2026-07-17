#!/usr/bin/env python3
"""修复 captcha.rs 中所有 .context("中文?)?  模式"""
import re
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()
# 找 .context("...失败?)?  模式，替换为 .context("...失败")?;
# 模式: .context(" + 中文 + ?)?  -> .context("中文")?;
new_content = re.sub(r'\.context\("([^"]*?)\?\)\?;', r'.context("\1")?;', content)
# 找 .context("...失败?)?  模式 (包含中文)
# 模式: .context("中文" + ?)?  -> .context("中文")?;  (中文可能不含 ")
count = content.count('.context("') - new_content.count('.context("')
print(f"Replaced {count} patterns")
# 找其他模式: ".中文?"  -> ".中文"  (字符串中的 ?  应该是 ")
# 模式: "中文?)?  -> "中文")?;  (多余括号也要去掉)
# 简化: 找 "中文?)?  替换为 "中文")?;  (按行处理)
lines = new_content.split("\n")
fixed = 0
for i, line in enumerate(lines, 1):
    # 找 "中文?)?  或  "中文?))  模式
    if '"' in line:
        # 计算引号
        quote_count = line.count('"')
        if quote_count % 2 == 1:
            # 末尾加 "
            # 但要小心：可能在注释里
            stripped = line.rstrip()
            # 检查最后非空白字符
            if not stripped.endswith('"'):
                # 加 "
                lines[i-1] = line.rstrip() + '"' + line[len(stripped):]
                print(f"L{i}: added closing quote")
                fixed += 1
print(f"Fixed {fixed} unbalanced lines")
with open(src, "w", encoding="utf-8", newline="") as f:
    f.write("\n".join(lines))
print("Saved")
