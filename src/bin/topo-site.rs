//! Public delivery has a closed set of inputs: docs, design exports, fictional
//! specimen, component notices and verified native releases. Never inventories.
#[path = "../workspace.rs"]
mod workspace;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};
const VERSION: &str = env!("CARGO_PKG_VERSION");
const SOURCE: &str = match option_env!("TOPO_SOURCE_REVISION") {
    Some(value) => value,
    None => "development",
};
struct Doc {
    slug: &'static str,
    title: &'static str,
    group: &'static str,
    source: &'static str,
}
impl Doc {
    fn url(&self) -> String {
        if self.slug.is_empty() {
            "/docs/".into()
        } else {
            format!("/docs/{}/", self.slug)
        }
    }
}
const DOCS: &[Doc] = &[
    Doc {
        slug: "",
        title: "Documentation",
        group: "Start",
        source: include_str!("../../docs/overview.md"),
    },
    Doc {
        slug: "quickstart",
        title: "Your first map",
        group: "Start",
        source: include_str!("../../docs/quickstart.md"),
    },
    Doc {
        slug: "install",
        title: "Install and update",
        group: "Start",
        source: include_str!("../../docs/install.md"),
    },
    Doc {
        slug: "using-the-map",
        title: "Use the map",
        group: "Guides",
        source: include_str!("../../docs/using-the-map.md"),
    },
    Doc {
        slug: "collecting",
        title: "Collect repositories",
        group: "Guides",
        source: include_str!("../../docs/collecting.md"),
    },
    Doc {
        slug: "evidence",
        title: "Understand evidence",
        group: "Guides",
        source: include_str!("../../docs/evidence.md"),
    },
    Doc {
        slug: "sharing",
        title: "Share a map",
        group: "Guides",
        source: include_str!("../../docs/sharing.md"),
    },
    Doc {
        slug: "agents",
        title: "Work with agents",
        group: "Guides",
        source: include_str!("../../docs/agents.md"),
    },
    Doc {
        slug: "cli",
        title: "CLI reference",
        group: "Reference",
        source: include_str!("../../docs/cli.md"),
    },
    Doc {
        slug: "graph",
        title: "Graph and JSON",
        group: "Reference",
        source: include_str!("../../docs/graph.md"),
    },
    Doc {
        slug: "privacy",
        title: "Privacy",
        group: "Reference",
        source: include_str!("../../docs/privacy.md"),
    },
    Doc {
        slug: "deployment",
        title: "Self-host deployment",
        group: "Reference",
        source: include_str!("../../docs/deployment.md"),
    },
    Doc {
        slug: "limitations",
        title: "Collection limits",
        group: "Reference",
        source: include_str!("../../docs/limitations.md"),
    },
    Doc {
        slug: "troubleshooting",
        title: "Troubleshooting",
        group: "Reference",
        source: include_str!("../../docs/troubleshooting.md"),
    },
];
const TARGETS: &[&str] = &[
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
];
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn nav(selected: &str) -> String {
    let mut output = String::new();
    let mut previous = "";
    for doc in DOCS {
        if previous != doc.group {
            if !previous.is_empty() {
                output.push_str("</section>");
            }
            output.push_str(&format!(
                "<section class=\"doc-nav-group\"><h2>{}</h2>",
                doc.group
            ));
            previous = doc.group;
        }
        output.push_str(&format!(
            "<a href=\"{}\"{}>{}</a>",
            doc.url(),
            if doc.slug == selected {
                " aria-current=\"page\""
            } else {
                ""
            },
            doc.title
        ));
    }
    output.push_str("</section>");
    output
}
fn frame(title: &str, description: &str, body: &str) -> String {
    format!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="description" content="{description}"><title>{title} · Topo</title><link rel="icon" href="/favicon.svg" type="image/svg+xml"><link rel="stylesheet" href="/tokens.css"><link rel="stylesheet" href="/site.css"></head><body><a class="skip" href="#content">Skip to content</a><header class="site-bar"><a class="brand" href="/"><img class="mark" src="/favicon.svg" alt="">Topo</a><span class="preview-label">Local-first preview {VERSION}</span><nav aria-label="Main navigation"><a href="/demo/">Demo</a><a href="/docs/">Docs</a><a href="/docs/install/">Install</a><a href="/changelog/">Changelog</a></nav></header>{body}</body></html>"##
    )
}
fn markdown(source: &str, public_notes: bool) -> (String, String) {
    let events: Vec<_> = Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .collect();
    let mut result = vec![];
    let mut contents = String::new();
    let mut ids = BTreeMap::<String, usize>::new();
    let mut private_link = false;
    for (index, event) in events.iter().enumerate() {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) if public_notes => {
                let url = dest_url.to_ascii_lowercase();
                private_link = url.strip_prefix("https://github.com/misty-step/topo")
                    .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(['/', '?', '#']));
                if private_link { continue; }
            }
            Event::End(TagEnd::Link) if private_link => {
                private_link = false;
                continue;
            }
            _ => {}
        }
        if let Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) = event
        {
            let title = events[index + 1..]
                .iter()
                .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(_))))
                .filter_map(|e| match e {
                    Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
                    Event::SoftBreak | Event::HardBreak => Some(" "),
                    _ => None,
                })
                .collect::<String>();
            let mut slug = String::new();
            for c in title.to_lowercase().chars() {
                if c.is_alphanumeric() {
                    slug.push(c);
                } else if !slug.ends_with('-') {
                    slug.push('-');
                }
            }
            let base = slug.trim_matches('-');
            let base = if base.is_empty() { "section" } else { base };
            let count = ids.entry(base.to_owned()).or_default();
            *count += 1;
            let slug = if *count == 1 {
                base.to_owned()
            } else {
                format!("{base}-{count}")
            };
            if *level == HeadingLevel::H2 {
                contents.push_str(&format!(
                    "<a href=\"#{}\">{}</a>",
                    escape(&slug),
                    escape(&title)
                ));
            }
            result.push(Event::Start(Tag::Heading {
                level: *level,
                id: Some(id.clone().unwrap_or_else(|| slug.into())),
                classes: classes.clone(),
                attrs: attrs.clone(),
            }));
        } else {
            result.push(event.clone());
        }
    }
    let mut html = String::new();
    html::push_html(&mut html, result.into_iter());
    (html, contents)
}
fn doc_page(index: usize, native_version: &str) -> String {
    let doc = &DOCS[index];
    let (content, contents) = markdown(&doc.source.replace("{{version}}", native_version), false);
    let navigation = nav(doc.slug);
    let mut steps = String::new();
    if index > 0 {
        let d = &DOCS[index - 1];
        steps.push_str(&format!(
            "<a href=\"{}\"><span>Previous</span>{}</a>",
            d.url(),
            d.title
        ));
    }
    if index + 1 < DOCS.len() {
        let d = &DOCS[index + 1];
        steps.push_str(&format!(
            "<a href=\"{}\"><span>Next</span>{}</a>",
            d.url(),
            d.title
        ));
    }
    let body = format!(
        r##"<main class="site-desk doc-desk" id="content" tabindex="-1"><aside class="site-pane doc-nav" aria-label="Documentation"><nav aria-label="Documentation chapters">{navigation}</nav></aside><section class="site-pane primary doc-content" tabindex="0" aria-label="{title}"><details class="doc-mobile"><summary>Documentation</summary><nav aria-label="Documentation chapters">{navigation}</nav></details><div class="doc-article-grid"><article>{content}<nav class="doc-steps" aria-label="Reading sequence">{steps}</nav><footer class="doc-foot">Topo {VERSION} · <a href="/docs/privacy/">Private maps stay private</a> · <a href="/third-party/">Third-party notices</a></footer></article><aside class="doc-toc" aria-label="On this page"><details open><summary>On this page</summary><nav aria-label="Page sections">{contents}</nav></details></aside></div></section></main><script src="/docs.js" defer></script>"##,
        title = escape(doc.title)
    );
    frame(
        doc.title,
        "Topo guides and references: start with a map, reveal deeper details as needed, and preserve evidence and privacy.",
        &body,
    )
}
fn write(root: &Path, path: &str, content: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, content)
}
fn released_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 25 * 1024 * 1024 {
        return Err(format!("Release asset budget/empty file: {}", path.display()).into());
    }
    Ok(fs::read(path)?)
}
fn keep_releases(out: &Path, previous: &Path, native_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Carry forward only verified versioned native executables/checksums. Never
    // copy arbitrary historical site files or inventories into public assets.
    for version in fs::read_dir(previous)? {
        let version = version?;
        let name = version.file_name().to_string_lossy().into_owned();
        let parts: Vec<_> = name.strip_prefix('v').unwrap_or("").split('.').collect();
        if parts.len() != 3
            || parts
                .iter()
                .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
            || !version.file_type()?.is_dir()
        {
            continue;
        }
        if name == format!("v{native_version}") {
            continue;
        }
        for target in TARGETS {
            let file = format!("topo-{target}");
            let bytes = released_bytes(&version.path().join(&file))?;
            let checksum = fs::read_to_string(version.path().join(format!("{file}.sha256")))?;
            let sha = format!("{:x}", Sha256::digest(&bytes));
            if checksum.split_whitespace().next() != Some(&sha) {
                return Err(format!("Historical release checksum mismatch: {name}/{file}").into());
            }
            write(out, &format!("releases/{name}/{file}"), bytes)?;
            write(out, &format!("releases/{name}/{file}.sha256"), checksum)?;
        }
    }
    Ok(())
}
fn build(
    out: &Path,
    releases: &Path,
    notices: &Path,
    previous: Option<&Path>,
    native_version: &str,
    native_source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if native_version.split('.').count() != 3 || native_version.split('.').any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
        return Err("Native version must be an explicit numeric SemVer release".into());
    }
    if native_source.len() != 40 || !native_source.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Native source must be an exact release commit".into());
    }
    if out.exists() {
        return Err("Site output exists; choose a new staging directory".into());
    }
    for target in TARGETS {
        released_bytes(&releases.join(format!("topo-{target}")))?;
    }
    let notices = fs::read_to_string(notices)?;
    fs::create_dir_all(out)?;
    let notice_body = format!(
        "<main class=\"site-desk doc-desk\" id=\"content\"><aside class=\"site-pane doc-nav\"><nav aria-label=\"Documentation chapters\">{}</nav></aside><section class=\"site-pane primary doc-content\" tabindex=\"0\" aria-label=\"Component notices\"><article>{notices}</article></section></main>",
        nav("third-party")
    );
    write(
        out,
        "third-party/index.html",
        frame(
            "Third-party notices",
            "Licenses and attribution for Topo's distributed native components.",
            &notice_body,
        ),
    )?;
    write(out, "tokens.css", include_str!("../../design/tokens.css"))?;
    write(out, "site.css", include_str!("../../site/site.css"))?;
    write(out, "docs.js", include_str!("../../site/docs.js"))?;
    // The mark follows the system palette: the base primary, then its light value.
    let primary = |n| {
        include_str!("../../design/tokens.css")
            .split("--color-primary:")
            .nth(n)
            .and_then(|v| v.split(';').next())
            .map(str::trim)
            .ok_or("Primary token missing")
    };
    let (dark, light) = (primary(1)?, primary(2)?);
    write(
        out,
        "favicon.svg",
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\"><style>svg{{color:{dark}}}@media (prefers-color-scheme:light){{svg{{color:{light}}}}}</style><path fill=\"currentColor\" d=\"M1 1h5v5H1zM10 10h5v5h-5z\"/><path fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" d=\"M6 3h6v7\"/></svg>"
        ),
    )?;
    let body = format!(
        r#"<main class="site-desk" id="content"><section class="site-pane primary demo-pane" data-fullscreen-surface aria-label="Interactive fictional example"><div class="pane-head"><span>Example workspace</span><span class="status">Fictional data</span><button type="button" data-fullscreen-toggle aria-label="Fullscreen" aria-pressed="false">{}<span data-fullscreen-label>Fullscreen</span></button><a href="/demo/">Open map</a></div><iframe src="/demo/" title="Topo's fictional example map" loading="eager" allowfullscreen></iframe></section><aside class="site-pane intro" aria-label="Get started"><h1>Your software, in view.</h1><p>A real map of the dependencies in your repositories. Start at a directory, zoom out to a repo, or see a few projects together—without losing the evidence.</p><div class="actions"><a class="action primary" href="/docs/quickstart/">Map your repo</a><a class="action" href="/docs/">Read the docs</a></div><pre>topo run --repo app=. --out topo-map</pre><p class="muted">Install once. Open the result in your browser. No account, source upload or model key.</p><hr class="rule"><dl class="facts"><dt>Collector</dt><dd>Rust · Go · JS/TS source</dd><dt>Runs</dt><dd>On your computer</dd><dt>Output</dt><dd>Offline map + JSON graph</dd><dt>Agents</dt><dd>The same evidence and revision</dd></dl><nav class="guide-links" aria-label="Documentation shortcuts"><a href="/docs/install/">Install for macOS or Linux</a><a href="/docs/using-the-map/">Learn the map controls</a><a href="/docs/agents/">Query with your agent</a><a href="/docs/sharing/">Share a map safely</a></nav><hr class="rule"><p class="muted">An early, read-only preview. Source declarations are not live runtime, health or complete infrastructure discovery. Unknown stays unknown.</p><p><a href="/docs/limitations/">Supported collection and limits</a></p></aside></main>"#,
        workspace::FULLSCREEN_ICON
    );
    write(
        out,
        "index.html",
        workspace::fullscreen(&frame(
            "Your software, in view",
            "Topo maps source dependencies across modules and repositories. A local Rust CLI with offline maps, JSON evidence and no source upload.",
            &body,
        ))?,
    )?;
    write(out, "demo/index.html", demo()?)?;
    for (index, doc) in DOCS.iter().enumerate() {
        let path = if doc.slug.is_empty() {
            "docs/index.html".into()
        } else {
            format!("docs/{}/index.html", doc.slug)
        };
        write(out, &path, doc_page(index, native_version))?;
    }
    let notes = "# Changelog\n\nNo generated release notes in this revision yet. Earlier native downloads remain available.\n";
    let (notes, _) = markdown(&notes, true);
    write(out, "changelog/index.html", frame("Changelog", "Release notes included in this Topo build.", &format!("<main class=\"site-desk doc-desk\" id=\"content\"><aside class=\"site-pane doc-nav\">{}</aside><section class=\"site-pane primary doc-content\"><article>{notes}</article></section></main>", nav("changelog"))))?;
    write(out, "manifest.json", serde_json::to_vec_pretty(&serde_json::json!({"schema":"topo.code.v1", "version":VERSION, "source_revision":SOURCE, "native_version":native_version, "native_source_revision":native_source}))?)?;
    write(
        out,
        "install.sh",
        include_str!("../../site/install.sh").replace("@@VERSION@@", native_version),
    )?;
    if let Some(previous) = previous {
        keep_releases(out, previous, native_version)?;
    }
    let mut manifest = vec![];
    for target in TARGETS {
        let name = format!("topo-{target}");
        let bytes = released_bytes(&releases.join(&name))?;
        let sha = format!("{:x}", Sha256::digest(&bytes));
        let path = format!("releases/v{native_version}/{name}");
        write(out, &path, &bytes)?;
        write(out, &format!("{path}.sha256"), format!("{sha}  {name}\n"))?;
        manifest.push(serde_json::json!({"target":target,"path":format!("/{path}"),"sha256":sha,"bytes":bytes.len()}));
    }
    write(
        out,
        "releases/manifest.json",
        serde_json::to_vec_pretty(&serde_json::json!({"version":native_version,"source_revision":native_source,"platforms":manifest}))?,
    )?;
    write(
        out,
        "404.html",
        frame(
            "Page not found",
            "This Topo page does not exist.",
            "<main class=\"site-desk\" id=\"content\"><section class=\"site-pane primary intro\"><h1>Page not found</h1><p>Try the <a href=\"/docs/\">documentation</a> or return to the <a href=\"/\">map example</a>.</p></section></main>",
        ),
    )?;
    write(
        out,
        "robots.txt",
        "User-agent: *\nAllow: /\nSitemap: https://topo.mistystep.io/sitemap.xml\n",
    )?;
    let urls = std::iter::once("https://topo.mistystep.io/".to_string())
        .chain(
            DOCS.iter()
                .map(|d| format!("https://topo.mistystep.io{}", d.url())),
        )
        .map(|url| format!("<url><loc>{url}</loc></url>"))
        .collect::<String>();
    write(
        out,
        "sitemap.xml",
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>"
        ),
    )?;
    println!(
        "Built public site at {} with {} docs and {} verified release assets; no inventory input.",
        out.display(),
        DOCS.len(),
        TARGETS.len()
    );
    Ok(())
}
// Adapt the public demo without editing the frozen visual specimen. Unlike h(),
// the native replaceChildren method stringifies absent optional children. Like
// exported maps, it uses the generated tokens (system light/dark) and primitives.
fn demo() -> Result<String, Box<dyn std::error::Error>> {
    let reference = include_str!("../../design/reference.html");
    let slot = "function renderDetail() {";
    let call = "E.dBody.replaceChildren(";
    if reference.matches(slot).count() != 1 || reference.matches(call).count() != 3 {
        return Err("Demo inspector rendering slots changed".into());
    }
    let mut page = reference.replace(call, "replaceDetailChildren(").replace(
        slot,
        &format!(
            "function replaceDetailChildren(...children) {{\n  E.dBody.replaceChildren(...children.filter(child => child != null && child !== false));\n}}\n{slot}"
        ),
    );
    let start = page.find("<style>").ok_or("Demo style slot missing")? + 7;
    let end = start + page[start..].find("</style>").ok_or("Demo style end missing")?;
    page.replace_range(
        start..end,
        concat!("\n", include_str!("../../design/tokens.css"), include_str!("../../design/components.css")),
    );
    workspace::fullscreen(&page)
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let get = |name: &str| {
        args.windows(2)
            .find(|p| p[0] == name)
            .map(|p| PathBuf::from(&p[1]))
    };
    let result=match(get("--out"),get("--releases"),get("--notices"),get("--native-version"),get("--native-source")){(Some(out),Some(releases),Some(notices),Some(version),Some(source))=>build(&out,&releases,&notices,get("--previous-releases").as_deref(),version.to_str().unwrap_or(""),source.to_str().unwrap_or("")),_=>Err("Usage: topo-site --out NEW_DIRECTORY --releases RELEASE_DIRECTORY --notices NOTICE_HTML --native-version SEMVER --native-source SHA [--previous-releases PUBLISHED_RELEASE_DIRECTORY]".into())};
    if let Err(e) = result {
        eprintln!("topo-site: {e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_notes_unlink_private_source_without_changing_text_or_public_links() {
        let source = "## [0.4.0](https://github.com/misty-step/topo/compare/v0.3.0...v0.4.0)\n\n* Keep **useful** `notes` ([#20][issue]) ([f1618aa](https://github.com/misty-step/topo/commit/f1618aa)).\n\n[issue]: https://github.com/misty-step/topo/issues/20\n\n[Install](/docs/install/) · [Parser](https://github.com/pulldown-cmark/pulldown-cmark)\n";
        let (notes, _) = markdown(source, true);
        assert_eq!(notes, "<h2 id=\"0-4-0\">0.4.0</h2>\n<ul>\n<li>Keep <strong>useful</strong> <code>notes</code> (#20) (f1618aa).</li>\n</ul>\n<p><a href=\"/docs/install/\">Install</a> · <a href=\"https://github.com/pulldown-cmark/pulldown-cmark\">Parser</a></p>\n");
        assert!(markdown(source, false).0.contains("href=\"https://github.com/misty-step/topo/issues/20\""));
        for url in ["https://github.com/misty-step/topo", "https://GITHUB.com/Misty-Step/Topo?tab=readme", "https://github.com/misty-step/topo#notes"] {
            assert!(!markdown(&format!("[note]({url})"), true).0.contains("<a "));
        }
        for url in ["https://github.com/misty-step/topology", "https://github.com/misty-step/topo-public", "https://topo.mistystep.io/docs/"] {
            assert!(markdown(&format!("[note]({url})"), true).0.contains("<a "));
        }
    }

    #[test]
    fn section_links_resolve_for_inline_and_duplicate_headings() {
        let (content, links) = markdown(
            "# Guide\n\n## Read `graph.json` & evidence\n\nText\n\n## Read `graph.json` & evidence\n\n<details>\n<summary>Advanced</summary>\n\n## A deeper section\n\nDetails\n\n</details>\n",
            false,
        );
        for id in [
            "read-graph-json-evidence",
            "read-graph-json-evidence-2",
            "a-deeper-section",
        ] {
            assert!(links.contains(&format!("href=\"#{id}\"")));
            assert!(content.contains(&format!("id=\"{id}\"")));
        }
        assert!(content.contains("<code>graph.json</code>"));
        assert!(content.contains("<details>"));
    }
}
