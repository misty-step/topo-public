# Work with agents

Let an agent query the same graph you explore in the browser. It does not need
to read screenshots or maintain a second inventory. Topo itself makes no model call.

## Give it the run, not a picture

Supply the local `graph.json` path, the question and the intended repository or
module. The agent needs filesystem permission to read it. Do not upload private
maps to an external model/provider unless that sharing is separately approved.

Start by listing scopes:

```sh
topo scopes --graph "$HOME/topo-maps/app-001/graph.json"
```

Use exact emitted IDs, not names guessed from a diagram. Directory modules have
IDs such as `module:app:src`; their known boundary consumers can live elsewhere.

## Find consumers or dependencies

For a file that actually exists in your graph:

```sh
topo query --graph "$HOME/topo-maps/app-001/graph.json" \
  --scope module:app:src \
  --entity file:app:src/store.ts \
  --direction consumers
```

Use `dependencies` for outgoing relationships and `both` for both directions.
Outputs include matching direct relationships, neighbors and relevant receipts.
A result is not a transitive runtime blast-radius prediction.

<details>
<summary>Inspect a scope before choosing a file</summary>

```sh
topo query --graph "$HOME/topo-maps/app-001/graph.json" \
  --scope module:app:src
```

This returns the supplied projection's entities, edges and collection records.
Use those entity IDs in a subsequent query. Unknown scopes or out-of-scope entity
requests fail rather than return misleading absence.

</details>

## Pin the evidence revision

Read the run's `revision` from `manifest.json`, `graph.json` or a query response,
then add it to subsequent queries:

```sh
topo query --graph "$HOME/topo-maps/app-001/graph.json" \
  --entity file:app:src/store.ts --direction consumers \
  --revision "$REVISION"
```

Set `REVISION` to that actual run's value first. A wrong revision or changed graph
fails with a nonzero exit. This prevents accidental run substitution; it is not
authentication or permission to share.

## Preserve uncertainty in the answer

Return the IDs, source dates and declarations supporting an answer. Keep
collection errors, coverage limits and unresolved targets visible. Dirty bytes
may differ from HEAD; a dated import is not a live provider check.

Do not translate zero matches into absence, import declarations into deployed
bindings, or inferred candidates into mutation/retirement permission. If an agent
needs runtime confirmation, name that missing evidence rather than invent it.

<details>
<summary>Build a custom integration</summary>

The CLI is the supported local read interface. Invoke `scopes`/`query`, parse JSON
and handle nonzero exits. Prefer it for revision checks and scoped queries.
Reading the raw graph is also possible, but preserve its dates, IDs, direction,
coverage and integrity contract. [Graph and response reference](/docs/graph/).

No hosted API, MCP server, customer-data storage or accepted-revision workflow is
shipped here. The optional constrained json-render design adapter does not grant
data access or mutation actions. [Composition boundary](/docs/graph/#composition-boundary)
is described in the advanced graph section.

</details>
