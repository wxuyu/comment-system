//! Pure "cook" conversions: turn DB entities into the API DTOs.
//! Mirrors `internal/dao/cook.go` but takes related entities as arguments
//! (no DB fetch — keeping the core pure). Server glue supplies the relations.

use crate::entity::*;
use crate::markdown::marked;

/// Mirrors `Dao.CookComment`. `user`, `page`, `site` are the related rows.
pub fn cook_comment(
    c: &Comment,
    user: &User,
    page: &Page,
    site: Option<&Site>,
    email_hash: &dyn Fn(&str) -> String,
) -> CookedComment {
    let marked_content = marked(&c.content);
    CookedComment {
        id: c.id,
        content: c.content.clone(),
        content_marked: marked_content,
        user_id: c.user_id,
        nick: user.name.clone(),
        email_encrypted: email_hash(&user.email),
        link: user.link.clone(),
        ua: c.ua.clone(),
        date: crate::entity::format_datetime(c.created_at),
        is_collapsed: c.is_collapsed,
        is_pending: c.is_pending,
        is_pinned: c.is_pinned,
        is_allow_reply: c.is_allow_reply(),
        is_verified: if user.is_admin { true } else { c.is_verified },
        rid: c.rid,
        badge_name: user.badge_name.clone(),
        badge_color: user.badge_color.clone(),
        ip: c.ip.clone(),
        ip_region: None,
        visible: true,
        vote_up: c.vote_up,
        vote_down: c.vote_down,
        page_key: c.page_key.clone(),
        page_url: page_url(page, site),
        site_name: c.site_name.clone(),
    }
}

/// Mirrors `Dao.CookPage`.
pub fn cook_page(p: &Page) -> CookedPage {
    CookedPage {
        id: p.id,
        admin_only: p.admin_only,
        key: p.key.clone(),
        url: page_url(p, None),
        title: p.title.clone(),
        site_name: p.site_name.clone(),
        vote_up: p.vote_up,
        vote_down: p.vote_down,
        pv: p.pv,
        date: crate::entity::format_datetime(p.created_at),
    }
}

/// Mirrors `Dao.CookSite`.
pub fn cook_site(s: &Site) -> CookedSite {
    let urls = crate::split_and_trim(&s.urls, ',');
    let first_url = urls.first().cloned().unwrap_or_default();
    CookedSite {
        id: s.id,
        name: s.name.clone(),
        urls: urls.clone(),
        urls_raw: s.urls.clone(),
        first_url,
    }
}

/// Mirrors `Dao.CookUser`.
pub fn cook_user(u: &User) -> CookedUser {
    CookedUser {
        id: u.id,
        name: u.name.clone(),
        email: u.email.clone(),
        link: u.link.clone(),
        badge_name: u.badge_name.clone(),
        badge_color: u.badge_color.clone(),
        is_admin: u.is_admin,
        receive_email: u.receive_email,
    }
}

/// Mirrors `Dao.UserToCookedForAdmin`.
pub fn cook_user_for_admin(u: &User, comment_count: i64) -> CookedUserForAdmin {
    CookedUserForAdmin {
        user: cook_user(u),
        last_ip: u.last_ip.clone(),
        last_ua: u.last_ua.clone(),
        is_in_conf: u.is_in_conf,
        comment_count,
    }
}

/// Mirrors `Dao.CookNotify`.
pub fn cook_notify(n: &Notify) -> CookedNotify {
    CookedNotify {
        id: n.id,
        user_id: n.user_id,
        comment_id: n.comment_id,
        is_read: n.is_read,
        is_emailed: n.is_emailed,
        read_link: n.read_link.clone(),
    }
}

/// Mirrors `Dao.GetPageAccessibleURL`. The accessible URL is the page's own
/// `accessible_url` if set, else the first site URL + page key.
pub fn page_url(page: &Page, site: Option<&Site>) -> String {
    if !page.accessible_url.is_empty() {
        return page.accessible_url.clone();
    }
    if let Some(s) = site {
        let urls = crate::split_and_trim(&s.urls, ',');
        if let Some(first) = urls.first() {
            return format!("{}{}", first.trim_end_matches('/'), page.key);
        }
    }
    page.key.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooks_page_without_site() {
        let mut p = Page::default();
        p.key = "/blog/1".into();
        let cooked = cook_page(&p);
        assert_eq!(cooked.key, "/blog/1");
        assert_eq!(cooked.url, "/blog/1");
    }

    #[test]
    fn page_url_prefers_site_first_url() {
        let mut p = Page::default();
        p.key = "/blog/1".into();
        let mut s = Site::default();
        s.urls = "https://example.com,https://other.com".into();
        assert_eq!(page_url(&p, Some(&s)), "https://example.com/blog/1");
    }

    #[test]
    fn cooks_user_admin_gets_verified() {
        let mut u = User::default();
        u.is_admin = true;
        let mut c = Comment::default();
        c.is_verified = false;
        let _ = cook_comment(&c, &u, &Page::default(), None, &|e| format!("hash({})", e));
    }
}
