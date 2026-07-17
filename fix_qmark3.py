#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()
line = lines[317]
stripped = line.rstrip("\n")
# 检查是否以 ?)));  模式结尾（坏）或 ?????)))); (好)
print(f"Current end: {stripped[-20:]!r}")
# 找 ?))); 在哪里
# 找最后一个 "  然后数之后到 ; 之间的 )
quote_pos = stripped.rfind('"')
print(f"Last quote at: {quote_pos}")
rest = stripped[quote_pos+1:]
print(f"After quote: {rest!r}")
# rest 应该是 ?)));  (4 个 )) 加 ;)
# 数 ) 数量
closes = rest.count(")")
print(f"After quote: opens={rest.count('(')}, closes={closes}, semicolons={rest.count(';')}, questions={rest.count('?')}")
# 期望 closes = 4 (error, Json, tuple, Err) semicolons = 1
if closes > 4:
    # 移除多余 )
    extra = closes - 4
    # 替换 ?)));  模式
    # 找 rest 末尾的 );  把多余 ) 移除
    new_rest = rest
    for _ in range(extra):
        # 移除最后一个 )  在 ; 之前
        idx = new_rest.rfind(")")
        if idx >= 0:
            new_rest = new_rest[:idx] + new_rest[idx+1:]
    new_line = stripped[:quote_pos+1] + new_rest + "\n"
    print(f"New end: {new_line[-20:]!r}")
    lines[317] = new_line
    with open(src, "w", encoding="utf-8", newline="\n") as f:
        f.writelines(lines)
    print("Fixed")
