# Topo user stories

Existing consumer contracts, not a promise of complete runtime/organization
discovery. These preserve the original US-01 through US-09 capabilities with
three-digit IDs and explicit Statement/Criteria fields for the native checker.

## US-001 Collect source relationships

Statement: As a developer, I can map selected Git repositories' source relationships rather than guess their dependencies.

Criteria:
1. Rust, Go and JS/TS file/module requests and Cargo/npm/Go package declarations retain consumer → dependency direction, evidence sources, dates and source lines when available.
2. Comments/string lookalikes do not create imports; unresolved aliases remain unresolved.

## US-002 Read without executing or escaping scope

Statement: As a repository owner, I can collect without executing my project or exposing unrelated files.

Criteria:
1. Collection does not install, build, test or execute the target, mutate providers, consult model credentials or follow leaf/ancestor symlinks outside the selected repository.
2. Ignored/generated/credential paths are excluded and source bodies are not exported.
3. Denied/malformed/budget-skipped inputs remain unknown; dirty worktree hashes describe observed bytes, not a fabricated atomic HEAD.

## US-003 One portable graph

Statement: As a consumer or agent, I can inspect one revisioned evidence model shared by code and the UI.

Criteria:
1. JSON retains stable run-scoped IDs, evidence, collection limits and shared requested-package identity across supplied repositories.
2. Content revisions are verified on read and an unintended revision can be rejected; a checksum is not authorization.

## US-004 Navigate scale without changing identity

Statement: As a reader, I can move among directory-module, repository and selected-portfolio views without losing the same evidence revision.

Criteria:
1. Projections retain boundary consumers/dependencies, group member IDs and exact bundled edge IDs.
2. Containment does not enter runtime dependency traversal; views do not claim complete symbols, native-provider identity or organization authorization.

## US-005 Explore a real map

Statement: As a reader, I can inspect an interactive, readable map instead of a static report.

Criteria:
1. The approved plum tiled workspace retains selection/inspector, neighborhood focus, search, inventory/evidence, pan/zoom, compact controls and keyboard traversal.
2. The map works without inference, credentials or a live backend; the approved fictional R2 specimen is retained.
3. Every fictional demo entity has a readable inspector in map and inventory views. Absent optional sections are omitted, never rendered as literal null/undefined/false fragments; the frozen specimen and its exact 43-entity/69-relationship fixture remain unchanged.
4. The map, public site/docs and private home follow the system light or dark setting with DESIGN.md's plum palettes; changing it keeps selection, focus, filters and view.

## US-006 Query through code and agents

Statement: As an integrator, I can query the same knowledge through the CLI rather than read screenshots or maintain another inventory.

Criteria:
1. CLI lists scopes and returns scoped entities, dependency/consumer relationships and evidence as JSON.
2. Out-of-scope entities and wrong revisions fail rather than returning misleading absence; rendering an existing graph does not create a new inventory.

## US-007 Preserve dated infrastructure imports

Statement: As an operator, I can reuse a dated infrastructure baseline without pretending it is a fresh probe.

Criteria:
1. Import the current baseline of topo.infrastructure-map.v1 retaining original dates, direction, evidence, owner/cost context and gaps.
2. Proposed scenarios are excluded and import does not claim a new provider/host/health/restore observation.

## US-008 Export privately and without overwrite

Statement: As an owner, I can retain private, portable runs and share only deliberately selected inputs.

Criteria:
1. Each fresh private output directory contains offline scope HTML, graph JSON and a manifest; existing outputs are not overwritten.
2. Untrusted labels cannot break out of embedded JSON into executable HTML; offline and normal static-host navigation work.
3. Focused views do not redact a bundle; sharing requires deliberately selected intended inputs.

## US-009 Keep design and composition authority separate

Statement: As an implementer, I can reuse the approved design and constrained composition adapter without granting it knowledge or mutation authority.

Criteria:
1. DESIGN.md owns rules/tokens; generated exports and reusable primitives preserve the approved reference.
2. The official json-render catalog/admission adapter rejects unsupported styling, state writes, invented references and stale projections.
3. Composition does not grant backend access/commits; a production generative runtime remains a separately scoped capability.

## US-010 Start without an operator or build checkout

Statement: As a friend or coworker, I can install Topo and map my own repository without joining someone else's tailnet or obtaining their credentials.

Criteria:
1. The public HTTPS site serves consumer/developer docs, a clearly fictional interactive demo and native macOS Apple Silicon/Intel and Linux x86_64 downloads.
2. Installation verifies a versioned executable's SHA256, uses no sudo, and refuses accidental replacement; a deliberate force option allows update/reinstall.
3. The installed executable embeds parsers and map assets, runs without its build checkout or Rust/Go/Bun/Node toolchains, and collects without an account, provider/model key or source upload.
4. Only deliberately selected local inputs enter a private offline bundle; site/demo publication never includes customer/personal/client inventories.

## US-011 Expand a map and learn progressively

Statement: As a map reader, I can give the workspace the full screen and find the documentation depth appropriate to my task.

Criteria:
1. Embedded, standalone-demo and newly exported maps expose labelled fullscreen controls; expansion preserves selection, filters, focus, view and revision without reloading the map.
2. Unsupported or denied browser fullscreen falls back to page expansion; Exit/Escape restores focus and outside content without clearing map state.
3. The public docs hub groups getting started, task guides and exact references, provides working section/deep links and reveals advanced details when requested.
4. Published earlier release downloads remain available; new controls can be applied by rendering an existing valid graph to a new folder without recollecting or overwriting it.

## US-012 Read every project's state and proposed disposition

Statement: As an operator, I can read a complete supplied repository census against an accepted inventory and hosting option, without confusing recommendations with live facts.

Criteria:
1. The private host-readable table and JSON retain every supplied repository's purpose, dated runtime/location, last real activity, proposed focus/maintain/archive call and reason, with sources and explicit unknowns.
2. Configuration, positive presence, inactivity, bounded absence and peer reports remain distinct; export time does not refresh evidence.
3. Duplicate identities and missing runtime rows fail rather than hiding projects. Existing outputs are not overwritten and no repository, service, provider or hosting proposal is activated or archived.
