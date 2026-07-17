use pulldown_cmark::{html, Parser, Options};
fn main() {
    let md = "# Title\n\nThis is **bold**.";
    let parser = Parser::new_ext(md, Options::all());
    let mut out = String::new();
    html::push_html(&mut out, parser);
    println!("RESULT=[{}]", out);
}
