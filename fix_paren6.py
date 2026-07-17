#!/usr/bin/env python3
"""修复 line 284: 移除多余的 )"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs"

with open(src, "r", encoding="utf-8") as f:
    content = f.read()

# 284:  ?"))), -> ?"))
content = content.replace(
    'Json(ApiResponse::error(404, "评论不存在"))),',
    'Json(ApiResponse::error(404, "评论不存在"))),'
)
# 检查
print("Before:", '"评论不存在"))),' in content)
print("After:", '"评论不存在"))),' in content)

# 改为只有 2 个 )
content = content.replace(
    'Json(ApiResponse::error(404, "评论不存在"))),',
    'Json(ApiResponse::error(404, "评论不存在"))),'
)
# 上面 replace 没变。手动写
old = 'Json(ApiResponse::error(404, "\u8bc4\u8bba\u4e0d\u5b58\u5728"))),'
new = 'Json(ApiResponse::error(404, "\u8bc4\u8bba\u4e0d\u5b58\u5728"))),'
# 上面一样。我需要的是把 3 个 ) 变成 2 个 )，再加逗号
# 找 ?"))),  替换为 ?")),
content = content.replace('?")),', '?"))),')

# 写回
with open(src, "w", encoding="utf-8", newline="\n") as f:
    f.write(content)
print("Done")
