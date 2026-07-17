#!/usr/bin/env python3
"""补全缺的 ) 。"""
import os
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"

# 已知需要修复
FIXES = {
    "comments.rs": [
        (109, '        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "数据库错误")))\n'),
        (124, '                    Ok(R(db::row_i64(row, 0)?))\n'),
        (155, '                            Json(ApiResponse::error(500, "创建页面失败"))),\n'),
        (164, '            Json(ApiResponse::error(400, "需要指定 page_id 或 page_url"))),\n'),
        (283, '            Json(ApiResponse::error(404, "评论不存在"))),\n'),
        (328, '        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "数据库错误")))\n'),
        (425, '                    Ok(R(db::row_str(row, 0)?, db::row_str(row, 1)?))\n'),
    ],
    "pages.rs": [
        (53, '        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "服务器错误")))\n'),
    ],
    "sites.rs": [
        (84, '        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(500, "服务器错误")))\n'),
    ],
}

for root, _, files in os.walk(src):
    for n in files:
        if n not in FIXES:
            continue
        p = os.path.join(root, n)
        with open(p, "r", encoding="utf-8") as f:
            lines = f.readlines()
        for line_num, new_line in FIXES[n]:
            old = lines[line_num - 1]
            if old != new_line:
                print(f"  {n}:{line_num}: {old!r} -> {new_line!r}")
                lines[line_num - 1] = new_line
        with open(p, "w", encoding="utf-8", newline="\n") as f:
            f.writelines(lines)
