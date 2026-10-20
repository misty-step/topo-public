# CLI reference

Commands are local. No daemon, cloud login or model key is needed.

## Version and help

```sh
topo --version
topo --help
topo --licenses > notices.html
```

`--licenses` exports the embedded third-party component notices as HTML.
All commands print diagnostics to stderr and fail with a nonzero exit when the
request cannot be completed. `run`, `query`, `scopes` and `state` print JSON to stdout;
`render` writes a bundle without printing a new collection result.

## Collect

```sh
topo run --repo NAME=GIT_ROOT [--repo NAME=GIT_ROOT ...] \
  [--snapshot NAME=FILE ...] --out NEW_DIRECTORY
```

- `--repo`: explicit caller namespace and Git repository root, with at least one commit.
- `--snapshot`: already-redacted `topo.infrastructure-map.v1` inventory; imports only
  its `current` baseline and retains original dates. It does not probe providers.
- `--out`: a new directory. Existing directories are never replaced.

Repository namespaces are caller-supplied, not queried native hosting IDs. Paths
with spaces can be supplied as `--repo "app=/path with spaces/app"`.
A namespace is 1 to 128 ASCII alphanumeric or `.-_/` characters and cannot contain
`..`. Duplicate input namespaces are rejected, including across repository and
snapshot inputs. Repeat `--repo`/`--snapshot` for multiple inputs; other flags
occur once. Every flag requires a value.

The result contains:

| File | Purpose |
| --- | --- |
| `index.html` | Default portfolio or imported-snapshot view |
| `scope-*.html` | Self-contained sibling scope views |
| `graph.json` | Full immutable run graph and evidence |
| `manifest.json` | Revision and scope IDs/files |

All collections report partial relationship coverage; `errors` identifies
specific read/syntax failures when present. Empty error lists do not mean complete
runtime discovery. Output permissions are private on supported Unix systems.

## List scopes

```sh
topo scopes --graph topo-map/graph.json
```

Use the emitted exact scope IDs in a query. Directory module IDs have the shape
`module:NAME:relative/directory`; repository IDs are `repo:NAME`.

## Query

```sh
topo query --graph topo-map/graph.json \
  [--scope SCOPE_ID] [--entity ENTITY_ID] \
  [--direction dependencies|consumers|both] [--revision REVISION]
```

Without an entity, returns the requested scope's entities and known relationships.
Without a scope, reads the full supplied graph. With an entity, returns it,
matching direct relationships, neighbors and source receipts. Direction applies
to entity queries; scope-wide queries return their selected known edges.

Out-of-scope entity requests, unknown scopes, wrong revisions, malformed inputs
and content-hash mismatches fail with a nonzero exit status. No match is not proof
of absent infrastructure. Containment is not runtime dependency traversal.

## Render an existing graph

```sh
topo render --graph topo-map/graph.json --out rendered-map-001
```

Reuses the graph's exact knowledge/revision, without recollecting a repository.
The rendering assets are embedded in the executable.

## Export project state

```sh
topo state --input project-state.json --out NEW_DIRECTORY
```

Writes a private `projects.md` table and `projects.json` evidence dossier. Each
repository has a one-line purpose, dated live/location evidence, last real
activity, and a proposed **focus**, **maintain**, or **archive** call with reason.
The command does not discover deployments, refresh Git/provider dates, choose
recommendations, archive repositories, or modify the accepted source-map graph.
Supply the complete intended census; duplicate identities and empty runtime
lists fail. No results are truncated.

Input shape (`topo.project-state.v1`):

```json
{
  "schema": "topo.project-state.v1",
  "collected_at": "2026-10-12T00:00:00Z",
  "inventory_basis": "Path/reference to accepted inventory",
  "option_basis": "Path/reference to accepted hosting proposal; not current placement",
  "coverage": "Census boundary and activity method, including clone/worktree deduplication",
  "gaps": ["Providers not completely surveyed; no match is not absence"],
  "projects": [{
    "name": "owner/repository",
    "purpose": "One-line description",
    "purpose_source": "README path or dated repository metadata",
    "activity": {
      "date": null, "commit": null, "subject": "Unknown",
      "source": "Git unavailable; no activity established"
    },
    "runtime": [{
      "status": "unknown", "home": "No confirmed deployment",
      "date": "2026-10-12", "source": "Incomplete survey"
    }],
    "call": "maintain",
    "reason": "Resolve runtime and ownership before proposing retirement"
  }]
}
```

Runtime statuses: `observed` (dated positive presence, not health/traffic),
`configured` (live unconfirmed), `inactive`, `unknown`, `none` (explicit bounded
absence observation), `peer-reported` (unverified). Every runtime entry preserves
its own source and date. Use null activity date/commit when unknown; name the
activity method and scope in `coverage` and its receipt in `activity.source`.
Supplied text is escaped for Markdown/HTML. The JSON remains the full private
input dossier, not a public-site asset. Existing outputs are never overwritten.

## Compose a private current-map home

```sh
topo-home --state PROJECT_STATE_EXPORT --maps MAP_BUNDLE \
  --out NEW_DIRECTORY --stale-after-hours 48
```

This optional source-built binary composes existing exports; it does not collect,
upload or refresh their observations. The directory contains a catalogue,
`projects/` (the complete table and evidence), `maps/` (every supplied scope), and
a hosted-home `manifest.json`. Inputs and existing outputs are not overwritten.
Only regular expected export files are copied; map revisions/scopes are checked.

The home (`index.html`) is a launcher, not a map. It leads with one large
destination per typed `Infra` map, named from its scope label for display only
(`example-scope` reads Example Scope). Each shows entity counts by
kind from the graph (Hosts, Services, Databases, Domains, Vendors, Accounts; any
other kind stays visible as Other), `Partial` coverage from the catalogue, and
freshness: `Built` is graph generation, `Data` is the age of the oldest
evidence date (`+` when a date is unknown). Tile counts are graph entities;
`Evidence` in `Basis` counts dated sources, a different number. `Basis` also lists
per map the original date labels and the unknown-date count. Projects and Repos
(`Partial` source maps out of all dated projects) sit one level lower; the dated project
list with map coverage lives on `projects/`. Every map page gets a `Topo` link
home and the same freshness bar. Keys 1-4 follow the numbers on screen. Links name
their files (`index.html`), so every return route resolves hosted and from disk.

Every scope shows its original graph generation time and the dates of that
page's evidence. The catalogue and project page distinguish render time from
original inventory/runtime observations. Fresh/Stale/Unknown is recalculated in
the browser; the default threshold is 48 hours. Date-only observations use
00:00 UTC; unknown/future dates are not fresh. Re-rendering never refreshes old
infrastructure observations. The manifest retains input dates, graph revision,
partial coverage and the optional `TOPO_PUBLISH_SOURCE_REVISION` provenance.

This is a full selected bundle, not a per-scope redaction boundary. Hosting is
separate from the public marketing site; see [privacy](/docs/privacy/).

## Prepare freshly collected private source maps

```sh
topo-regenerate --state PROJECT_STATE_JSON --snapshot INFRASTRUCTURE_JSON \
  --repos REPOSITORIES_JSON --out NEW_DIRECTORY
```

This optional Linux publishing helper clones the selected GitHub default
branches using native Git/`gh` and an explicitly injected `GH_TOKEN`, then runs
`topo run`, `topo state` and `topo-home` from its own binary directory. The
repository input is a JSON array of `owner/repository` identities. Target
applications are not installed or executed. The helper does not deploy.

The infrastructure input is `topo.infrastructure.v1`: a nonempty `maps` array
with distinct IDs matching `[a-z0-9][a-z0-9_-]{0,63}`. Each has a `label`, a native
`topo.infrastructure-map.v1` `snapshot` containing only its complete supplied
`current` baseline, `observations`, approved `cloudflare_accounts` (`id`, `label`)
and `dns_domains` (`name`, existing snapshot `node`). Empty source lists are
permitted; they do not establish complete coverage. Shared vendors do not imply
shared account ownership. Keep this payload private and review its assignments.

Snapshot inputs use the generic `topo.infrastructure-map.v1` schema identifier.
Older private inputs need an explicit schema-field migration in a new external
file before use; keep their originals and all evidence dates unchanged.

Before regeneration, refresh the bounded sources using the existing publisher
credential; this helper performs GETs only and never deploys:

```sh
python3 tools/infrastructure.py --input PRIVATE_PAYLOAD --out FRESH_PRIVATE_FILE
```

It lists uploaded Worker scripts, Worker custom domains and D1 databases in
approved accounts, plus NS/MX for approved public domains. D1 pages are followed.
For bindings, the [first deployment returned](https://developers.cloudflare.com/api/resources/workers/subresources/scripts/subresources/deployments/methods/list/)
is the serving deployment; every positive-traffic version is read through the
[version GET](https://developers.cloudflare.com/api/resources/workers/subresources/scripts/subresources/versions/methods/get/).
Newer uploaded versions are never substituted. Split versions retain deployment,
version, percentage and observation dates separately.

Only safe binding names/types and explicitly allowlisted resource-reference
fields leave the native read path. Plain-text, JSON and secret values, runtime
configuration, database records and health are not retained or exposed.
Worker-to-D1 joins use exact IDs in the approved account; production service
bindings use exact Worker IDs there, never similar names or staging-to-production
guesses. Uninspected KV/R2/other resource bindings remain typed unknown references.
No service/database/repository name-based join is performed.

Prior binding observations remain dated evidence. Denied, missing, malformed or
incomplete provider metadata fails preparation before output/publication, rather
than replacing known evidence with an empty result. A prior exact binding cannot
be replaced by unavailable reference metadata. Account containment stays
nonoperational; resource bindings express configured dependencies, not calls,
health, traffic, billing or complete estate coverage.

The refresh output includes `freshness` (`topo.infrastructure-freshness.v1`),
copied by regeneration to `infrastructure-freshness.json`. Each map retains
per-source observation date, status, coverage, unknowns and gaps. Oldest known
observations govern age; undated/future observations cannot be Fresh. Status is
calculated at generation time; consumers must age the original dates, not that
generation date. Original source dates are never advanced by import.

Regeneration consumes the merged `topo.catalogue.v1` output from `topo-home`.
Both exact snapshot scopes must be typed `Infra` and share the hosted manifest's
graph revision before publication preparation succeeds. This reuses the typed
catalogue rather than writing a second project/map index. Account/provider
membership is `contains`, not an operational dependency; configured routing is
kept distinct from health.

Source repositories are freshly collected; original project-state and
unrefreshed infrastructure observations are retained. A successful manifest must
cover every selected repository and both infrastructure scopes. Private
subprocess output is captured rather than echoed. Preparation validates the
complete staged site before renaming it to a fresh output directory; failures
leave no partially prepared output and do not invoke deployment. Temporary
checkouts are private and removed.

The separate daily cloud workflow publishes only after preparation succeeds.
Post-deployment verification failure can occur after the served version changed;
read the actual deployment/version and authenticated manifest before any retry.
No failure receipt alone proves the previous served bundle survived.

## Error handling

Do not convert an error into an empty successful result. The CLI rejects malformed
flags, duplicate nonrepeatable flags, unknown scopes, out-of-scope entities,
unsupported graph schemas, graph content/revision mismatches and existing outputs.
Collection read/syntax/budget errors can be retained in a successful **partial**
run; inspect `collections[].errors` and `limits` before accepting an answer.

[Query response shapes](/docs/graph/#query-response-shapes) ·
[Troubleshooting](/docs/troubleshooting/) · [Limits](/docs/limitations/)
