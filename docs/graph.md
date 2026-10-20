# Graph reference

Use `graph.json` when building an integration. It is the complete supplied run,
not a screenshot, an HTML layout, an access-control rule or a partial-view export.
The CLI verifies its content revision before querying or rendering.

## Envelope and collection records

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | string | `topo.graph.v1` |
| `revision` | string | SHA256 revision of the serialized typed graph with the revision cleared |
| `collected_at` | string | Run's UTC timestamp |
| `entities` | object keyed by ID | Source, manifest, repository, directory, requested package or imported resource records |
| `edges` | object keyed by ID | Directed documentary/imported relationships |
| `sources` | object keyed by ID | Dated receipts, observed-byte hashes and context |
| `collections` | array | Input-level enumeration, read counts, dirty context, coverage, errors and limits |
| `scopes` | object keyed by ID | Permitted projections of this supplied run |

Collections contain `id`, `label`, `enumerated_files`, `read_files`, `coverage`,
nullable `head`, `dirty`, `errors` and `limits`. `coverage` remains `partial` even
when no errors occurred. File count is not a completeness or health judgment.

Use Topo's typed reader to verify revisions rather than hash pretty-printed JSON
with a generic tool. A revision identifies content; it does not authenticate its
author or authorize a read. Format changes require explicit version handling.

## Entity records and identity

Each entity has `id`, `label`, `entity_kind`, `category`, `status`, `summary`,
nullable `repo`/`path`, `sources`, `details` and `members`.

| ID shape | Meaning |
| --- | --- |
| `repo:app` | Repository under caller namespace `app` |
| `file:app:src/store.ts` | Source/manifest path in that input |
| `module:app:src` | Directory-module containment/scope |
| `package:npm:zod` | Shared requested npm package |
| `package:rust:serde` | Shared requested Rust package |
| `package:go:encoding/json` | Shared requested Go package |
| `package:node:bun:test` | Literal builtin/runtime request; specifier is not normalized into an invented alias |
| `source-package:app:store` | Known Go source package and file members |
| `specifier:app:…` | Unresolved request whose ID contains a digest |

These are examples, not guaranteed IDs in your graph. Enumerate actual entities.
Caller namespaces/path identities are not queried native GitHub IDs or rename
lineage. Shared requested package identity does not prove equal installed versions,
activated features or one deployed service.

Containment `members` are explicit known IDs. View-only aggregates and relationship
bundles appear in rendered projections, not as newly discovered graph resources.

## Directed edge records

Edges contain `id`, `from`, `to`, `relation`, `family`, `evidence`, `label`, `sources`
and nullable source `line`. Imported snapshots may supply `relation` separately
from their display `type`/family; absent `relation` retains the original `type`.
`from` is the consumer and `to` is its dependency for operational/source
relationships. `contains` is containment, not runtime flow; entity queries
exclude it from dependency/consumer relationships.

Source collection retains static `imports` and `declares` relations. Imported
relations keep their original direction and evidence. Do not infer meaning from
color, graph column or an edge's ID alone.

## Source receipts

Sources contain `id`, `label`, `result`, `date`, `level` and nullable `repo`, `path`,
`sha256`, `head`.

A source file's `sha256` identifies observed bytes. Its `head` is contextual:
current bytes may be dirty and collection is not atomic. Git enumeration receipts
have no source-body hash; imported observations can have no repository/path.
Dated provider reads are not newly refreshed by importing.

No source body is exported. Names, paths, declarations and imported details can
still be sensitive. [Privacy](/docs/privacy/) governs sharing the whole graph.

## Scope and bundle records

Scopes contain `id`, `label`, `kind`, nullable `repo`/`prefix`, and `focal_ids`.
Known connected boundary consumers/dependencies can accompany those focal IDs.
The query engine computes this supplied projection; a caller cannot create
permission by inventing a scope or entity ID.

`manifest.json` uses `topo.bundle.v1` and contains the run's `revision`,
`collected_at`, default `portfolio` file and a `scopes` array of `id`, `label`,
`map_kind`, `file`. `map_kind` is the product label, distinct from entity roles
and the internal scope `kind`: `snapshot` → `Infra`, `portfolio`/`project` →
`Project`, `repository` → `Repo`, `module` → `Module`, `architecture` →
`Architecture`. Unknown scope kinds are refused rather than mislabeled.
Rendered projection metadata/navigation carries the same labels. Use supplied
filenames rather than derive them from display labels.
The browser output keeps sibling files for offline navigation.

## Private home identity and coverage catalogue

`topo-home` composes the existing project-state export and map bundle into a
private home; `catalogue.json` (`topo.catalogue.v1`) is linked from that home
and identified by the home manifest's `catalogue` record. No collection,
runtime refresh, adoption or publication happens during composition.

The catalogue retains every supplied dated project, including projects without
maps. Each project has `id` (`project:` plus the case-folded census name),
`name`, explicit `aliases`, `map_kind: Project`, original `evidence`,
nullable `repo_scope`/`source_collection`, `module_scopes` and precise `gaps`.
Only explicit names/aliases join a repository scope; conflicting identity
claims fail before a new output directory is created. Alias paths are not
extra projects, native provider identities or ownership assertions.

`source_coverage` is `partial` or `unavailable`. `module_coverage` is
`partial-directory-only` or `unavailable`: directory views do not establish
semantic modules. `architecture_coverage` remains `unavailable` until real
system-design relationships are assessed; an import chart does not satisfy it.
`runtime_source_binding` remains `unverified`, even for a project with dated
runtime observations and a source map. Collection errors, limits and source
HEAD/dirty context accompany the matching project.

The catalogue's `maps` lists typed scope/file references and interpretation
limits. Infrastructure remains in the supplied graph and Infra scope, not
manufactured from package imports. `graph_revision`, `project_state_sha256`
and `project_state_observed_at` identify the inputs; render time never refreshes
observation dates. All output remains private. Installing code without
installing its reviewed private inputs does not establish expanded coverage.


## Query response shapes

Without `--entity`, `query` returns `revision`, nullable `scope`, an `entities`
array, an `edges` array and `collections`. This is not the same keyed envelope as
the full graph file.

With `--entity`, it returns `revision`, nullable `scope`, the entity,
`relationships`, `neighbors` and relevant `sources` as arrays. Neighbor inclusion
can contain the selected entity itself. Queries return known **direct** matches,
not transitive failure simulation.

`scopes` returns the ID-keyed scope object. [CLI reference](/docs/cli/) gives exact
arguments and errors; [agent guide](/docs/agents/) shows how to use them safely.

<details id="composition-boundary">
<summary>Composition and proposed hosted integration boundaries</summary>

The official json-render catalog/admission adapter is a constrained developer
design component. It accepts trusted projection references and supported
composition, not arbitrary styles, state writes, facts or mutation authority.
It is not a running generative renderer or an authenticated hosted API.

Topo {{version}} has no accepted-revision database, autonomous maintenance,
provider/model connector or side-effect API. Model judgments cannot grant future
access or turn unknown resources into proven facts. Keep integrations local and
read-only unless separately scoped authority is established.

</details>
