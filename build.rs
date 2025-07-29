use std::process::Command;
fn main() {
    let tag = Command::new("git")
        .args(&["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_TAG={}", tag.trim());
    println!("cargo:warning=GIT_TAG={}", tag.trim());
}