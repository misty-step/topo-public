// Exercise the shipped JS/Wasm modules in workerd; only upstream assets/sink fail.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFile, writeFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createHash } from "node:crypto";
const require = createRequire(import.meta.url);
const { Miniflare, Response: MFResponse } = createRequire(require.resolve("wrangler/package.json"))("miniflare");
const manifest = JSON.parse(await readFile(".cache/site/manifest.json", "utf8"));
const version = "00000000-1111-2222-3333-444444444444";
const canary = "TOPO_PRIVATE_CANARY_not_an_inventory";
const modulePath = resolve(".cache/worker/index.js");
const digest = createHash("sha256").update(await readFile(modulePath)).digest("hex");
const proof = { moduleSha256: digest, source: manifest.source_revision, cases: [] };
const modules = Object.fromEntries(await Promise.all((await readdir(".cache/worker"))
  .filter(name => name.endsWith(".js") || name.endsWith(".wasm"))
  .map(async name => [name, { type: name.endsWith(".wasm") ? "wasm" : "esm", contents: await readFile(resolve(".cache/worker", name)) }])));

async function exercise(sink = "ok", dsn = "https://public@sentry.invalid/1") {
  let fault = "none";
  const envelopes = [];
  let attempts = 0;
  let leaked = false;
  const mf = new Miniflare({
    workers: [{ config: {
      name: "topo-public-proof",
      manifest: { mainModule: "index.js", modules },
      compatibilityDate: "2026-10-09", compatibilityFlags: ["nodejs_compat"],
      env: {
        SENTRY_DSN: { type: "text", value: dsn },
        SENTRY_ENVIRONMENT: { type: "text", value: "verification" },
        CF_VERSION_METADATA: { type: "json", value: { id: version } },
        ASSETS: { type: "fetcher", handler: async request => {
        if (fault === "throw") throw new Error(canary);
        if (fault === "500") return new MFResponse(canary, { status: 500 });
        if (fault === "manifest") return MFResponse.json({ source_revision: canary });
        const route = new URL(request.url).pathname;
        if (route === "/manifest.json") return MFResponse.json(manifest);
        if (route === "/") return new MFResponse(await readFile(".cache/site/index.html"));
        return new MFResponse("Not found", { status: 404 });
        } },
      },
    }, dev: { outboundService: { type: "fetcher", handler: async request => {
      assert.equal(new URL(request.url).hostname, "sentry.invalid");
      attempts++;
      const body = await request.text();
      leaked ||= body.includes(canary);
      envelopes.push(body);
      if (sink === "hang") await new Promise(r => setTimeout(r, 4000));
      return new MFResponse("{}", { status: sink === "reject" ? 503 : 200 });
    } } } }],
  });
  async function get(route = "/health", method = "GET") {
    const start = performance.now();
    const response = await mf.dispatchFetch(`https://topo.invalid${route}?private=${canary}`, {
      method,
      headers: {
        Authorization: `Bearer ${canary}`, Cookie: `session=${canary}`,
        "User-Agent": canary, "sentry-trace": "12345678901234567890123456789012-1234567890123456-1",
        baggage: `sentry-transaction=${canary},sentry-user_id=${canary}`,
      },
      ...(method === "POST" ? { body: canary } : {}),
    });
    const body = await response.text();
    assert(performance.now() - start < 1500, "Telemetry must not block public responses");
    return { response, body };
  }
  try {
    await mf.ready;
    let result = await get();
    assert.equal(result.response.status, 200, result.body);
    assert.equal(result.response.headers.get("Cache-Control"), "no-store");
    assert.deepEqual(JSON.parse(result.body), {
      status: "ok", service: "topo-site", environment: "verification",
      source: manifest.source_revision, release: `topo-site@${manifest.source_revision}`, version,
    });
    assert.equal((await get("/health", "HEAD")).body, "");
    assert.equal((await get("/health", "POST")).response.status, 405);
    assert.equal((await get("/missing")).response.status, 404);
    assert.equal((await get("/debug-sentry")).response.status, 404);
    assert.equal(attempts, 0);
    fault = "manifest";
    assert.equal((await get()).response.status, 503);
    // Same-category floods remain one event per minute/isolate.
    for (let i = 0; i < 3; i++) assert.equal((await get()).response.status, 503);
    fault = "throw";
    assert.equal((await get("/")).response.status, 503);
    fault = "500";
    const unavailable = await get("/");
    assert.equal(unavailable.response.status, 503);
    assert(!unavailable.body.includes(canary));
    fault = "none";
    assert.equal((await get("/")).response.status, 200);
    assert.equal((await get()).response.status, 200);
    // Let native waitUntil/flush complete; no application delivery loop or retry.
    await new Promise(r => setTimeout(r, sink === "hang" ? 4500 : 500));
    assert.equal(leaked, false, "Private request/exception/baggage must not reach the wire");
    if (dsn.includes("sentry.invalid")) {
      assert.equal(attempts, 2);
      const events = envelopes.flatMap(body => body.split("\n").map(line => {
        try { return JSON.parse(line); } catch { return {}; }
      })).filter(item => item.platform === "javascript");
      assert.equal(events.length, 2);
      assert.deepEqual(events.map(event => event.tags.category).sort(), ["public-assets", "public-health"]);
      for (const event of events) {
        assert.equal(event.release, `topo-site@${manifest.source_revision}`);
        assert.equal(event.environment, "verification");
        assert.equal(event.tags.provider_version, version);
        assert.equal(event.sdk.name, "sentry.javascript.cloudflare");
        assert.equal(event.sdk.version, "11.8.0");
        assert.deepEqual(event.user, { ip_address: "0.0.0.0" });
        assert.deepEqual(Object.keys(event).sort(), ["environment", "event_id", "fingerprint", "level", "message", "platform", "release", "sdk", "tags", "timestamp", "user"].sort());
      }
    } else assert.equal(attempts, 0);
    proof.cases.push({ sink, configured: !!dsn, attempts, publicRecovery: 200, rejectedPost: 405, privateCanaryAbsent: true });
  } finally {
    await mf.dispose();
  }
}
await exercise();
await exercise("reject");
await exercise("hang");
await exercise("disabled", "");
await exercise("invalid", "not-a-dsn");
assert.equal(createHash("sha256").update(await readFile(modulePath)).digest("hex"), digest);
await writeFile(".cache/public-errors-proof.json", JSON.stringify(proof, null, 2) + "\n");
console.log(JSON.stringify(proof));
