//! Input validation helpers. Mirrors `utils.ValidateEmail` / `ValidateURL`.
use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());

/// Validate an email string. Mirrors Go's `utils.ValidateEmail`.
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.len() > 254 {
        return false;
    }
    EMAIL_RE.is_match(email)
}

/// Validate a URL string. Mirrors Go's `utils.ValidateURL`.
pub fn is_valid_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() {
        return false;
    }
    match url::Url::parse(url) {
        Ok(parsed) => parsed.scheme() == "http" || parsed.scheme() == "https",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_basics() {
        assert!(is_valid_email("a@example.com"));
        assert!(!is_valid_email("not-an-email"));
        assert!(!is_valid_email("a@b"));
        assert!(!is_valid_email(""));
    }

    #[test]
    fn url_basics() {
        assert!(is_valid_url("https://example.com/x"));
        assert!(!is_valid_url("javascript:alert(1)"));
        assert!(!is_valid_url("not a url"));
    }
}
