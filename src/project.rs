use crate::model::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub fn scope_file(id: &str) -> String {
    format!("scope-{}.html", &hash(id.as_bytes())[..16])
}
fn tier(g: &Graph, e: &Edge) -> &'static str {
    if e.evidence == "inference" {
        "inference"
    } else if e.sources.iter().any(|s| {
        g.sources
            .get(s)
            .is_some_and(|s| ["api", "cli", "dns"].contains(&s.level.as_str()))
    }) {
        "live"
    } else {
        "config"
    }
}
fn bucket(n: &Entity) -> String {
    if n.category == "stdlib" {
        "Standard library".into()
    } else if n.id.contains(":cf-worker-") {
        "Cloudflare Workers".into()
    } else if n.id.contains(":cf-r2-") {
        "Cloudflare R2".into()
    } else if n.id.contains(":cf-d1-") {
        "Cloudflare D1".into()
    } else if n.id.contains(":cf-durable-") {
        "Cloudflare Durable Objects".into()
    } else if n.id.contains(":exe-") && n.entity_kind != "provider" {
        "exe.dev workspaces".into()
    } else if n.id.contains(":do-domain-") {
        "DigitalOcean zone records".into()
    } else if n.id.contains(":cf-zone-") {
        "Cloudflare zones".into()
    } else {
        format!("{} {}", n.entity_kind, n.category)
    }
}
fn group(
    g: &Graph,
    id: String,
    label: String,
    members: &[String],
    kind: &str,
    scope_link: Option<String>,
) -> Value {
    let statuses: BTreeSet<_> = members
        .iter()
        .filter_map(|id| g.entities.get(id).map(|n| n.status.clone()))
        .collect();
    let sources: BTreeSet<_> = members
        .iter()
        .flat_map(|id| {
            g.entities
                .get(id)
                .into_iter()
                .flat_map(|n| n.sources.iter().cloned())
        })
        .collect();
    let details: Vec<_> = members
        .iter()
        .filter_map(|id| {
            g.entities
                .get(id)
                .map(|n| format!("{} · {} · {}", n.label, n.entity_kind, n.status))
        })
        .collect();
    json!({"id":id,"label":format!("{label} · {}",members.len()),"kind":kind,"entity_kind":"aggregate","category":"code","status":if statuses.len()==1{statuses.iter().next().cloned().unwrap()}else{"unknown".into()},
  "summary":format!("View aggregate of {} known entities, not a new resource. Mixed statuses remain unknown; counts cover only supplied evidence.",members.len()),
  "owner":"Unknown","cost":"Unknown; not totaled","repos":[],"sources":sources,"details":details,"members":members,"scope_link":scope_link})
}
fn node(n: &Entity, kind: &str) -> Value {
    json!({"id":n.id,"label":n.label,"kind":kind,"entity_kind":n.entity_kind,"category":if n.category=="hosting"{"compute"}else{&n.category},"status":n.status,"summary":n.summary,"owner":"Unknown","cost":"Unknown; not measured",
 "repos":[],"sources":n.sources,"details":n.details.iter().cloned().chain(n.path.iter().map(|p|format!("Source path: {p}"))).collect::<Vec<_>>(),"members":if n.members.is_empty(){vec![n.id.clone()]}else{n.members.clone()}})
}

pub fn selected(g: &Graph, scope: &Scope) -> (BTreeSet<String>, Vec<Edge>) {
    let focal: BTreeSet<_> = scope.focal_ids.iter().cloned().collect();
    if scope.kind == "portfolio" {
        let mut ids: BTreeSet<String> = g
            .entities
            .values()
            .filter(|n| {
                n.repo
                    .as_ref()
                    .is_some_and(|r| focal.contains(&format!("repo:{r}")))
            })
            .map(|n| n.id.clone())
            .collect();
        let mut edges = vec![];
        for e in g.edges.values() {
            if ids.contains(&e.from) || ids.contains(&e.to) {
                edges.push(e.clone());
            }
        }
        for e in &edges {
            ids.insert(e.from.clone());
            ids.insert(e.to.clone());
        }
        return (ids, edges);
    }
    let mut ids = focal.clone();
    let mut edges = vec![];
    for edge in g.edges.values() {
        if focal.contains(&edge.from)
            || focal.contains(&edge.to)
            || scope.kind == "repository"
                && (scope
                    .repo
                    .as_ref()
                    .is_some_and(|r| edge.from == format!("repo:{r}")))
        {
            ids.insert(edge.from.clone());
            ids.insert(edge.to.clone());
            edges.push(edge.clone());
        }
    }
    (ids, edges)
}

pub fn projection(g: &Graph, scope_id: &str) -> Result<Value> {
    let portfolio = scope_id == "portfolio";
    let scope = if portfolio {
        Scope {
            id: "portfolio".into(),
            label: "Selected repositories".into(),
            kind: "portfolio".into(),
            repo: None,
            prefix: None,
            focal_ids: g
                .entities
                .values()
                .filter(|n| n.entity_kind == "repository")
                .map(|n| n.id.clone())
                .collect(),
        }
    } else {
        g.scopes
            .get(scope_id)
            .ok_or("Unknown supplied scope")?
            .clone()
    };
    let (mut ids, edges) = if portfolio {
        (
            g.entities
                .values()
                .filter(|n| {
                    n.repo.as_ref().is_some_and(|r| !r.starts_with("snapshot:"))
                        || n.entity_kind == "package"
                })
                .map(|n| n.id.clone())
                .collect(),
            g.edges
                .values()
                .filter(|e| !e.id.starts_with("snapshot:"))
                .cloned()
                .collect::<Vec<_>>(),
        )
    } else {
        selected(g, &scope)
    };
    let focal: BTreeSet<_> = scope.focal_ids.iter().cloned().collect();
    if ids.is_empty() {
        ids.extend(focal.clone());
    }
    let mut mapping: BTreeMap<String, String> = BTreeMap::new();
    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    if portfolio {
        for n in g
            .entities
            .values()
            .filter(|n| n.entity_kind == "repository")
        {
            let mut v = node(n, "product");
            v["category"] = json!("compute");
            v["scope_link"] = json!(scope_file(&n.id));
            nodes.insert(n.id.clone(), v);
        }
        for id in &ids {
            if let Some(n) = g.entities.get(id) {
                if let Some(repo) = &n.repo {
                    let rid = format!("repo:{repo}");
                    if g.entities.contains_key(&rid) {
                        mapping.insert(id.clone(), rid);
                    }
                }
            }
        }
    } else if scope.kind == "repository" && focal.len() > 28 {
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for id in &focal {
            let n = &g.entities[id];
            let parts: Vec<_> = n.path.as_deref().unwrap_or("").split('/').collect();
            let prefix = if parts.len() > 2 {
                parts[..2].join("/")
            } else if parts.len() > 1 {
                parts[0].into()
            } else {
                "Root files".into()
            };
            groups.entry(prefix).or_default().push(id.clone());
        }
        for (prefix, members) in groups {
            let id = format!("aggregate:{}:{prefix}", scope.id);
            let target = format!("module:{}:{prefix}", scope.repo.as_deref().unwrap_or(""));
            let link = g.scopes.contains_key(&target).then(|| scope_file(&target));
            for member in &members {
                mapping.insert(member.clone(), id.clone());
            }
            nodes.insert(id.clone(), group(g, id, prefix, &members, "product", link));
        }
    } else if scope.kind == "snapshot" {
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for id in &focal {
            let n = &g.entities[id];
            let key = bucket(n);
            if key.starts_with("Cloudflare ")
                || key == "exe.dev workspaces"
                || key == "DigitalOcean zone records"
            {
                groups.entry(key).or_default().push(id.clone());
            } else {
                let kind = if n.entity_kind == "product" {
                    "product"
                } else if n.entity_kind == "domain" {
                    "domain"
                } else {
                    "provider"
                };
                nodes.insert(id.clone(), node(n, kind));
                mapping.insert(id.clone(), id.clone());
            }
        }
        for (label, members) in groups {
            let id = format!("aggregate:{}:{label}", scope.id);
            let mut v = group(g, id.clone(), label, &members, "provider", None);
            v["category"] = json!("compute");
            for member in &members {
                mapping.insert(member.clone(), id.clone());
            }
            nodes.insert(id, v);
        }
    }
    let mut dependency_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for id in &ids {
        if mapping.contains_key(id) {
            continue;
        }
        let Some(n) = g.entities.get(id) else {
            continue;
        };
        if focal.contains(id) {
            mapping.insert(id.clone(), id.clone());
            let mut v = node(n, "product");
            if let Some(prefix) = &scope.prefix {
                if let Some(path) = &n.path {
                    v["label"] = json!(path.strip_prefix(&(prefix.clone() + "/")).unwrap_or(path));
                }
            }
            nodes.insert(id.clone(), v);
        } else if portfolio && n.category == "packages" {
            let label = if n.id.starts_with("package:go:") {
                "Go external modules"
            } else if n.id.starts_with("package:rust:") {
                "Rust crates"
            } else if n.id.starts_with("package:npm:") {
                "npm packages"
            } else {
                "Unresolved import requests"
            };
            dependency_groups
                .entry(label.into())
                .or_default()
                .push(id.clone());
        } else if n.category == "stdlib" {
            dependency_groups
                .entry("Standard libraries".into())
                .or_default()
                .push(id.clone());
        } else {
            mapping.insert(id.clone(), id.clone());
            let consumer = edges.iter().any(|e| e.from == *id && focal.contains(&e.to));
            let mut v = node(
                n,
                if n.entity_kind == "repository" || consumer {
                    "tool"
                } else {
                    "provider"
                },
            );
            if consumer {
                v["category"] = json!("context");
            }
            if n.entity_kind == "repository" {
                v["category"] = json!("context");
                v["scope_link"] = json!(scope_file(&n.id));
            }
            if n.entity_kind == "source_package" {
                if let (Some(repo), Some(path)) = (&n.repo, &n.path) {
                    let target = format!("module:{repo}:{path}");
                    if g.scopes.contains_key(&target) {
                        v["scope_link"] = json!(scope_file(&target));
                    }
                }
            }
            nodes.insert(id.clone(), v);
        }
    }
    for (label, members) in dependency_groups {
        let id = format!("aggregate:{}:{}", scope.id, label);
        let standard = label == "Standard libraries";
        let mut v = group(g, id.clone(), label, &members, "provider", None);
        v["category"] = json!(if standard { "stdlib" } else { "packages" });
        for member in &members {
            mapping.insert(member.clone(), id.clone());
        }
        nodes.insert(id, v);
    }
    // Compress repeated imports, preserving relation/direction/source IDs and the
    // exact underlying edge IDs. No containment edge enters operational traversal.
    let mut bundled: BTreeMap<(String, String, String), Vec<&Edge>> = BTreeMap::new();
    for e in &edges {
        let (Some(from), Some(to)) = (mapping.get(&e.from), mapping.get(&e.to)) else {
            continue;
        };
        if from == to {
            continue;
        }
        bundled
            .entry((from.clone(), to.clone(), e.family.clone()))
            .or_default()
            .push(e);
    }
    let projected:Vec<_>=bundled.into_iter().map(|((from,to,family),items)|{
  let mix:BTreeSet<_>=items.iter().map(|e|tier(g,e)).collect();let sources:BTreeSet<_>=items.iter().flat_map(|e|e.sources.iter().cloned()).collect();
  let forced=if mix.contains("inference"){"inference"}else if mix.len()>1{"config"}else{mix.iter().next().copied().unwrap_or("config")};
  json!({"from":from,"to":to,"type":family,"label":if items.len()==1{items[0].label.clone()}else{format!("{} underlying relationships · evidence {}",items.len(),mix.into_iter().collect::<Vec<_>>().join(" / "))},"evidence":if forced=="inference"{"inference"}else{"read"},"tier_override":forced,"sources":sources,"edge_ids":items.iter().map(|e|e.id.clone()).collect::<Vec<_>>()})
 }).collect();
    let used_sources: BTreeSet<_> = nodes
        .values()
        .chain(projected.iter())
        .flat_map(|v| {
            v["sources"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
        })
        .collect();
    let sources:Vec<_>=g.sources.values().filter(|s|used_sources.contains(&s.id)).map(|s|json!({"id":s.id,"label":s.label,"result":format!("{}{}{}",s.result,s.path.as_ref().map(|p|format!(" Source path: {p}.")).unwrap_or_default(),s.sha256.as_ref().map(|h|format!(" SHA256 {h}.")).unwrap_or_default()),"date":s.date,"level":s.level,"url":null})).collect();
    let relevant: Vec<_> = g
        .collections
        .iter()
        .filter(|c| {
            portfolio && !c.id.starts_with("snapshot:")
                || !portfolio
                    && scope
                        .repo
                        .as_ref()
                        .is_some_and(|r| c.id == format!("collection:{r}") || &c.id == r)
        })
        .collect();
    let gaps:Vec<_>=relevant.iter().flat_map(|c|c.limits.iter().map(move|limit|json!({"area":c.label,"detail":limit,"needed":"Independent runtime/provider or richer source evidence; a map is not proof of health or absence."})).chain(c.errors.iter().map(move|error|json!({"area":c.label,"detail":error,"needed":"Repair access/parser support and recollect; retain previous observations."})))).collect();
    let coverage = format!(
        "{} · {} visible entities/groups from {} underlying entities; {} projected relationships from {} exact edges. Source-only declarations and dated imports, not live health. Graph {}.",
        scope.label,
        nodes.len(),
        ids.len(),
        projected.len(),
        edges.len(),
        &g.revision[..12]
    );
    let mut navigation = if g.scopes.contains_key("portfolio") {
        vec![json!({"id":"portfolio","label":"Project · selected repositories","map_kind":MapKind::Project,"file":"index.html"})]
    } else {
        vec![]
    };
    navigation.extend(
        g.scopes
            .values()
            .filter(|s| s.id != "portfolio")
            .map(|s| Ok(json!({"id":s.id,"label":format!("{} · {}",s.map_kind()?.label(),s.label),"map_kind":s.map_kind()?,"file":scope_file(&s.id)})))
            .collect::<Result<Vec<_>>>()?,
    );
    let findings = vec![
        json!({"title":"One graph across scopes","detail":format!("Revision {}. Scope changes the projection, not entity identity or evidence. Shared packages are not duplicated between repositories.",g.revision),"evidence":[]}),
        json!({"title":"Read-only source and snapshot collection","detail":format!("{} input collections. No application execution, provider mutation, model call or fresh infrastructure probe. Scope and collection coverage remain partial.",g.collections.len()),"evidence":[]}),
    ];
    Ok(
        json!({"meta":{"date":g.collected_at,"coverage":coverage,"limits":["Repository views preserve dirty-worktree bytes through source hashes; Git HEAD is context, not a byte pin.","Package/source imports are not deployed bindings or runtime health. Imported account observations retain their original dates.","Aggregates retain member IDs and edge bundles; graph.json and the Rust query command expose underlying evidence.","Only caller-selected repositories/snapshots were collected; this is not a complete organization discovery."],"title":scope.label,"map_kind":scope.map_kind()?,"scope_id":scope.id,"scopes":navigation,"graph_revision":g.revision},
 "nodes":nodes.into_values().collect::<Vec<_>>(),"edges":projected,"sources":sources,"findings":findings,"gaps":gaps,"directions":[],"alternatives":[]}),
    )
}
