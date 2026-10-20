# Topo

A local-first, evidence-backed map of your software. A native Rust executable
collects selected repositories and emits the approved interactive workspace plus
one portable JSON graph. Cloudflare delivers the public site, fictional demo,
docs and downloads through a Rust Worker. Private scans and maps stay local.

## Consumer quickstart

```sh
curl -fsSL https://topo.mistystep.io/install.sh -o install-topo.sh
# Inspect it before running.
sh install-topo.sh
export PATH="$HOME/.local/bin:$PATH"

topo run --repo app=. --out topo-map
```

Open `topo-map/index.html`. Use a Git repository root with at least one commit and
a fresh output directory. No account, source upload, model key, Rust/Go/Bun/Node
toolchain or build checkout is needed by the installed executable. Git is needed
for collection. Supported downloads: macOS Apple Silicon/Intel and Linux x86_64
(Ubuntu 22.04+ compatible glibc; WSL2 is Linux). See [install docs](docs/install.md).

- Site: https://topo.mistystep.io/
- Fictional example: https://topo.mistystep.io/demo/
- Consumer walkthrough: https://topo.mistystep.io/docs/quickstart/
- Progressive documentation hub: https://topo.mistystep.io/docs/
- [CLI reference](docs/cli.md) · [Agents/graph](docs/agents.md)
- [Privacy and sharing](docs/privacy.md) · [Known limits](docs/limitations.md)

This is a read-only early preview, not complete infrastructure discovery or a
maintained customer-data service. No application, installer, provider, model call
or customer inventory upload runs during collection. Source declarations do not
prove installed versions, deployed runtime bindings, health, costs or restore.

## Developer build

Rust 1.99.0 and Git. [SPEC.md](SPEC.md) indexes the behavior contracts.

```sh
cargo build --locked -j 2 --bin topo
cargo test --locked -j 2
./target/debug/topo run --repo app=/path/to/git-root --out .cache/run-001
```

Rust `syn` and embedded Tree-sitter grammars collect Rust, Go and JS/TS static
requests. Parsers never execute the target project. Comments/string lookalikes
are not imports. Unresolved aliases stay unresolved. The executable embeds CSS,
viewer behavior and the reference shell; no runtime build-machine path or sidecar
parser is required. The small browser JS controller is the retained approved
renderer, not a backend or knowledge authority.

`graph.json`, `manifest.json` and self-contained sibling HTML views describe one
immutable run revision. Directory/module, repository and selected-portfolio
projections retain shared IDs, member IDs and exact bundled edge IDs. Containment
is not a runtime dependency. Filesystem permissions and deliberate sharing govern
these private exports; a scope selector or checksum is not access control.

Existing output folders are never overwritten. Supported file reads reject leaf
and ancestor symlinks and exclude ignored/generated/credential paths. Budgets are
2,000 supported files, 1 MiB/file and 16 MiB source bytes per repository input.
Failed/denied/skipped/unsupported inputs remain unknown, never proof of absence.
Dirty worktree hashes describe observed bytes, not a fabricated atomic HEAD.

## Dated inventory compatibility

```sh
topo run --snapshot estate=/path/to/redacted-map.json --out imported-map-001
```

Imports the `current` scenario of native infrastructure snapshots, retaining original
evidence dates, direction, owner/cost context and gaps. Proposed scenarios are
excluded. An import is not a fresh API, DNS, host, billing, health or restore check.
Inventories must already be appropriate for intended recipients; Topo is not a
universal secret-redaction service.

## Project state export (native 0.2.3)

```sh
topo state --input project-state.json --out project-state-001
```

Exports the supplied complete repository census as a private, unstyled
`projects.md` table plus `projects.json`: purpose, dated runtime/location,
last real activity, and focus/maintain/archive proposals with reasons.
Accepted inventory and hosting-option references, per-row sources and unknowns
remain explicit. This is not provider discovery, automatic recommendation,
retirement, or a migration. Original evidence dates are never refreshed by export.
[Input contract](docs/cli.md#export-project-state). Private project dossiers do
not enter the public site. Distribution downloads have their own release cadence.

## Design and optional composition tooling

`DESIGN.md` owns visual rules and tokens. Generated `design/tokens.{css,json}`
and `design/components.css` reproduce the approved plum tiled workspace.
`design/reference.html` is the frozen 43-entity/69-edge fictional specimen, not
private infrastructure or a second token authority. The served demo and new
exports restyle it with the generated tokens and `design/components.css`, which
follow the system light or dark setting, and add a shared fullscreen controller
without editing that frozen reference.
Fullscreen preserves map state; unsupported/denied browser mode falls back to an
in-page expansion with Escape/exit and focus restoration. Rendered states matter;
lint/schema success alone is not visual acceptance.

The full source check runs `tests/demo-browser.mjs` in real Chromium: all 43 inspectors in map/inventory views, desktop/phone, system
light/dark palettes, selection, focus and native/denied fullscreen (including
the home embed). Screenshots and JSON results are retained in the public
candidate's `browser-proof` directory.
To rerun against a local Worker or the public site, use
`bun tests/demo-browser.mjs http://127.0.0.1:8798 .cache/browser-proof` after
`bunx --no-install playwright install chromium`. `CHROMIUM_EXECUTABLE` optionally
selects an already-installed Chromium.

The official json-render catalog/admission adapter in `design/` retains its
closed map-first vocabulary and trusted projection references. It rejects
unsupported styles/state writes/invented references; it is not authorization or
a production generative registry/runtime. No inference is needed for maps.

For this optional design/composition development path, install Bun/Node:

```sh
bun install --frozen-lockfile
bun run design:build
bun run check
```

Bun/TypeScript is tooling for official SDK integration, not a consumer runtime
requirement. [Scope/generative proposals](docs/scopes-and-generative-ui.md) remain
separate from implemented source collection and projections.

## Private current-map home (0.2.3)

`topo-home` composes existing private project-state and map exports into a
small catalogue with the existing viewer. Every scope retains generated-at and
original observation dates, with age-based stale/unknown status. Partial source
coverage and dated runtime evidence are not converted into complete or live facts.
See [the command](docs/cli.md#compose-a-private-current-map-home) and
[privacy/recovery](docs/privacy.md#opt-in-private-hosting).

The home leads with the typed Infra maps (hosts, services, databases, domains,
vendors, accounts), freshness as `Built` versus oldest `Data`, and `Partial`
coverage; repos and projects are one level lower. `DESIGN.md#home` owns its
type and texture rules.

Private hosting is optional. `wrangler.maps.toml` is a template with no account,
route, Access issuer, audience, or private hostname. Supply explicit external
operator settings as described in [deployment configuration](docs/deployment.md).
Scans, inventories, source selections, credentials, and deployment receipts stay
outside the source tree. No bundled owner inventory is loaded by default.

`topo-regenerate` prepares a new private bundle from its explicit state, snapshot,
and repository inputs. It does not deploy. Failed preparation leaves the current
publication unchanged. After uncertain deployment or readback, reconcile the
provider version and manifest before retry or rollback.

## Public source checks

This repository distributes development source. It does not automate hosted
publication, native releases, or private map regeneration. The original private
repository retains its history, configuration, deployment custody, and backups.
The public-source review draft must never merge into private main.

Pull requests run advisory TypeScript and Python checks on hosted runners.
The full suite runs twice daily and on manual dispatch. PR cancellation cannot
cancel it. Every job skips in a private repository.

The full suite runs `bun run check`, native relocation tests for Linux x86_64 and
macOS ARM64/Intel, the public Worker error boundary, the Chromium demo, and
`tests/private-worker-auth.mjs` against built private JS/Wasm. Synthetic Access
keys stay in memory; no provider account or deployment settings are required.
The private test records artifact hashes and verifies host, method, and Access
refusals before any asset read. Full proof artifacts expire after 14 days.

On failure, the full workflow uses its ephemeral GitHub token to create or update
one open issue labelled `ci-failure`, with the run URL and SHA. It creates the
label if necessary. Only that job can write issues; all other permissions are
read-only. A manual `drill=true` deliberately fails to verify notification.

```sh
gh workflow run full.yml --ref SOURCE_BRANCH
gh workflow run full.yml --ref SOURCE_BRANCH -f drill=true
```

The generic bootstrap places a dispatch-only workflow at this same path so the
source branch can be tested before merge. Hosted runs and the notification drill
need their own observed evidence. Local checks do not prove them.
[CI and hosting decision](docs/adr/001-code-and-private-publication.md).

## Build a public site

Public inputs are a closed list of consumer docs, generated design exports,
fictional reference, component notices, and verified native executables. Real
maps, settings, inventories, and screenshots are not site-builder inputs.

Put one tested executable per supported target in `.cache/releases`, then run:

```sh
cargo run --locked --bin topo-notices -- .cache/third-party.html
cargo run --locked --bin topo-site -- \
	--releases .cache/releases --out .cache/site-next --notices .cache/third-party.html \
	--native-version VERSION --native-source EXACT_SOURCE_SHA
python3 tools/check_site_links.py .cache/site-next
```

Supply the numeric native version and its full source commit. Every advertised
target must exist and fit the 25 MiB per-asset limit. The builder writes versioned
executables, checksums, installer, docs, and demo. Optional `--previous-releases`
verifies and carries forward versioned binaries and checksums from an explicit
prior directory. Site output directories must be fresh.

The docs hub groups Start/Guides/Reference, derives section links from headings,
and provides a reading sequence. `site/docs.js` opens disclosures for deep links;
ordinary docs work without it. The Changelog page states when no generated notes
are included in the source revision.

Topo does not declare a first-party open-source license. Third-party library
licenses remain applicable. Their notices are available at `/third-party/` and
embedded in `topo --licenses`; `topo-notices` generates them from locked
dependencies. The pinned TypeScript grammar's upstream MIT text is retained in
`site/licenses/` because its crate archive omits LICENSE.
[Grammar provenance](site/licenses/README.md) records the exact upstream revision.

## Self-host and recover

`wrangler.toml` and `wrangler.maps.toml` are templates. Supply operator settings
explicitly through [deployment configuration](docs/deployment.md). Keep operational
defaults, credentials, private datasets, and deployment receipts outside source.
Hosting, promotion, readback, and recovery belong to your deployment system.

The public entry wraps the Rust asset handler with the native Cloudflare Sentry
SDK. An operator can supply their own public DSN and environment. `/health`
returns the environment, source, release, and provider version after checking
the public manifest. It does not inspect private maps. Events contain fixed
failure categories and those identities, never request or exception contents.
Delivery has a one-envelope buffer and a two-second aborting flush, with no
retries, tracing, or logs. An explicit `0.0.0.0` user address prevents inferred IPs.
`node tests/public-errors.mjs` exercises the built modules in local workerd with
synthetic assets and sinks. It writes `.cache/public-errors-proof.json`.

The source workflow builds Workers without publishing them. For a local build,
install the locked Bun dependencies, Rust wasm target, and worker-build 0.8.7 in
your development environment. Then use the build-only Wrangler command:

```sh
bunx --no-install wrangler build --config wrangler.toml
```

Choose your own external uptime and alerting setup. A healthy endpoint or captured
error alone does not establish uptime coverage or notification delivery. Preserve
previous compatible artifacts and verify their manifests after recovery. This
stateless public site has no durable customer database. Private datasets and
consumer maps need separate backup and restore procedures.
