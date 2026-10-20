//! Generate notices from locked, downloaded Rust dependencies, never from a
//! scanned target or private inventory. These are public distribution inputs.
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
    process::Command,
};
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let out = env::args()
        .nth(1)
        .ok_or("Usage: topo-notices NEW_OUTPUT_HTML")?;
    if Path::new(&out).exists() {
        return Err("Notice output already exists".into());
    }
    let result = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .output()?;
    if !result.status.success() {
        return Err("Locked dependency metadata failed".into());
    }
    let metadata: Value = serde_json::from_slice(&result.stdout)?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("Package metadata missing")?;
    let root = packages
        .iter()
        .find(|p| p["name"] == "topo")
        .ok_or("Topo package missing")?["id"]
        .as_str()
        .ok_or("Package ID missing")?;
    let nodes: BTreeMap<_, _> = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("Dependency resolution missing")?
        .iter()
        .map(|n| (n["id"].as_str().unwrap(), n))
        .collect();
    let mut included = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if included.insert(id) {
            for d in nodes[id]["dependencies"]
                .as_array()
                .ok_or("Dependency list missing")?
            {
                pending.push(d.as_str().ok_or("Dependency ID missing")?);
            }
        }
    }
    let mut html = String::from(
        "<h1>Third-party notices</h1><p>Topo uses the following Rust components. Their authors retain their rights and applicable licenses. This notice does not assign an open-source license to Topo itself.</p><p>The Rust standard library is provided under MIT OR Apache-2.0; <a href=\"https://www.rust-lang.org/policies/licenses\">Rust licensing</a>.</p>",
    );
    let mut sorted: Vec<_> = packages
        .iter()
        .filter(|p| p["id"].as_str().is_some_and(|id| included.contains(id)) && p["name"] != "topo")
        .collect();
    sorted.sort_by_key(|p| p["name"].as_str().unwrap());
    for p in sorted {
        let name = p["name"].as_str().ok_or("Name missing")?;
        let version = p["version"].as_str().ok_or("Version missing")?;
        let license = p["license"].as_str().unwrap_or("See included license file");
        let dir = Path::new(p["manifest_path"].as_str().ok_or("Manifest path missing")?)
            .parent()
            .ok_or("Manifest parent missing")?;
        let mut files = BTreeSet::new();
        if let Some(file) = p["license_file"].as_str() {
            files.insert(dir.join(file));
        }
        for file in fs::read_dir(dir)? {
            let file = file?;
            let label = file.file_name().to_string_lossy().to_uppercase();
            if file.file_type()?.is_file()
                && (label.starts_with("LICENSE")
                    || label.starts_with("LICENCE")
                    || label.starts_with("COPYING")
                    || label.starts_with("NOTICE"))
            {
                files.insert(file.path());
            }
        }
        // The pinned grammar crate omits its upstream LICENSE from the crate
        // archive. Retain the exact version's upstream MIT text in source.
        if files.is_empty() && name == "tree-sitter-typescript" && version == "0.23.2" {
            files.insert(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("site/licenses/tree-sitter-typescript-LICENSE"),
            );
        }
        if files.is_empty() {
            return Err(
                format!("No license text found for {name}; review before distribution").into(),
            );
        }
        html.push_str(&format!(
            "<h2>{} {}</h2><p>{}</p>",
            escape(name),
            escape(version),
            escape(license)
        ));
        if let Some(url) = p["repository"].as_str() {
            html.push_str(&format!(
                "<p>Upstream: <a href=\"{}\">{}</a></p>",
                escape(url),
                escape(url)
            ));
        }
        for file in files {
            html.push_str(&format!(
                "<details><summary>{}</summary><pre>{}</pre></details>",
                escape(&file.file_name().unwrap().to_string_lossy()),
                escape(
                    &fs::read_to_string(file)?
                        .lines()
                        .map(str::trim_end)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            ));
        }
    }
    if let Some(parent) = Path::new(&out).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, html)?;
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("topo-notices: {e}");
        std::process::exit(1);
    }
}
