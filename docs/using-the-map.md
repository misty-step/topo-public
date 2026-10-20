# Read the map

The same workspace serves directory/module, repository and selected-repository
portfolio views. Scope changes what is projected, not entity identity or design.

## Navigate

The scope menu changes views. Click a repository or a group with **Open scoped
map** in its inspector to drill down. Groups are view aggregates, not newly
discovered resources. Their inspector retains member names/IDs; graph JSON retains
exact underlying relationships. On a phone-sized screen, **Scope** opens the menu.

Focal entities sit in the center. Consumers/context are on the left and
dependencies on the right. Wires point **consumer → dependency**. The inspector
also states **from** and **to**, so position alone never establishes direction.

## Focus, search and inventory

Select a node to read its evidence and relationships. **Focus** shows its known
neighborhood. **Search** highlights matching nodes; kind/status/category controls
change the visible projection. **Reset** restores the initial controls.

The map is pannable. Use **+**, **−** and **Fit** for zoom; large graphs aggregate
or scroll instead of shrinking labels into unreadable dots. Inventory gives a
sortable/searchable list and relationship table. Evidence shows coverage,
findings, gaps and source receipts.

## Fullscreen

The square-corner **Fullscreen** button is in the workspace bar. On the marketing
page, the example has its own labelled Fullscreen control above the embedded map.
It expands the existing map, without reloading it or losing selection, filters,
focus, view or the graph revision.

Exit with **Exit fullscreen** or **Escape**. Supported browsers use native
fullscreen. When the browser does not allow it, Topo fills the page instead;
browser chrome can remain visible. Outside marketing content is inactive while
the map is expanded, and keyboard focus returns to the control on exit.

This is available in the public demo and maps exported/rendered with Topo
{{version}}. Older HTML is immutable; rerender its valid graph into a fresh folder
with the current CLI to gain the control without recollecting.

## Evidence is not health

Relation color describes the kind of relationship, not service health. Line
patterns describe evidence: documentary/configuration, positive dated provider
read, or inference. Source-only runs normally show documentary import/declaration
relationships. A provider snapshot import retains its original dates; it is not
a live refresh.

Unknown is not healthy, absent, free or retired. A source/package declaration does
not prove installation, deployed bindings, traffic, restore or failure impact.
Collection limits and unresolved targets are part of the result.

## Keyboard

| Key | Action |
| --- | --- |
| Tab | Enter controls and map nodes |
| Arrow keys | Move between map nodes |
| Enter | Inspect the focused node |
| F | Focus its neighborhood |
| Escape | Exit fullscreen first; otherwise leave focus/selection or close contextual UI |
| / | Search the current view |
| + / − / 0 | Zoom in / out / fit |
| 1 / 2 / 3 / 4 | Map / Inventory / Directions / Evidence |

The imported comparison/direction vocabulary remains available, but source-only
runs do not manufacture operating-model recommendations.

<details>
<summary>Reading crowded views and boundaries</summary>

Choose a directory or repository rather than zoom every label down to an unreadable
size. The focal column stays readable; known incoming consumers outside that
focal directory can remain as boundary context. Aggregates retain underlying
members/edge IDs for [exact queries](/docs/agents/).

</details>

[Understand receipts and uncertainty](/docs/evidence/) or
[query the same knowledge with an agent](/docs/agents/).
