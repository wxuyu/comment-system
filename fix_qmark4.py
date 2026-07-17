#!/usr/bin/env python3
"""修复 line 318: 把 ???)));  替换为 "") ) ) ;  (加 关闭字符串的 " 和 缺的 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"
with open(src, "r", encoding="utf-8") as f:
    lines = f.readlines()
line = lines[317]
stripped = line.rstrip("\n")
# 找最后一个 "  位置
quote_pos = stripped.rfind('"')
rest = stripped[quote_pos+1:]
print(f"After last quote: {rest!r}")
# 当前 rest 应该是 ???))));  模式
# 期望: 4 个 ) 加 ;
# 数当前 )
closes = rest.count(")")
print(f"Closes: {closes}")
# 期望 4 个 ), 如果是 4 (说明我之前加回去了 1 个但还缺 ")，加 1 个 "
# 实际上是 4 (被我 fix_qmark3 改过), 我需要加回 "
if closes == 4:
    # 找 )???));  模式，在前面加 "
    # rest 末尾是 ???)); 
    # 第一个 ? 是字符内容（在中文里），不是我们要的
    # 我们要的是 rest 的开始
    # rest 全部是 ?????))));  但实际 rest 是 CJK
    # 实际 rest 是 截)))));  4 个 )
    # 所以问题不是 数量，而是没有 " 关闭字符串
    # 实际上我之前已经 fix_qmark3 把 5 改 4 了，但缺 "
    # 上面 quote_pos 是 72，但是 rfind 返回的是第一个 "
    # 实际是 string 的开始 "
    # 我需要 rfind 找的是 string close "，但只有 1 个 "
    # 所以 quote_pos 是 72（string open）
    # 没有 string close "
    pass
# 重新思考：corruption 是 ?????)  替换了 "")"  (3 字符)
# 现在 line 318 是 ????)"  4 个 ), no "
# 我加 1 个 " 然后 4 个 ) 加 ;
# 但 rest 是 ????)"  + ; 
# 实际现在的 rest 是 ????"  + ;  4 个 ) 加 ;
# 我把第一个 ? 替换为 "  ?
# 等等让我先打印实际内容
print(f"stripped last 30: {stripped[-30:]!r}")
# 找模式: 截)))));  -> 截"))))); (加 ")
# 实际末尾是 截)))));  (4 个 ))
# 应该改为 截"))))); (在第一个 ? 后加 ")
# 即 截" + 4 个 ) + ;
# 当前是 截 + 4 个 ) + ;   (缺 ")
# 在 截 后面加 "  即可
# stripped 倒数 5 字符: )))));  
# 倒数第 6 字符: 截 (中文字)
# 在 截 后面 (即倒数第 5 字符前) 加 "
# 即: 截 + " + )))));  -> 截")))));  (5 + 2 = 7 字符)
# 当前: 截 + )))));  = 截")))));  (5 字符)
# 缺的 1 个 " 加在 截 后面即可
# 但 截 是单字符，加 " 就是 截"  共 2 字符
# 实际: 截)))));  5 字符 + ;  = 6 字符
# 改: 截")))));  7 字符 + ;  = 8 字符
# 现在 last 5 是 )))));  后面是 \n
# 倒数第 6 是 截
# 我要在 截 后加 "  变成 截"  就是 倒数 6 字符变为 截"  + 后 5 字符
new_stripped = stripped[:-5] + '")))));'  # 截" + 4 个 ) + ;
# 上面不对。让我直接用字符操作
# stripped = "        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(403, \"内容包含疑似垃圾信息，已被拦截)))));"
# 我要在 拦 后面加 "  变成 拦"  + ?  + 4 个 ) + ;
# 找 拦 的位置
idx = stripped.rfind("截")
print(f"截 position: {idx}")
if idx > 0:
    new_stripped = stripped[:idx+1] + '"' + stripped[idx+1:]
    new_line = new_stripped + "\n"
    print(f"New line: {new_line!r}")
    lines[317] = new_line
    with open(src, "w", encoding="utf-8", newline="\n") as f:
        f.writelines(lines)
    print("Fixed")
