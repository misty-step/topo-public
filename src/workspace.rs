//! One maintained fullscreen enhancement for the fictional demo, public embed
//! and exported offline maps. The approved reference remains a frozen specimen.
//! Callers already carry the generated tokens; repeating them here would reset
//! the components' compact-layout token overrides.
pub const FULLSCREEN_ICON: &str = r#"<svg viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M6 2H2v4m8-4h4v4M2 10v4h4m4 0h4v-4"/></svg>"#;
pub fn fullscreen(page: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut page = page.to_owned();
    if page.contains("id=\"keys-btn\"") {
        let slot = "<button class=\"barbtn\" type=\"button\" id=\"keys-btn\"";
        if !page.contains(slot) {
            return Err("Workspace controls slot missing".into());
        }
        page=page.replace(slot,&format!("<button class=\"barbtn\" type=\"button\" data-fullscreen-toggle aria-label=\"Fullscreen\" aria-pressed=\"false\" title=\"Fullscreen\">{FULLSCREEN_ICON}</button>{slot}"));
    }
    if !page.contains("data-fullscreen-toggle") {
        return Err("Fullscreen control missing".into());
    }
    if !page.contains("</head>") || !page.contains("</body>") {
        return Err("Workspace document slots missing".into());
    }
    page = page.replace(
        "</head>",
        &format!("<style>\n{}\n</style></head>", include_str!("../web/fullscreen.css")),
    );
    page = page.replace(
        "</body>",
        &format!(
            "<script>\n{}\n</script></body>",
            include_str!("../web/fullscreen.js")
        ),
    );
    Ok(page)
}
