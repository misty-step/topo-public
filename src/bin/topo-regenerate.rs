//! Opt-in publication preparation. The ordinary `topo` collector stays local.
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

struct Work(PathBuf);
impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execute(command: &mut Command, operation: &str) -> Result<()> {
    // Source paths, repository names and collection receipts must not enter CI logs.
    let output = command
        .output()
        .map_err(|_| format!("{operation} could not start"))?;
    if !output.status.success() {
        return Err(format!("{operation} failed ({})", output.status).into());
    }
    Ok(())
}
fn value<'a>(args: &'a [String], flag: &str) -> Result<&'a str> {
    args.chunks_exact(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
        .ok_or_else(|| format!("Missing {flag}").into())
}
fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["--help"] {
        println!(
            "topo-regenerate --state PROJECT_STATE_JSON --snapshot INFRASTRUCTURE_JSON --repos REPOSITORIES_JSON --out NEW_DIR\nRecollect selected GitHub default branches, retain scoped observation dates, prepare private static assets. No deployment."
        );
        return Ok(());
    }
    let flags = ["--state", "--snapshot", "--repos", "--out"];
    if args.len() != 8
        || flags
            .iter()
            .any(|flag| args.chunks_exact(2).filter(|pair| pair[0] == *flag).count() != 1)
        || args
            .chunks_exact(2)
            .any(|pair| pair[1].is_empty() || pair[1].starts_with("--"))
    {
        return Err("Expected --state, --snapshot, --repos and --out exactly once".into());
    }
    let out = Path::new(value(&args, "--out")?);
    if out.exists() {
        return Err("Output already exists; use a fresh run directory".into());
    }
    let infrastructure: Value =
        serde_json::from_reader(fs::File::open(value(&args, "--snapshot")?)?)
            .map_err(|_| "Invalid infrastructure payload")?;
    let infrastructure_scopes = infrastructure["maps"]
        .as_array()
        .ok_or("Missing infrastructure scopes")?;
    let scope_ids: std::collections::BTreeSet<_> = infrastructure_scopes
        .iter()
        .filter_map(|m| m["id"].as_str())
        .collect();
    if infrastructure["schema"] != "topo.infrastructure.v1"
        || infrastructure_scopes.is_empty()
        || scope_ids.len() != infrastructure_scopes.len()
        || scope_ids.iter().any(|id| {
            id.is_empty()
                || id.len() > 64
                || !id.as_bytes()[0].is_ascii_alphanumeric()
                || !id.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_'
                })
        })
        || infrastructure["freshness"]["schema"] != "topo.infrastructure-freshness.v1"
    {
        return Err("Expected reviewed infrastructure scopes with freshness".into());
    }
    let repositories: Vec<String> = serde_json::from_slice(&fs::read(value(&args, "--repos")?)?)
        .map_err(|_| "Repository selection must be a JSON array of GitHub identities")?;
    if repositories.is_empty() {
        return Err("No repositories selected".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for repo in &repositories {
        let Some((owner, name)) = repo.split_once('/') else {
            return Err("Repository selection contains an invalid GitHub identity".into());
        };
        if repo.len() > 128
            || repo.contains("..")
            || [owner, name].iter().any(|part| {
                part.is_empty()
                    || part.starts_with('.')
                    || !part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
            })
            || !seen.insert(repo.to_ascii_lowercase())
        {
            return Err(
                "Repository selection contains an invalid or duplicate GitHub identity".into(),
            );
        }
    }
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let work = Work(parent.join(format!(
        ".topo-work-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    )));
    fs::create_dir(&work.0)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&work.0, fs::Permissions::from_mode(0o700))?;
    }
    let binaries = env::current_exe()?
        .parent()
        .ok_or("Missing binary directory")?
        .to_path_buf();
    let mut collect = Command::new(binaries.join("topo"));
    collect.arg("run");
    for (index, repo) in repositories.iter().enumerate() {
        let checkout = work.0.join(format!("repo-{index}"));
        execute(
            Command::new("git")
                .args([
                    "-c",
                    "core.hooksPath=/dev/null",
                    "-c",
                    "credential.helper=",
                    "-c",
                    "credential.helper=!gh auth git-credential",
                    "clone",
                    "--depth",
                    "1",
                    "--no-tags",
                ])
                .arg(format!("https://github.com/{repo}.git"))
                .arg(&checkout)
                .env("GIT_TERMINAL_PROMPT", "0"),
            &format!("Selected repository {index} checkout"),
        )?;
        collect
            .arg("--repo")
            .arg(format!("{repo}={}", checkout.display()));
    }
    let maps = work.0.join("maps");
    for scope in infrastructure_scopes {
        let id = scope["id"]
            .as_str()
            .ok_or("Missing infrastructure identity")?;
        let path = work.0.join(format!("{id}.json"));
        serde_json::to_writer(fs::File::create(&path)?, &scope["snapshot"])?;
        collect
            .arg("--snapshot")
            .arg(format!("{id}={}", path.display()));
    }
    collect.arg("--out").arg(&maps);
    execute(&mut collect, "Source collection")?;
    let state = work.0.join("state");
    execute(
        Command::new(binaries.join("topo"))
            .args(["state", "--input", value(&args, "--state")?, "--out"])
            .arg(&state),
        "Project-state export",
    )?;
    let site = work.0.join("site");
    execute(
        Command::new(binaries.join("topo-home"))
            .arg("--state")
            .arg(&state)
            .arg("--maps")
            .arg(&maps)
            .arg("--out")
            .arg(&site)
            .args(["--stale-after-hours", "48"]),
        "Private site composition",
    )?;
    let manifest: Value = serde_json::from_reader(fs::File::open(site.join("manifest.json"))?)?;
    if manifest["schema"] != "topo.home.v1"
        || manifest["coverage"]["source_repositories"].as_u64() != Some(repositories.len() as u64)
    {
        return Err("Prepared site manifest does not cover every selected repository".into());
    }
    let catalogue: Value = serde_json::from_reader(fs::File::open(site.join("catalogue.json"))?)?;
    let infra_maps = catalogue["maps"]
        .as_array()
        .ok_or("Missing typed catalogue maps")?;
    let typed_scopes: std::collections::BTreeSet<_> = infra_maps
        .iter()
        .filter(|map| map["map_kind"] == "Infra")
        .filter_map(|map| map["scope"].as_str())
        .collect();
    let expected_scopes: std::collections::BTreeSet<_> =
        scope_ids.iter().map(|id| format!("snapshot:{id}")).collect();
    let expected_scopes = expected_scopes.iter().map(String::as_str).collect();
    if catalogue["schema"] != "topo.catalogue.v1"
        || catalogue["graph_revision"] != manifest["graph_revision"]
        || typed_scopes != expected_scopes
    {
        return Err("Typed catalogue does not cover every infrastructure scope".into());
    }
    let freshness = site.join("infrastructure-freshness.json");
    serde_json::to_writer(fs::File::create(&freshness)?, &infrastructure["freshness"])?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(freshness, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(site, out)?;
    println!(
        "Private site prepared from {} fresh default-branch checkouts and {} dated infrastructure scopes; scoped provider observations are in infrastructure-freshness.json. No deployment.",
        repositories.len(), infrastructure_scopes.len()
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!(
            "topo-regenerate: Private site preparation failed: {error}; previous publication unchanged."
        );
        std::process::exit(1);
    }
}
