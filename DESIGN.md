---
version: alpha
name: Topo
description: A compact, map-first workspace for maintained system knowledge, from one module to an organization.
colors:
  primary: "#d59be5"
  desk: "#120b17"
  pane: "#25172c"
  inset: "#1c1122"
  raise: "#392342"
  sel-bg: "#56345f"
  sel-fg: "#fff7ff"
  fg: "#f2e8f3"
  fg-hi: "#fff7ff"
  muted: "#b9a6bd"
  chip-count-selected: "#cbb9d0"
  chip-selected-surface: "#4a3d50"
  border-inactive: "rgba(89,89,89,0.667)"
  control: "rgba(242,232,243,0.04)"
  control-hover: "rgba(242,232,243,0.08)"
  control-on: "rgba(242,232,243,0.18)"
  line: "rgba(242,232,243,0.1)"
  line-strong: "rgba(242,232,243,0.25)"
  scrollbar: "rgba(242,232,243,0.22)"
  entity-border: "rgba(242,232,243,0.45)"
  focal-border: "rgba(242,232,243,0.34)"
  node-hover: "#2f1e37"
  focal-hover: "#43294d"
  hosting: "#a6afe9"
  data: "#83c79b"
  auth: "#e4c36e"
  inference: "#d59be5"
  communication: "#ef8195"
  delivery: "#7fc4c8"
  monitoring: "#e4a078"
  dns: "#c79573"
  business: "#b9a6bd"
  other: "#b9a6bd"
  light-primary: "#88439a"
  light-desk: "#f3e9f2"
  light-pane: "#fdf8fc"
  light-inset: "#f8f1f7"
  light-raise: "#eeddf2"
  light-sel-bg: "#e9cbf0"
  light-sel-fg: "#1f1122"
  light-fg: "#38263c"
  light-fg-hi: "#1f1122"
  light-muted: "#66566a"
  light-chip-count-selected: "#564559"
  light-chip-selected-surface: "#e1dbe1"
  light-border-inactive: "rgba(56,38,60,0.28)"
  light-control: "rgba(56,38,60,0.035)"
  light-control-hover: "rgba(56,38,60,0.07)"
  light-control-on: "rgba(56,38,60,0.14)"
  light-line: "rgba(56,38,60,0.1)"
  light-line-strong: "rgba(56,38,60,0.34)"
  light-scrollbar: "rgba(56,38,60,0.28)"
  light-entity-border: "rgba(56,38,60,0.58)"
  light-focal-border: "rgba(56,38,60,0.45)"
  light-node-hover: "#f5edf5"
  light-focal-hover: "#e9d1ee"
  light-hosting: "#535aad"
  light-data: "#267b4c"
  light-auth: "#816512"
  light-inference: "#88439a"
  light-communication: "#b83753"
  light-delivery: "#0c767b"
  light-monitoring: "#a14f09"
  light-dns: "#6c4932"
  light-business: "#66566a"
  light-other: "#66566a"
typography:
  mono:
    fontFamily: &mono '"JetBrainsMono Nerd Font","JetBrainsMono NF","JetBrains Mono","GeistMono Nerd Font",ui-monospace,"Cascadia Mono","SF Mono",Menlo,Consolas,"DejaVu Sans Mono","Liberation Mono",monospace'
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.45
  control:
    fontFamily: *mono
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.45
  metadata:
    fontFamily: *mono
    fontSize: 11px
    fontWeight: 400
    lineHeight: 1.45
  focal:
    fontFamily: *mono
    fontSize: 13px
    fontWeight: 600
    lineHeight: 1.25
  inspector-title:
    fontFamily: *mono
    fontSize: 16px
    fontWeight: 600
    lineHeight: 1.3
  home-name:
    fontFamily: *mono
    fontSize: 4.25rem
    fontWeight: 600
    lineHeight: 1
    letterSpacing: -0.035em
  home-count:
    fontFamily: *mono
    fontSize: 2.25rem
    fontWeight: 600
    lineHeight: 1.05
    letterSpacing: -0.02em
  home-body:
    fontFamily: *mono
    fontSize: 1.0625rem
    fontWeight: 400
    lineHeight: 1.45
  home-meta:
    fontFamily: *mono
    fontSize: 0.9375rem
    fontWeight: 400
    lineHeight: 1.45
rounded:
  none: 0px
spacing:
  hairline: 1px
  border: 2px
  xs: 4px
  sm: 6px
  md: 8px
  pane-gap: 10px
  pane-gap-compact: 6px
  lg: 12px
  xl: 16px
  section: 20px
  desktop-bar: 28px
  compact-bar: 36px
  desktop-control: 26px
  compact-control: 34px
  inspector-width: 368px
  inspector-compact: 320px
components:
  workspace:
    backgroundColor: "{colors.desk}"
    textColor: "{colors.fg}"
    typography: "{typography.mono}"
    rounded: "{rounded.none}"
    padding: "{spacing.pane-gap}"
  pane:
    backgroundColor: "{colors.pane}"
    textColor: "{colors.fg}"
    rounded: "{rounded.none}"
  topology-canvas:
    backgroundColor: "{colors.inset}"
    textColor: "{colors.fg}"
  focal-entity:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.fg-hi}"
    typography: "{typography.focal}"
  focal-entity-hover:
    backgroundColor: "{colors.focal-hover}"
    textColor: "{colors.fg-hi}"
  dependency-entity-hover:
    backgroundColor: "{colors.node-hover}"
    textColor: "{colors.fg}"
  entity-selected:
    backgroundColor: "{colors.sel-bg}"
    textColor: "{colors.sel-fg}"
  metadata:
    backgroundColor: "{colors.pane}"
    textColor: "{colors.muted}"
    typography: "{typography.metadata}"
  chip-count-selected:
    backgroundColor: "{colors.chip-selected-surface}"
    textColor: "{colors.chip-count-selected}"
    typography: "{typography.metadata}"
  control:
    backgroundColor: "{colors.pane}"
    textColor: "{colors.fg}"
    typography: "{typography.control}"
    height: "{spacing.desktop-control}"
  control-fill:
    backgroundColor: "{colors.control}"
  control-hover:
    backgroundColor: "{colors.control-hover}"
  control-on:
    backgroundColor: "{colors.control-on}"
  active-pane-border:
    textColor: "{colors.primary}"
  inactive-pane-border:
    textColor: "{colors.border-inactive}"
  divider:
    textColor: "{colors.line}"
  control-border:
    textColor: "{colors.line-strong}"
  scroll-thumb:
    textColor: "{colors.scrollbar}"
  entity-border:
    textColor: "{colors.entity-border}"
  focal-border:
    textColor: "{colors.focal-border}"
  hosting-wire:
    textColor: "{colors.hosting}"
  data-wire:
    textColor: "{colors.data}"
  auth-wire:
    textColor: "{colors.auth}"
  inference-wire:
    textColor: "{colors.inference}"
  communication-wire:
    textColor: "{colors.communication}"
  delivery-wire:
    textColor: "{colors.delivery}"
  monitoring-wire:
    textColor: "{colors.monitoring}"
  dns-wire:
    textColor: "{colors.dns}"
  business-wire:
    textColor: "{colors.business}"
  other-wire:
    textColor: "{colors.other}"
---

# Topo design standard

## Overview

**Approved visual baseline.** On 2026-10-08, Phaedrus approved the Topo R2
reset: “awesome. beautiful. we should lock this in to the DESIGN.md standard.”
The accepted reference is the compact plum tiled workspace. Its frozen,
entirely fictional capture is `design/reference.html`.

Topo is a working map, not a dashboard that happens to contain a diagram.
Keep the topology, compact controls, relationship legend and evidence inspector
in one coherent workspace. Density comes from meaningful structure and useful
interaction, not small illegible text. The reference draws its palette from
Omarchy Ivory Blush and its window grammar from the original Example Scope map. It does
not contain Example Scope client inventory.

Every run uses this visual language. The dataset, scope, detail and selected lens
may change; the product's typography, palette, pane grammar and interaction
identity do not. This approval locks the visual baseline, not a backend stack,
AI provider, React application, or provisioning authority.

**Authority:** this file owns token values and design rules. `design/tokens.json`
and `design/tokens.css` are generated exports, not competing editable sources.
`design/components.css` owns reusable selector structure. The reference capture
is a frozen visual specimen, not a second token registry. Token changes must be
reviewed against the rendered map, focused neighborhood and compact inspector.

## Colors

Use a plum-black desk, plum panes and a slightly darker graph surface. Raised
focal entities, lavender active-window borders and a pale selected-node outline
establish hierarchy. Do not replace these with white cards or a blue SaaS shell.

Every surface follows the reader's system light or dark setting; there is no
theme control. `light-<role>` is that role's system-light value. Light keeps
the tiled plum grammar: ivory-blush desk, blush-white panes, a slightly darker
graph surface, plum ink, deep orchid active borders, lilac raised entities and a
lilac selection with an ink outline. Relationship families keep their hues,
deepened to at least 4.5:1 as text on panes. Components name only the base
roles, so the design.md linter reports `light-*` colors as unreferenced.

Relationship families have consistent colors: hosting lavender-blue, data green,
authentication yellow, inference lilac, communication rose, delivery cyan,
monitoring apricot, DNS brown, and business/other muted plum. Color describes the
relation, **not health**. Use labels and line patterns as redundant encoding.
Source-code imports/calls need explicit relation names; they must not be mislabeled
as AI inference. A future family can reuse an existing hue if its label is clear.

Evidence encoding is independent of relationship color: solid for a positive
provider/API/CLI/DNS observation; dotted for documentary/configured evidence;
dashed for an inferred candidate. An inferred edge never becomes a verified fact
because it shares a color with one. Status glyphs distinguish observed,
configured, historical and unknown; “observed” does not imply healthy.

Control text sits over the pane plus its alpha fill; `control` describes the
underlying surface and `control-fill` the overlay. Inspect the composited result.
The selected count token preserves R2's contrast fix. Do not use ordinary muted
text on a selected chip: the original combination measured 4.45:1. Alpha colors
must be checked on their actual composited surface, not against assumed white.

## Typography

Monospace throughout. Use the bundled/system fallback stack in the tokens; no
remote font dependency is required to read an exported map. Body 13px, controls
and dependencies 12px, metadata 11px, focal entities 13px semibold, inspector
heading 16px semibold. Normal text has no tracked all-caps treatment.

Do not introduce a display headline above the map. Long entity names wrap or
truncate with an accessible full name; they do not force the root page wider.
Maintain readable text by grouping, drilling down or scrolling before shrinking
the entire organization to a microscopic graph. Browser text zoom remains usable.

## Layout

The application shell fills the viewport and does not become a long landing
page. Each long surface scrolls inside its own pane. The desktop workspace bar
is 28px; panes have 10px gaps, 2px borders and zero radius. The ordinary inspector
is 368px wide, reducing to 320px on narrower desktop layouts. Controls sit above
the map; relationship/evidence filters sit below it.

The map's center holds the selected level's main entities. Context and consumers
sit on the left; dependencies sit on the right. These are **semantic roles**, not
fixed product/provider kinds or geographic coordinates. Directed operational
relationships run **consumer → dependency**, including left-side consumers whose
edges point into the focus. An inspector always states direction in words.

**Fullscreen is a workspace operation.** Keep a compact, labelled square-corner
control in the map bar and a text-labelled control beside the marketing embed.
Expand the same live workspace, not a new iframe or screenshot: preserve selection,
focus, filters, view and revision. Native browser fullscreen is preferred; unsupported
or denied mode falls back to filling the page. Exit/Escape restores the prior pane
and keyboard focus. Expanded embeds make outside content inactive. The frozen
reference remains untouched; maintained exports/demo apply the shared enhancement.

Scope is navigation, not a second application:

| Scope | Typical central detail | Boundary/context | Expansion |
| --- | --- | --- | --- |
| Module | The focal module and useful internal units | Callers, imports, data, external services | Internal submodules or symbols when evidence supports them |
| Repository | Packages, modules, executables and deployable services | Shared libraries, CI, runtime, vendors | A package/module or a connected service |
| Project/system | Related repositories and services | Shared identity, delivery and infrastructure | A repository, service or module |
| Organization | Grouped projects/repos/services and shared providers | Cross-project dependencies and coverage limits | A project/repo, retaining shared entity IDs |

At larger scales, **aggregate rather than miniaturize**. Groups retain counts,
source dates, coverage and unresolved conflicts. Bundled edges retain relation,
direction, evidence mix and drill-down to the exact underlying edges. A group
is not a newly discovered resource. Containment is not runtime dependency.
Do not hide an unresolved finding inside an apparently healthy aggregate.

Scope breadcrumb, selected entity, lens, revision and filters are restorable view
state. The same entity keeps its identity across scopes. A shared database or
vendor is not cloned or billed again for every repository that uses it.
See `docs/scopes-and-generative-ui.md` for the proposed projection contract.

On compact screens (the reference switches at 760px), use a 36px workspace bar,
6px pane gaps, at least 34px controls, a pannable map, collapsible filters and a
bottom inspector up to 72dvh. Start with the focal column readable. Selecting a
node opens details; closing details returns focus. Keep a labelled list/table
alternative. Do not make “organization scope” mean “all nodes rendered on a phone.”

## Home

The private home is a launcher in front of the workspace, not a map. These rules
apply to the home only; the map workspace keeps its sizes, density and motion.

- **Order:** one large destination per typed `Infra` map first, then Projects and
  Repos one level lower. Repos open as a disclosure; every map stays reachable.
- **Type:** `home-*` tokens. Nothing under 15px; sizes are rem so browser text
  size applies. Map names are destination labels, not a hero headline.
- **Words:** one word per label or action. No sentences in the home. Rules and
  original dates open behind `Basis`.
- **Freshness:** `Built` is when the map was generated. `Data` is the age of the
  oldest observation, `+` when a date is unknown. Status is Fresh, Stale or
  Unknown by the same rule as the map bar; `Partial` marks incomplete coverage.
  Never imply a runtime probe.
- **Texture:** flat 2D contour hairlines at the edge of a destination, at most
  20% opacity, deterministic per map. Not terrain, and no gradient, shadow or glass.
- **Motion:** press, hover, the Repos disclosure and the Basis popover only.
  Keyboard navigation is instant. Reduced motion removes movement; hover motion
  is gated to fine pointers.

## Elevation & Depth

Tonal surfaces and active-window borders carry depth. No gradients, card shadows,
blurred glass, floating bubble groups or decorative 3D terrain. Reserve motion
for direct manipulation and compact inspector transitions; respect reduced motion.

## Shapes

Sharp rectangles, compact square status glyphs and narrow semantic edge swatches.
A focal entity is raised and semibold; a dependency has a colored side stripe;
a tool can use a double stripe; a domain is a quieter boundary label. Entity
kind is not inferred solely from appearance. Selection, keyboard focus and
observation status remain independently recognizable.

## Components

Use the actual CSS primitives in `design/components.css`, with the generated
native CSS tokens. Their semantic attributes and controller behavior matter as
much as their appearance. The future framework registry must reproduce these
contracts; the current seed is not a production React component library.

| Component | Reusable structure | Behavioral contract |
| --- | --- | --- |
| WorkspaceFrame | `.bar`, `.desk`, `.pane`, `.pane.is-active` | Keep the map present; tiles scroll independently; active border follows focus |
| ScopeNavigator | Compact breadcrumb/workspace controls | Change authorized scope without changing visual identity; retain a return path |
| CoverageStrip | `.hint`, `.notice`, status glyphs | Show complete/partial/denied/failed coverage and observation dates; no health fiction |
| TopologyCanvas | `.map-viewport`, `.map-world`, `.edges`, `.node`, `.band` | Deterministic layout, directed edges, focus, pan/zoom, keyboard traversal and accessible alternative |
| EntityNode | `.node`, kind/status/selection modifiers | One entity ID; full accessible name; select opens its inspector; rerender retains focus |
| RelationshipLegend | `.legend`, `.chip`, `.sw` | Relation color and evidence pattern are independent; toggles expose state through ARIA |
| InspectorDock | `.detail-head`, `.detail-body`, `.facts`, `.rel`, `.srcrow` | Resolve IDs against the authorized revision; preserve provenance and uncertainty |
| Inventory | `.table-wrap`, `.inv-table` | Sort/filter the same projection; compact rows keep labels; horizontal scroll stays local |
| EvidenceList | `.srcs`, `.finding`, `.notice` | Source-linked accepted assertions, contradictions and gaps; no free-standing AI facts |
| CollectionQueue | Existing pane/list grammar; behavior not implemented | Denial preserves prior knowledge; preview is not a committed update |
| InvestigationBrief | Existing focused map/inspector grammar; behavior not implemented | Show bounded goal, known evidence and open questions without replacing the workspace |

Keyboard minimum: Tab into controls/map; arrow keys move through map entities;
Enter selects; focus toggles the neighborhood; Escape closes contextual UI;
search and reset remain available. Announce selection/results and maintain focus
through rerender. Exports remain readable without inference, credentials or a
live collector.

**Generative UI boundary:** json-render is a constrained composition adapter,
not a design authority or fact store. Code supplies the frame, authorized graph
projection and revision. Generation can select a supported lens, focus entity
and useful evidence/maintenance/investigation sections. It cannot supply arbitrary
HTML/CSS/JavaScript, colors, geometry, provider URLs, graph facts or mutation actions.
`design/catalog.ts` demonstrates the closed vocabulary using official
`@json-render/core`; `design/admit.ts` enforces the required map-first shape and
literal authorized references. A schema-valid tree alone is insufficient.

Keep the last valid ordinary map during partial/invalid generation. Do not allow
streaming output to remove the map, coverage strip or return path. Composition
must not automatically execute actions. Reading accepted knowledge works with
inference disabled. Semantic decisions may use Jev/Clef behind their existing
transport boundary; the renderer does not choose credentials or grant authority.

## Do's and Don'ts

- **Do** reproduce the approved workspace before exploring new treatments.
- **Do** derive CSS/DTCG tokens from this file; change the source, not the exports.
- **Do** keep stable identities, scope boundaries and provenance at every scale.
- **Do** separate graph aggregation, semantic interpretation and authorization.
- **Do** use readable aggregation, drill-down, search and an accessible inventory.
- **Do** exercise actual rendered states; lint is not visual acceptance.
- **Don't** use six-node toy diagrams to represent a large estate.
- **Don't** add hero headings, rounded provider blobs or report-first navigation.
- **Don't** let generation replace the shell with generic cards or fabricate facts.
- **Don't** equate reachability with certain failure when redundancy is unknown.
- **Don't** label unknown, denied or partial coverage empty, healthy, free or retired.
- **Don't** expose inaccessible resources or their counts through an aggregate.
- **Don't** promise infinite-scale performance from a 43-entity reference. Scale
  budgets, semantic grouping and real module/repo/org acceptance need measurement.

Source: Phaedrus's 2026-10-08 visual approval and module/repository/organization
scope request. Exact token values follow the approved fictional R2 artifact.
Generative composition and multiscale projection details are implementation
recommendations; their adapter checks do not establish backend behavior.
