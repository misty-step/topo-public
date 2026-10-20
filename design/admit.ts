import { z } from "zod";
import { catalog, components } from "./catalog.ts";

const element = z.object({
  type: z.enum(Object.keys(components) as [keyof typeof components, ...(keyof typeof components)[]]),
  props: z.record(z.string(), z.unknown()),
  children: z.array(z.string()).max(8),
}).strict();
const wire = z.object({ root: z.string(), elements: z.record(z.string(), element) }).strict();
export type Spec = z.infer<typeof wire>;

// Supplied by trusted code from one authorized snapshot, never from a model spec.
// Membership here checks reference integrity; it is NOT backend authorization.
export interface ProjectionContext {
  projectionId: string;
  revision: number;
  entityIds: ReadonlySet<string>;
  receiptIds: ReadonlySet<string>;
  collectionIds: ReadonlySet<string>;
  investigationIds: ReadonlySet<string>;
}

export function admit(spec: unknown, context: ProjectionContext): Spec {
  const parsed = wire.parse(spec); // Reject state, on/watch/visible/repeat/slots, and unknown fields.
  const entries = Object.entries(parsed.elements);
  if (entries.length > 16) throw Error("Composition exceeds the bounded workspace");
  // Explicit literal-prop validation disallows expressions and avoids relying
  // on permissive catalog behavior for arbitrary extra properties.
  for (const [, e] of entries) components[e.type].props.parse(e.props);
  const checked = catalog.validate(parsed);
  if (!checked.success) throw Error("Spec does not match the json-render catalog");
  const root = parsed.elements[parsed.root];
  if (!root || root.type !== "WorkspaceFrame" || root.props.projectionId !== context.projectionId)
    throw Error("Workspace must reference the supplied authorized projection");
  const required = ["ScopeNavigator", "CoverageStrip", "TopologyCanvas", "InspectorDock", "RelationshipLegend"];
  if (root.children.length !== required.length || root.children.some((id, i) => parsed.elements[id]?.type !== required[i]))
    throw Error("Composition must retain the map-first workspace and coverage");

  const seen = new Set<string>();
  const visit = (id: string) => {
    const e = parsed.elements[id];
    if (!e || seen.has(id)) throw Error("Dangling, cyclic or multiply parented element");
    seen.add(id);
    if (e.type !== "WorkspaceFrame" && e.type !== "InspectorDock" && e.children.length)
      throw Error("Leaf components cannot replace the workspace structure");
    if (e.type === "InspectorDock" && e.children.some(child => !["EvidenceList", "CollectionQueue", "InvestigationBrief"].includes(parsed.elements[child]?.type)))
      throw Error("Unsupported inspector section");
    for (const [key, allowed] of [
      ["focusEntityId", context.entityIds], ["entityId", context.entityIds],
      ["receiptIds", context.receiptIds], ["collectionIds", context.collectionIds],
      ["investigationId", context.investigationIds],
    ] as const) {
      const value = e.props[key];
      for (const ref of value == null ? [] : Array.isArray(value) ? value : [value]) {
        if (typeof ref !== "string" || !allowed.has(ref)) throw Error(`Reference outside projection: ${key}`);
      }
    }
    e.children.forEach(visit);
  };
  visit(parsed.root);
  if (seen.size !== entries.length) throw Error("Unreachable elements are not accepted");
  return parsed;
}
