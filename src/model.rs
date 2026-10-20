use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub fn hash(text: &[u8]) -> String {
    format!("{:x}", Sha256::digest(text))
}
pub fn now() -> Result<String> {
    let out = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()?;
    if !out.status.success() {
        return Err("Cannot read UTC time".into());
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub label: String,
    pub entity_kind: String,
    pub category: String,
    pub status: String,
    pub summary: String,
    pub repo: Option<String>,
    pub path: Option<String>,
    pub sources: Vec<String>,
    pub details: Vec<String>,
    pub members: Vec<String>,
}
impl Entity {
    pub fn new(
        id: String,
        label: String,
        kind: &str,
        category: &str,
        repo: Option<String>,
    ) -> Self {
        Self {
            id,
            label,
            entity_kind: kind.into(),
            category: category.into(),
            status: "configured".into(),
            summary:
                "Declared in source/configuration; installed and runtime state were not probed."
                    .into(),
            repo,
            path: None,
            sources: vec![],
            details: vec![],
            members: vec![],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub relation: String,
    pub family: String,
    pub evidence: String,
    pub label: String,
    pub sources: Vec<String>,
    pub line: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub label: String,
    pub result: String,
    pub date: String,
    pub level: String,
    pub repo: Option<String>,
    pub path: Option<String>,
    pub sha256: Option<String>,
    pub head: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub label: String,
    pub enumerated_files: usize,
    pub read_files: usize,
    pub coverage: String,
    pub head: Option<String>,
    pub dirty: bool,
    pub errors: Vec<String>,
    pub limits: Vec<String>,
}

/// Product map kinds are not entity roles or evidence tiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapKind {
    Infra,
    Project,
    Repo,
    Module,
    Architecture,
}
impl MapKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Infra => "Infra",
            Self::Project => "Project",
            Self::Repo => "Repo",
            Self::Module => "Module",
            Self::Architecture => "Architecture",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub repo: Option<String>,
    pub prefix: Option<String>,
    pub focal_ids: Vec<String>,
}
impl Scope {
    pub fn map_kind(&self) -> Result<MapKind> {
        match self.kind.as_str() {
            "snapshot" => Ok(MapKind::Infra),
            "portfolio" | "project" => Ok(MapKind::Project),
            "repository" => Ok(MapKind::Repo),
            "module" => Ok(MapKind::Module),
            "architecture" => Ok(MapKind::Architecture),
            _ => Err("Unknown map scope kind; cannot label it truthfully".into()),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub schema: String,
    pub revision: String,
    pub collected_at: String,
    pub entities: BTreeMap<String, Entity>,
    pub edges: BTreeMap<String, Edge>,
    pub sources: BTreeMap<String, Source>,
    pub collections: Vec<Collection>,
    pub scopes: BTreeMap<String, Scope>,
}
impl Graph {
    pub fn new() -> Result<Self> {
        Ok(Self {
            schema: "topo.graph.v1".into(),
            revision: String::new(),
            collected_at: now()?,
            entities: BTreeMap::new(),
            edges: BTreeMap::new(),
            sources: BTreeMap::new(),
            collections: vec![],
            scopes: BTreeMap::new(),
        })
    }
    pub fn entity(&mut self, node: Entity) {
        if let Some(old) = self.entities.get_mut(&node.id) {
            old.sources.extend(node.sources);
            old.sources.sort();
            old.sources.dedup();
        } else {
            self.entities.insert(node.id.clone(), node);
        }
    }
    pub fn edge(
        &mut self,
        from: &str,
        to: &str,
        relation: &str,
        family: &str,
        label: String,
        source: &str,
        line: Option<usize>,
    ) {
        let id = hash(format!("{from}\0{to}\0{relation}\0{source}\0{line:?}").as_bytes());
        self.edges.insert(
            id.clone(),
            Edge {
                id,
                from: from.into(),
                to: to.into(),
                relation: relation.into(),
                family: family.into(),
                evidence: "config".into(),
                label,
                sources: vec![source.into()],
                line,
            },
        );
    }
    pub fn seal(&mut self) -> Result<()> {
        for edge in self.edges.values() {
            if !self.entities.contains_key(&edge.from) || !self.entities.contains_key(&edge.to) {
                return Err("Dangling graph edge".into());
            }
            for source in &edge.sources {
                if !self.sources.contains_key(source) {
                    return Err("Missing edge evidence".into());
                }
            }
        }
        for node in self.entities.values() {
            for source in &node.sources {
                if !self.sources.contains_key(source) {
                    return Err("Missing entity evidence".into());
                }
            }
        }
        self.revision.clear();
        self.revision = hash(&serde_json::to_vec(self)?);
        Ok(())
    }
}
