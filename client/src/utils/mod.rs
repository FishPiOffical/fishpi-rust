pub mod auth;

pub use auth::AuthService;

use regex::Regex;

pub fn strip_html_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").to_string()
}