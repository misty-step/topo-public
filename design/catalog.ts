import { defineCatalog, defineSchema } from "@json-render/core";
import { z } from "zod";

// Standard flat-tree wire shape, with no generated state, expressions, events,
// arbitrary visibility, or built-in state-mutation actions. This does not select
// a UI framework; a future platform registry supplies component implementations.
export const schema = defineSchema((s) => ({
  spec: s.object({
    root: s.string(),
    elements: s.record(s.object({
      type: s.ref("catalog.components"),
      props: s.propsOf("catalog.components"),
      children: s.array(s.string()),
    })),
  }),
  catalog: s.object({
    components: s.map({
      props: s.zod(),
      slots: s.array(s.string()),
      description: s.string(),
    }),
    actions: s.map({ params: s.zod(), description: s.string() }),
  }),
}));

const id = z.string().min(1).max(256);
const empty = z.object({}).strict();
export const components = {
  WorkspaceFrame: {
    props: z.object({ projectionId: id }).strict(),
    slots: ["default"],
    description: "Fixed approved plum tiled workspace. Retain navigation, coverage, map, inspector and legend.",
  },
  ScopeNavigator: {
    props: empty,
    description: "Code resolves the authorized scope breadcrumb and return path from the projection.",
  },
  CoverageStrip: {
    props: empty,
    description: "Code displays source dates and complete/partial/denied/failed coverage. Never infer health.",
  },
  TopologyCanvas: {
    props: z.object({
      lens: z.enum(["dependencies", "consumers", "ownership"]),
      focusEntityId: id.nullable(),
    }).strict(),
    description: "Deterministic map of the supplied projection. Focus IDs must already be in the authorized projection.",
  },
  InspectorDock: {
    props: z.object({ entityId: id.nullable() }).strict(),
    slots: ["default"],
    description: "Fixed evidence dock. Content may choose supported evidence, collection or investigation sections.",
  },
  RelationshipLegend: {
    props: empty,
    description: "Code derives relation colors and evidence patterns from the same map revision.",
  },
  EvidenceList: {
    props: z.object({ receiptIds: z.array(id).max(32) }).strict(),
    description: "Display supplied authorized receipts. No invented assertions or raw HTML.",
  },
  CollectionQueue: {
    props: z.object({ collectionIds: z.array(id).max(32) }).strict(),
    description: "Read-only supplied collection records. Denied collection does not establish absence.",
  },
  InvestigationBrief: {
    props: z.object({ investigationId: id }).strict(),
    description: "Read-only existing bounded investigation. This component cannot launch an agent or execute a tool.",
  },
};

export const catalog = defineCatalog(schema, { components, actions: {} });
