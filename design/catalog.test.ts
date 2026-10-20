import { describe, expect, test } from "bun:test";
import { admit, type ProjectionContext } from "./admit.ts";
import { catalog } from "./catalog.ts";

const entity = "module:example/beacon/billing";
function context(scope: "module" | "repository" | "organization"): ProjectionContext {
  return {
    projectionId: `projection:${scope}:example@7`, revision: 7,
    entityIds: new Set([entity, "service:ledger", "provider:postgres"]),
    receiptIds: new Set(["receipt:fixture"]),
    collectionIds: new Set(["collection:fixture"]),
    investigationIds: new Set(["investigation:fixture"]),
  };
}
function workspace(c: ProjectionContext) {
  return {
    root: "workspace",
    elements: {
      workspace: { type: "WorkspaceFrame", props: { projectionId: c.projectionId }, children: ["nav", "coverage", "graph", "dock", "legend"] },
      nav: { type: "ScopeNavigator", props: {}, children: [] },
      coverage: { type: "CoverageStrip", props: {}, children: [] },
      graph: { type: "TopologyCanvas", props: { lens: "dependencies", focusEntityId: entity }, children: [] },
      dock: { type: "InspectorDock", props: { entityId: entity }, children: ["evidence"] },
      evidence: { type: "EvidenceList", props: { receiptIds: ["receipt:fixture"] }, children: [] },
      legend: { type: "RelationshipLegend", props: {}, children: [] },
    },
  };
}

describe("json-render admission at the Topo composition boundary", () => {
  test("accepts an authorized map-first composition through the actual official SDK", () => {
    const c = context("repository");
    const spec = workspace(c);
    expect(catalog.validate(spec).success).toBe(true);
    expect(() => admit(spec, c)).not.toThrow();
  });
  test("rejects an otherwise schema-valid composition that removes the map", () => {
    const c = context("repository"); const s = workspace(c);
    s.elements.workspace.children = ["nav", "coverage", "dock", "legend"];
    expect(catalog.validate(s).success).toBe(true);
    expect(() => admit(s, c)).toThrow("map-first");
  });
  test("rejects an entity outside the trusted authorized projection", () => {
    const c = context("module"); const s = workspace(c);
    s.elements.graph.props.focusEntityId = "module:another-client/private";
    expect(() => admit(s, c)).toThrow("outside projection");
  });
  test("rejects invented evidence even when the entity is permitted", () => {
    const c = context("module"); const s = workspace(c);
    s.elements.evidence.props.receiptIds = ["receipt:invented-restore"];
    expect(() => admit(s, c)).toThrow("outside projection");
  });
  test("rejects generated styles and state-write events rather than stripping them", () => {
    const c = context("repository"); const s = workspace(c);
    for (const bad of [
      { ...s, state: { graph: "replacement" } },
      { ...s, elements: { ...s.elements, graph: { ...s.elements.graph, on: { press: { action: "setState" } } } } },
      { ...s, elements: { ...s.elements, graph: { ...s.elements.graph, props: { ...s.elements.graph.props, style: { background: "white" } } } } },
    ]) expect(() => admit(bad, c)).toThrow();
  });
  test("rejects a mismatched projection instead of accepting stale composition", () => {
    const c = context("module"); const s = workspace(c);
    s.elements.workspace.props.projectionId = "projection:module:example@6";
    expect(() => admit(s, c)).toThrow("supplied authorized projection");
  });
});
