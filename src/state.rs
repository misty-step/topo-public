use crate::model::{Result, now};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::{collections::BTreeSet, fs, io::Write, path::Path};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Call {
    Focus,
    Maintain,
    Archive,
}
impl Call {
    fn label(&self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::Maintain => "maintain",
            Self::Archive => "archive",
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Status {
    Observed,
    Configured,
    Inactive,
    Unknown,
    None,
    PeerReported,
}
impl Status {
    fn label(&self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Configured => "configured; live unconfirmed",
            Self::Inactive => "inactive",
            Self::Unknown => "unknown",
            Self::None => "nothing (bounded observation)",
            Self::PeerReported => "peer-reported; unverified",
        }
    }
}
#[derive(Deserialize, Serialize)]
struct Runtime {
    status: Status,
    home: String,
    date: String,
    source: String,
}
#[derive(Deserialize, Serialize)]
struct Activity {
    date: Option<String>,
    commit: Option<String>,
    subject: String,
    source: String,
}
#[derive(Deserialize, Serialize)]
struct Project {
    name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    aliases: Vec<String>,
    purpose: String,
    purpose_source: String,
    activity: Activity,
    runtime: Vec<Runtime>,
    call: Call,
    reason: String,
}
#[derive(Deserialize, Serialize)]
struct Inventory {
    schema: String,
    collected_at: String,
    inventory_basis: String,
    option_basis: String,
    coverage: String,
    gaps: Vec<String>,
    projects: Vec<Project>,
}
// Supplied labels are text, not Markdown/HTML or executable links.
fn text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\n' | '\r' => out.push(' '),
            '\\' | '|' | '`' | '*' | '_' | '[' | ']' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
fn private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path)?.write_all(bytes)?;
    Ok(())
}
pub fn export(input: &Path, out: &Path) -> Result<serde_json::Value> {
    let inventory: Inventory = serde_json::from_slice(&fs::read(input)?)?;
    if inventory.schema != "topo.project-state.v1" || inventory.projects.is_empty() {
        return Err("Expected nonempty topo.project-state.v1 inventory".into());
    }
    let mut names = BTreeSet::new();
    for p in &inventory.projects {
        for identity in std::iter::once(&p.name).chain(&p.aliases) {
            if identity.trim().is_empty() || !names.insert(identity.to_ascii_lowercase()) {
                return Err("Empty or conflicting project identity/alias".into());
            }
        }
        if p.purpose.trim().is_empty() || p.reason.trim().is_empty() || p.runtime.is_empty() {
            return Err(
                "Every repository needs purpose, runtime (unknown is allowed), and proposal reason"
                    .into(),
            );
        }
    }
    let counts = |call| {
        inventory
            .projects
            .iter()
            .filter(|p| p.call.label() == call)
            .count()
    };
    let mut md = format!(
        "# Topo project state\n\nObserved inputs: {}. Exported: {}.\n\nProposals only — archive nothing. Provider presence is not health or traffic; configuration and peer reports are not live proof. No match is not estate-wide absence. Dates below are original observations, not export time.\n\nInventory basis: {}\n\nOption basis: {}\n\nCoverage: {}\n\n{} repositories: {} focus, {} maintain, {} archive proposals.\n\n| Repository | What it is | Live / where (dated) | Last real activity | Proposed call | Reason |\n| --- | --- | --- | --- | --- | --- |\n",
        text(&inventory.collected_at),
        now()?,
        text(&inventory.inventory_basis),
        text(&inventory.option_basis),
        text(&inventory.coverage),
        inventory.projects.len(),
        counts("focus"),
        counts("maintain"),
        counts("archive"),
    );
    for p in &inventory.projects {
        let runtime = p
            .runtime
            .iter()
            .map(|r| format!("{}: {} [{}]", r.status.label(), r.home, r.date))
            .collect::<Vec<_>>()
            .join("; ");
        let activity = format!(
            "{} · {} · {}",
            p.activity.date.as_deref().unwrap_or("Unknown"),
            p.activity
                .commit
                .as_deref()
                .unwrap_or("no commit established"),
            p.activity.subject
        );
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            text(&p.name),
            text(&p.purpose),
            text(&runtime),
            text(&activity),
            p.call.label(),
            text(&p.reason)
        ));
    }
    md.push_str("\n## Unknowns and limits\n\n");
    for gap in &inventory.gaps {
        md.push_str(&format!("- {}\n", text(gap)));
    }
    md.push_str("\n## Evidence by repository\n\n");
    for p in &inventory.projects {
        md.push_str(&format!(
            "### {}\n\nPurpose: {}\n\nActivity: {}\n\n",
            text(&p.name),
            text(&p.purpose_source),
            text(&p.activity.source)
        ));
        for r in &p.runtime {
            md.push_str(&format!(
                "- {} [{}]: {} — {}\n",
                r.status.label(),
                text(&r.date),
                text(&r.home),
                text(&r.source)
            ));
        }
        md.push('\n');
    }
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(out)?;
    private_file(&out.join("projects.md"), md.as_bytes())?;
    private_file(
        &out.join("projects.json"),
        &serde_json::to_vec_pretty(&inventory)?,
    )?;
    Ok(
        serde_json::json!({"output":out,"repositories":inventory.projects.len(),"focus":counts("focus"),"maintain":counts("maintain"),"archive":counts("archive")}),
    )
}
