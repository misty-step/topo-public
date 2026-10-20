# CLI

A user installs Topo from the public site, maps a Git repository, lists its
scopes, queries entities and relationships as JSON, re-renders an existing
graph, and exports a project-state table. Every request it cannot answer fails
with a non-zero exit. None of them return an empty success.

## Sub-features

- `cli-install`: the published `install.sh` verifies SHA256, installs without sudo (prints `Installed`), and refuses to replace an existing binary without `--force` (exit 1, `Already exists`).
- `cli-run`: `topo run --repo NAME=ROOT --out NEW_DIR` writes `index.html`, `scope-*.html`, `graph.json` and `manifest.json`.
- `cli-evidence`: static imports and package declarations become entities and edges. Commented imports and ignored `.env` content do not.
- `cli-scopes`: `topo scopes` lists `portfolio`, `repo:NAMESPACE` and `module:NAMESPACE:dir` IDs.
- `cli-query`: `topo query` returns an entity's direct relationships. It accepts the right `--revision`.
- `cli-render`: `topo render` rewrites a bundle from an existing graph with the same revision.
- `cli-state`: `topo state` exports `projects.md` and `projects.json` from a `topo.project-state.v1` input.
- `cli-refusals`: existing output, unknown scope, wrong revision, tampered graph, non-repository path, repository with no commits, and an invalid namespace each exit exactly 1 with their own message.

## How to get to it (user POV)

- `curl -fsSL https://topo.mistystep.io/install.sh -o install-topo.sh && sh install-topo.sh`, as on the site's quickstart.
- `topo run --repo app=. --out topo-map`, then `topo scopes`, `topo query`, `topo render` and `topo state`, as in `docs/cli.md`.
- Agents use the same commands and read the JSON. See `docs/agents.md`.

## Driving it with verify-topo

Preconditions:

- `.agents/skills/verify-topo/verify-topo doctor` passes. The public site answers.
- `TOPO_VERIFY_RUN` is fresh, or its `map/` was removed by `cleanup`.

- **Install.** Run `.agents/skills/verify-topo/verify-topo cli`. The `install` step runs the downloaded installer with `TOPO_INSTALL_DIR=$TOPO_VERIFY_RUN/bin`. It must exit 0 and print `Installed `. `install-again-refused` reruns the installer and must exit 1 with `Already exists: `.
- **Collect.** Step `run` maps the fictional fixture under namespace `example/beacon`, which matches the fictional project's name so the composed home can link the project to its map. The fixture has a TS web app importing `./client` and `react`, and a Rust API with `mod orders` and `use serde`. The checks require `package:npm:react` and `package:rust:serde` in `map/graph.json`. They also require that the commented `./ghost` import and the ignored `.env` value `fixture-not-a-secret` appear nowhere in the outputs.
- **Scopes and query.** Step `scopes` must list `repo:example/beacon` and `module:example/beacon:web/src`. Step `query-entity` (`--entity file:example/beacon:web/src/app.ts --direction dependencies`) must return an edge to `file:example/beacon:web/src/client.ts`. Step `query-revision` passes the manifest's revision and must succeed.
- **Render.** Step `render` writes `rendered/`, and its manifest revision must equal the original.
- **Project state.** Step `state` exports the fictional `example/beacon` row, and both `state-out/projects.md` and `projects.json` must exist.
- **Refusals.** Steps `existing-output`, `unknown-scope`, `wrong-revision`, `tampered-graph` (a label edited with `jq`), `not-a-repo`, `empty-repo` and `bad-namespace` must each exit exactly 1, with, in order, `Output already exists`, `Unknown scope`, `Graph revision mismatch`, `does not match its content`, `Git rev-parse failed` (both repository cases) and `Invalid caller namespace`. A crash (101), a signal or a missing binary (127) fails the step.
- **Unmerged code.** Run `.agents/skills/verify-topo/verify-topo cli --source` to drive this checkout's `target/debug/topo` instead of the published binary.
- **Proof.** `evidence/cli/transcript.txt` plus each step's `.stdout` and `.stderr`.

## Gotchas

- The installer serves the released native version (`/manifest.json` `native_version`), not this checkout. A CLI change is proven only with `--source` before release, and with the installer after release.
- `topo run` stdout is the run summary, where `entities` is a count. The full entity map is in `graph.json`.
- `graph.json` is pretty-printed. Tamper with `jq`, not a `sed` pattern that may not match.
- An empty `errors` list does not mean complete discovery. Every collection reports `coverage: partial`.
- The installer needs `git` on PATH and refuses unsupported platforms. Only Linux x86_64 and macOS have downloads.
- `topo-home` and `topo-regenerate` are source-built only. They are not in the installer.
