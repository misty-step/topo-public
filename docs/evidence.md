# Understand evidence

A map shows what supplied evidence supports, plus its gaps. Before changing a
system, inspect the relationship, its receipt and the collection date.

## Start with the inspector

Select a file, package or resource. The inspector states relationship direction
in words and links it to source records. **Evidence** lists findings, source
receipts, coverage and unresolved gaps for the current view.

For source collections, a receipt identifies a relative path, observed-byte
SHA256, date and contextual Git HEAD. A source line, when available, lets you
find the declaration in your checkout. The source body is not in the export.

## Separate kind from certainty

Relationship color describes its family, not health. Line patterns encode
independent evidence:

| Pattern | Meaning | Not established |
| --- | --- | --- |
| Dotted | Documentary/source/configuration evidence | Installation, active execution or live traffic |
| Solid | Positive dated provider/API/CLI/DNS observation in an imported map | Current health, complete enumeration or restore |
| Dashed | Inferred candidate from supplied evidence | A verified fact or permission to mutate |

Source-only runs normally show documentary imports and package declarations.
Imported observations keep their original dates. Importing is not a new probe.

## Read unknowns literally

An unresolved alias/import has not been resolved by the supported collector.
It is not a manufactured vendor and does not mean no dependency exists.

A file that was denied, malformed, ignored or over budget remains outside the
known relationship set. Zero matches means zero matches in the supplied evidence,
not absent infrastructure. Unknown is not healthy, empty, free or retired.

<details>
<summary>How aggregates preserve evidence</summary>

Large views group known entities rather than shrink every label. Group members
retain their IDs and bundled relationships retain exact underlying edge IDs in
the graph. Mixed evidence is not promoted to the strongest member's certainty.
Drill down or [query the graph](/docs/agents/) to inspect the underlying facts.
Containment membership is not an operational dependency.

</details>

<details>
<summary>What would justify a runtime or retirement decision?</summary>

Static imports alone cannot establish deployed bindings, actual callers,
redundancy, traffic, data authority, incremental cost or a working restore.
Those need separate current evidence and appropriate authorization.
Topo {{version}} does not collect or grant those automatically.

</details>
