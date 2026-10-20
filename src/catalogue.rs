//! Join dated project identities to supplied maps, never infer a runtime binding.
use crate::model::{Collection, Graph, MapKind, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize)]
pub struct Map<'a> {
    pub scope: &'a str,
    pub label: &'a str,
    pub map_kind: MapKind,
    pub file: &'a str,
    pub repo: Option<&'a str>,
    pub coverage: &'static str,
    pub interpretation: &'static str,
}
#[derive(Serialize)]
pub struct Project<'a> {
    pub id: String,
    pub name: &'a str,
    pub aliases: Vec<&'a str>,
    pub map_kind: MapKind,
    pub evidence: &'a Value,
    pub repo_scope: Option<&'a str>,
    pub module_scopes: Vec<&'a str>,
    pub source_collection: Option<&'a Collection>,
    pub source_coverage: &'static str,
    pub module_coverage: &'static str,
    pub architecture_coverage: &'static str,
    pub runtime_source_binding: &'static str,
    pub gaps: Vec<String>,
}
#[derive(Serialize)]
pub struct Catalogue<'a> {
    pub schema: &'static str,
    pub graph_revision: &'a str,
    pub project_state_observed_at: &'a str,
    pub project_state_sha256: String,
    pub partial: bool,
    pub gaps: &'a Value,
    pub maps: Vec<Map<'a>>,
    pub projects: Vec<Project<'a>>,
}

pub fn compose<'a>(
    inventory: &'a Value,
    state_bytes: &[u8],
    graph: &'a Graph,
    manifest: &'a Value,
) -> Result<Catalogue<'a>> {
    let rows = inventory["projects"]
        .as_array()
        .ok_or("Missing project identities")?;
    let observed = inventory["collected_at"]
        .as_str()
        .ok_or("Missing project observation date")?;
    // Names and aliases establish joins, not purpose strings, resource labels or paths.
    let mut identities = BTreeMap::new();
    for row in rows {
        let name = row["name"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("Empty project identity")?;
        let aliases = row
            .get("aliases")
            .map(|v| v.as_array().ok_or("Invalid project aliases"))
            .transpose()?;
        for identity in std::iter::once(name).chain(
            aliases
                .into_iter()
                .flatten()
                .map(|v| v.as_str().unwrap_or("")),
        ) {
            if identity.trim().is_empty()
                || identities
                    .insert(identity.to_ascii_lowercase(), name)
                    .is_some()
            {
                return Err("Empty, duplicate or conflicting project identity/alias".into());
            }
        }
    }
    let mut maps = vec![];
    let mut scopes = BTreeSet::new();
    for entry in manifest["scopes"].as_array().ok_or("Missing map scopes")? {
        let id = entry["id"].as_str().ok_or("Missing map scope ID")?;
        let scope = graph.scopes.get(id).ok_or("Unknown manifest scope")?;
        if !scopes.insert(id) {
            return Err("Duplicate manifest scope".into());
        }
        maps.push(Map {
            scope: id, label: &scope.label, map_kind: scope.map_kind()?,
            file: entry["file"].as_str().ok_or("Missing map file")?, repo: scope.repo.as_deref().filter(|_| scope.kind != "snapshot"),
            coverage: "partial",
            interpretation: match scope.kind.as_str() {
                "snapshot" => "Dated infrastructure observations; no runtime refresh",
                "module" => "Directory-scoped static source relationships; not a semantic module or system-design proof",
                "repository" | "portfolio" => "Selected static source relationships; not deployed architecture",
                _ => "Supplied evidence only; not completeness or runtime health",
            },
        });
    }
    if scopes.len() != graph.scopes.len() {
        return Err("Manifest omits graph scopes".into());
    }
    let mut repository_maps = BTreeMap::new();
    for scope in graph.scopes.values().filter(|s| s.kind == "repository") {
        let Some(name) = scope
            .repo
            .as_deref()
            .and_then(|r| identities.get(&r.to_ascii_lowercase()))
        else {
            continue;
        };
        if repository_maps.insert(*name, scope).is_some() {
            return Err("Multiple repository maps claim one project identity".into());
        }
    }
    let mut projects = vec![];
    for row in rows {
        let name = row["name"].as_str().unwrap();
        let aliases: Vec<_> = row["aliases"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let repo = repository_maps.get(name).copied();
        let collection = repo.and_then(|s| s.repo.as_deref()).and_then(|r| {
            graph
                .collections
                .iter()
                .find(|c| c.id.strip_prefix("collection:") == Some(r))
        });
        let modules: Vec<_> = repo.map_or_else(Vec::new, |repo| {
            graph
                .scopes
                .values()
                .filter(|s| s.kind == "module" && s.repo == repo.repo)
                .map(|s| s.id.as_str())
                .collect()
        });
        let mut gaps = vec![
            "Architecture/system-design relationships have not been assessed.".into(),
            "Runtime observations do not establish a binding to these source bytes.".into(),
        ];
        if let Some(c) = collection {
            gaps.extend(c.errors.iter().cloned());
            gaps.extend(c.limits.iter().cloned());
        } else {
            gaps.push("No source collection receipt for this dated identity; unsupported, inaccessible or unselected sources remain visible.".into());
        }
        if modules.is_empty() {
            gaps.push(
                "No directory module map supplied; semantic module coverage is unavailable.".into(),
            );
        } else {
            gaps.push(
                "Directory scopes are partial static source views, not verified semantic modules."
                    .into(),
            );
        }
        let module_coverage = if modules.is_empty() {
            "unavailable"
        } else {
            "partial-directory-only"
        };
        projects.push(Project {
            id: format!("project:{}", name.to_ascii_lowercase()),
            name,
            aliases,
            map_kind: MapKind::Project,
            evidence: row,
            repo_scope: repo.map(|s| s.id.as_str()),
            module_scopes: modules,
            source_collection: collection,
            source_coverage: if collection.is_some() {
                "partial"
            } else {
                "unavailable"
            },
            module_coverage,
            architecture_coverage: "unavailable",
            runtime_source_binding: "unverified",
            gaps,
        });
    }
    Ok(Catalogue {
        schema: "topo.catalogue.v1",
        graph_revision: &graph.revision,
        project_state_observed_at: observed,
        project_state_sha256: crate::model::hash(state_bytes),
        partial: true,
        gaps: &inventory["gaps"],
        maps,
        projects,
    })
}
