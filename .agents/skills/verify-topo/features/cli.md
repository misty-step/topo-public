# CLI

A user installs Topo from the public site, maps a Git repository, lists its
scopes, queries entities and relationships as JSON, re-renders an existing
graph, and exports a project-state table. Every request it cannot answer fails
with a non-zero exit. None of them return an empty success.

## Sub-features

- `cli-install`: the published `install.sh` verifies SHA256, installs without sudo, and refuses to replace an existing binary without `--force`.
- `cli-run`: `topo run --repo NAME=ROOT --out NEW_DIR` writes `index.html`, `scope-*.html`, `graph.json` and `manifest.json`.
- `cli-evidence`: static imports and package declarations become entities and edges. Commented imports and ignored `.env` content do not.
- `cli-scopes`: `topo scopes` lists `portfolio`, `repo:NAME` and `module:NAME:dir` IDs.
- `cli-query`: `topo query` returns an entity's direct relationships. It accepts the right `--revision`.
- `cli-render`: `topo render` rewrites a bundle from an existing graph with the same revision.
- `cli-state`: `topo state` exports `projects.md` and `projects.json` from a `topo.project-state.v1` input.
- `cli-refusals`: existing output, unknown scope, wrong revision, tampered graph, non-repository path, repository with no commits, and an invalid namespace each exit non-zero.

## How to get to it (user POV)

- `curl -fsSL https://topo.mistystep.io/install.sh -o install-topo.sh && sh install-topo.sh`, as on the site's quickstart.
- `topo run --repo app=. --out topo-map`, then `topo scopes`, `topo query`, `topo render` and `topo state`, as in `docs/cli.md`.
- Agents use the same commands and read the JSON. See `docs/agents.md`.

## Driving it with verify-topo

Preconditions:

- `verify-topo doctor` passes. The public site answers.
- `TOPO_VERIFY_RUN` is fresh, or its `map/` was removed by `cleanup`.

- **Install.** Run `verify-topo cli`. The `install` step runs the downloaded installer with `TOPO_INSTALL_DIR=$TOPO_VERIFY_RUN/bin`. It must print the version and `Installed`. `install-again-refused` reruns the installer and must exit non-zero with `Already exists`.
- **Collect.** Step `run` maps the fictional `beacon` fixture: a TS web app importing `./client` and `react`, and a Rust API with `mod orders` and `use serde`. The checks require `package:npm:react` and `package:rust:serde` in `map/graph.json`. They also require that the commented `./ghost` import and the ignored `.env` value `fixture-not-a-secret` appear nowhere in the outputs.
- **Scopes and query.** Step `scopes` must list `repo:beacon` and `module:beacon:web/src`. Step `query-entity` (`--entity file:beacon:web/src/app.ts --direction dependencies`) must return an edge to `file:beacon:web/src/client.ts`. Step `query-revision` passes the manifest's revision and must succeed.
- **Render.** Step `render` writes `rendered/`, and its manifest revision must equal the original.
- **Project state.** Step `state` exports the fictional `example/beacon` row to `state-out/projects.md` and `projects.json`.
- **Refusals.** Steps `existing-output`, `unknown-scope`, `wrong-revision`, `tampered-graph` (a label edited with `jq`), `not-a-repo`, `empty-repo` and `bad-namespace` must each exit non-zero. The transcript records the message, for example `Output already exists`, `Unknown scope`, `Graph revision mismatch`, `Graph revision does not match its content`, `Git rev-parse failed` or `Invalid caller namespace`.
- **Unmerged code.** Run `verify-topo cli --source` to drive this checkout's `target/debug/topo` instead of the published binary.
- **Proof.** `evidence/cli/transcript.txt` plus each step's `.stdout` and `.stderr`.

## Gotchas

- The installer serves the released native version (`/manifest.json` `native_version`), not this checkout. A CLI change is proven only with `--source` before release, and with the installer after release.
- `topo run` stdout is the run summary, where `entities` is a count. The full entity map is in `graph.json`.
- `graph.json` is pretty-printed. Tamper with `jq`, not a `sed` pattern that may not match.
- An empty `errors` list does not mean complete discovery. Every collection reports `coverage: partial`.
- The installer needs `git` on PATH and refuses unsupported platforms. Only Linux x86_64 and macOS have downloads.
- `topo-home` and `topo-regenerate` are source-built only. They are not in the installer.
