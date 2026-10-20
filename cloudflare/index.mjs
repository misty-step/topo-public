import * as Sentry from "@sentry/cloudflare";
import site from "./build/index.js";
import manifest from "../.cache/site/manifest.json";

const source = manifest.source_revision;
const release = `topo-site@${source}`;
const categories = new Set(["public-health", "public-assets", "public-worker"]);
const lastSent = new Map();

function identity(env) {
  return {
    service: "topo-site",
    environment: env.SENTRY_ENVIRONMENT === "production" ? "production" : "verification",
    source,
    release,
    version: /^[a-f0-9-]{36}$/.test(env.CF_VERSION_METADATA?.id ?? "")
      ? env.CF_VERSION_METADATA.id : "local",
  };
}

const handler = {
  async fetch(request, env, ctx) {
    let category = "public-worker";
    try {
      if (["GET", "HEAD"].includes(request.method) && new URL(request.url).pathname === "/health") {
        category = "public-health";
        const response = await env.ASSETS.fetch("https://topo.mistystep.io/manifest.json");
        if (response.status !== 200 || (await response.json()).source_revision !== source) {
          throw new Error("Public manifest unavailable");
        }
        return new Response(request.method === "HEAD" ? null : JSON.stringify({ status: "ok", ...identity(env) }), {
          headers: { "Content-Type": "application/json", "Cache-Control": "no-store" },
        });
      }
      // worker-build exports a proxied WorkerEntrypoint class, not a fetch object.
      // Compose it instead of stacking Sentry's non-configurable fields on its proxy.
      category = "public-assets";
      const response = await new site(ctx, env).fetch(request);
      if (response.status >= 500) throw new Error("Public assets unavailable");
      return response;
    } catch {
      // Never forward exception text: even a public request can contain private data.
      Sentry.captureMessage(`Topo ${category} unavailable`, { level: "error", tags: { category } });
      return new Response("Public site temporarily unavailable", { status: 503, headers: { "Cache-Control": "no-store" } });
    }
  },
};

export default Sentry.withSentry(env => ({
  dsn: env.SENTRY_DSN,
  environment: identity(env).environment,
  release,
  cacheClient: false,
  defaultIntegrations: false,
  // Veto even an incoming sampled trace; a rate of zero can inherit its decision.
  tracesSampler: () => 0,
  tracePropagationTargets: [],
  enableLogs: false,
  beforeSendMetric: () => null,
  sendDefaultPii: false,
  sendClientReports: false,
  maxBreadcrumbs: 0,
  transportOptions: { bufferSize: 1 },
  dataCollection: { userInfo: false, cookies: false, httpHeaders: false, httpBodies: [], urlQueryParams: false },
  beforeSend(event) {
    const category = categories.has(event.tags?.category) ? event.tags.category : "public-worker";
    const now = Date.now();
    if (now - (lastSent.get(category) ?? -Infinity) < 60_000) return null;
    lastSent.set(category, now);
    // Rebuild, rather than redact, the complete event. No request, stack or context survives.
    return {
      event_id: event.event_id,
      timestamp: event.timestamp,
      platform: "javascript",
      level: "error",
      message: `Topo ${category} unavailable`,
      fingerprint: ["topo-site", category],
      environment: identity(env).environment,
      release,
      // Relay otherwise infers the sender IP for JavaScript events. This fixed
      // non-identifying address suppresses that inference without a provider policy change.
      user: { ip_address: "0.0.0.0" },
      tags: { category, service: "topo-site", source, provider_version: identity(env).version },
    };
  },
}), handler);
