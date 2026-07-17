//! 管理后台 UI 路由
//!
//! 服务嵌入式的静态管理界面（SPA）：
//! - `/admin` → SPA 入口
//! - `/admin/login` → 登录页
//!
//! 静态文件直接嵌入二进制（include_str!），无需额外部署前端。

use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};

/// 处理 `/admin/*` 路径：返回嵌入的 SPA 资源
pub async fn admin_spa(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches("/admin").trim_start_matches('/');

    // 默认路径：返回 index.html
    let resource_path = if path.is_empty() || path == "login" {
        "static/admin/index.html"
    } else {
        match path {
            "app.js" => "static/admin/app.js",
            "style.css" => "static/admin/style.css",
            "favicon.ico" => return StatusCode::NO_CONTENT.into_response(),
            _ => "static/admin/index.html", // SPA 兜底
        }
    };

    let content_type = match resource_path {
        p if p.ends_with(".html") => "text/html; charset=utf-8",
        p if p.ends_with(".js") => "application/javascript; charset=utf-8",
        p if p.ends_with(".css") => "text/css; charset=utf-8",
        _ => "application/octet-stream",
    };

    match load_embedded(resource_path) {
        Some(content) => ([(header::CONTENT_TYPE, content_type)], content).into_response(),
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

fn load_embedded(path: &str) -> Option<String> {
    match path {
        "static/admin/index.html" => Some(include_str!("../static/admin/index.html").to_string()),
        "static/admin/app.js" => Some(include_str!("../static/admin/app.js").to_string()),
        "static/admin/style.css" => Some(include_str!("../static/admin/style.css").to_string()),
        _ => None,
    }
}
