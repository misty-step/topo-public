# Collect repositories

Choose what Topo reads and understand what a collection represents. Topo never
installs or runs the target project's application, tests or scripts.

## Choose explicit inputs

Use a Git **root**, not a directory inside it. Directory views are chosen after
collection. At least one commit is required, but current uncommitted source is
included when readable.

```sh
topo run --repo app="$HOME/work/my-app" \
  --out "$HOME/topo-maps/app-001"
```

`app` is your namespace for this input. It becomes part of file, module and
repository IDs. Keep it consistent between runs when you want names to remain
comparable. It is not a queried GitHub/provider identifier or rename lineage.

## Map a system together

```sh
topo run \
  --repo api="$HOME/work/api" \
  --repo web="$HOME/work/web" \
  --out "$HOME/topo-maps/system-001"
```

Namespaces must be distinct. Shared requested packages retain one ID across
supplied repositories; this does not prove identical installed versions.

Start in the portfolio, drill into a repository and choose a directory/module.
Boundary consumers outside that directory remain available when known. Groups
retain membership; they are not newly discovered resources.

## Understand the read boundary

Topo enumerates tracked and nonignored untracked files, then reads supported
source/manifests with bounded no-follow reads. Ignored, generated and credential
paths and leaf/ancestor symlinks are excluded. Source bodies are not exported.

The CLI's JSON result reports read counts, dirty context, errors and limits.
Even an empty error list means **partial** relationships, not complete runtime
discovery. [Supported files and budgets](/docs/limitations/).

<details>
<summary>Dirty worktrees, Git context and source hashes</summary>

Each readable source file has a receipt with a SHA256 of the bytes actually
observed. Git HEAD provides context; those bytes need not equal committed HEAD.
A changing worktree is not an atomic checkout. Preserve dates, collection errors
and hashes when interpreting differences between runs.

</details>

## Refresh without losing history

Run again into a fresh directory. Topo does not overwrite a previous run or
continuously refresh it. Choose the newer folder explicitly; an old browser tab
still contains the old graph.

To rebuild HTML without reading repositories again:

```sh
topo render --graph "$HOME/topo-maps/app-001/graph.json" \
  --out "$HOME/topo-maps/app-001-rendered"
```

This preserves that graph's revision. In {{version}}, newly rendered maps gain
fullscreen controls; older map HTML is not modified in place.

<details>
<summary>Import an existing infrastructure snapshot</summary>

```sh
topo run --snapshot estate=/path/to/redacted-map.json \
  --out "$HOME/topo-maps/import-001"
```

The compatibility input is `topo.infrastructure-map.v1`. Only its `current`
baseline is imported; proposed scenarios are excluded. Dates, direction,
evidence, owner/cost context and gaps remain dated, not refreshed observations.
Topo does not contact a cloud account, host, DNS service or billing API.
The inventory must already be suitable for the intended recipients.

</details>

[Understand the evidence](/docs/evidence/) before treating a relationship as
installation, execution, health or failure-impact proof.
