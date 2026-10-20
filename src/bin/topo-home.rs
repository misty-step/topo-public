//! Compose explicitly selected private exports; no collection or publication.
#[path = "../catalogue.rs"]
mod catalogue;
#[path = "../model.rs"]
mod model;
use pulldown_cmark::{Event, Options, Parser, html};
use serde_json::{Value, json};
use std::{collections::BTreeSet, env, fs, io::Write, path::Path};
type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const STYLE: &str = "body.topo-catalogue{margin:0;background:var(--color-desk);color:var(--color-fg);font:13px/1.45 var(--font-mono)}.topo-catalogue a{color:var(--color-primary)}.topo-catalogue main{padding:16px;max-width:1200px;margin:auto;overflow:auto}.topo-catalogue h1,.topo-catalogue h2{font-size:18px}.topo-catalogue table{border-collapse:collapse;min-width:900px}.topo-catalogue th,.topo-catalogue td{border:1px solid var(--color-line-strong);padding:8px;text-align:left;vertical-align:top}.topo-fresh{flex-shrink:0;padding:6px 10px;background:var(--color-pane);border-top:1px solid var(--color-line-strong);font:11px/1.45 var(--font-mono);color:var(--color-muted)}.topo-fresh strong{color:var(--color-fg);margin-right:12px}.topo-fresh[data-status=Stale] strong{color:var(--color-monitoring)}.topo-fresh span{display:inline-block;margin-right:12px}body[data-view]{padding-bottom:44px;box-sizing:border-box}body[data-view]>.topo-fresh{position:fixed;bottom:0;left:0;right:0;z-index:100}@media(max-width:760px){body[data-view]{padding-bottom:104px}}";
const SCRIPT: &str = r#"(()=>{'use strict';
const parse=s=>{if(/^\d{4}-\d{2}-\d{2}$/.test(s))s+='T00:00:00Z';s=s.replace(/(\.\d{3})\d+(?=Z|[+-])/,'$1');const n=Date.parse(s);return Number.isFinite(n)?n:null};
function refresh(){for(const el of document.querySelectorAll('.topo-fresh')){const dates=JSON.parse(el.dataset.dates),now=Date.now(),known=dates.map(s=>({raw:s,time:parse(s)})).filter(d=>d.time!==null&&d.time<=now).sort((a,b)=>a.time-b.time);const unknown=known.length!==dates.length||!dates.length;const stale=known.length&&now-known[0].time>=Number(el.dataset.hours)*3600000;const status=stale?'Stale':unknown?'Unknown':'Fresh';el.dataset.status=status;el.querySelector('strong').textContent=status+(unknown?' · unknown observation dates':'');el.querySelector('.topo-observed').textContent='Observed: '+(known.length?known[0].raw+(known.length>1?' to '+known[known.length-1].raw:''):'Unknown');}}
refresh();setInterval(refresh,60000);
})();"#;
const FRESH_CSS: &str = include_str!("../../web/fresh.css");
const FRESH_JS: &str = include_str!("../../web/fresh.js");
const HOME_CSS: &str = include_str!("../../web/home.css");
const HOME_JS: &str = include_str!("../../web/home.js");
const BAR_CSS: &str = include_str!("../../web/bar.css");
const FRESH_LINE: &str = "<b class=\"st\"><i class=\"sq\"></i><span class=\"w\">Unknown</span></b><span>Built <b class=\"built\">–</b></span><span>Data <b class=\"data\">–</b></span>";
const ARROW: &str = "<svg class=\"arrow\" viewBox=\"0 0 20 20\" aria-hidden=\"true\" focusable=\"false\"><path d=\"M3 10h13M11 4.5 16.5 10 11 15.5\"/></svg>";
const CHEVRON: &str = "<svg class=\"chev\" viewBox=\"0 0 20 20\" aria-hidden=\"true\" focusable=\"false\"><path d=\"M7 4.5 12.5 10 7 15.5\"/></svg>";
const BACK: &str = "<svg viewBox=\"0 0 14 14\" aria-hidden=\"true\" focusable=\"false\"><path d=\"M5 2l5 5-5 5\"/></svg>";
// Home counts by typed entity kind. A kind not listed stays visible as Other, never dropped.
const KINDS: [(&str, &[&str]); 6] = [
    ("Hosts", &["host"]),
    ("Services", &["service", "product"]),
    ("Databases", &["database"]),
    ("Domains", &["domain"]),
    ("Vendors", &["provider"]),
    ("Accounts", &["account"]),
];
struct Entry<'a> {
    id: &'a str,
    file: &'a str,
    label: &'a str,
    kind: model::MapKind,
    coverage: &'static str,
    dates: Vec<(String, usize)>,
}
fn replace_once(html: &mut String, from: &str, to: &str) -> Result<()> {
    let at = html
        .find(from)
        .ok_or("Map chrome slot missing; viewer markup changed")?;
    html.replace_range(at..at + from.len(), to);
    Ok(())
}
fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
/// Scope labels become readable titles without changing source identities.
fn title(label: &str) -> String {
    label
        .split(['-', '_', '.'])
        .filter(|w| !w.is_empty())
        .map(|w| match w {
            w if w.bytes().any(|b| b.is_ascii_digit()) => w.to_uppercase(),
            w => capitalize(w),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn fresh_attrs(
    name: Option<&str>,
    built: &str,
    dates: &[(String, usize)],
    hours: u64,
) -> Result<String> {
    Ok(format!(
        "data-fresh data-status=\"Unknown\" data-hours=\"{hours}\" data-built=\"{}\" data-dates=\"{}\"{}",
        escape(built),
        escape(&serde_json::to_string(dates)?),
        name.map(|n| format!(" data-name=\"{}\"", escape(n)))
            .unwrap_or_default()
    ))
}
fn basis(hours: u64) -> String {
    format!(
        "<details class=\"basis\"><summary>Basis</summary><div class=\"tip\"><dl><div><dt>Counts</dt><dd>Entities</dd></div><div><dt>Built</dt><dd>Map generated</dd></div><div><dt>Data</dt><dd>Oldest evidence</dd></div><div><dt>+</dt><dd>Older possible</dd></div><div><dt>Unknown</dt><dd>Date missing</dd></div><div><dt>Evidence</dt><dd>Dated sources</dd></div><div><dt>Stale</dt><dd>Over {hours}h</dd></div><div><dt>Partial</dt><dd>Gaps remain</dd></div><div><dt>Scope</dt><dd>Not a boundary</dd></div><div><dt>Probe</dt><dd>None</dd></div></dl><div class=\"dates\"></div></div></details>"
    )
}
fn unit(state: &mut u32) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    f64::from(*state) / f64::from(u32::MAX)
}
/// One closed, smooth contour: a wobbling ellipse through `n` points.
fn ring(
    c: (f64, f64),
    r: (f64, f64),
    n: usize,
    wobble: &[(f64, f64)],
    drift: f64,
    places: usize,
) -> String {
    let at = |j: usize| {
        let t = (j % n) as f64 / n as f64 * std::f64::consts::TAU;
        let m = 1.0
            + wobble
                .iter()
                .enumerate()
                .map(|(k, (a, p))| a * ((k + 1) as f64 * t + p + drift).sin())
                .sum::<f64>();
        (c.0 + t.cos() * r.0 * m, c.1 + t.sin() * r.1 * m)
    };
    let mut d = format!("M{:.places$} {:.places$}", at(0).0, at(0).1);
    for j in 0..n {
        let (p0, p1, p2, p3) = (at(j + n - 1), at(j), at(j + 1), at(j + 2));
        d += &format!(
            "C{:.places$} {:.places$} {:.places$} {:.places$} {:.places$} {:.places$}",
            p1.0 + (p2.0 - p0.0) / 6.0,
            p1.1 + (p2.1 - p0.1) / 6.0,
            p2.0 - (p3.0 - p1.0) / 6.0,
            p2.1 - (p3.1 - p1.1) / 6.0,
            p2.0,
            p2.1
        );
    }
    d + "Z"
}
fn mark() -> String {
    let wobble = [(0.14, 0.2), (0.07, 0.4)];
    let paths: String = [11.0, 7.0, 3.2]
        .iter()
        .map(|r| {
            format!(
                "<path d=\"{}\"/>",
                ring((14.0, 14.0), (*r, *r), 20, &wobble, *r, 1)
            )
        })
        .collect();
    format!(
        "<svg class=\"mark-glyph\" viewBox=\"0 0 28 28\" aria-hidden=\"true\" focusable=\"false\">{paths}</svg>"
    )
}
/// Decorative 2D contours, deterministic per map; nothing here carries data.
fn terrain(seed: &str) -> String {
    let mut s = u32::from_str_radix(&model::hash(seed.as_bytes())[..8], 16).unwrap_or(1) | 1;
    let wobble: Vec<_> = (1..=4)
        .map(|k| {
            (
                0.16 / f64::from(k) * (0.6 + unit(&mut s)),
                unit(&mut s) * std::f64::consts::TAU,
            )
        })
        .collect();
    let c = (330.0 + unit(&mut s) * 50.0, 55.0 + unit(&mut s) * 30.0);
    let paths: String = (0..9)
        .map(|i| {
            let r = 14.0 + 22.0 * f64::from(i);
            format!(
                "<path d=\"{}\"/>",
                ring(c, (r * 1.25, r * 0.9), 36, &wobble, 0.15 * f64::from(i), 0)
            )
        })
        .collect();
    format!(
        "<svg class=\"terrain\" viewBox=\"0 0 480 340\" preserveAspectRatio=\"xMaxYMin slice\" aria-hidden=\"true\" focusable=\"false\">{paths}</svg>"
    )
}
fn counts(g: &model::Graph, repo: &str) -> String {
    let mut found = [0usize; KINDS.len()];
    let mut total = 0;
    for e in g
        .entities
        .values()
        .filter(|e| e.repo.as_deref() == Some(repo))
    {
        total += 1;
        if let Some(i) = KINDS
            .iter()
            .position(|(_, kinds)| kinds.contains(&e.entity_kind.as_str()))
        {
            found[i] += 1;
        }
    }
    let other = total - found.iter().sum::<usize>();
    KINDS
        .iter()
        .map(|(label, _)| *label)
        .zip(found)
        .chain((other > 0).then_some(("Other", other)))
        .map(|(label, n)| format!("<div><dt>{label}</dt><dd>{n}</dd></div>"))
        .collect()
}
fn tile(n: usize, e: &Entry, g: &model::Graph, hours: u64) -> Result<String> {
    let name = title(e.label);
    let kind = e.kind.label();
    let repo = g.scopes[e.id].repo.as_deref().unwrap_or(e.id);
    let cov = if e.coverage.is_empty() {
        String::new()
    } else {
        format!("<span>{}</span>", capitalize(e.coverage))
    };
    Ok(format!(
        "<a class=\"map\" href=\"maps/{file}\" id=\"map-{n}\" data-key=\"{n}\" aria-keyshortcuts=\"{n}\" aria-labelledby=\"map-{n}-name map-{n}-fresh\" aria-describedby=\"map-{n}-counts\">{}<span class=\"key\" aria-hidden=\"true\">{n}</span><h3 class=\"name\" id=\"map-{n}-name\">{}<span class=\"kind\"><span>{}</span>{cov}</span></h3><dl class=\"counts\" id=\"map-{n}-counts\">{}</dl><p class=\"fresh\" id=\"map-{n}-fresh\" {}>{FRESH_LINE}</p><span class=\"go\">Open{ARROW}</span><i class=\"age\" aria-hidden=\"true\"><i></i></i></a>",
        terrain(e.id),
        escape(&name),
        escape(&kind),
        counts(g, repo),
        fresh_attrs(Some(&title(e.label)), &g.collected_at, &e.dates, hours)?,
        file = escape(e.file),
    ))
}
/// Infrastructure first, from the typed maps; Projects and Repos one level lower.
fn home_html(entries: &[Entry], g: &model::Graph, projects: usize, hours: u64) -> Result<String> {
    let infra: Vec<_> = entries
        .iter()
        .filter(|e| e.kind == model::MapKind::Infra)
        .collect();
    let repos: Vec<_> = entries
        .iter()
        .filter(|e| e.kind == model::MapKind::Repo)
        .collect();
    let mut tiles = String::new();
    for (i, e) in infra.iter().enumerate() {
        tiles.push_str(&tile(i + 1, e, g, hours)?);
    }
    let infrastructure = if infra.is_empty() {
        String::new()
    } else {
        format!(
            "<section aria-labelledby=\"h-infra\"><h2 class=\"sec\" id=\"h-infra\">Infrastructure</h2><nav class=\"maps\" aria-label=\"Infrastructure\">{tiles}</nav></section>"
        )
    };
    let key = infra.len() + 1;
    let mut more = format!(
        "<a class=\"row\" href=\"projects/index.html\" data-key=\"{key}\" aria-keyshortcuts=\"{key}\"><span class=\"k\" aria-hidden=\"true\">{key}</span><span class=\"lab\">Projects</span><span class=\"n\">{projects}</span>{ARROW}</a>"
    );
    if !repos.is_empty() {
        let mut rows = String::new();
        for e in &repos {
            rows.push_str(&format!("<li><a href=\"maps/{}\" {}><i class=\"sq\"></i><span class=\"rn\">{}</span><span class=\"rb\"><span class=\"w sr\">Unknown</span><b class=\"built\">–</b></span></a></li>", escape(e.file), fresh_attrs(None, &g.collected_at, &e.dates, hours)?, escape(e.label)));
        }
        let key = key + 1;
        let partial = if repos.iter().any(|e| e.coverage == "partial") {
            "<span class=\"cov\">Partial</span>"
        } else {
            ""
        };
        more.push_str(&format!("<details class=\"row-details repos\" data-key=\"{key}\"><summary class=\"row\" aria-keyshortcuts=\"{key}\"><span class=\"k\" aria-hidden=\"true\">{key}</span><span class=\"lab\">Repos</span>{partial}<span class=\"n\">{} of {projects}</span>{CHEVRON}</summary><ul class=\"repolist\">{rows}<li class=\"all\"><a href=\"maps/index.html\"><span class=\"rn\">All</span><span class=\"rb\">{ARROW}</span></a></li></ul></details>", repos.len()));
    }
    Ok(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Topo</title><style>{}{FRESH_CSS}{HOME_CSS}</style></head><body class=\"home\"><header class=\"top\"><a class=\"mark\" href=\"index.html\" aria-label=\"Topo\">{}<span>Topo</span></a></header><main>{infrastructure}<section class=\"more\" aria-label=\"More\">{more}</section></main><footer class=\"foot\"><span class=\"rev\" title=\"Graph revision\">{}</span><a class=\"cat\" href=\"catalogue.json\">Catalogue</a>{}</footer><script>{FRESH_JS}{HOME_JS}</script></body></html>",
        include_str!("../../design/tokens.css"),
        mark(),
        escape(&g.revision[..8]),
        basis(hours)
    ))
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("Missing {key}").into())
}
fn read(path: &Path) -> Result<Vec<u8>> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err("Expected a regular export file, not a symlink".into());
    }
    Ok(fs::read(path)?)
}
fn directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(path)?;
    Ok(())
}
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path)?.write_all(bytes)?;
    Ok(())
}
fn dates(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}
fn freshness(generated: &str, observations: &[String], hours: u64) -> Result<String> {
    Ok(format!(
        "<aside class=\"topo-fresh\" aria-label=\"Map freshness\" data-hours=\"{hours}\" data-dates=\"{}\"><strong>Unknown</strong><span>Generated: {}</span><span class=\"topo-observed\">Observed: {}</span><span>Stale after {hours}h · date-only evidence uses 00:00 UTC · not a runtime probe</span></aside>",
        escape(&serde_json::to_string(observations)?),
        escape(generated),
        escape(&observations.join(", "))
    ))
}
fn page(title: &str, content: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Topo · {}</title><style>{}{STYLE}</style></head><body class=\"topo-catalogue\"><main>{content}</main><script>{SCRIPT}</script></body></html>",
        escape(title),
        include_str!("../../design/tokens.css")
    )
}
fn projection(html: &str) -> Result<Value> {
    let marker = "<script type=\"application/json\" id=\"infra-data\">";
    let start = html.find(marker).ok_or("Map projection missing")? + marker.len();
    let end = start
        + html[start..]
            .find("</script>")
            .ok_or("Map projection unclosed")?;
    Ok(serde_json::from_str(&html[start..end])?)
}
fn map_file(file: &str) -> bool {
    file == "index.html"
        || file
            .strip_prefix("scope-")
            .and_then(|s| s.strip_suffix(".html"))
            .is_some_and(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}
fn run() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args == ["--help"] {
        println!(
            "topo-home --state STATE_EXPORT_DIR --maps MAP_BUNDLE_DIR --out NEW_DIR [--stale-after-hours 48]\nCompose current private exports without refreshing their observations or publishing."
        );
        return Ok(());
    }
    let flags = ["--state", "--maps", "--out", "--stale-after-hours"];
    if ![6, 8].contains(&args.len())
        || args
            .chunks_exact(2)
            .any(|p| !flags.contains(&p[0].as_str()) || p[1].is_empty() || p[1].starts_with("--"))
        || flags
            .iter()
            .any(|f| args.chunks_exact(2).filter(|p| p[0] == *f).count() > 1)
    {
        return Err("Expected --state, --maps and --out exactly once".into());
    }
    let value = |flag: &str| {
        args.chunks_exact(2)
            .find(|p| p[0] == flag)
            .map(|p| p[1].as_str())
    };
    let state = Path::new(value("--state").ok_or("Missing --state")?);
    let maps = Path::new(value("--maps").ok_or("Missing --maps")?);
    let out = Path::new(value("--out").ok_or("Missing --out")?);
    let hours: u64 = value("--stale-after-hours").unwrap_or("48").parse()?;
    if hours == 0 {
        return Err("Stale threshold must be positive".into());
    }
    if out.exists() {
        return Err("Output already exists; use a fresh directory".into());
    }
    let state_bytes = read(&state.join("projects.json"))?;
    let inventory: Value = serde_json::from_slice(&state_bytes)?;
    let projects = inventory["projects"].as_array().ok_or("Missing projects")?;
    if inventory["schema"] != "topo.project-state.v1" || projects.is_empty() {
        return Err("Expected nonempty project-state export".into());
    }
    let markdown = String::from_utf8(read(&state.join("projects.md"))?)?;
    let graph_bytes = read(&maps.join("graph.json"))?;
    let mut graph: model::Graph = serde_json::from_slice(&graph_bytes)?;
    let revision = graph.revision.clone();
    graph.seal()?;
    if graph.schema != "topo.graph.v1" || graph.revision != revision {
        return Err("Map graph revision/schema mismatch".into());
    }
    let manifest_bytes = read(&maps.join("manifest.json"))?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    if manifest["schema"] != "topo.bundle.v1" || manifest["revision"] != revision {
        return Err("Map manifest revision/schema mismatch".into());
    }
    let scopes = manifest["scopes"].as_array().ok_or("Missing map scopes")?;
    let catalogue = catalogue::compose(&inventory, &state_bytes, &graph, &manifest)?;
    let generated = model::now()?;
    let source_revision = env::var("TOPO_PUBLISH_SOURCE_REVISION")
        .or_else(|_| env::var("GITHUB_SHA"))
        .unwrap_or_else(|_| "local-unsubmitted".into());
    let state_dates = dates(
        std::iter::once(text(&inventory, "collected_at")?.to_string()).chain(
            projects
                .iter()
                .flat_map(|p| p["runtime"].as_array().into_iter().flatten())
                .map(|r| r["date"].as_str().unwrap_or("Unknown").to_string()),
        ),
    );
    let repo_count = graph
        .scopes
        .values()
        .filter(|scope| scope.kind == "repository")
        .count();
    let mut entries = vec![];
    let mut page_metadata = vec![];
    let mut seen = BTreeSet::new();
    // Resolve and validate all page inputs before creating an output directory.
    let mut pages = vec![];
    for scope in scopes {
        let id = text(scope, "id")?;
        let file = text(scope, "file")?;
        let label = text(scope, "label")?;
        if !map_file(file) || !graph.scopes.contains_key(id) || !seen.insert(file) {
            return Err("Invalid or duplicate scope file".into());
        }
        let original = String::from_utf8(read(&maps.join(file))?)?;
        let data = projection(&original)?;
        if data["meta"]["graph_revision"] != revision || data["meta"]["scope_id"] != id {
            return Err("Map page revision/scope mismatch".into());
        }
        // One record per evidence source: the dates behind Data, kept with their counts.
        let mut records = std::collections::BTreeMap::<String, usize>::new();
        for source in data["sources"].as_array().ok_or("Missing page sources")? {
            *records
                .entry(source["date"].as_str().unwrap_or("Unknown").to_string())
                .or_default() += 1;
        }
        let coverage = catalogue
            .maps
            .iter()
            .find(|m| m.file == file)
            .map_or("", |m| m.coverage);
        let kind = graph.scopes[id].map_kind()?;
        let name = if kind == model::MapKind::Infra {
            title(label)
        } else {
            label.to_string()
        };
        let mut hosted = original;
        replace_once(
            &mut hosted,
            "<header class=\"bar\">",
            &format!(
                "<header class=\"bar\"><a class=\"home-link\" href=\"../index.html\" aria-label=\"Topo home\">{BACK}<span>Topo</span></a>"
            ),
        )?;
        replace_once(
            &mut hosted,
            "<a class=\"review-link\" href=\"index.html\">Review</a>",
            "",
        )?;
        let head = hosted.find("</head>").ok_or("Map head missing")?;
        hosted.insert_str(head, &format!("<style>{FRESH_CSS}{BAR_CSS}</style>"));
        let body = hosted.rfind("</body>").ok_or("Map body missing")?;
        let cov = if coverage.is_empty() {
            String::new()
        } else {
            format!("<span class=\"cov\">{}</span>", capitalize(coverage))
        };
        hosted.insert_str(body, &format!("<footer class=\"fresh-bar\" aria-label=\"Freshness\"><p class=\"fresh\" {}>{FRESH_LINE}</p>{cov}{}</footer><script>{FRESH_JS}</script>", fresh_attrs(Some(&name), &graph.collected_at, &records.iter().map(|(d, n)| (d.clone(), *n)).collect::<Vec<_>>(), hours)?, basis(hours)));
        page_metadata.push(json!({"scope":id,"kind":graph.scopes[id].kind,"map_kind":kind,"file":format!("maps/{file}"),"generated_at":graph.collected_at,"observed_dates":records.keys().collect::<Vec<_>>()}));
        entries.push(Entry {
            id,
            file,
            label,
            kind,
            coverage,
            dates: records.into_iter().collect(),
        });
        pages.push((file, hosted));
    }
    if !seen.contains("index.html") {
        return Err("Default map missing".into());
    }
    // The home stays short: identities and their map coverage live on the project page.
    let mut coverage = String::from("<h2>Dated projects and map coverage</h2><ul>");
    for project in &catalogue.projects {
        coverage.push_str(&format!("<li>Project · {} — ", escape(project.name)));
        if let Some(map) = catalogue
            .maps
            .iter()
            .find(|m| Some(m.scope) == project.repo_scope)
        {
            coverage.push_str(&format!(
                "<a href=\"../maps/{}\">Repo · partial source map</a>",
                escape(map.file)
            ));
        } else {
            coverage.push_str("Repo · source map unavailable");
        }
        coverage.push_str(&format!(
            "; Module · {}; Architecture · unavailable; runtime/source binding unverified</li>",
            project.module_coverage
        ));
    }
    coverage.push_str("</ul>");
    let home = home_html(&entries, &graph, projects.len(), hours)?;
    let mut state_html = String::new();
    let parsed = Parser::new_ext(&markdown, Options::ENABLE_TABLES).map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        other => other,
    });
    html::push_html(&mut state_html, parsed);
    let project_page = format!(
        "<p><a href=\"../index.html\">Topo</a> · <a href=\"projects.json\">JSON evidence</a> · <a href=\"projects.md\">Markdown</a></p>{}{state_html}{coverage}",
        freshness(&generated, &state_dates, hours)?
    );
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    directory(out)?;
    directory(&out.join("maps"))?;
    directory(&out.join("projects"))?;
    write(&out.join("index.html"), home.as_bytes())?;
    write(
        &out.join("projects/index.html"),
        page("Project state", &project_page).as_bytes(),
    )?;
    write(&out.join("projects/projects.json"), &state_bytes)?;
    write(&out.join("projects/projects.md"), markdown.as_bytes())?;
    write(
        &out.join("catalogue.json"),
        &serde_json::to_vec_pretty(&catalogue)?,
    )?;
    write(&out.join("maps/graph.json"), &graph_bytes)?;
    write(&out.join("maps/manifest.json"), &manifest_bytes)?;
    for (file, hosted) in pages {
        write(&out.join("maps").join(file), hosted.as_bytes())?;
    }
    let hosted_manifest = json!({"schema":"topo.home.v1","generated_at":generated,"source_revision":source_revision,"graph_revision":revision,"stale_after_hours":hours,"catalogue":{"schema":"topo.catalogue.v1","file":"catalogue.json"},"coverage":{"projects":projects.len(),"source_repositories":repo_count,"partial":true,"runtime_recollected":false},"project_state":{"collected_at":inventory["collected_at"],"observed_dates":state_dates},"pages":page_metadata});
    write(
        &out.join("manifest.json"),
        &serde_json::to_vec_pretty(&hosted_manifest)?,
    )?;
    println!(
        "Private catalogue composed: {} project rows, {repo_count} partial repository maps; original observation dates preserved.",
        projects.len()
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("topo-home: {error}");
        std::process::exit(1);
    }
}
