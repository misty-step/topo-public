use crate::{model::*, project::*};
use serde_json::Value;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

pub fn safe_json(value: &Value) -> Result<String> {
    Ok(serde_json::to_string(value)?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026"))
}
fn html(g: &Graph, scope: &str) -> Result<String> {
    let projection = projection(g, scope)?;
    let mut template = include_str!("../design/reference.html").to_string();
    let data_start = template
        .find("<script type=\"application/json\" id=\"infra-data\">")
        .ok_or("Missing reference data slot")?;
    let content = data_start + "<script type=\"application/json\" id=\"infra-data\">".len();
    let end = content
        + template[content..]
            .find("</script>")
            .ok_or("Unclosed data slot")?;
    template.replace_range(content..end, &safe_json(&projection)?);
    let start = template.find("<style>").ok_or("Missing styles")? + 7;
    let end = start
        + template[start..]
            .find("</style>")
            .ok_or("Unclosed styles")?;
    let tokens = include_str!("../design/tokens.css");
    let components = include_str!("../design/components.css");
    template.replace_range(start..end,&format!("\n{tokens}\n{components}\n.t-imports{{--c:var(--color-delivery)}}.t-declares{{--c:var(--color-muted)}}.t-other{{--c:var(--color-other)}}\n.bar{{grid-template-columns:auto minmax(150px,260px) minmax(0,1fr)}}#topo-scope{{width:100%;max-width:100%;font-size:var(--text-control)}}#topo-scope-btn{{display:none}}#topo-scope-compact{{width:100%;max-width:100%}}@media(max-width:760px){{.bar{{grid-template-columns:1fr auto}}.sampleflag{{display:none}}#topo-scope-btn{{display:inline-block}}}}\n"));
    let script_start = template.find("<script>").ok_or("Missing controller")? + 8;
    let script_end = script_start
        + template[script_start..]
            .find("</script>")
            .ok_or("Unclosed controller")?;
    template.replace_range(script_start..script_end, include_str!("../web/viewer.js"));
    template=template.replace("<title>Topo · Visual reset</title>","<title>Topo · Source map</title>")
 .replace("<span class=\"sampleflag\" title=\"Fictional data; no connected tools\">Demo</span>","<span class=\"sampleflag\">Read-only</span><button id=\"topo-scope-btn\" class=\"barbtn\" type=\"button\" aria-expanded=\"false\" aria-controls=\"topo-scope-dock\">Scope</button>")
 .replace("<div class=\"keys\" id=\"keys\"", "<div class=\"keys\" id=\"topo-scope-dock\" role=\"region\" aria-label=\"Map scope\" hidden><label for=\"topo-scope-compact\">Map scope</label><select id=\"topo-scope-compact\" class=\"sel\"></select></div><div class=\"keys\" id=\"keys\"")
 .replace("href=\"../\"", "href=\"index.html\"")
 .replace("<div class=\"detail-body\" id=\"detail-body\">", "<div class=\"detail-body\" id=\"detail-body\" tabindex=\"0\" role=\"region\" aria-label=\"Evidence inspector\">")
 .replace("<h1 class=\"bar-title\">Topo / demo workspace</h1>","<div class=\"bar-title\"><label class=\"sr-only\" for=\"topo-scope\">Map scope</label><select id=\"topo-scope\" class=\"sel\" aria-label=\"Map scope\"></select></div>");
    // The original title markup uses text-only spacing; assert the real slot, not
    // a silently ignored patch. The native selector is the scope navigation owner.
    if !template.contains("id=\"topo-scope\"") {
        let start = template
            .find("<h1 class=\"bar-title\"")
            .ok_or("Workspace title slot missing")?;
        let end = start + template[start..].find("</h1>").ok_or("Title end missing")? + 5;
        template.replace_range(start..end,"<div class=\"bar-title\"><label class=\"sr-only\" for=\"topo-scope\">Map scope</label><select id=\"topo-scope\" class=\"sel\" aria-label=\"Map scope\"></select></div>");
    }
    // Preserve the selector; title text belongs to the accessible document title.
    template = template.replace(
        "document.querySelector('.bar-title').textContent=D.meta.title||'Topo';",
        "document.title='Topo · '+(D.meta.title||'Source map');",
    );
    crate::workspace::fullscreen(&template)
}
fn private_file(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
pub fn bundle(g: &Graph, out: &Path) -> Result<()> {
    // Never overwrite a foreign artifact or an input checkout on refresh.
    if out.exists() {
        return Err("Output already exists; use a fresh run directory".into());
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(out)?;
    #[cfg(unix)]
    fs::set_permissions(out, fs::Permissions::from_mode(0o700))?;
    private_file(&out.join("graph.json"), &serde_json::to_vec_pretty(g)?)?;
    let default_scope = if g.scopes.contains_key("portfolio") {
        "portfolio"
    } else {
        g.scopes.keys().next().ok_or("No renderable scopes")?
    };
    private_file(&out.join("index.html"), html(g, default_scope)?.as_bytes())?;
    for id in g.scopes.keys().filter(|id| id.as_str() != "portfolio") {
        private_file(&out.join(scope_file(id)), html(g, id)?.as_bytes())?;
    }
    let scopes = g.scopes.values().map(|s| Ok(serde_json::json!({"id":s.id,"label":s.label,"map_kind":s.map_kind()?,"file":if s.id=="portfolio"{"index.html".into()}else{scope_file(&s.id)}}))).collect::<Result<Vec<_>>>()?;
    let manifest = serde_json::json!({"schema":"topo.bundle.v1","revision":g.revision,"collected_at":g.collected_at,"portfolio":"index.html","scopes":scopes});
    private_file(
        &out.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}
