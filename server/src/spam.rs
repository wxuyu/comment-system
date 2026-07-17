//! 垃圾内容检测模块

use regex::Regex;

/// 检测评论内容是否为垃圾信息
pub fn is_spam(content: &str, nickname: &str, website: Option<&str>, _email: &str) -> bool {
    // 检查空内容
    if content.trim().is_empty() || content.len() < 2 {
        return false;
    }

    // 检查是否包含过多链接（超过 3 个）
    let link_count = content.matches("http").count() + content.matches("www.").count();
    if link_count > 3 {
        return true;
    }

    // 检查是否纯数字/乱码
    let alpha_ratio = content.chars().filter(|c| c.is_alphabetic()).count() as f64
        / content.len().max(1) as f64;
    if content.len() > 10 && alpha_ratio < 0.1 {
        return true;
    }

    // 检查常见垃圾关键词
    let spam_keywords = [
        "buy now", "click here", "free money", "casino", "viagra",
        "赚钱", "兼职", "日结", "加微信", "加QQ", "免费领取",
        "点击领取", "点击查看", "快速致富",
    ];

    let lower = content.to_lowercase();
    for kw in &spam_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    // 检查网站是否为垃圾域名
    if let Some(url) = website {
        if !url.is_empty() {
            let spam_domains = ["casino", "porn", "xxx", "bet", "loan"];
            let lower_url = url.to_lowercase();
            for domain in &spam_domains {
                if lower_url.contains(domain) {
                    return true;
                }
            }
        }
    }

    // 检查昵称是否为垃圾模式
    let nickname_patterns = [
        Regex::new(r"[a-zA-Z]{20,}").unwrap(),     // 超长英文字母
        Regex::new(r"\d{8,}").unwrap(),              // 超长数字序列
    ];

    for pattern in &nickname_patterns {
        if pattern.is_match(nickname) {
            return true;
        }
    }

    false
}
