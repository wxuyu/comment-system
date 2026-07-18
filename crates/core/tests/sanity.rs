//! Sanity tests for the pure core modules (no DB / I/O required).
use artalk_core::config::Config;
use artalk_core::cook;
use artalk_core::crypto::{
    check_password, md5_hex, set_password_encrypt, sign_user_token, verify_user_token,
};
use artalk_core::entity::{Comment, Page, Site, User};
use artalk_core::markdown::marked;
use artalk_core::validate::{is_valid_email, is_valid_url};
use chrono::Utc;

fn sample_user() -> User {
    User {
        id: 7,
        name: "tester".into(),
        email: "Tester@Example.com".into(),
        link: "https://example.com".into(),
        ..Default::default()
    }
}

fn sample_comment() -> Comment {
    Comment {
        id: 42,
        content: "**hello** <script>alert(1)</script> world".into(),
        page_key: "/post-1".into(),
        site_name: "site-a".into(),
        user_id: 7,
        ip: "127.0.0.1".into(),
        ..Default::default()
    }
}

#[test]
fn jwt_round_trip() {
    let conf = Config::default();
    let user = sample_user();
    let token = sign_user_token(&user, &conf.app_key, 3600).expect("sign");
    let claims = verify_user_token(&token, &conf.app_key, None).expect("verify");
    assert_eq!(claims.user_id, 7);
}

#[test]
fn password_bcrypt_round_trip() {
    let mut u = sample_user();
    set_password_encrypt(&mut u, "sup3rsecret").expect("encrypt");
    assert!(u.password.starts_with("(bcrypt)"));
    assert!(check_password(&u.password, "sup3rsecret"));
    assert!(!check_password(&u.password, "wrong"));
}

#[test]
fn md5_is_lowercase_hex() {
    let h = md5_hex("Test@Example.com");
    assert_eq!(h.len(), 32);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(h, md5_hex("Test@Example.com"));
}

#[test]
fn markdown_sanitises_scripts() {
    let out = marked("**hi** <script>alert(1)</script>");
    assert!(out.contains("<strong>hi</strong>"));
    assert!(!out.to_lowercase().contains("<script"));
}

#[test]
fn cook_comment_fields() {
    let conf = Config::default();
    let user = sample_user();
    let page = Page {
        key: "/post-1".into(),
        title: "Post 1".into(),
        site_name: "site-a".into(),
        ..Default::default()
    };
    let site = Site {
        name: "site-a".into(),
        ..Default::default()
    };
    let c = sample_comment();
    let cooked = cook::cook_comment(&c, &user, &page, Some(&site), &|e: &str| {
        md5_hex(&e.to_lowercase())
    });
    assert_eq!(cooked.id, 42);
    assert_eq!(cooked.nick, "tester");
    assert_eq!(cooked.email_encrypted, md5_hex("tester@example.com"));
    assert!(cooked.content_marked.contains("<strong>hello</strong>"));
    assert_eq!(cooked.page_key, "/post-1");
    assert_eq!(cooked.site_name, "site-a");
}

#[test]
fn validate_helpers() {
    assert!(is_valid_email("a@b.com"));
    assert!(!is_valid_email("nope"));
    assert!(is_valid_url("https://x.io/p"));
    assert!(!is_valid_url("not a url"));
}

#[test]
fn now_formatted_shape() {
    let _ = Utc::now();
}
