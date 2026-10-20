mod collect;
mod imports;
mod model;
mod project;
mod render;
mod state;
mod workspace;
use model::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

fn argument(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|p| p[0] == key).map(|p| p[1].clone())
}
fn values(args: &[String], key: &str) -> Vec<String> {
    args.windows(2)
        .filter(|p| p[0] == key)
        .map(|p| p[1].clone())
        .collect()
}
fn input(s: &str) -> Result<(String, PathBuf)> {
    let (name, path) = s.split_once('=').ok_or("Expected namespace=path")?;
    if name.is_empty()
        || name.len() > 128
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-_/".contains(c))
        || name.contains("..")
    {
        return Err("Invalid caller namespace".into());
    }
    Ok((name.to_string(), PathBuf::from(path)))
}
fn read_graph(path: &str) -> Result<Graph> {
    let mut g: Graph = serde_json::from_slice(&fs::read(path)?)?;
    if g.schema != "topo.graph.v1" {
        return Err("Unsupported Topo graph schema".into());
    }
    let expected = g.revision.clone();
    g.seal()?;
    if g.revision != expected {
        return Err("Graph revision does not match its content".into());
    }
    Ok(g)
}
fn query(g: &Graph, args: &[String]) -> Result<Value> {
    if argument(args, "--revision").is_some_and(|r| r != g.revision) {
        return Err("Graph revision mismatch".into());
    }
    let scope_id = argument(args, "--scope");
    let (permitted, edges) = if let Some(scope) = scope_id.as_ref() {
        let scope = g.scopes.get(scope).ok_or("Unknown scope")?;
        project::selected(g, scope)
    } else {
        (
            g.entities.keys().cloned().collect(),
            g.edges.values().cloned().collect(),
        )
    };
    if let Some(id) = argument(args, "--entity") {
        if !permitted.contains(&id) {
            return Err("Entity is outside the requested supplied scope".into());
        }
        let direction = argument(args, "--direction").unwrap_or("both".into());
        if !["both", "dependencies", "consumers"].contains(&direction.as_str()) {
            return Err("Direction must be dependencies, consumers or both".into());
        }
        let relationships: Vec<_> = edges
            .iter()
            .filter(|e| {
                e.relation != "contains"
                    && match direction.as_str() {
                        "dependencies" => e.from == id,
                        "consumers" => e.to == id,
                        _ => e.from == id || e.to == id,
                    }
            })
            .collect();
        let mut neighbors = BTreeSet::new();
        for e in &relationships {
            neighbors.insert(e.from.clone());
            neighbors.insert(e.to.clone());
        }
        let sources: BTreeSet<_> = relationships
            .iter()
            .flat_map(|e| e.sources.iter().cloned())
            .chain(g.entities[&id].sources.iter().cloned())
            .collect();
        Ok(
            json!({"revision":g.revision,"scope":scope_id,"entity":g.entities.get(&id),"relationships":relationships,"neighbors":neighbors.iter().filter_map(|id|g.entities.get(id)).collect::<Vec<_>>(),"sources":sources.iter().filter_map(|id|g.sources.get(id)).collect::<Vec<_>>()}),
        )
    } else {
        Ok(
            json!({"revision":g.revision,"scope":scope_id,"entities":permitted.iter().filter_map(|id|g.entities.get(id)).collect::<Vec<_>>(),"edges":edges,"collections":g.collections}),
        )
    }
}
fn validate_args(args: &[String]) -> Result<()> {
    let allowed: &[&str] = match args[0].as_str() {
        "run" => &["--repo", "--snapshot", "--out"],
        "render" => &["--graph", "--out"],
        "state" => &["--input", "--out"],
        "query" => &[
            "--graph",
            "--scope",
            "--entity",
            "--direction",
            "--revision",
        ],
        "scopes" => &["--graph"],
        _ => return Err("Unknown command; use topo --help".into()),
    };
    if !args[1..].len().is_multiple_of(2) {
        return Err("Every flag requires a value".into());
    }
    let mut seen = BTreeSet::new();
    for pair in args[1..].chunks_exact(2) {
        if !allowed.contains(&pair[0].as_str()) || pair[1].starts_with("--") {
            return Err("Unknown flag or missing value".into());
        }
        if !seen.insert(&pair[0]) && !["--repo", "--snapshot"].contains(&pair[0].as_str()) {
            return Err("Duplicate flag".into());
        }
    }
    Ok(())
}
fn run() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|a| a == "--version" || a == "version")
    {
        println!("topo {} ({})", env!("CARGO_PKG_VERSION"), option_env!("TOPO_SOURCE_REVISION").unwrap_or("development"));
        return Ok(());
    }
    if args.is_empty() || args[0] == "help" || args[0] == "--help" {
        println!(
            "Topo — local, read-only source maps and dated inventory imports\n\n  topo run --repo NAME=PATH [--repo NAME=PATH ...] [--snapshot NAME=FILE] --out NEW_DIR\n  topo render --graph FILE --out NEW_DIR\n  topo state --input FILE --out NEW_DIR\n  topo query --graph FILE [--scope SCOPE_ID] [--entity ID] [--direction dependencies|consumers|both] [--revision REVISION]\n  topo scopes --graph FILE\n  topo --version\n  topo --licenses > notices.html\n\nNo app execution, model calls, provider mutation or automatic retirement. Source/config relationships are not runtime proof."
        );
        return Ok(());
    }
    if args.len() == 1 && args[0] == "--licenses" {
        print!("{}", include_str!("../site/third-party.html"));
        return Ok(());
    }
    validate_args(&args)?;
    match args[0].as_str() {
        "run" => {
            let out = argument(&args, "--out").ok_or("--out is required")?;
            let out = Path::new(&out);
            if out.exists() {
                return Err("Output already exists; use a fresh run directory".into());
            }
            let mut g = Graph::new()?;
            let mut names = BTreeSet::new();
            for target in values(&args, "--repo") {
                let (name, root) = input(&target)?;
                if !names.insert(name.clone()) {
                    return Err("Duplicate input namespace".into());
                }
                collect::repository(&mut g, &collect::Input { name, root })?;
            }
            for target in values(&args, "--snapshot") {
                let (name, path) = input(&target)?;
                if !names.insert(name.clone()) {
                    return Err("Duplicate input namespace".into());
                }
                collect::snapshot(&mut g, &name, &path)?;
            }
            if g.collections.is_empty() {
                return Err("Supply at least one --repo or --snapshot".into());
            }
            let repositories: Vec<_> = g
                .entities
                .values()
                .filter(|n| n.entity_kind == "repository")
                .map(|n| n.id.clone())
                .collect();
            if !repositories.is_empty() {
                g.scopes.insert(
                    "portfolio".into(),
                    Scope {
                        id: "portfolio".into(),
                        label: "Selected repositories".into(),
                        kind: "portfolio".into(),
                        repo: None,
                        prefix: None,
                        focal_ids: repositories,
                    },
                );
            }
            g.seal()?;
            render::bundle(&g, out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"revision":g.revision,"entities":g.entities.len(),"relationships":g.edges.len(),"sources":g.sources.len(),"scopes":g.scopes.len(),"output":out,"collections":g.collections})
                )?
            );
        }
        "render" => {
            let g = read_graph(&argument(&args, "--graph").ok_or("--graph is required")?)?;
            render::bundle(
                &g,
                Path::new(&argument(&args, "--out").ok_or("--out is required")?),
            )?;
        }
        "query" => {
            let g = read_graph(&argument(&args, "--graph").ok_or("--graph is required")?)?;
            println!("{}", serde_json::to_string_pretty(&query(&g, &args)?)?);
        }
        "scopes" => {
            let g = read_graph(&argument(&args, "--graph").ok_or("--graph is required")?)?;
            println!("{}", serde_json::to_string_pretty(&g.scopes)?);
        }
        "state" => {
            let result = state::export(
                Path::new(&argument(&args, "--input").ok_or("--input is required")?),
                Path::new(&argument(&args, "--out").ok_or("--out is required")?),
            )?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => return Err("Unknown command; use topo --help".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("topo: {error}");
        std::process::exit(1);
    }
}
