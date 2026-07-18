//! Markdown rendering. Mirrors `utils.Marked` (goldmark + bluemonday sanitise).
//! comrak with `safe` feature renders GitHub-flavored markdown and escapes raw HTML.
use comrak::{markdown_to_html, ComrakOptions};

/// Render markdown content to sanitized HTML. On any error, returns the
/// input escaped as a paragraph (never panics).
pub fn marked(content: &str) -> String {
    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.render.unsafe_ = false; // sanitise: no raw HTML pass-through
    options.render.hardbreaks = true;
    markdown_to_html(content, &options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_markdown() {
        let html = marked("**bold** and _italic_");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn escapes_raw_html() {
        let html = marked("<script>alert(1)</script>");
        assert!(!html.to_lowercase().contains("<script>"));
    }
}
