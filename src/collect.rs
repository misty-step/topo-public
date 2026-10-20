use crate::model::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
};
use syn::{spanned::Spanned, visit::Visit};

const MAX_FILE: u64 = 1024 * 1024;
const MAX_FILES: usize = 2000;
const MAX_BYTES: usize = 16 * 1024 * 1024;
pub struct Input {
    pub name: String,
    pub root: PathBuf,
}
// Descriptor-relative no-follow opens prevent both leaf and ancestor symlinks,
// including a concurrently replaced ancestor, from escaping the selected root.
#[cfg(unix)]
fn source_text(root: &Path, path: &str) -> std::io::Result<String> {
    use std::{
        ffi::CString,
        os::fd::{AsRawFd, FromRawFd},
    };
    let parts: Vec<_> = Path::new(path).components().collect();
    let mut handle = fs::File::open(root)?;
    for (index, part) in parts.iter().enumerate() {
        let Component::Normal(name) = part else {
            return Err(std::io::Error::other("Invalid path"));
        };
        let name = CString::new(name.as_encoded_bytes())
            .map_err(|_| std::io::Error::other("Invalid path"))?;
        let flags = libc::O_RDONLY
            | libc::O_NOFOLLOW
            | libc::O_CLOEXEC
            | if index + 1 < parts.len() {
                libc::O_DIRECTORY
            } else {
                libc::O_NONBLOCK
            };
        let fd = unsafe { libc::openat(handle.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        handle = unsafe { fs::File::from_raw_fd(fd) };
    }
    let metadata = handle.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_FILE {
        return Err(std::io::Error::other("Scope/budget exclusion"));
    }
    let mut text = String::new();
    handle.take(MAX_FILE + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_FILE {
        return Err(std::io::Error::other("File grew beyond budget"));
    }
    Ok(text)
}
#[cfg(not(unix))]
fn source_text(_root: &Path, _path: &str) -> std::io::Result<String> {
    Err(std::io::Error::other(
        "Source collection requires Unix no-follow support",
    ))
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("--no-optional-locks")
        // Even read-only Git status can execute a repository's fsmonitor hook.
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "Git {} failed; no absence established",
            args.first().unwrap_or(&"read")
        )
        .into());
    }
    Ok(output.stdout)
}
fn supported(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    ["ts", "tsx", "js", "jsx", "mjs", "cjs", "rs", "go"].contains(
        &Path::new(path)
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .as_ref(),
    ) || ["package.json", "Cargo.toml", "go.mod"].contains(&name.as_ref())
}
fn code(path: &str) -> bool {
    !["package.json", "Cargo.toml", "go.mod"].contains(
        &Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .as_ref(),
    )
}
fn allowed(path: &str) -> bool {
    !Path::new(path).is_absolute()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        && !path.split('/').any(|p| {
            p.starts_with(".env")
                || ["node_modules", "target", ".git", ".cache", "vendor"].contains(&p)
        })
}
fn file_id(repo: &str, path: &str) -> String {
    format!("file:{repo}:{path}")
}
fn clean_spec(s: &str) -> bool {
    !s.is_empty()
        && s.len() < 256
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "@/._-:$".contains(c))
}
fn dependency(
    g: &mut Graph,
    from: &str,
    ecosystem: &str,
    spec: &str,
    source: &str,
    line: Option<usize>,
    relation: &str,
) {
    let (id, label, category) = if clean_spec(spec)
        && (!spec.contains(':') || spec.starts_with("node:") || spec.starts_with("bun:"))
    {
        let builtin = ecosystem == "node"
            && (spec.starts_with("node:") || spec.starts_with("bun:"))
            || ecosystem == "rust" && ["std", "core", "alloc"].contains(&spec)
            || ecosystem == "go" && !spec.split('/').next().unwrap_or("").contains('.');
        let namespace = if ecosystem == "node" && !builtin {
            "npm"
        } else {
            ecosystem
        };
        (
            format!("package:{namespace}:{spec}"),
            spec.to_string(),
            if builtin { "stdlib" } else { "packages" },
        )
    } else {
        (
            format!("unresolved:{}", hash(spec.as_bytes())),
            "Unresolved import".into(),
            "code",
        )
    };
    let mut node = Entity::new(id.clone(), label, "package", category, None);
    node.sources.push(source.into());
    if id.starts_with("unresolved:") {
        node.status = "unknown".into();
        node.summary =
            "Unsupported specifier; raw content withheld. No external resource was probed.".into();
    }
    g.entity(node);
    g.edge(
        from,
        &id,
        relation,
        if relation == "declares" {
            "declares"
        } else {
            "imports"
        },
        format!("{relation}: {ecosystem} dependency"),
        source,
        line,
    );
}
fn unresolved(g: &mut Graph, repo: &str, from: &str, spec: &str, source: &str, line: usize) {
    let id = format!("specifier:{repo}:{}", hash(spec.as_bytes()));
    let mut n = Entity::new(
        id.clone(),
        if clean_spec(spec) {
            format!("{spec} · unresolved")
        } else {
            "Unresolved import".into()
        },
        "specifier",
        "packages",
        None,
    );
    n.status = "unknown".into();
    n.summary="Literal import with no established package/module target. Alias, inline namespace or undeclared external package remains unresolved.".into();
    n.sources.push(source.into());
    g.entity(n);
    g.edge(
        from,
        &id,
        "imports",
        "imports",
        "Literal request; target identity unresolved".into(),
        source,
        Some(line),
    );
}
fn normalized(path: &Path) -> Option<String> {
    let mut parts = vec![];
    for part in path.components() {
        match part {
            Component::Normal(v) => parts.push(v.to_string_lossy().to_string()),
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(parts.join("/"))
}
fn resolve_js(path: &str, spec: &str, known: &BTreeSet<String>) -> Option<String> {
    if !spec.starts_with('.') {
        return None;
    }
    let base = normalized(&Path::new(path).parent().unwrap_or(Path::new("")).join(spec))?;
    let mut candidates = vec![base.clone()];
    if base.ends_with(".js") {
        candidates.push(base.trim_end_matches(".js").to_string() + ".ts");
        candidates.push(base.trim_end_matches(".js").to_string() + ".tsx");
    }
    for ext in ["ts", "tsx", "js", "jsx", "mjs", "cjs"] {
        candidates.push(format!("{base}.{ext}"));
        candidates.push(format!("{base}/index.{ext}"));
    }
    candidates.into_iter().find(|p| known.contains(p))
}
fn use_paths(tree: &syn::UseTree, mut prefix: Vec<String>, out: &mut Vec<Vec<String>>) {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            use_paths(&p.tree, prefix, out)
        }
        syn::UseTree::Name(p) => {
            prefix.push(p.ident.to_string());
            out.push(prefix)
        }
        syn::UseTree::Rename(p) => {
            prefix.push(p.ident.to_string());
            out.push(prefix)
        }
        syn::UseTree::Glob(_) => out.push(prefix),
        syn::UseTree::Group(p) => {
            for item in &p.items {
                use_paths(item, prefix.clone(), out)
            }
        }
    }
}
struct RustImports {
    items: Vec<(Vec<String>, usize)>,
}
impl<'ast> Visit<'ast> for RustImports {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let mut paths = vec![];
        use_paths(&node.tree, vec![], &mut paths);
        for path in paths {
            self.items.push((path, node.span().start().line));
        }
    }
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if node.content.is_none() {
            self.items.push((
                vec!["self".into(), node.ident.to_string()],
                node.span().start().line,
            ));
        }
        syn::visit::visit_item_mod(self, node);
    }
}
fn resolve_rust(
    path: &str,
    parts: &[String],
    known: &BTreeSet<String>,
    crate_base: &Path,
) -> Option<String> {
    if parts.is_empty() {
        return None;
    }
    let parent = Path::new(path).parent().unwrap_or(Path::new(""));
    let stem = Path::new(path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let this = if stem == "mod" || stem == "lib" || stem == "main" {
        parent.to_path_buf()
    } else {
        parent.join(stem.as_ref())
    };
    let (mut base, mut offset) = match parts[0].as_str() {
        "crate" => (crate_base.join("src"), 1),
        "self" => (this, 1),
        "super" => (this.parent()?.to_path_buf(), 1),
        _ => (crate_base.join("src"), 0),
    };
    while offset < parts.len() && parts[offset] == "super" {
        base = base.parent()?.to_path_buf();
        offset += 1;
    }
    for length in (offset + 1..=parts.len()).rev() {
        let mut candidate = base.clone();
        for p in &parts[offset..length] {
            candidate.push(p);
        }
        for p in [candidate.with_extension("rs"), candidate.join("mod.rs")] {
            if let Some(s) = normalized(&p) {
                if known.contains(&s) {
                    return Some(s);
                }
            }
        }
    }
    None
}

pub fn repository(g: &mut Graph, input: &Input) -> Result<()> {
    let root = input.root.canonicalize()?;
    let top = String::from_utf8(git(&root, &["rev-parse", "--show-toplevel"])?)?;
    if Path::new(top.trim()).canonicalize()? != root {
        return Err("--repo must name a Git repository root".into());
    }
    let head = String::from_utf8(git(&root, &["rev-parse", "HEAD"])?)?
        .trim()
        .to_string();
    let start_status = git(
        &root,
        &["status", "--porcelain", "-z", "--ignore-submodules=all"],
    )?;
    let repo = &input.name;
    let rid = format!("repo:{repo}");
    let git_source = format!("source:{repo}:git");
    g.sources.insert(git_source.clone(),Source{id:git_source.clone(),label:format!("{repo} local Git enumeration"),result:format!("HEAD {head}; live worktree, not an atomic committed checkout. Namespace supplied by caller; native hosting ID was not queried."),date:g.collected_at.clone(),level:"config".into(),repo:Some(repo.clone()),path:None,sha256:None,head:Some(head.clone())});
    let mut repository = Entity::new(
        rid.clone(),
        repo.clone(),
        "repository",
        "context",
        Some(repo.clone()),
    );
    repository.sources.push(git_source.clone());
    g.entity(repository);
    let paths = git(
        &root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?;
    let all: BTreeSet<String> = paths
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8_lossy(p).to_string())
        .filter(|p| allowed(p) && supported(p))
        .collect();
    let mut collection=Collection{id:format!("collection:{repo}"),label:repo.clone(),enumerated_files:all.len(),read_files:0,coverage:"partial".into(),head:Some(head.clone()),dirty:!start_status.is_empty(),errors:vec![],limits:vec!["Read-only source collection; no installs, tests, application execution, provider reads or model calls.".into(),"Only Rust, Go, JS/TS static imports and Cargo/npm/Go declarations. Runtime bindings, scripts, aliases, conditional execution and call graphs need separate evidence.".into(),"Secrets/ignored/generated paths excluded; symlinks never followed. Per-file hashes identify observed bytes; a dirty worktree is not HEAD's content.".into(),"Repository identity uses the explicit caller namespace; rename/native-provider identity is not established.".into(),"Directory scopes, not complete semantic/symbol extraction. Go workspaces/nested manifests, Cargo target/workspace dependency tables, JS path-alias configs and dynamic dependencies need additional resolution.".into(),"Budgets: 2,000 supported files, 1 MiB per file, 16 MiB total source bytes. A skipped path remains unknown.".into()]};
    if all.len() > MAX_FILES {
        collection
            .errors
            .push(format!("File budget exceeded: {} > {MAX_FILES}", all.len()));
    }
    let mut texts: BTreeMap<String, String> = BTreeMap::new();
    let mut bytes = 0;
    for path in all.iter().take(MAX_FILES) {
        let id = file_id(repo, path);
        let mut node = Entity::new(
            id.clone(),
            path.clone(),
            if code(path) {
                "source_file"
            } else {
                "manifest"
            },
            "code",
            Some(repo.clone()),
        );
        node.path = Some(path.clone());
        node.sources.push(git_source.clone());
        match if bytes < MAX_BYTES {
            source_text(&root, path)
        } else {
            Err(std::io::Error::other("Collection byte budget"))
        } {
            Ok(text) if bytes + text.len() <= MAX_BYTES => {
                bytes += text.len();
                let sid = format!("source:{repo}:{path}");
                g.sources.insert(sid.clone(),Source{id:sid.clone(),label:format!("{repo}/{path}"),result:"Static source/config declaration; not installed or runtime proof. No source body exported.".into(),date:g.collected_at.clone(),level:"config".into(),repo:Some(repo.clone()),path:Some(path.clone()),sha256:Some(hash(text.as_bytes())),head:Some(head.clone())});
                node.sources = vec![sid];
                texts.insert(path.clone(), text);
                collection.read_files += 1;
            }
            _ => {
                node.status = "unknown".into();
                node.summary="Listed path was not read (permission, symlink, file/collection budget or encoding). No absence conclusion.".into();
                collection.errors.push(format!("{path}: not read"));
            }
        }
        g.entity(node);
    }
    let known: BTreeSet<_> = texts.keys().filter(|p| code(p)).cloned().collect();
    let mut manifest_dirs = vec![];
    let mut rust_aliases = BTreeMap::new();
    let mut npm_names = BTreeSet::new();
    let mut go_module = String::new();
    for (path, text) in &texts {
        let from = format!("repo:{repo}");
        let source = format!("source:{repo}:{path}");
        match Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .as_ref()
        {
            "package.json" => match serde_json::from_str::<Value>(text) {
                Ok(value) => {
                    for field in [
                        "dependencies",
                        "devDependencies",
                        "peerDependencies",
                        "optionalDependencies",
                    ] {
                        if let Some(deps) = value[field].as_object() {
                            for (name, requirement) in deps {
                                npm_names.insert(name.clone());
                                dependency(g, &from, "npm", name, &source, None, "declares");
                                if let Some(edge) = g.edges.values_mut().find(|e| {
                                    e.from == from
                                        && e.to == format!("package:npm:{name}")
                                        && e.sources.contains(&source)
                                }) {
                                    edge.label = format!(
                                        "{field}: {name}; requirement {}",
                                        if requirement
                                            .as_str()
                                            .is_some_and(|s| s.len() < 100 && !s.contains("://"))
                                        {
                                            requirement.as_str().unwrap()
                                        } else {
                                            "not exported"
                                        }
                                    );
                                }
                            }
                        }
                    }
                }
                Err(_) => collection
                    .errors
                    .push(format!("{path}: manifest parse failed")),
            },
            "Cargo.toml" => match toml::from_str::<toml::Value>(text) {
                Ok(value) => {
                    manifest_dirs.push(
                        Path::new(path)
                            .parent()
                            .unwrap_or(Path::new(""))
                            .to_path_buf(),
                    );
                    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                        if let Some(deps) = value.get(section).and_then(toml::Value::as_table) {
                            for (name, info) in deps {
                                let actual = info
                                    .get("package")
                                    .and_then(toml::Value::as_str)
                                    .unwrap_or(name);
                                rust_aliases.insert(name.replace('-', "_"), actual.to_string());
                                dependency(g, &from, "rust", actual, &source, None, "declares");
                            }
                        }
                    }
                }
                Err(_) => collection
                    .errors
                    .push(format!("{path}: manifest parse failed")),
            },
            "go.mod" => {
                let mut block = false;
                for (line, text) in text.lines().enumerate() {
                    let parts: Vec<_> = text.split_whitespace().collect();
                    if parts.first() == Some(&"module") && parts.len() > 1 {
                        go_module = parts[1].trim_matches('"').into();
                    }
                    if parts.first() == Some(&"require") {
                        block = parts.get(1) == Some(&"(");
                        if !block && parts.len() > 1 {
                            dependency(
                                g,
                                &from,
                                "go",
                                parts[1],
                                &source,
                                Some(line + 1),
                                "declares",
                            );
                        }
                    } else if block {
                        if parts.first() == Some(&")") {
                            block = false;
                        } else if let Some(name) = parts.first() {
                            if !name.starts_with("//") {
                                dependency(
                                    g,
                                    &from,
                                    "go",
                                    name,
                                    &source,
                                    Some(line + 1),
                                    "declares",
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for language in ["typescript", "go"] {
        let files: Vec<_> = texts
            .iter()
            .filter(|(path, _)| {
                if language == "go" {
                    path.ends_with(".go")
                } else {
                    ["ts", "tsx", "js", "jsx", "mjs", "cjs"].contains(
                        &Path::new(path)
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .as_ref(),
                    )
                }
            })
            .collect();
        if files.is_empty() {
            continue;
        }
        let parsed = files
            .into_iter()
            .map(|(path, text)| crate::imports::parse(path, text))
            .collect::<Result<Vec<_>>>();
        match parsed {
            Ok(results) => {
                for parsed in results {
                    if parsed.error.is_some() {
                        collection
                            .errors
                            .push(format!("{}: syntax diagnostics", parsed.path));
                    }
                    let from = file_id(repo, &parsed.path);
                    let source = format!("source:{repo}:{}", parsed.path);
                    for import in parsed.imports {
                        let local = if language == "typescript" {
                            resolve_js(&parsed.path, &import.specifier, &known)
                        } else if !go_module.is_empty()
                            && import.specifier.starts_with(&(go_module.clone() + "/"))
                        {
                            let prefix = import.specifier[go_module.len() + 1..].to_string() + "/";
                            {
                                let members: Vec<_> = known
                                    .iter()
                                    .filter(|p| {
                                        Path::new(p).parent()
                                            == Some(Path::new(prefix.trim_end_matches('/')))
                                            && p.ends_with(".go")
                                    })
                                    .map(|p| file_id(repo, p))
                                    .collect();
                                if members.is_empty() {
                                    None
                                } else {
                                    let id = format!(
                                        "source-package:{repo}:{}",
                                        prefix.trim_end_matches('/')
                                    );
                                    let mut n = Entity::new(
                                        id.clone(),
                                        prefix.trim_end_matches('/').into(),
                                        "source_package",
                                        "code",
                                        Some(repo.clone()),
                                    );
                                    n.path = Some(prefix.trim_end_matches('/').into());
                                    n.members = members;
                                    n.sources.push(git_source.clone());
                                    n.summary="Go source-package directory. Known files retained as containment, not runtime edges; build tags and compiled membership were not evaluated.".into();
                                    g.entity(n);
                                    Some(id)
                                }
                            }
                        } else {
                            None
                        };
                        if let Some(path) = local {
                            let target = if language == "go" {
                                path
                            } else {
                                file_id(repo, &path)
                            };
                            g.edge(
                                &from,
                                &target,
                                "imports",
                                "imports",
                                format!("Static {language} import"),
                                &source,
                                Some(import.line),
                            );
                        } else if language == "typescript" && import.specifier.starts_with('.') {
                            let id =
                                format!("unresolved:{repo}:{}", hash(import.specifier.as_bytes()));
                            let mut n = Entity::new(
                                id.clone(),
                                "Unresolved local import".into(),
                                "boundary",
                                "code",
                                None,
                            );
                            n.status = "unknown".into();
                            n.sources.push(source.clone());
                            g.entity(n);
                            g.edge(
                                &from,
                                &id,
                                "imports",
                                "imports",
                                "Target not established in the readable source set".into(),
                                &source,
                                Some(import.line),
                            );
                        } else {
                            let spec = if language == "typescript" {
                                if import.specifier.starts_with('@') {
                                    import
                                        .specifier
                                        .split('/')
                                        .take(2)
                                        .collect::<Vec<_>>()
                                        .join("/")
                                } else {
                                    import.specifier.split('/').next().unwrap_or("").to_string()
                                }
                            } else {
                                import.specifier
                            };
                            if language == "typescript"
                                && !npm_names.contains(&spec)
                                && !spec.starts_with("node:")
                                && !spec.starts_with("bun:")
                            {
                                unresolved(g, repo, &from, &spec, &source, import.line);
                            } else {
                                dependency(
                                    g,
                                    &from,
                                    if language == "go" { "go" } else { "node" },
                                    &spec,
                                    &source,
                                    Some(import.line),
                                    "imports",
                                );
                            }
                        }
                    }
                }
            }
            Err(_) => collection.errors.push(format!(
                "{language}: native parser failed; relationships remain incomplete"
            )),
        }
    }
    for (path, text) in &texts {
        if !path.ends_with(".rs") {
            continue;
        }
        match syn::parse_file(text) {
            Ok(file) => {
                let mut imports = RustImports { items: vec![] };
                imports.visit_file(&file);
                let from = file_id(repo, path);
                let source = format!("source:{repo}:{path}");
                let base = manifest_dirs
                    .iter()
                    .filter(|p| Path::new(path).starts_with(p))
                    .max_by_key(|p| p.components().count())
                    .cloned()
                    .unwrap_or_default();
                for (parts, line) in imports.items {
                    if let Some(target) = resolve_rust(path, &parts, &known, &base) {
                        g.edge(
                            &from,
                            &file_id(repo, &target),
                            "imports",
                            "imports",
                            "Static Rust use/mod declaration; cfg conditions not evaluated".into(),
                            &source,
                            Some(line),
                        );
                    } else if let Some(first) = parts.first() {
                        if ["crate", "self", "super"].contains(&first.as_str()) {
                            collection
                                .errors
                                .push(format!("{path}:{line}: local Rust path not resolved"));
                        } else if rust_aliases.contains_key(first)
                            || ["std", "core", "alloc"].contains(&first.as_str())
                        {
                            dependency(
                                g,
                                &from,
                                "rust",
                                rust_aliases.get(first).unwrap_or(first),
                                &source,
                                Some(line),
                                "imports",
                            );
                        } else {
                            unresolved(g, repo, &from, first, &source, line);
                        }
                    }
                }
            }
            Err(_) => collection
                .errors
                .push(format!("{path}: Rust syntax diagnostics")),
        }
    }
    let mut scope = Scope {
        id: rid.clone(),
        label: repo.clone(),
        kind: "repository".into(),
        repo: Some(repo.clone()),
        prefix: None,
        focal_ids: known.iter().map(|p| file_id(repo, p)).collect(),
    };
    if scope.focal_ids.is_empty() {
        scope.focal_ids.push(rid.clone());
    }
    g.scopes.insert(rid.clone(), scope);
    let dirs: BTreeSet<_> = known
        .iter()
        .flat_map(|p| {
            Path::new(p)
                .parent()
                .into_iter()
                .flat_map(Path::ancestors)
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
        })
        .filter(|p| !p.is_empty())
        .collect();
    for prefix in dirs {
        let focal_ids = known
            .iter()
            .filter(|p| p.starts_with(&(prefix.clone() + "/")))
            .map(|p| file_id(repo, p))
            .collect();
        let id = format!("module:{repo}:{prefix}");
        g.scopes.insert(
            id.clone(),
            Scope {
                id,
                label: format!("{repo} / {prefix}"),
                kind: "module".into(),
                repo: Some(repo.clone()),
                prefix: Some(prefix),
                focal_ids,
            },
        );
    }
    if git(&root, &["rev-parse", "HEAD"])? != format!("{head}\n").as_bytes()
        || git(
            &root,
            &["status", "--porcelain", "-z", "--ignore-submodules=all"],
        )? != start_status
    {
        collection
            .errors
            .push("Git head/status changed during collection; not an atomic snapshot".into());
    }
    collection.errors.sort();
    collection.errors.dedup();
    // File enumeration may be complete, but the supported relationship model is
    // deliberately partial. This collection can never prove runtime absence.
    g.collections.push(collection);
    Ok(())
}

pub fn snapshot(g: &mut Graph, name: &str, path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)?;
    let raw: Value = serde_json::from_str(&text)?;
    if raw["schema"] != "topo.infrastructure-map.v1" {
        return Err("Unsupported inventory snapshot schema".into());
    }
    let scenario = raw["scenarios"]
        .as_array()
        .and_then(|s| s.iter().find(|s| s["id"] == "current"))
        .ok_or("Snapshot has no current baseline")?;
    let ids: BTreeSet<String> = scenario["node_ids"]
        .as_array()
        .ok_or("Missing baseline nodes")?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let edge_ids: BTreeSet<String> = scenario["edge_ids"]
        .as_array()
        .ok_or("Missing baseline edges")?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    for source in raw["sources"]
        .as_array()
        .ok_or("Missing snapshot sources")?
    {
        let sid = format!(
            "snapshot:{name}:source:{}",
            source["id"].as_str().ok_or("Invalid snapshot source")?
        );
        g.sources.insert(
            sid.clone(),
            Source {
                id: sid,
                label: source["label"].as_str().unwrap_or("Imported source").into(),
                result: format!(
                    "Imported dated snapshot, NOT refreshed by this run. {}",
                    source["result"].as_str().unwrap_or("")
                ),
                date: source["date"].as_str().unwrap_or("Unknown").into(),
                level: source["level"].as_str().unwrap_or("config").into(),
                repo: None,
                path: None,
                sha256: Some(hash(text.as_bytes())),
                head: None,
            },
        );
    }
    let sources = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|s| format!("snapshot:{name}:source:{s}"))
            .collect()
    };
    let mut focal = vec![];
    for node in raw["nodes"].as_array().ok_or("Missing snapshot nodes")? {
        let raw_id = node["id"].as_str().ok_or("Invalid snapshot node")?;
        if !ids.contains(raw_id) {
            continue;
        }
        let id = format!("snapshot:{name}:{raw_id}");
        focal.push(id.clone());
        let mut n = Entity::new(
            id,
            node["label"].as_str().unwrap_or(raw_id).into(),
            node["kind"].as_str().unwrap_or("resource"),
            node["category"].as_str().unwrap_or("business"),
            Some(format!("snapshot:{name}")),
        );
        n.status = node["status"].as_str().unwrap_or("unknown").into();
        n.summary = format!(
            "Dated imported observation; not refreshed here. {}",
            node["summary"].as_str().unwrap_or("")
        );
        n.sources = sources(&node["sources"]);
        n.details = node["details"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
        if let Some(owner) = node["owner"].as_str() {
            n.details.push(format!("Recorded snapshot owner: {owner}"));
        }
        if let Some(costs) = node["costs"].as_array() {
            if !costs.is_empty() {
                n.details.push(format!(
                    "Recorded snapshot cost entries (dated, separate bases; not totaled): {}",
                    serde_json::to_string(costs)?
                ));
            }
        }
        n.details.push(format!("Snapshot last verified: {}. Health, custody and restore are not established by presence.",node["last_verified"].as_str().unwrap_or("Unknown")));
        g.entity(n);
    }
    for edge in raw["edges"].as_array().ok_or("Missing snapshot edges")? {
        if !edge_ids.contains(edge["id"].as_str().unwrap_or("")) {
            continue;
        }
        let id = format!("snapshot:{name}:{}", edge["id"].as_str().unwrap_or(""));
        g.edges.insert(
            id.clone(),
            Edge {
                id,
                from: format!("snapshot:{name}:{}", edge["from"].as_str().unwrap_or("")),
                to: format!("snapshot:{name}:{}", edge["to"].as_str().unwrap_or("")),
                relation: edge["relation"].as_str().or_else(|| edge["type"].as_str()).unwrap_or("other").into(),
                family: edge["type"].as_str().unwrap_or("other").into(),
                evidence: edge["evidence"].as_str().unwrap_or("config").into(),
                label: edge["label"].as_str().unwrap_or("").into(),
                sources: sources(&edge["sources"]),
                line: None,
            },
        );
    }
    let id = format!("snapshot:{name}");
    g.scopes.insert(
        id.clone(),
        Scope {
            id: id.clone(),
            label: name.into(),
            kind: "snapshot".into(),
            repo: Some(id.clone()),
            prefix: None,
            focal_ids: focal,
        },
    );
    let imported_gaps: Vec<_> = raw["gaps"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|v| {
            format!(
                "Imported gap · {}: {}. Needed: {}",
                v["area"].as_str().unwrap_or("Unknown area"),
                v["detail"].as_str().unwrap_or("Unresolved coverage"),
                v["needed"].as_str().unwrap_or("Further evidence")
            )
        })
        .collect();
    g.collections.push(Collection{id,label:format!("Imported {name}"),enumerated_files:0,read_files:1,coverage:"partial".into(),head:None,dirty:false,errors:vec![],limits:vec!["Dated existing snapshot imported unchanged in direction/provenance; no provider API, DNS, host, billing or health refresh.".into(),"Only the current baseline is imported; proposed scenarios are not observations. Original collection gaps remain unknown.".into()].into_iter().chain(imported_gaps).collect()});
    Ok(())
}
