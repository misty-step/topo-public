use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
static FIXTURE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(std::env::var("HOME").unwrap())
            .join(".cache/tmp")
            .join(format!(
                "topo-test-{}-{}-{}",
                std::process::id(),
                FIXTURE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn repo(&self, name: &str) {
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .arg(self.0.join(name))
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(self.0.join(name))
                .args(["add", "."])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(self.0.join(name))
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "core.hooksPath=/dev/null",
                    "commit",
                    "-qm",
                    "fixture",
                    "--no-gpg-sign"
                ])
                .status()
                .unwrap()
                .success()
        );
    }
    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_topo"))
            .args(args)
            .output()
            .unwrap()
    }
    fn graph(&self) -> Value {
        serde_json::from_slice(&fs::read(self.0.join("out/graph.json")).unwrap()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn source_fixture() -> Fixture {
    let f = Fixture::new();
    for name in ["alpha", "beta"] {
        f.write(&format!("{name}/package.json"),r#"{"dependencies":{"zod":"^4"},"scripts":{"preinstall":"touch APPLICATION_EXECUTED"}}"#);
        f.write(&format!("{name}/src/main.ts"),"// import 'COMMENT_IS_NOT_AN_IMPORT';\nimport {child} from './child.js';\nimport 'zod';\nimport 'node:fs';\nconst text=\"require('STRING_IS_NOT_AN_IMPORT')\";\nthrow new Error('APPLICATION MUST NEVER EXECUTE');\nimport 'bun:test';\n");
        f.write(
            &format!("{name}/src/child.ts"),
            "import '@/unresolved-alias';export const child=1;\n",
        );
        f.repo(name);
    }
    f.write(
        "alpha/Cargo.toml",
        "[package]\nname='fixture'\nversion='0.1.0'\n[dependencies]\nserde='1'\n",
    );
    f.write(
        "alpha/src/lib.rs",
        "use serde::Serialize;\nuse std::path::Path;\n",
    );
    f.write("alpha/go.mod", "module example.invalid/alpha\n\ngo 1.26\n");
    f.write(
        "alpha/main.go",
        "package main\nimport (\"database/sql\";\"example.invalid/alpha/store\")\n",
    );
    f.write(
        "alpha/store/store.go",
        "package store\nimport \"database/sql\"\n",
    );
    f.write("alpha/store/other.go", "package store\n");
    // Ignored credential file, leaf symlink and replaced directory symlink must
    // not enter parser input or exported relationships.
    f.write("alpha/.gitignore", ".env\n");
    f.write("alpha/.env", "PRIVATE_VALUE=ENV_MUST_NOT_BE_READ\n");
    f.write("outside.ts", "import 'OUTSIDE_MUST_NOT_BE_READ';\n");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(f.0.join("outside.ts"), f.0.join("alpha/src/link.ts")).unwrap();
        f.write("alpha/scoped/original.ts", "export const original=1;\n");
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(f.0.join("alpha"))
                .args(["add", "scoped"])
                .status()
                .unwrap()
                .success()
        );
        fs::remove_dir_all(f.0.join("alpha/scoped")).unwrap();
        fs::create_dir(f.0.join("outside")).unwrap();
        fs::write(
            f.0.join("outside/original.ts"),
            "import 'ANCESTOR_MUST_NOT_BE_READ';",
        )
        .unwrap();
        symlink(f.0.join("outside"), f.0.join("alpha/scoped")).unwrap();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.write(
            "alpha/.git/fsmonitor-test",
            "#!/bin/sh\ntouch FSMONITOR_EXECUTED\nprintf 'token\\0'\n",
        );
        let hook = f.0.join("alpha/.git/fsmonitor-test");
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(f.0.join("alpha"))
                .args(["config", "core.fsmonitor"])
                .arg(hook)
                .status()
                .unwrap()
                .success()
        );
    }
    let args = vec![
        "run".into(),
        "--repo".into(),
        format!("alpha={}", f.0.join("alpha").display()),
        "--repo".into(),
        format!("beta={}", f.0.join("beta").display()),
        "--out".into(),
        f.0.join("out").to_string_lossy().to_string(),
    ];
    let output = f.run(&args.iter().map(String::as_str).collect::<Vec<_>>());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    f
}
#[test]
fn real_collectors_preserve_shared_identity_direction_and_read_boundary() {
    let f = source_fixture();
    let g = f.graph();
    let entities = g["entities"].as_object().unwrap();
    let edges = g["edges"].as_object().unwrap();
    assert!(entities.contains_key("package:npm:zod"));
    assert!(entities.contains_key("package:node:bun:test"));
    assert!(!entities.values().any(|n| n["label"] == "node:bun:test"));
    assert!(entities.contains_key("package:rust:serde"));
    assert!(entities.contains_key("package:go:database/sql"));
    assert!(edges.values().any(|e| e["from"] == "file:alpha:src/main.ts"
        && e["to"] == "file:alpha:src/child.ts"
        && e["line"] == 2));
    for repo in ["alpha", "beta"] {
        assert!(edges.values().any(
            |e| e["from"] == format!("file:{repo}:src/main.ts") && e["to"] == "package:npm:zod"
        ));
    }
    assert_eq!(
        entities["source-package:alpha:store"]["members"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        entities
            .values()
            .any(|n| n["label"] == "@/unresolved-alias · unresolved" && n["status"] == "unknown")
    );
    let text = serde_json::to_string(&g).unwrap();
    for marker in [
        "ENV_MUST_NOT_BE_READ",
        "OUTSIDE_MUST_NOT_BE_READ",
        "ANCESTOR_MUST_NOT_BE_READ",
        "COMMENT_IS_NOT_AN_IMPORT",
        "STRING_IS_NOT_AN_IMPORT",
    ] {
        assert!(!text.contains(marker), "Leaked {marker}");
    }
    assert!(!f.0.join("alpha/APPLICATION_EXECUTED").exists());
    assert!(
        !f.0.join("alpha/FSMONITOR_EXECUTED").exists(),
        "Git status executed the target's fsmonitor hook"
    );
    assert_eq!(
        g["sources"]["source:alpha:src/main.ts"]["sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert!(
        g["collections"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["coverage"] == "partial")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(f.0.join("out")).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(f.0.join("out/graph.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[test]
fn query_rejects_wrong_revision_and_out_of_scope_without_overwriting_artifacts() {
    let f = source_fixture();
    let graph = f.0.join("out/graph.json");
    let path = graph.to_str().unwrap();
    let output = f.run(&[
        "query",
        "--graph",
        path,
        "--scope",
        "module:alpha:src",
        "--entity",
        "file:beta:src/main.ts",
    ]);
    assert!(!output.status.success());
    let output = f.run(&["query", "--graph", path, "--revision", "stale"]);
    assert!(!output.status.success());
    let output = f.run(&[
        "query",
        "--graph",
        path,
        "--scope",
        "module:alpha:src",
        "--entity",
        "file:alpha:src/child.ts",
        "--direction",
        "consumers",
    ]);
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["relationships"].as_array().unwrap().len(), 1);
    assert_eq!(result["relationships"][0]["from"], "file:alpha:src/main.ts");
    let before = fs::read(&graph).unwrap();
    let output = f.run(&[
        "render",
        "--graph",
        path,
        "--out",
        f.0.join("out").to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    assert_eq!(before, fs::read(&graph).unwrap());
}
#[test]
fn dated_import_retains_provenance_and_excludes_proposed_resources() {
    let f = Fixture::new();
    let raw = json!({"schema":"topo.infrastructure-map.v1","sources":[{"id":"old","label":"Dated API read","date":"2022-01-01","level":"api","result":"Known resource present"}],"nodes":[{"id":"known","label":"Known </script><img src=x>","kind":"product","category":"business","status":"live","sources":["old"]},{"id":"proposed","label":"Not observed","kind":"provider","category":"compute","status":"configured","sources":["old"]}],"edges":[],"scenarios":[{"id":"current","node_ids":["known"],"edge_ids":[]},{"id":"proposal","node_ids":["proposed"],"edge_ids":[]}]});
    f.write("snapshot.json", &serde_json::to_string(&raw).unwrap());
    let target = format!("dated={}", f.0.join("snapshot.json").display());
    let out = f.0.join("out");
    let result = f.run(&["run", "--snapshot", &target, "--out", out.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let g = f.graph();
    assert_eq!(
        g["sources"]["snapshot:dated:source:old"]["date"],
        "2022-01-01"
    );
    assert!(g["entities"].get("snapshot:dated:proposed").is_none());
    let manifest: Value =
        serde_json::from_slice(&fs::read(out.join("manifest.json")).unwrap()).unwrap();
    let file = manifest["scopes"][0]["file"].as_str().unwrap();
    let html = fs::read_to_string(out.join(file)).unwrap();
    assert!(!html.contains("Known </script><img"));
    assert!(html.contains("Known \\u003c/script\\u003e\\u003cimg"));
}

#[test]
fn project_state_export_is_private_and_cannot_inject_or_overwrite() {
    let f = Fixture::new();
    let input = json!({
        "schema":"topo.project-state.v1", "collected_at":"2026-10-12T00:00:00Z",
        "inventory_basis":"accepted inventory", "option_basis":"Option A",
        "coverage":"one selected repository", "gaps":["Runtime health unknown"],
        "projects":[{
            "name":"owner/project", "purpose":"</table><script>alert(1)</script>\n# Forged",
            "purpose_source":"local README", "call":"archive", "reason":"Proposal only",
            "activity":{"date":null,"commit":null,"subject":"Unknown","source":"Git inaccessible"},
            "runtime":[{"status":"unknown","home":"No confirmed deployment","date":"2026-10-12","source":"Incomplete survey"}]
        }]
    });
    f.write("input.json", &serde_json::to_string(&input).unwrap());
    let input_path = f.0.join("input.json");
    let out = f.0.join("out");
    let args = [
        "state",
        "--input",
        input_path.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ];
    let result = f.run(&args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let md = fs::read(out.join("projects.md")).unwrap();
    let rendered = String::from_utf8(md.clone()).unwrap();
    assert!(!rendered.contains("<script>"));
    assert!(!rendered.contains("\n# Forged"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&out).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(out.join("projects.md"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(out.join("projects.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert!(!f.run(&args).status.success());
    assert_eq!(fs::read(out.join("projects.md")).unwrap(), md);
}

#[test]
fn private_home_joins_identities_without_hiding_unmapped_projects_or_inventing_architecture() {
    let f = Fixture::new();
    f.write("alpha/Cargo.toml", "[package]\nname='alpha'\nversion='0.1.0'\n");
    f.write("alpha/src/main.rs", "fn main() {}\n");
    f.repo("alpha");
    let snapshot = json!({"schema":"topo.infrastructure-map.v1","sources":[{"id":"dated","label":"Bounded provider read","date":"2022-01-01","level":"api"}],"nodes":[{"id":"vendor","label":"Real hosting vendor","kind":"provider","category":"compute","status":"live","sources":["dated"]},{"id":"account","label":"Observed account","kind":"account","category":"compute","status":"live","sources":["dated"]}],"edges":[{"id":"membership","from":"vendor","to":"account","relation":"contains","type":"other","evidence":"read","sources":["dated"]}],"scenarios":[{"id":"current","node_ids":["vendor","account"],"edge_ids":["membership"]}]});
    f.write("infra.json", &snapshot.to_string());
    let repo = format!("alpha={}", f.0.join("alpha").display());
    let infra = format!("infra={}", f.0.join("infra.json").display());
    assert!(f.run(&["run", "--repo", &repo, "--snapshot", &infra, "--out", f.0.join("maps").to_str().unwrap()]).status.success());
    let row = |name: &str, aliases: Vec<&str>| json!({
        "name":name,"aliases":aliases,"purpose":"Known dated purpose","purpose_source":"README",
        "activity":{"date":null,"commit":null,"subject":"Unknown","source":"Git unavailable"},
        "runtime":[{"status":"unknown","home":"Unresolved host","date":"2022-01-01","source":"Bounded survey"}],
        "call":"maintain","reason":"Proposal only"
    });
    let inventory = json!({"schema":"topo.project-state.v1","collected_at":"2022-01-01","inventory_basis":"Dated census","option_basis":"Proposal only","coverage":"Partial","gaps":["No health probe"],"projects":[row("owner/alpha",vec!["alpha"]),row("owner/unreadable",vec![])]});
    f.write("input.json", &inventory.to_string());
    assert!(f.run(&["state","--input",f.0.join("input.json").to_str().unwrap(),"--out",f.0.join("state").to_str().unwrap()]).status.success());
    let home = f.0.join("home");
    let result = Command::new(env!("CARGO_BIN_EXE_topo-home")).args(["--state",f.0.join("state").to_str().unwrap(),"--maps",f.0.join("maps").to_str().unwrap(),"--out",home.to_str().unwrap()]).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let catalogue: Value = serde_json::from_slice(&fs::read(home.join("catalogue.json")).unwrap()).unwrap();
    assert_eq!(catalogue["projects"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>(), ["project:owner/alpha","project:owner/unreadable"].into_iter().collect());
    let mapped = &catalogue["projects"][0];
    assert_eq!(mapped["repo_scope"], "repo:alpha");
    assert_eq!(mapped["source_coverage"], "partial");
    assert_eq!(mapped["module_coverage"], "partial-directory-only");
    assert_eq!(mapped["architecture_coverage"], "unavailable");
    assert_eq!(mapped["runtime_source_binding"], "unverified");
    assert_eq!(mapped["evidence"]["runtime"], inventory["projects"][0]["runtime"]);
    assert_eq!(catalogue["projects"][1]["source_coverage"], "unavailable");
    assert!(catalogue["projects"][1]["repo_scope"].is_null());
    assert_eq!(catalogue["project_state_observed_at"], "2022-01-01");
    let kinds = catalogue["maps"].as_array().unwrap().iter().map(|m| m["map_kind"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(kinds, ["Project","Repo","Module","Infra"].into_iter().collect());
    // The home leads with the typed Infra map's graph counts, keeps original dates and no project identity; a repo reads 1 of 2.
    let html = fs::read_to_string(home.join("index.html")).unwrap();
    assert!(html.contains("<dt>Vendors</dt><dd>1</dd>") && html.contains("<dt>Accounts</dt><dd>1</dd>") && html.contains("2022-01-01"));
    assert!(html.contains("1 of 2") && !html.contains("owner/unreadable"));
    // Every page reaches its neighbours by a link that names a file, so each return route works hosted and from disk.
    let mut pages = vec![home.join("index.html"), home.join("projects/index.html")];
    pages.extend(fs::read_dir(home.join("maps")).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "html")));
    for page in pages {
        let text = fs::read_to_string(&page).unwrap();
        for href in text.split("href=\"").skip(1).map(|s| s.split('"').next().unwrap()).filter(|h| !h.is_empty() && !h.starts_with("http") && !h.starts_with("data:") && !h.starts_with('#')) {
            let target = page.parent().unwrap().join(href.split(['#', '?']).next().unwrap());
            assert!(target.is_file(), "{} links to {href}, which is not a file", page.display());
        }
    }
    let graph: Value = serde_json::from_slice(&fs::read(home.join("maps/graph.json")).unwrap()).unwrap();
    assert_eq!(graph["sources"]["snapshot:infra:source:dated"]["date"], "2022-01-01");
    assert_eq!(graph["entities"]["snapshot:infra:vendor"]["entity_kind"], "provider");
    assert_eq!(graph["edges"]["snapshot:infra:membership"]["relation"], "contains");
    let queried = f.run(&["query","--graph",home.join("maps/graph.json").to_str().unwrap(),"--entity","snapshot:infra:vendor","--direction","dependencies"]);
    assert!(queried.status.success());
    let queried: Value = serde_json::from_slice(&queried.stdout).unwrap();
    assert_eq!(queried["relationships"], json!([]), "Account containment became a runtime dependency");
    // Conflicting alias claims must fail before creating a replacement publication.
    let before = fs::read(home.join("catalogue.json")).unwrap();
    let mut conflicting = inventory.clone();
    conflicting["projects"][1]["aliases"] = json!(["ALPHA"]);
    f.write("state/projects.json", &conflicting.to_string());
    let refused = f.0.join("refused");
    assert!(!Command::new(env!("CARGO_BIN_EXE_topo-home")).args(["--state",f.0.join("state").to_str().unwrap(),"--maps",f.0.join("maps").to_str().unwrap(),"--out",refused.to_str().unwrap()]).status().unwrap().success());
    assert!(!refused.exists());
    assert_eq!(fs::read(home.join("catalogue.json")).unwrap(), before);
}

#[test]
fn regeneration_uses_explicit_fictional_scopes_and_rejects_unsafe_ids() {
    let f = Fixture::new();
    f.write("example/demo.git/src/main.rs", "fn main() {}\n");
    f.repo("example/demo.git");
    let state = json!({
        "schema":"topo.project-state.v1", "collected_at":"2022-01-01",
        "inventory_basis":"Fictional census", "option_basis":"Proposal only",
        "coverage":"Partial", "gaps":[], "projects":[{
            "name":"example/demo", "aliases":[], "purpose":"Fictional demo",
            "purpose_source":"Fixture", "activity":{"date":null,"commit":null,"subject":"Unknown","source":"Fixture"},
            "runtime":[{"status":"unknown","home":"Unknown","date":"2022-01-01","source":"Fixture"}],
            "call":"maintain", "reason":"Proposal only"
        }]
    });
    f.write("state.json", &state.to_string());
    f.write("repos.json", "[\"example/demo\"]");
    let snapshot = json!({
        "schema":"topo.infrastructure-map.v1",
        "sources":[{"id":"dated","label":"Fictional dated input","date":"2022-01-01","level":"api"}],
        "nodes":[{"id":"vendor","label":"Example vendor","kind":"provider","category":"compute","status":"configured","sources":["dated"]}],
        "edges":[], "scenarios":[{"id":"current","node_ids":["vendor"],"edge_ids":[]}]
    });
    let maps: Vec<_> = ["example-one", "example-two", "example-three"]
        .into_iter().map(|id| json!({"id":id,"label":id,"snapshot":snapshot})).collect();
    let input = json!({"schema":"topo.infrastructure.v1","maps":maps,"freshness":{"schema":"topo.infrastructure-freshness.v1"}});
    f.write("infra.json", &input.to_string());
    let run = |out: &str| Command::new(env!("CARGO_BIN_EXE_topo-regenerate"))
        .args(["--state", f.0.join("state.json").to_str().unwrap(),
            "--snapshot", f.0.join("infra.json").to_str().unwrap(),
            "--repos", f.0.join("repos.json").to_str().unwrap(),
            "--out", f.0.join(out).to_str().unwrap()])
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", format!("url.file://{}/.insteadOf", f.0.display()))
        .env("GIT_CONFIG_VALUE_0", "https://github.com/")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .output().unwrap();
    let result = run("site");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let catalogue: Value = serde_json::from_slice(&fs::read(f.0.join("site/catalogue.json")).unwrap()).unwrap();
    let scopes: std::collections::BTreeSet<_> = catalogue["maps"].as_array().unwrap().iter()
        .filter(|map| map["map_kind"] == "Infra").map(|map| map["scope"].as_str().unwrap()).collect();
    assert_eq!(scopes, ["snapshot:example-one", "snapshot:example-two", "snapshot:example-three"].into_iter().collect());
    let html = fs::read_to_string(f.0.join("site/index.html")).unwrap();
    assert!(html.contains("Example One") && html.contains("Example Three") && html.contains("2022-01-01"));
    for bad in [json!([]), json!([maps[0], maps[0]]), json!([{ "id":"../outside", "snapshot":snapshot }])] {
        let mut invalid = input.clone();
        invalid["maps"] = bad;
        f.write("infra.json", &invalid.to_string());
        assert!(!run("refused").status.success());
        assert!(!f.0.join("refused").exists());
    }
}

fn generated_site_fixture() -> Fixture {
    let f = Fixture::new();
    for target in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu",
    ] {
        f.write(
            &format!("releases/topo-{target}"),
            "synthetic release fixture",
        );
    }
    f.write("notices.html", "<p>Synthetic test notices.</p>");
    let result = Command::new(env!("CARGO_BIN_EXE_topo-site"))
        .arg("--out")
        .arg(f.0.join("site"))
        .arg("--releases")
        .arg(f.0.join("releases"))
        .arg("--notices")
        .arg(f.0.join("notices.html"))
        .args(["--native-version", "0.0.0", "--native-source", "0000000000000000000000000000000000000000"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    f
}

#[test]
fn generated_site_internal_hrefs_resolve() {
    let f = generated_site_fixture();
    let result = Command::new("python3")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tools/check_site_links.py"))
        .arg(f.0.join("site"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn demo_filters_optional_inspector_children_without_changing_evidence() {
    let f = generated_site_fixture();
    let reference = include_str!("../design/reference.html");
    let page = fs::read_to_string(f.0.join("site/demo/index.html")).unwrap();
    let data = |html: &str| {
        html.split("<script type=\"application/json\" id=\"infra-data\">")
            .nth(1)
            .unwrap()
            .split("</script>")
            .next()
            .unwrap()
            .to_owned()
    };
    assert_eq!(data(&page), data(reference));
    let fixture: serde_json::Value = serde_json::from_str(&data(&page)).unwrap();
    assert_eq!(fixture["nodes"].as_array().unwrap().len(), 43);
    assert_eq!(fixture["edges"].as_array().unwrap().len(), 69);
    assert_eq!(page.matches("replaceDetailChildren(").count(), 4);
    assert_eq!(page.matches("E.dBody.replaceChildren(").count(), 1);
    assert!(page.contains("children.filter(child => child != null && child !== false)"));
    assert!(page.contains("data-fullscreen-toggle"));
}
