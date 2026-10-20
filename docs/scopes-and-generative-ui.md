# One Topo workspace, many scopes

**User direction, 2026-10-08:** the approved visual language should be repeatable
for any project/organization, useful when scoped to a repository or even a module,
and useful in aggregate at organization scale. The shape below is a proposed
implementation contract, not implemented ingestion or a backend API.

## Scope is not zoom

Zoom changes geometry. Scope changes the question, containment boundary, relevant
entities, evidence coverage and useful level of detail. Authorization determines
what may be read; changing scope never grants new access.

Keep one evidence-backed graph and stable identities. A view is a projection of a
revision, not another writable estate or generated inventory. The browser and
agents query the same underlying model. Module, repository, system/project and
organization views are related projections, not four independently maintained
maps or four different visual products.

Containment and operational dependencies are separate facts. A repository
contains a module; a module imports another module; a service uses a database.
Containment must not enter outage reachability as if every child were a runtime
dependency. Dependencies always run consumer → dependency. Inferred static calls,
configured bindings and observed runtime connections retain different evidence.

### Stable identity

- Repository identity includes its host/account and native repository ID. Name,
  URL and current checkout path are attributes, not identity by themselves.
- A module has a scoped identity inside its repository; language/package/symbol
  identity and source revision/path are recorded explicitly. Moves/renames need
  an alias or lineage decision, not a silent delete/create or a name-based merge.
- Provider resources use provider, account and native IDs. A shared vendor,
  database or library has one identity even when many repos use it.
- A deployed service and its source module/repo are distinct entities connected
  by evidence-backed implementation/deployment relationships. One module can have
  multiple deployments; one service can use multiple modules/repos.
- Context membership is not a second ownership authority. A shared entity can
  appear in several projections without acquiring several owners or duplicated
  cost. Organization ownership, repository containment and account custody must
  not be collapsed into one parent field.

## Projection request and result

A future request names scope identity, base revision, lens, desired detail,
filters and (optionally) a focal entity. The server first restricts its knowledge
to caller permissions, then projects the requested scope and authorized boundary
context. A client-selected scope string is not authorization.

A result should contain:

- Projection ID and graph revision; scope breadcrumb and permitted navigation.
- Stable entity IDs; explicit focal/context/dependency roles; typed edges with
  direction, accepted/inferred disposition and evidence references.
- Visible groups and membership references to the **same** underlying entities.
  An aggregate is a view object, not a newly discovered provider resource.
- Boundary relationships: imports/callers, shared services, runtime/provider
  dependencies and relevant ownership. Out-of-scope does not mean absent.
- Source dates and collection coverage per source/scope: complete, partial,
  denied or failed. Repository read completeness does not establish complete
  organization or provider coverage.
- Underlying membership/edge lookup or cursor for permitted drill-down, search,
  inventory and exports. No hidden truncation marketed as the full estate.
- The selection/anchor mapping needed to keep a module selected when its parent
  repo/project becomes a collapsed node in the broader view.

Cross-boundary details require independent read authority. Show an opaque boundary
only when the relationship itself is allowed to be disclosed. Do not reveal
inaccessible names, resource counts or prices through an aggregate. Unknown
coverage is not a known zero. Denied collection preserves prior observations.

## Working at different scales

| Scope | Ordinary starting projection | Useful drill-down |
| --- | --- | --- |
| Module | Focal module, callers, imports, owned data and external services | Submodules/symbols or a supplied call path |
| Repository | Packages, modules, executables and deployable services | A module/package; its consumers and dependencies |
| Project/system | Connected repos/services plus shared delivery/runtime | One repo or service, preserving cross-repo edges |
| Organization | Grouped projects/repos/services and shared provider/account resources | Project → repo → module, or shared-resource consumers |

Use the **same shell and component set**. The central column changes its level of
detail; contextual consumers remain left, dependencies right, and evidence stays
in the inspector. The graph is not necessarily an acyclic tree: cycles,
shared dependencies and alternate execution paths must remain representable.

Aggregation is deterministic over supplied evidence and permitted membership.
Semantic suggestions can recommend group labels or investigations; they do not
merge entity identity. An edge bundle retains direction, relation and evidence
mix. Mixed/unknown/stale/conflicting members remain visible in its summary. Cost
summaries deduplicate resource IDs and retain currency, period and billing basis;
partial amounts are not presented as complete totals. Do not add month-to-date
usage to monthly tariffs.

Plan viewport-sized projections, local neighborhood focus, edge bundling,
paginated inventory/search and incremental expansion before a new graph database
or a canvas-only UI. Preserve an accessible text/list surface. Do not fit every
function in every repo into one organization diagram or render ten thousand
interactive DOM nodes to call the product “scalable.”

**Acceptance still needed:** representative real module and repo evidence;
shared resource appearing across scopes without duplicated identity/cost; retained
selection on drill-up/down; a cross-repo consumer; a denied/partial source; cycles;
large authorized estates; keyboard and compact navigation; measured latency,
visible-node/edge budgets and offline export size. The 43-entity reference proves
visual quality at its bounded size, not organization-scale performance.

## json-render: composition, not invention

Official json-render separates schema (grammar), catalog (vocabulary), component
registry (implementation) and action handlers (execution). Topo provides its own
catalog instead of generic Card/Metric/Stack components that could erase the
approved map identity.

The seed's `design/catalog.ts` uses the official `@json-render/core` custom-schema
API. Its flat-tree wire format can be paired with a future platform registry.
The restricted schema does not inherit the React schema's built-in state writers
or its default instruction to invent realistic sample data. Production graph
facts come from accepted evidence, not generated `state`.

The closed catalog supplies WorkspaceFrame, ScopeNavigator, CoverageStrip,
TopologyCanvas, InspectorDock, RelationshipLegend, EvidenceList, CollectionQueue
and InvestigationBrief. Props contain literal reference IDs and bounded lens
choices. No arbitrary HTML, JavaScript, CSS, styling props, URLs, graph nodes,
prices, health assertions, event handlers or mutation actions are accepted.

`design/admit.ts` demonstrates the composition boundary:

1. Validate the literal wire shape and props through the official catalog.
2. Require the fixed map-first frame, navigation, coverage, canvas, dock and legend.
3. Reject dangling/cyclic/multiply parented/unreachable elements and unsupported
   dock children. Keep bounded element/prop counts.
4. Resolve all references against a trusted authorized projection/context, not
   IDs or evidence records supplied by the model. Reject stale projection IDs.
5. Only a complete admitted composition may replace the current valid view. A
   consumer must retain the ordinary map during partial, invalid or unavailable
   generation; this fallback renderer is not implemented by the seed.

The code gate checks reference integrity and composition. It is **not backend
access control**. The real service must independently authenticate, authorize and
validate every read/proposal/commit. A component registry or model role cannot
turn a local preview into a provider update. Navigation handlers belong to code;
starting an agent investigation later requires explicit scope, access and budget.

Good generation: “Focus billing's external dependencies and show the supplied
restore receipts and collection gaps in the dock.” Bad generation: “Create a new
white dashboard, declare the host healthy, invent a backup, and retire it.”

A generative model can propose the lens or prepare source-linked explanations.
Jev/Clef can choose among prepared semantic candidates; deterministic code owns
identities, arithmetic, aggregation, layout and permissions. Model failure leaves
the ordinary map useful. No model call or private-data disclosure is needed for
the design seed or its checks.

### Existing Jev experiment

json-render's official [Jev guide](https://json-render.dev/docs/jev) already
explores composition from prepared atomic candidates. That is a promising fit for
choosing supported lenses/dock content without letting a model invent facts.
However, the guide marks the composer **unreleased/experimental** and its evaluator
uses Vercel Gateway's experimental evaluation endpoint. It is not the fleet's
OpenRouter Decisions adapter. Do not switch transports, copy auth stores, or claim
production compatibility from the guide. Source-pin and evaluate a separately
scoped integration if commissioned; the stable catalog does not require it.

## First real vertical slice

Import one repo's known graph plus manual vendor context, derive one module view
and the repo view at the same revision, and place that repo into an organization
view with one genuinely shared resource. Render all three through the approved
components; inspect provenance and coverage; query the same projections through
the agent API/CLI; export the same revision. Exercise it with inference disabled.

A read-only composition prototype can follow, choosing one focus/lens and supplied
evidence sections through the catalog. Scope ingestion, shared-revision persistence,
aggregation and a framework registry require grooming/implementation; the design
seed does not silently select a production frontend or backend stack.

Sources: Phaedrus's visual/scoping direction, 2026-10-08;
[json-render catalog](https://json-render.dev/docs/catalog),
[schemas](https://json-render.dev/docs/schemas),
[custom schema](https://json-render.dev/docs/custom-schema),
[Jev experiment](https://json-render.dev/docs/jev). Product code owns technical
truth; this contract records the intended next implementation boundary.
