use std::{error::Error, fs, process::Command};

fn google(args: &[&str]) -> Result<Vec<u8>, Box<dyn Error>> {
    let out = Command::new("node")
        .arg("node_modules/@google/design.md/dist/index.js")
        .args(args).output()?;
    if !out.status.success() {
        return Err(format!("design.md command failed:\n{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(out.stdout)
}

fn main() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(".cache")?;
    let report = google(&["lint", "DESIGN.md"])?;
    fs::write(".cache/design-lint.json", &report)?;
    println!("{}", String::from_utf8_lossy(&report));
    fs::write(".cache/official-tokens.json", google(&["export", "--format", "dtcg", "DESIGN.md"])?)?;
    if !Command::new("bun").arg("design/export.ts").status()?.success() {
        return Err("Native token export failed".into());
    }
    let reference = fs::read_to_string("design/reference.html")?;
    let start = reference.find("<style>").ok_or("Reference style missing")? + 7;
    let end = reference[start..].find("</style>").ok_or("Reference style end missing")? + start;
    let css = format!("\n{}\n{}\n", fs::read_to_string("design/tokens.css")?, fs::read_to_string("design/components.css")?);
    let mut preview = reference.clone();
    preview.replace_range(start..end, &css);
    // The legacy reference's evidence prose has one inline color reference.
    preview = preview.replace("color:var(--fg)", "color:var(--color-fg)");
    fs::write(".cache/preview.html", preview)?;
    println!("Built native CSS, DTCG tokens and .cache/preview.html from DESIGN.md");
    Ok(())
}
