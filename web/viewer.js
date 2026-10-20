(() => {
'use strict';

/* ---------- vocabulary ---------- */
const VIEWS = ['map', 'inventory', 'directions', 'evidence'];
const TYPES = ['hosting', 'data', 'auth', 'inference', 'communication', 'monitoring', 'delivery', 'dns', 'imports', 'declares', 'other'];
const KINDS = ['product', 'provider', 'tool', 'domain'];
const KIND_PL = { product: 'Focus', provider: 'Dependencies', tool: 'Context', domain: 'Boundaries' };
const KIND_ONE = { product: 'Focal entity', provider: 'Dependency', tool: 'Context', domain: 'Boundary' };
const STATUSES = ['live', 'configured', 'historical', 'unknown'];
const STATUS_ONE = { live: 'Live', configured: 'Configured', historical: 'Historical', unknown: 'Unknown' };
const CATS = ['compute', 'data', 'identity', 'communication', 'delivery', 'observability', 'ai', 'business', 'domain', 'code', 'packages', 'stdlib', 'context'];
const CAT_ONE = { compute: 'compute', data: 'data', identity: 'identity', communication: 'communication', delivery: 'delivery', observability: 'observability', ai: 'AI', business: 'business', domain: 'domain' };
const CAT_TITLE = { compute: 'Compute', data: 'Data', identity: 'Identity', communication: 'Communication', delivery: 'Delivery', observability: 'Observability', ai: 'AI', business: 'Business', domain: 'Domain' };
const LEVEL_RANK = { api: 0, cli: 1, dns: 2, config: 3, pricing: 4, other: 5 };
/* Evidence tiers by how a link was read: live = positive provider proof from a successful API, CLI or DNS read (metadata, not traffic or health);
   config = documentary: repository source or config, docs, pricing, or a record of a failed or unattributed probe; inference = reasoned. */
const TIERS = ['live', 'config', 'inference'];
const TIER_ONE = { live: 'provider read', config: 'documentary', inference: 'inferred' };
const TIER_TITLE = { live: 'Positive provider proof: a successful API, CLI or DNS read of account or repository metadata; not traffic or health', config: 'Documentary: repository source or config, docs, pricing, or a record of a failed or unattributed probe; not positive provider proof', inference: 'Inferred: reasoned from indirect evidence, not stated in a source' };
/* Bands: left = reach and operations, right = runtime and what watches it. Each band borrows the hue of the relationship that lands in it. */
const BANDS = {
  domains: { label: 'Domains', side: 'L', tone: 'dns' },
  domain: { label: 'Domain services', side: 'L', tone: 'dns' },
  delivery: { label: 'Delivery', side: 'L', tone: 'delivery' },
  communication: { label: 'Communication', side: 'L', tone: 'communication' },
  business: { label: 'Business', side: 'L', tone: 'business' },
  compute: { label: 'Compute', side: 'R', tone: 'hosting' },
  data: { label: 'Data', side: 'R', tone: 'data' },
  identity: { label: 'Identity', side: 'R', tone: 'auth' },
  observability: { label: 'Observability', side: 'R', tone: 'monitoring' },
  ai: { label: 'AI', side: 'R', tone: 'inference' },
};
Object.assign(CAT_ONE, {code:'source code', packages:'packages', stdlib:'standard library', context:'context'});
Object.assign(CAT_TITLE, {code:'Source code', packages:'Packages', stdlib:'Standard libraries', context:'Context'});
Object.assign(BANDS, {code:{label:'Source boundaries', side:'R', tone:'imports'},packages:{label:'Packages',side:'R',tone:'imports'},stdlib:{label:'Standard libraries',side:'R',tone:'imports'},context:{label:'Repository context',side:'L',tone:'delivery'}});
const BAND_ORDER = Object.keys(BANDS);
const GEO = { top: 40, bandHead: 20, nodeGap: 5, bandGap: 14, prodGap: 10, colGap: 150, pad: 18, bottom: 22 };

/* ---------- helpers ---------- */
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
const arr = (x) => (Array.isArray(x) ? x : []);
const str = (x) => (typeof x === 'string' ? x : x == null ? '' : String(x));
const strs = (x) => arr(x).map((v) => str(v).trim()).filter(Boolean);
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
const isUnknown = (s) => !s || /^unknown\b/i.test(String(s).trim());
const byLabel = (a, b) => a.label.localeCompare(b.label, 'en', { sensitivity: 'base' });
const textW = (s, px) => s.length * px * 0.6;
const plural = (n, one, many) => `${n} ${n === 1 ? one : many || one + 's'}`;

function h(tag, a, ...kids) {
  const el = document.createElement(tag);
  if (a) for (const k of Object.keys(a)) {
    const v = a[k];
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'text') el.textContent = v;
    else if (k === 'vars') for (const [p, val] of Object.entries(v)) el.style.setProperty(p, val);
    else if (k === 'css') el.style.cssText = v;
    else if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2), v);
    else el.setAttribute(k, v === true ? '' : String(v));
  }
  for (const c of kids.flat(Infinity)) if (c != null && c !== false) el.append(c.nodeType ? c : String(c));
  return el;
}
function sv(tag, a) {
  const el = document.createElementNS('http://www.w3.org/2000/svg', tag);
  if (a) for (const k of Object.keys(a)) if (a[k] != null) el.setAttribute(k, String(a[k]));
  return el;
}
function safeUrl(s) {
  try { const u = new URL(str(s)); return u.protocol === 'https:' || u.protocol === 'http:' ? u.href : ''; } catch (e) { return ''; }
}
const sw = (tone, tier) => h('i', { class: `sw t-${tone}${tier === 'config' ? ' dot' : tier === 'inference' ? ' dash' : ''}`, 'aria-hidden': 'true' });
/* Scroll one pane so an element shows below the pane's sticky header (inset). Unlike scrollIntoView,
   this never scrolls ancestors or the window, so the bar and toolbars stay in place. */
function scrollWithin(box, el, mode, inset) {
  const r = el.getBoundingClientRect();
  const top = r.top - box.getBoundingClientRect().top - box.clientTop - inset;
  const view = box.clientHeight - inset;
  if (mode === 'start') box.scrollTop += top;
  else if (mode === 'center') box.scrollTop += top - (view - r.height) / 2;
  else if (top < 0) box.scrollTop += top;
  else if (top + r.height > view) box.scrollTop += Math.min(top, top + r.height - view);
}

/* ---------- data ---------- */
function load() {
  const el = document.getElementById('infra-data');
  const text = el ? el.textContent.trim() : '';
  let raw = {};
  let error = null;
  if (text && !text.startsWith('/*')) {
    try { raw = JSON.parse(text) || {}; } catch (e) { error = String((e && e.message) || e); }
  }
  const warn = [];
  const meta = raw.meta && typeof raw.meta === 'object' ? raw.meta : {};
  const M = { date: str(meta.date), coverage: str(meta.coverage), limits: strs(meta.limits), title:str(meta.title), scopeId:str(meta.scope_id), scopes:arr(meta.scopes), revision:str(meta.graph_revision) };

  const nodes = [];
  const byId = new Map();
  arr(raw.nodes).forEach((r) => {
    if (!r || typeof r !== 'object') return;
    const id = str(r.id).trim();
    if (!id || byId.has(id)) { warn.push(`node id "${id}" is empty or duplicated`); return; }
    if (!KINDS.includes(r.kind)) warn.push(`node ${id} has kind "${str(r.kind)}"`);
    const kind = KINDS.includes(r.kind) ? r.kind : 'provider';
    if (!CATS.includes(r.category)) warn.push(`node ${id} has category "${str(r.category)}"`);
    const category = CATS.includes(r.category) ? r.category : kind === 'domain' ? 'domain' : 'business';
    if (!STATUSES.includes(r.status)) warn.push(`node ${id} has status "${str(r.status)}"`);
    const n = {
      id, kind, category, entityKind:str(r.entity_kind)||KIND_ONE[kind], members:strs(r.members), scopeLink:/^scope-[a-f0-9]{16}\.html$/.test(r.scope_link)?r.scope_link:null,
      label: str(r.label).trim() || id,
      status: STATUSES.includes(r.status) ? r.status : 'unknown',
      summary: str(r.summary).trim(),
      repos: strs(r.repos),
      owner: str(r.owner).trim() || 'Unknown',
      cost: str(r.cost).trim() || 'Unknown',
      strengths: strs(r.strengths),
      weaknesses: strs(r.weaknesses),
      sources: strs(r.sources),
      details: strs(r.details),
      idx: nodes.length,
      links: [],
    };
    nodes.push(n);
    byId.set(id, n);
  });

  const edges = [];
  arr(raw.edges).forEach((r, i) => {
    if (!r || typeof r !== 'object') return;
    const from = str(r.from).trim();
    const to = str(r.to).trim();
    if (!byId.has(from) || !byId.has(to)) { warn.push(`relationship ${from} to ${to} names a missing node`); return; }
    if (!TYPES.includes(r.type)) warn.push(`relationship ${from} to ${to} has type "${str(r.type)}"`);
    if (r.evidence !== 'read' && r.evidence !== 'inference') warn.push(`relationship ${from} to ${to} has evidence "${str(r.evidence)}"`);
    edges.push({
      id: 'e' + i, from, to,
      type: TYPES.includes(r.type) ? r.type : 'other',
      label: str(r.label).trim(),
      evidence: r.evidence === 'read' ? 'read' : 'inference',
      sources: strs(r.sources), forcedTier:TIERS.includes(r.tier_override)?r.tier_override:null,
    });
  });

  const sources = [];
  const srcById = new Map();
  arr(raw.sources).forEach((r) => {
    if (!r || typeof r !== 'object') return;
    const id = str(r.id).trim();
    if (!id || srcById.has(id)) { warn.push(`source id "${id}" is empty or duplicated`); return; }
    const level = str(r.level).trim();
    const s = { id, label: str(r.label).trim() || id, result: str(r.result).trim(), date: str(r.date).trim(), url: safeUrl(r.url), level: level in LEVEL_RANK ? level : 'other', idx: sources.length };
    sources.push(s);
    srcById.set(id, s);
  });

  const directions = arr(raw.directions).filter((r) => r && typeof r === 'object').map((r, i) => ({
    id: str(r.id).trim() || 'direction-' + (i + 1),
    rank: Number.isFinite(Number(r.rank)) ? Number(r.rank) : i + 1,
    name: str(r.name).trim(), stance: str(r.stance).trim(), decision: str(r.decision).trim(),
    moves: strs(r.moves), cost: str(r.cost).trim(), risk: strs(r.risk), givesUp: strs(r.givesUp),
    ergonomics: strs(r.ergonomics), prerequisites: strs(r.prerequisites), sources: strs(r.sources),
  })).sort((a, b) => a.rank - b.rank);
  const alternatives = arr(raw.alternatives).filter((r) => r && typeof r === 'object').map((r) => ({
    name: str(r.name).trim(), why: str(r.why).trim(), tradeoff: str(r.tradeoff).trim(), use: str(r.use).trim(), source: safeUrl(r.source), sourceText: str(r.source).trim(), sources: strs(r.sources),
  }));
  const findings = arr(raw.findings).filter((r) => r && typeof r === 'object').map((r) => ({ title: str(r.title).trim(), detail: str(r.detail).trim(), evidence: strs(r.evidence) }));
  const gaps = arr(raw.gaps).filter((r) => r && typeof r === 'object').map((r) => ({ area: str(r.area).trim(), detail: str(r.detail).trim(), needed: str(r.needed).trim() }));

  for (const e of edges) {
    e.tier = e.forcedTier || (e.evidence === 'inference' ? 'inference' : e.sources.some((id) => ['api', 'cli', 'dns'].includes((srcById.get(id) || {}).level)) ? 'live' : 'config');
  }
  for (const e of edges) {
    const a = byId.get(e.from);
    const b = byId.get(e.to);
    a.links.push({ e, other: b, dir: 'out' });
    if (a !== b) b.links.push({ e, other: a, dir: 'in' });
  }
  for (const n of nodes) {
    const ps = new Set();
    const qs = new Set();
    for (const l of n.links) (l.other.kind === 'product' ? ps : qs).add(l.other.id);
    n.productCount = ps.size;
    n.otherCount = qs.size;
    n.indirect = n.kind === 'tool' && n.productCount === 0;
    n.levels = [...new Set(n.sources.map((id) => (srcById.get(id) || {}).level).filter(Boolean))].sort((a, b) => LEVEL_RANK[a] - LEVEL_RANK[b]);
  }

  const cited = new Map(sources.map((s) => [s.id, { nodes: [], edges: [], findings: [], directions: [], alternatives: [] }]));
  const cite = (id, bucket, obj, where) => {
    const c = cited.get(id);
    if (!c) { warn.push(`${where} cites missing source "${id}"`); return; }
    c[bucket].push(obj);
  };
  nodes.forEach((n) => n.sources.forEach((id) => cite(id, 'nodes', n, `node ${n.id}`)));
  edges.forEach((e) => e.sources.forEach((id) => cite(id, 'edges', e, `relationship ${e.from} to ${e.to}`)));
  findings.forEach((f) => f.evidence.forEach((id) => cite(id, 'findings', f, `finding "${f.title}"`)));
  directions.forEach((d) => d.sources.forEach((id) => cite(id, 'directions', d, `direction ${d.id}`)));
  alternatives.forEach((a) => a.sources.forEach((id) => cite(id, 'alternatives', a, `alternative "${a.name}"`)));

  return { meta: M, nodes, byId, edges, sources, srcById, directions, alternatives, findings, gaps, cited, warn, error, empty: nodes.length === 0 };
}

const D = load();
const TYPES_IN = [...TYPES, 'other'].filter((t) => D.edges.some((e) => e.type === t));
const KINDS_IN = KINDS.filter((k) => D.nodes.some((n) => n.kind === k));
const STATUSES_IN = STATUSES.filter((s) => D.nodes.some((n) => n.status === s));
const count = (list, f) => list.reduce((a, x) => a + (f(x) ? 1 : 0), 0);
function edgeLevel(e) {
  let best = null;
  for (const id of e.sources) { const s = D.srcById.get(id); if (s && (best == null || LEVEL_RANK[s.level] < LEVEL_RANK[best])) best = s.level; }
  return best;
}
/* One phrase per link for map tooltips, detail rows and the inventory, so the three always agree. */
function evidenceText(e) {
  const lvl = edgeLevel(e) || 'uncited';
  return e.tier === 'inference' ? 'inferred' : e.tier === 'live' ? `read via ${lvl}` : `documentary (${lvl})`;
}
function bandKey(n) {
  if (n.kind === 'product') return null;
  if (n.kind === 'domain') return 'domains';
  return BANDS[n.category] ? n.category : 'business';
}
D.nodes.forEach((n) => {
  n.hay = [n.label, n.id, n.kind, n.category, n.status, n.summary, n.owner, n.cost, n.repos.join(' '), n.strengths.join(' '), n.weaknesses.join(' '), n.details.join(' '), n.sources.join(' ')].join(' ').toLowerCase();
  n.mhay = [n.label, n.id, n.entityKind, CAT_ONE[n.category], n.owner, n.repos.join(' ')].join(' ').toLowerCase();
});

/* Node names inside prose become links to the map. */
const LABEL_TO_ID = new Map();
D.nodes.forEach((n) => { if (n.label.length >= 3 && !LABEL_TO_ID.has(n.label)) LABEL_TO_ID.set(n.label, n.id); });
const LABEL_RE = LABEL_TO_ID.size
  ? new RegExp('(?<![\\w.@/-])(' + [...LABEL_TO_ID.keys()].sort((a, b) => b.length - a.length).map((s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|') + ')(?![\\w-]|\\.\\w)', 'g')
  : null;
function rich(text) {
  if (!text) return [];
  if (!LABEL_RE) return [text];
  const out = [];
  let last = 0;
  for (const m of text.matchAll(LABEL_RE)) {
    if (m.index > last) out.push(text.slice(last, m.index));
    out.push(h('button', { type: 'button', class: 'ref', 'data-goto': LABEL_TO_ID.get(m[1]), title: 'Show on map', text: m[1] }));
    last = m.index + m[1].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

/* ---------- state ---------- */
const S = {
  view: 'map',
  selected: null,
  hover: null,
  focus: false,
  kinds: new Set(KINDS_IN),
  statuses: new Set(STATUSES_IN),
  cat: 'all',
  types: new Set(TYPES_IN),
  evidence: new Set(TIERS),
  query: '',
  scale: null,
  inv: { tab: 'nodes', q: '', kind: 'all', status: 'all', cat: 'all', type: 'all', evidence: 'all', sort: { nodes: ['label', 1], edges: ['from', 1] } },
  srcQ: '',
  srcLevel: 'all',
  flash: null,
};
const DEFAULT_KINDS = new Set(KINDS_IN);
/* A crowded default map shrinks labels below reading size. Long-tail nodes start hidden, stay one chip away, and are always in Inventory:
   domains when there are many, and tools that link only to other providers rather than to a product. */
if (count(D.nodes, (n) => n.kind === 'domain') > 14) DEFAULT_KINDS.delete('domain');
const INDIRECT = count(D.nodes, (n) => n.indirect);
const DEFAULT_LONGTAIL = true;
S.kinds = new Set(DEFAULT_KINDS);
S.longTail = DEFAULT_LONGTAIL;

/* ---------- elements ---------- */
const E = {
  desk: $('#desk'), barState: $('#bar-state'), keys: $('#keys'), keysBtn: $('#keys-btn'),
  vp: $('#map-vp'), world: $('#map-world'), scale: $('#map-scale'), svg: $('#map-edges'), bands: $('#map-bands'), nodes: $('#map-nodes'), mapEmpty: $('#map-empty'),
  q: $('#map-q'), qCount: $('#map-q-count'), kindChips: $('#kind-chips'), statusChips: $('#status-chips'), catSel: $('#cat-sel'), focusBtn: $('#focus-btn'), legend: $('#legend'),
  detail: $('#detail'), dHead: $('#detail-head'), dBody: $('#detail-body'),
  invQ: $('#inv-q'), invWrap: $('#inv-table-wrap'), invCount: $('#inv-count'), invFilters: $('#inv-filters'),
  dirDoc: $('#dir-doc'), evDoc: $('#ev-doc'), live: $('#live'),
};
let LAY = null;
let VIS = { nodes: [], edges: [] };
let mapKey = '';
let curScale = 1;
let cursorId = null;
let lastDetail = '';
let dirBuilt = false;
let evBuilt = false;

function announce(msg) {
  E.live.textContent = '';
  window.setTimeout(() => { E.live.textContent = msg; }, 40);
}

/* ---------- map layout ---------- */
const nodeBaseH = (n) => (n.kind === 'product' ? 32 : n.kind === 'domain' ? 22 : 26);

function pav(v) {
  const blocks = [];
  for (const x of v) {
    blocks.push({ sum: x, n: 1 });
    while (blocks.length > 1) {
      const b = blocks[blocks.length - 1];
      const a = blocks[blocks.length - 2];
      if (a.sum / a.n <= b.sum / b.n) break;
      a.sum += b.sum; a.n += b.n; blocks.pop();
    }
  }
  const out = [];
  for (const b of blocks) for (let k = 0; k < b.n; k++) out.push(b.sum / b.n);
  return out;
}

function wrapLines(text, capacity) {
  const cap = Math.max(1, Math.floor(capacity));
  let lines = 1;
  let cur = 0;
  for (const word of text.split(/\s+/).filter(Boolean)) {
    if (cur && cur + 1 + word.length <= cap) { cur += 1 + word.length; continue; }
    if (!cur && word.length <= cap) { cur = word.length; continue; }
    if (cur) { lines += 1; cur = 0; }
    lines += Math.ceil(word.length / cap) - 1;
    cur = word.length % cap || cap;
  }
  return lines;
}

function layoutGraph(nodes, edges, focusId) {
  const I = new Map();
  const nb = new Map(nodes.map((n) => [n.id, []]));
  for (const e of edges) { nb.get(e.from).push(e.to); nb.get(e.to).push(e.from); }
  const subs = new Map();
  if (focusId) for (const e of edges) {
    const other = e.from === focusId ? e.to : e.to === focusId ? e.from : null;
    if (!other || other === focusId) continue;
    if (!subs.has(other)) subs.set(other, []);
    subs.get(other).push(e);
  }
  const bands = new Map();
  const P = [];
  for (const n of nodes) {
    const band = bandKey(n);
    const col = band ? BANDS[band].side : 'C';
    const lines = (subs.get(n.id) || []).slice().sort((a, b) => TYPES.indexOf(a.type) - TYPES.indexOf(b.type));
    const sub = lines.slice(0, 5);
    const fs = n.kind === 'product' ? 13 : 12;
    let need = textW(n.label, fs) + 16 + 24;
    for (const e of sub) need = Math.max(need, textW(e.label || e.type, 11) + 40);
    const it = { n, id: n.id, col, band, fs, need, sub, more: lines.length - sub.length, x: 0, y: 0, w: 0, h: 0, labelH: 0, order: n.idx };
    I.set(n.id, it);
    if (col === 'C') P.push(it);
    else { if (!bands.has(band)) bands.set(band, []); bands.get(band).push(it); }
  }
  const colW = { L: 0, C: 0, R: 0 };
  for (const it of I.values()) colW[it.col] = Math.max(colW[it.col], clamp(it.need, it.col === 'C' ? 150 : 136, focusId ? 270 : it.col === 'C' ? 264 : 248));
  /* Long labels wrap rather than truncate; a monospace face makes the line count predictable. */
  for (const it of I.values()) {
    it.w = colW[it.col];
    const pad = (it.n.kind === 'product' ? 11 : it.n.kind === 'domain' ? 9 : 12) + 9 + 4;
    it.labelH = nodeBaseH(it.n) + (wrapLines(it.n.label, (it.w - pad - 16) / (it.fs * 0.6)) - 1) * it.fs * 1.25;
    const subCap = (it.w - pad - 16) / (11 * 0.6);
    const subH = it.sub.length ? 6 + it.sub.reduce((a, e) => a + wrapLines(e.label || e.type, subCap) * 14 + 1, 0) + (it.more > 0 ? 15 : 0) : 0;
    it.h = Math.ceil(it.labelH + subH);
  }

  const side = { L: [], R: [] };
  for (const key of BAND_ORDER) if (bands.has(key)) side[BANDS[key].side].push({ key, items: bands.get(key).sort((a, b) => byLabel(a.n, b.n)), y: 0 });
  const cy = (it) => it.y + it.h / 2;
  const stack = (s) => {
    let y = GEO.top;
    for (const b of side[s]) {
      b.y = y;
      y += GEO.bandHead;
      for (const it of b.items) { it.y = y; y += it.h + GEO.nodeGap; }
      y += GEO.bandGap - GEO.nodeGap;
    }
    return y;
  };
  const placeProducts = () => {
    const ideal = new Map();
    for (const p of P) {
      const ys = nb.get(p.id).map((id) => I.get(id)).filter((o) => o && o.col !== 'C').map(cy);
      ideal.set(p.id, ys.length ? ys.reduce((a, b) => a + b, 0) / ys.length : NaN);
    }
    P.sort((a, b) => {
      const ia = ideal.get(a.id);
      const ib = ideal.get(b.id);
      if (Number.isNaN(ia) || Number.isNaN(ib)) return Number.isNaN(ia) - Number.isNaN(ib) || a.order - b.order;
      return ia - ib || a.order - b.order;
    });
    const c = [];
    let acc = 0;
    for (const p of P) { c.push(acc); acc += p.h + GEO.prodGap; }
    const target = P.map((p, i) => ideal.get(p.id) - p.h / 2 - c[i]);
    for (let i = 0; i < target.length; i++) if (Number.isNaN(target[i])) target[i] = i ? target[i - 1] : GEO.top;
    const z = pav(target);
    let minY = Infinity;
    P.forEach((p, i) => { p.y = z[i] + c[i]; minY = Math.min(minY, p.y); });
    if (minY < GEO.top) P.forEach((p) => { p.y += GEO.top - minY; });
  };
  const reorder = () => {
    for (const s of ['L', 'R']) for (const b of side[s]) {
      const key = new Map();
      b.items.forEach((it, i) => {
        const ns = nb.get(it.id).map((id) => I.get(id)).filter(Boolean);
        const pref = ns.filter((o) => o.col === 'C');
        const use = pref.length ? pref : ns.filter((o) => o.band !== it.band);
        key.set(it.id, use.length ? use.reduce((a, o) => a + cy(o), 0) / use.length : 1e9 + i);
      });
      b.items.sort((a, c) => key.get(a.id) - key.get(c.id) || byLabel(a.n, c.n));
    }
  };
  for (let k = 0; k < 6; k++) { stack('L'); stack('R'); placeProducts(); reorder(); }
  const hL = stack('L');
  const hR = stack('R');
  placeProducts();
  const hC = P.reduce((m, p) => Math.max(m, p.y + p.h + GEO.prodGap), GEO.top);
  const H = Math.max(hL, hR, hC) + GEO.bottom;

  const segs = [];
  for (const e of edges) {
    const a = I.get(e.from);
    const b = I.get(e.to);
    if (!a || !b || a === b) continue;
    let kind;
    if (a.col === b.col) kind = a.col === 'C' ? 'arcC' : 'arc';
    else if (a.col === 'C' || b.col === 'C') kind = 'span';
    else kind = 'cross';
    const portSide = (me, other) => {
      if (kind === 'arc') return me.col === 'L' ? 'l' : 'r';
      if (kind === 'arcC') return 'r';
      if (me.col === 'C') return other.col === 'L' ? 'l' : 'r';
      return me.col === 'L' ? 'r' : 'l';
    };
    segs.push({ e, a, b, kind, sa: portSide(a, b), sb: portSide(b, a), ya: 0, yb: 0 });
  }
  const ports = new Map();
  const addPort = (it, s, seg, end, oy) => {
    const k = it.id + '|' + s;
    if (!ports.has(k)) ports.set(k, []);
    ports.get(k).push({ seg, end, oy, it });
  };
  for (const s of segs) { addPort(s.a, s.sa, s, 'a', cy(s.b)); addPort(s.b, s.sb, s, 'b', cy(s.a)); }
  for (const list of ports.values()) {
    list.sort((p, q) => p.oy - q.oy);
    const it = list[0].it;
    const n = list.length;
    const usable = Math.max(0, Math.min(it.h, 34) - 10);
    const step = n > 1 ? Math.min(6, usable / (n - 1)) : 0;
    const mid = it.y + (it.sub.length ? 3 + it.labelH / 2 : it.h / 2);
    list.forEach((p, i) => { p.seg[p.end === 'a' ? 'ya' : 'yb'] = mid + (i - (n - 1) / 2) * step; });
  }
  let bulgeL = 0;
  let bulgeR = 0;
  for (const s of segs) {
    if (s.kind !== 'arc' && s.kind !== 'arcC') continue;
    s.bulge = s.kind === 'arcC' ? Math.min(GEO.colGap * 0.42, 14 + 0.25 * Math.abs(s.ya - s.yb)) : Math.min(110, 14 + 0.28 * Math.abs(s.ya - s.yb));
    if (s.kind === 'arc') { if (s.a.col === 'L') bulgeL = Math.max(bulgeL, s.bulge); else bulgeR = Math.max(bulgeR, s.bulge); }
  }
  const cols = ['L', 'C', 'R'].filter((c) => (c === 'C' ? P.length : side[c].length));
  const colX = {};
  let x = GEO.pad + bulgeL;
  cols.forEach((c, i) => { colX[c] = x; x += colW[c] + (i < cols.length - 1 ? GEO.colGap : 0); });
  const W = x + GEO.pad + bulgeR;
  for (const it of I.values()) it.x = colX[it.col];

  const free = [];
  if (P.length) {
    const boxes = P.map((p) => [p.y - 5, p.y + p.h + 5]).sort((a, b) => a[0] - b[0]);
    let prev = GEO.top - 22;
    for (const [t, bt] of boxes) { if (t - prev > 6) free.push({ lo: prev, hi: t, used: 0 }); prev = Math.max(prev, bt); }
    free.push({ lo: prev, hi: prev + 400, used: 0 });
  }
  for (const s of segs) {
    const x1 = s.sa === 'l' ? s.a.x : s.a.x + s.a.w;
    const x2 = s.sb === 'l' ? s.b.x : s.b.x + s.b.w;
    const y1 = s.ya;
    const y2 = s.yb;
    if (s.kind === 'arc' || s.kind === 'arcC') {
      const b = s.bulge * (s.sa === 'l' ? -1 : 1);
      s.d = `M${x1},${y1} C${x1 + b},${y1} ${x2 + b},${y2} ${x2},${y2}`;
    } else if (s.kind === 'cross' && free.length) {
      const L = s.a.col === 'L' ? { x: x1, y: y1 } : { x: x2, y: y2 };
      const R = s.a.col === 'L' ? { x: x2, y: y2 } : { x: x1, y: y1 };
      const want = (L.y + R.y) / 2;
      let best = free[0];
      let bestD = Infinity;
      for (const f of free) {
        const d = want < f.lo ? f.lo - want : want > f.hi ? want - f.hi : 0;
        if (d < bestD) { bestD = d; best = f; }
      }
      const k = best.used++;
      const yc = clamp(want, best.lo + 3, Math.max(best.lo + 3, best.hi - 3)) + (k % 2 ? -1 : 1) * Math.ceil(k / 2) * 4;
      const cxL = colX.C - 26;
      const cxR = colX.C + colW.C + 26;
      const mL = (L.x + cxL) / 2;
      const mR = (cxR + R.x) / 2;
      s.d = `M${L.x},${L.y} C${mL},${L.y} ${mL},${yc} ${cxL},${yc} L${cxR},${yc} C${mR},${yc} ${mR},${R.y} ${R.x},${R.y}`;
    } else {
      const dx = Math.max(30, Math.abs(x2 - x1) * 0.5) * (x2 >= x1 ? 1 : -1);
      s.d = `M${x1},${y1} C${x1 + dx},${y1} ${x2 - dx},${y2} ${x2},${y2}`;
    }
  }
  const bandBoxes = [];
  for (const s of ['L', 'R']) for (const b of side[s]) bandBoxes.push({ key: b.key, x: colX[s], y: b.y, w: colW[s], n: b.items.length });
  const caps = [];
  if (colX.L != null) caps.push({ text: 'Consumers / context', x: colX.L });
  if (colX.C != null) caps.push({ text: 'Focus', x: colX.C });
  if (colX.R != null) caps.push({ text: 'Dependencies', x: colX.R });
  return { W: Math.ceil(W), H: Math.ceil(H), items: [...I.values()], byId: I, segs, bands: bandBoxes, caps, colX, colW, cols };
}

/* ---------- map ---------- */
/* One predicate for both the visible set and the map cache key, so they cannot drift apart. */
function nodePasses(n) {
  return S.kinds.has(n.kind) && (S.longTail || !n.indirect) && S.statuses.has(n.status) && (S.cat === 'all' || n.kind === 'product' || n.category === S.cat);
}
function visibleGraph() {
  const sel = S.selected;
  let ns = D.nodes.filter((n) => n.id === sel || nodePasses(n));
  let ids = new Set(ns.map((n) => n.id));
  let es = D.edges.filter((e) => ids.has(e.from) && ids.has(e.to) && S.types.has(e.type) && S.evidence.has(e.tier));
  if (S.cat !== 'all') {
    /* A category view keeps only the products that touch that category. */
    const touched = new Set();
    for (const e of es) {
      const a = D.byId.get(e.from);
      const b = D.byId.get(e.to);
      if (a.kind === 'product' && b.kind !== 'product') touched.add(a.id);
      if (b.kind === 'product' && a.kind !== 'product') touched.add(b.id);
    }
    ns = ns.filter((n) => n.kind !== 'product' || n.id === sel || touched.has(n.id));
    ids = new Set(ns.map((n) => n.id));
    es = es.filter((e) => ids.has(e.from) && ids.has(e.to));
  }
  if (S.focus && sel && ids.has(sel)) {
    es = es.filter((e) => e.from === sel || e.to === sel);
    const keep = new Set([sel]);
    es.forEach((e) => { keep.add(e.from); keep.add(e.to); });
    ns = ns.filter((n) => keep.has(n.id));
  }
  return { nodes: ns, edges: es };
}
function currentMapKey() {
  const n = S.selected ? D.byId.get(S.selected) : null;
  const forced = n && !nodePasses(n) ? n.id : '';
  return [[...S.kinds].sort(), S.longTail, [...S.statuses].sort(), S.cat, [...S.types].sort(), [...S.evidence].sort(), S.focus ? S.selected : '', S.cat !== 'all' ? S.selected : '', forced].join('|');
}
function nodeAria(n) {
  const rel = n.links.length;
  return `${n.label}. ${n.entityKind}, ${CAT_ONE[n.category]}, ${STATUS_ONE[n.status].toLowerCase()}. ${plural(rel, 'relationship')}.`;
}

function renderMap() {
  const focused = document.activeElement && document.activeElement.closest ? document.activeElement.closest('#map-nodes .node') : null;
  const refocus = focused ? focused.dataset.node : null;
  S.hover = null;
  mapKey = currentMapKey();
  VIS = visibleGraph();
  const none = VIS.nodes.length === 0;
  E.world.hidden = none;
  E.mapEmpty.hidden = !none;
  if (none) {
    LAY = null;
    E.mapEmpty.replaceChildren(D.empty ? unfinished() : h('p', { text: 'No nodes match these filters.' }), D.empty ? null : h('button', { class: 'btn', type: 'button', 'data-act': 'reset', text: 'Reset filters' }));
    updateMapControls();
    return;
  }
  LAY = layoutGraph(VIS.nodes, VIS.edges, S.focus && S.selected ? S.selected : null);
  const eo = clamp(0.62 - VIS.edges.length / 320, 0.3, 0.6);
  E.svg.style.setProperty('--eo', String(eo));
  E.svg.setAttribute('width', LAY.W);
  E.svg.setAttribute('height', LAY.H);
  E.svg.setAttribute('viewBox', `0 0 ${LAY.W} ${LAY.H}`);
  const segs = LAY.segs.slice().sort((a, b) => TIERS.indexOf(b.e.tier) - TIERS.indexOf(a.e.tier));
  E.svg.replaceChildren(...segs.map((s) => {
    const e = s.e;
    const g = sv('g', { class: `edge t-${e.type}${e.tier === 'config' ? ' cfg' : e.tier === 'inference' ? ' inf' : ''}`, 'data-edge': e.id, 'data-from': e.from, 'data-to': e.to });
    const t = sv('title');
    t.textContent = `${D.byId.get(e.from).label} to ${D.byId.get(e.to).label}: ${e.type}, ${evidenceText(e)}${e.label ? '. ' + e.label : ''}`;
    g.append(t, sv('path', { class: 'hit', d: s.d }), sv('path', { class: 'ln', d: s.d }));
    return g;
  }));
  E.bands.replaceChildren(
    ...LAY.caps.map((c) => h('div', { class: 'colcap', css: `left:${c.x}px;top:12px`, text: c.text })),
    ...LAY.bands.map((b) => h('div', { class: `band t-${BANDS[b.key].tone}`, css: `left:${b.x}px;top:${b.y}px` }, sw(BANDS[b.key].tone), h('b', { text: BANDS[b.key].label }), String(b.n))),
  );
  if (!LAY.byId.has(cursorId)) cursorId = (LAY.byId.get(S.selected) || LAY.items.filter((i) => i.col === 'C').sort((a, b) => a.y - b.y)[0] || LAY.items[0]).id;
  E.nodes.replaceChildren(...LAY.items.slice().sort((a, b) => ['L', 'C', 'R'].indexOf(a.col) - ['L', 'C', 'R'].indexOf(b.col) || a.y - b.y).map((it) => {
    const n = it.n;
    const b = h('button', {
      type: 'button', class: `node k-${n.kind} s-${n.status} t-${it.band ? BANDS[it.band].tone : 'business'}`, 'data-node': n.id,
      'aria-pressed': String(S.selected === n.id), 'aria-label': nodeAria(n), tabindex: n.id === cursorId ? '0' : '-1',
      title: `${n.label} (${n.status})`,
      css: `left:${it.x}px;top:${it.y}px;width:${it.w}px;height:${it.h}px`,
    }, h('span', { class: 'nl' }, h('span', { class: 'lab', text: n.label }), h('i', { class: 'stg', 'aria-hidden': 'true' })));
    for (const e of it.sub) b.append(h('span', { class: 'sub' }, sw(e.type, e.tier), h('span', { text: e.label || e.type })));
    if (it.more > 0) b.append(h('span', { class: 'sub' }, h('span', { text: `+${it.more} more` })));
    return b;
  }));
  applyScale();
  if (refocus) {
    const target = LAY.byId.has(refocus) ? refocus : S.selected && LAY.byId.has(S.selected) ? S.selected : null;
    if (target) moveCursor(target);
  }
  paint();
  updateMapControls();
}

function autoScale() {
  const vw = E.vp.clientWidth - 16;
  const vh = E.vp.clientHeight - 16;
  if (!LAY || vw <= 0 || vh <= 0) return 1;
  const fit = Math.min(vw / LAY.W, vh / LAY.H);
  return clamp(fit, vw < 700 ? 0.8 : 1, 1.15);
}
function applyScale() {
  if (!LAY) return;
  curScale = S.scale == null ? autoScale() : S.scale;
  E.world.style.width = LAY.W * curScale + 'px';
  E.world.style.height = LAY.H * curScale + 'px';
  E.scale.style.width = LAY.W + 'px';
  E.scale.style.height = LAY.H + 'px';
  E.scale.style.transform = `scale(${curScale})`;
}
function zoomTo(s, ax, ay) {
  if (!LAY) return;
  s = clamp(s, 0.3, 1.8);
  const vx = ax == null ? E.vp.clientWidth / 2 : ax;
  const vy = ay == null ? E.vp.clientHeight / 2 : ay;
  const wx = (E.vp.scrollLeft + vx - E.world.offsetLeft) / curScale;
  const wy = (E.vp.scrollTop + vy - E.world.offsetTop) / curScale;
  S.scale = s;
  applyScale();
  E.vp.scrollLeft = wx * curScale + E.world.offsetLeft - vx;
  E.vp.scrollTop = wy * curScale + E.world.offsetTop - vy;
}
function fitMap() {
  if (!LAY) return;
  const vw = E.vp.clientWidth - 16;
  const vh = E.vp.clientHeight - 16;
  S.scale = vw > 0 && vh > 0 ? clamp(Math.min(vw / LAY.W, vh / LAY.H), 0.3, 1.6) : 1;
  applyScale();
}

function paint() {
  if (!LAY) return;
  const active = S.hover && LAY.byId.has(S.hover) ? S.hover : S.selected && LAY.byId.has(S.selected) ? S.selected : null;
  const near = new Set();
  const hot = new Set();
  if (active) {
    near.add(active);
    for (const e of VIS.edges) if (e.from === active || e.to === active) { hot.add(e.id); near.add(e.from); near.add(e.to); }
  }
  const q = S.query.trim().toLowerCase();
  const match = q ? new Set(VIS.nodes.filter((n) => n.mhay.includes(q)).map((n) => n.id)) : null;
  for (const b of E.nodes.children) {
    const id = b.dataset.node;
    const sel = id === S.selected;
    b.classList.toggle('is-sel', sel);
    b.setAttribute('aria-pressed', String(sel));
    b.classList.toggle('is-near', !!active && near.has(id));
    b.classList.toggle('is-match', !!match && match.has(id));
    b.classList.toggle('is-dim', active ? !near.has(id) : !!match && !match.has(id));
  }
  const lit = !!active || !!match;
  E.svg.classList.toggle('has-active', lit);
  const raise = [];
  for (const g of Array.from(E.svg.children)) {
    const isHot = active ? hot.has(g.dataset.edge) : !!match && (match.has(g.dataset.from) || match.has(g.dataset.to));
    g.classList.toggle('is-hot', isHot);
    if (isHot) raise.push(g);
  }
  raise.forEach((g) => E.svg.appendChild(g));
  if (q) {
    const hidden = count(D.nodes, (n) => n.mhay.includes(q)) - (match ? match.size : 0);
    E.qCount.textContent = `${plural(match.size, 'match', 'matches')}${hidden > 0 ? `, ${hidden} hidden by filters` : ''}`;
  } else E.qCount.textContent = '';
}

function updateMapControls() {
  E.focusBtn.disabled = !S.selected;
  E.focusBtn.setAttribute('aria-pressed', String(S.focus));
  E.focusBtn.textContent = S.focus ? 'Exit focus' : 'Focus';
  for (const c of E.kindChips.children) c.setAttribute('aria-pressed', String(c.dataset.kind ? S.kinds.has(c.dataset.kind) : S.longTail));
  for (const c of E.statusChips.children) c.setAttribute('aria-pressed', String(S.statuses.has(c.dataset.status)));
  for (const c of $$('[data-type]', E.legend)) c.setAttribute('aria-pressed', String(S.types.has(c.dataset.type)));
  for (const c of $$('[data-evidence]', E.legend)) c.setAttribute('aria-pressed', String(S.evidence.has(c.dataset.evidence)));
  E.catSel.value = S.cat;
  $('#view-map').dataset.focus = String(S.focus);
}

function buildMapControls() {
  E.kindChips.replaceChildren(...KINDS_IN.map((k) => h('button', { type: 'button', class: 'chip', 'data-kind': k, 'aria-pressed': 'true' },
    h('i', { class: `kg kg-${k}`, 'aria-hidden': 'true' }), KIND_PL[k], h('span', { class: 'c', text: String(count(D.nodes, (n) => n.kind === k)) }))),
  ...(INDIRECT ? [h('button', { type: 'button', class: 'chip', 'data-longtail': 'true', 'aria-pressed': String(S.longTail), title: 'Tools that link only to other providers, not to a product' }, 'Indirect tools', h('span', { class: 'c', text: String(INDIRECT) }))] : []));
  const statusTitle = { live: 'Observed in a live provider or DNS read; not necessarily a passing product journey', configured: 'Set up in config or an account; not observed in a live read', historical: 'Past use', unknown: 'Status not established' };
  E.statusChips.replaceChildren(...STATUSES_IN.map((s) => h('button', { type: 'button', class: 'chip', 'data-status': s, 'aria-pressed': 'true', title: statusTitle[s] },
    h('i', { class: `stg s-${s}`, 'aria-hidden': 'true' }), s, h('span', { class: 'c', text: String(count(D.nodes, (n) => n.status === s)) }))));
  const catCount = (c) => count(D.nodes, (n) => n.kind !== 'product' && n.category === c);
  E.catSel.replaceChildren(h('option', { value: 'all', text: 'All categories' }), ...CATS.filter((c) => catCount(c) > 0).map((c) => h('option', { value: c, text: `${CAT_TITLE[c]} (${catCount(c)})` })));
  const typeChips = TYPES_IN.map((t) => h('button', { type: 'button', class: 'chip', 'data-type': t, 'aria-pressed': 'true', title: `Show or hide ${t} relationships` },
    sw(t), t, h('span', { class: 'c', text: String(count(D.edges, (e) => e.type === t)) })));
  const evChips = TIERS.map((tier) => {
    const n = count(D.edges, (e) => e.tier === tier);
    return h('button', { type: 'button', class: 'chip', 'data-evidence': tier, 'aria-pressed': 'true', title: TIER_TITLE[tier], disabled: n ? null : true }, sw('other', tier), TIER_ONE[tier], h('span', { class: 'c', text: String(n) }));
  });
  E.legend.replaceChildren(...typeChips, typeChips.length ? h('span', { class: 'sep', 'aria-hidden': 'true' }) : null, ...evChips);
}

function moveCursor(id) {
  const b = E.nodes.querySelector(`[data-node="${CSS.escape(id)}"]`);
  if (!b) return;
  for (const o of E.nodes.querySelectorAll('[tabindex="0"]')) o.tabIndex = -1;
  b.tabIndex = 0;
  cursorId = id;
  b.focus();
}
function neighborItem(it, key) {
  const colItems = (c) => LAY.items.filter((o) => o.col === c).sort((a, b) => a.y - b.y);
  if (key === 'ArrowUp' || key === 'ArrowDown' || key === 'Home' || key === 'End') {
    const list = colItems(it.col);
    const i = list.indexOf(it);
    if (key === 'Home') return list[0];
    if (key === 'End') return list[list.length - 1];
    return list[i + (key === 'ArrowDown' ? 1 : -1)];
  }
  const ci = LAY.cols.indexOf(it.col) + (key === 'ArrowRight' ? 1 : -1);
  if (ci < 0 || ci >= LAY.cols.length) return null;
  const y = it.y + it.h / 2;
  return colItems(LAY.cols[ci]).reduce((best, o) => (!best || Math.abs(o.y + o.h / 2 - y) < Math.abs(best.y + best.h / 2 - y) ? o : best), null);
}

/* ---------- selection and views ---------- */
function select(id, opts = {}) {
  const prev = S.selected;
  S.selected = id && D.byId.has(id) ? id : null;
  if (!S.selected) S.focus = false;
  if (S.selected && opts.cursor !== false) cursorId = S.selected;
  render();
  if (S.selected && S.selected !== prev) {
    const n = D.byId.get(S.selected);
    announce(`${n.label} selected. ${plural(n.links.length, 'relationship')}.`);
  }
}
function setView(v, push = true) {
  if (!VIEWS.includes(v)) return;
  const changed = S.view !== v;
  S.view = v;
  if (v !== 'map') S.hover = null;
  if (v !== 'evidence') S.flash = null;
  if (changed && push) { try { history.pushState(null, '', '#' + hashFor()); } catch (e) { /* some file:// contexts refuse history writes; the view still switches */ } }
  render();
  setActivePane(document.getElementById('view-' + v));
}
function hashFor() {
  let s = S.view;
  if (S.view === 'inventory' && S.inv.tab === 'edges') s += '/relationships';
  if ((S.view === 'map' || S.view === 'inventory') && S.selected) s += '/' + encodeURIComponent(S.selected) + (S.view === 'map' && S.focus ? '/focus' : '');
  if (S.view === 'evidence' && S.flash) s += '/' + encodeURIComponent(S.flash);
  return s;
}
function readHash() {
  const parts = location.hash.replace(/^#/, '').split('/').filter(Boolean).map(p=>{try{return decodeURIComponent(p)}catch{return p}});
  const v = VIEWS.includes(parts[0]) ? parts[0] : 'map';
  S.view = v;
  S.flash = null;
  if (v === 'inventory' && parts[1] === 'relationships') { S.inv.tab = 'edges'; parts.splice(1, 1); }
  if (v === 'map' || v === 'inventory') {
    S.selected = parts[1] && D.byId.has(parts[1]) ? parts[1] : null;
    S.focus = v === 'map' && !!S.selected && parts[2] === 'focus';
    if (S.selected) cursorId = S.selected;
  }
  if (v === 'evidence' && parts[1]) S.flash = parts[1];
}
function setActivePane(p) {
  if (!p) return;
  for (const o of $$('.pane')) o.classList.toggle('is-active', o === p);
}

function render() {
  document.body.dataset.view = S.view;
  for (const t of $$('.tab')) {
    const on = t.dataset.view === S.view;
    t.setAttribute('aria-selected', String(on));
    t.tabIndex = on ? 0 : -1;
  }
  for (const v of VIEWS) document.getElementById('view-' + v).hidden = v !== S.view;
  if (S.view === 'map') {
    if (currentMapKey() !== mapKey || (!LAY && !D.empty)) renderMap();
    else { paint(); updateMapControls(); }
  }
  if (S.view === 'inventory') renderInventory();
  if (S.view === 'directions' && !dirBuilt) { renderDirections(); dirBuilt = true; }
  if (S.view === 'evidence') { if (!evBuilt) { renderEvidence(); evBuilt = true; } flashSource(); }
  renderDetail();
  const hs = hashFor();
  if (location.hash.replace(/^#/, '') !== hs) { try { history.replaceState(null, '', '#' + hs); } catch (e) { /* see setView */ } }
}

/* ---------- detail ---------- */
function unknownText(s) { return isUnknown(s) ? h('span', { class: 'unk', text: s || 'Unknown' }) : s; }
function srcList(ids) {
  if (!ids.length) return h('span', { class: 'unk', text: 'None cited' });
  return h('div', { class: 'srcs' }, ids.map((id) => {
    const s = D.srcById.get(id);
    return h('div', { class: 'srcrow' }, h('span', { class: 'lvl', text: s ? s.level : 'missing' }),
      h('button', { type: 'button', class: 'link', 'data-src': id, title: s ? s.result : 'Source not found in data', text: s ? s.label : id }));
  }));
}
function renderDetail() {
  const show = !D.empty && (S.view === 'map' || S.view === 'inventory');
  E.detail.hidden = !show;
  E.desk.classList.toggle('solo', !show);
  if (!show) return;
  const n = S.selected ? D.byId.get(S.selected) : null;
  E.detail.classList.toggle('is-open', !!n);
  E.detail.dataset.node = n ? n.id : '';
  const key = (n ? n.id : '') + '|' + S.view + '|' + S.focus;
  if (key === lastDetail) return;
  const sameNode = lastDetail.split('|')[0] === (n ? n.id : '');
  lastDetail = key;
  if (!n) { renderOverview(); return; }
  E.dHead.replaceChildren(
    h('div', { class: 'grow' }, h('h2', { class: 'd-title', tabindex: '-1', text: n.label }), h('div', { class: 'd-sub', text: `${n.entityKind}, ${CAT_ONE[n.category]}` })),
    h('button', { type: 'button', class: 'btn', 'data-act': 'clear', text: 'Close' }),
  );
  const facts = h('dl', { class: 'facts' },
    h('dt', { text: 'Status' }), h('dd', { 'data-field': 'status' }, STATUS_ONE[n.status]),
    h('dt', { text: 'Owner' }), h('dd', { 'data-field': 'owner' }, unknownText(n.owner)),
    h('dt', { text: 'Cost' }), h('dd', { 'data-field': 'cost' }, unknownText(n.cost)),
    h('dt', { text: 'Repos' }), h('dd', { 'data-field': 'repos' }, n.repos.length ? n.repos.join(', ') : h('span', { class: 'unk', text: 'None recorded' })),
    n.kind === 'product' ? [h('dt', { text: 'Neighbors' }), h('dd', { text: plural(n.otherCount, 'provider or tool', 'providers and tools') })] : [h('dt', { text: 'Used by' }), h('dd', { text: plural(n.productCount, 'product') })],
    h('dt', { text: 'Sources' }), h('dd', { 'data-field': 'sources' }, srcList(n.sources)),
  );
  const groups = TYPES_IN.map((t) => [t, n.links.filter((l) => l.e.type === t)]).filter(([, ls]) => ls.length);
  const rels = h('div', { class: 'd-sec' }, h('h3', { text: `Relationships (${n.links.length})` }),
    n.links.length ? groups.map(([t, ls]) => h('div', { class: 'relgroup' },
      h('div', { class: 'h3' }, sw(t), t),
      ls.sort((a, b) => byLabel(a.other, b.other)).map((l) => {
        return h('div', { class: 'rel' },
          sw(t, l.e.tier),
          h('span', null, h('span', { class: 'muted', text: l.dir === 'out' ? 'to ' : 'from ' }), h('button', { type: 'button', class: 'link', 'data-select': l.other.id, text: l.other.label })),
          h('span', { class: 'ev', title: TIER_TITLE[l.e.tier], text: evidenceText(l.e) }),
          l.e.label ? h('span', { class: 'lbl', text: l.e.label }) : null);
      }))) : h('p', { class: 'muted', text: 'No relationships recorded.' }));
  const listSec = (title, items, cls) => (items.length ? h('div', { class: 'd-sec' }, h('h3', { text: title }), h('ul', { class: 'list ' + (cls || '') }, items.map((s) => h('li', null, rich(s))))) : null);
  const inMap = S.view === 'map';
  E.dBody.replaceChildren(...[
    n.summary ? h('p', { class: 'd-sum' }, rich(n.summary)) : null,
    facts,
    rels,
    listSec('Strengths for agent operation', n.strengths, 'plus'),
    listSec('Weaknesses for agent operation', n.weaknesses, 'minus'),
    listSec('Details', n.details),
    h('div', { class: 'actions' },
      inMap ? h('button', { type: 'button', class: 'btn', 'data-act': 'focus', 'aria-pressed': String(S.focus), text: S.focus ? 'Exit focus' : 'Focus neighborhood' }) : h('button', { type: 'button', class: 'btn', 'data-goto': n.id, text: 'Show on map' }),
      inMap ? h('button', { type: 'button', class: 'btn', 'data-act': 'inventory', text: 'Show in inventory' }) : null,
      n.scopeLink ? h('a', {class:'btn',href:n.scopeLink,text:'Open scoped map'}) : null),
  ].filter(Boolean));
  if (!sameNode) E.dBody.scrollTop = 0;
}
function renderOverview() {
  E.detail.dataset.node = '';
  E.dHead.replaceChildren(h('div', { class: 'grow' }, h('h2', { class: 'd-title', text: 'Overview' }), h('div', { class: 'd-sub', text: D.meta.date ? `Generated ${D.meta.date}` : 'Collection date not recorded' })));
  if (D.empty) { E.dBody.replaceChildren(unfinished()); return; }
  const rec = D.directions[0];
  const kc = (k) => count(D.nodes, (n) => n.kind === k);
  const tierCount = TIERS.map((tier) => count(D.edges, (e) => e.tier === tier)).join(', ');
  E.dBody.replaceChildren(...[
    D.meta.coverage ? h('p', { class: 'd-sum' }, D.meta.coverage) : null,
    h('div', { class: 'stat' },
      KINDS_IN.map((k) => h('div', null, h('span', { text: KIND_PL[k] }), h('b', { text: String(kc(k)) }))),
      h('div', null, h('span', { text: 'Relationships' }), h('b', { text: String(D.edges.length) })),
      h('div', null, h('span', { text: 'Provider, documentary, inferred' }), h('b', { text: tierCount })),
      h('div', null, h('span', { text: 'Sources' }), h('b', { text: String(D.sources.length) })),
      h('div', null, h('span', { text: 'Open gaps' }), h('b', { text: String(D.gaps.length) }))),
    rec ? h('div', { class: 'd-sec' }, h('h3', { text: 'Proposed operating model' }),
      h('div', { class: 'recbox' }, h('div', { class: 'nm', text: rec.name }), rec.stance ? h('div', { class: 'muted', text: rec.stance }) : null, rec.decision ? h('p', { class: 'd-sum' }, rich(rec.decision)) : null),
      h('div', { class: 'actions' }, h('button', { type: 'button', class: 'btn', 'data-view-go': 'directions', text: 'Compare directions' }))) : null,
    h('div', { class: 'd-sec' }, h('h3', { text: 'Reading the map' }),
      h('ul', { class: 'list' },
        h('li', null, 'Focal files, groups or repositories sit in the center. Context is left; dependencies are right. Column position does not determine edge direction.'),
        h('li', null, 'Line color is the relationship type. Solid lines are positive provider proof from a successful API, CLI or DNS read. Dotted lines are documentary: repository source or config, docs, pricing, or a record of a failed or unattributed probe. Dashed lines are inferred. No line style shows traffic or service health.'),
        h('li', null, 'The square on each node is its status: filled for a live observation in a provider or DNS read (not necessarily a passing product journey), outlined when only configured, a dash when historical, dotted when unknown.'),
        KINDS_IN.filter((k) => !DEFAULT_KINDS.has(k)).map((k) => h('li', null, `${KIND_PL[k]} (${count(D.nodes, (n) => n.kind === k)}) start hidden so labels stay readable. Turn them on with the ${KIND_PL[k]} chip; Inventory lists everything.`)),
        DEFAULT_LONGTAIL ? null : h('li', null, `Tools that link only to other providers (${INDIRECT}) start hidden so labels stay readable. Turn them on with Indirect tools; Inventory lists everything.`),
        h('li', null, 'Select a node for its owner, cost, repos, status and sources.'))),
  ].filter(Boolean));
}
function unfinished() {
  if (D.error) return h('div', { class: 'notice' }, h('h2', { text: 'Embedded data could not be read' }), h('p', { text: D.error }), h('p', { text: 'Rebuild this file from valid JSON.' }));
  return h('div', { class: 'notice', 'data-state': 'unfinished' }, h('h2', { text: 'Unfinished: research data not embedded' }),
    h('p', { text: 'This file is the page shell only. It shows no inventory, relationships or directions until the final research payload is embedded.' }));
}

/* ---------- inventory ---------- */
const nodeBtn = (id) => { const n = D.byId.get(id); return h('button', { type: 'button', class: 'link', 'data-select': id, text: n ? n.label : id }); };
const srcIds = (ids) => (ids.length ? h('span', { class: 'ids' }, ids.map((id) => h('button', { type: 'button', class: 'idbtn', 'data-src': id, title: D.srcById.get(id) ? D.srcById.get(id).label : 'Source not found in data', text: id }))) : h('span', { class: 'unk', text: 'None' }));
const NODE_COLS = [
  { key: 'label', label: 'Name', get: (n) => n.label, cell: (n) => h('button', { type: 'button', class: 'link cell-name', 'data-select': n.id, text: n.label }) },
  { key: 'kind', label: 'Kind', get: (n) => n.entityKind, cell: (n) => n.entityKind },
  { key: 'category', label: 'Category', get: (n) => CAT_ONE[n.category], cell: (n) => CAT_ONE[n.category] },
  { key: 'status', label: 'Status', get: (n) => STATUSES.indexOf(n.status), cell: (n) => STATUS_ONE[n.status] },
  { key: 'owner', label: 'Owner', text: true, get: (n) => n.owner, cell: (n) => unknownText(n.owner) },
  { key: 'cost', label: 'Cost', text: true, get: (n) => n.cost, cell: (n) => unknownText(n.cost) },
  { key: 'repos', label: 'Repos', text: true, get: (n) => n.repos.join(', '), cell: (n) => (n.repos.length ? n.repos.join(', ') : h('span', { class: 'unk', text: 'None' })) },
  { key: 'links', label: 'Links', num: true, get: (n) => n.links.length, cell: (n) => String(n.links.length) },
  { key: 'levels', label: 'Seen via', get: (n) => (n.levels.length ? LEVEL_RANK[n.levels[0]] : 99), cell: (n) => (n.levels.length ? n.levels.join(', ') : h('span', { class: 'unk', text: 'No source' })) },
  { key: 'sources', label: 'Sources', num: true, get: (n) => n.sources.length, cell: (n) => srcIds(n.sources) },
];
const EDGE_COLS = [
  { key: 'from', label: 'From', get: (e) => D.byId.get(e.from).label, cell: (e) => nodeBtn(e.from) },
  { key: 'type', label: 'Type', get: (e) => TYPES.indexOf(e.type), cell: (e) => h('span', null, sw(e.type, e.tier), ' ', e.type) },
  { key: 'to', label: 'To', get: (e) => D.byId.get(e.to).label, cell: (e) => nodeBtn(e.to) },
  { key: 'label', label: 'Label', text: true, get: (e) => e.label, cell: (e) => e.label || h('span', { class: 'unk', text: 'None' }) },
  { key: 'evidence', label: 'Evidence', get: (e) => TIERS.indexOf(e.tier) * 10 + (LEVEL_RANK[edgeLevel(e)] ?? 9), cell: (e) => evidenceText(e) },
  { key: 'sources', label: 'Sources', num: true, get: (e) => e.sources.length, cell: (e) => srcIds(e.sources) },
];
function cmp(col, dir) {
  return (a, b) => {
    const x = col.get(a);
    const y = col.get(b);
    if (col.text) {
      const ux = isUnknown(x) || x === '';
      const uy = isUnknown(y) || y === '';
      if (ux !== uy) return ux ? 1 : -1;
    }
    const r = typeof x === 'number' && typeof y === 'number' ? x - y : String(x).localeCompare(String(y), 'en', { sensitivity: 'base', numeric: true });
    return r * dir;
  };
}
function invFilterControls() {
  const I = S.inv;
  const mk = (id, label, value, opts) => h('label', null, h('span', { class: 'sr-only', text: label }),
    h('select', { class: 'sel', id, 'data-inv-filter': id }, opts.map(([v, t]) => h('option', { value: v, text: t, selected: v === value ? true : null }))));
  if (I.tab === 'nodes') return [
    mk('inv-kind', 'Kind', I.kind, [['all', 'All kinds'], ...KINDS_IN.map((k) => [k, KIND_PL[k]])]),
    mk('inv-status', 'Status', I.status, [['all', 'All statuses'], ...STATUSES.filter((s) => D.nodes.some((n) => n.status === s)).map((s) => [s, STATUS_ONE[s]])]),
    mk('inv-cat', 'Category', I.cat, [['all', 'All categories'], ...CATS.filter((c) => D.nodes.some((n) => n.category === c)).map((c) => [c, CAT_TITLE[c]])]),
  ];
  return [
    mk('inv-type', 'Type', I.type, [['all', 'All types'], ...TYPES_IN.map((t) => [t, t])]),
    mk('inv-evidence', 'Evidence', I.evidence, [['all', 'All evidence'], ['live', 'Provider read'], ['config', 'Documentary'], ['inference', 'Inferred']]),
  ];
}
let invKey = '';
let invSel = null;
function renderInventory() {
  const I = S.inv;
  const nodesTab = I.tab === 'nodes';
  const key = [I.tab, I.q, I.kind, I.status, I.cat, I.type, I.evidence, ...I.sort[I.tab]].join('|');
  if (key !== invKey) {
    invKey = key;
    buildInventory(nodesTab);
    invSel = null;
  }
  const sel = S.selected;
  for (const tr of E.invWrap.querySelectorAll('tr.row')) {
    tr.classList.toggle('is-sel', !tr.dataset.edge && tr.dataset.row === sel);
    tr.classList.toggle('is-near', !!tr.dataset.edge && !!sel && (tr.dataset.from === sel || tr.dataset.to === sel));
  }
  if (sel !== invSel) {
    invSel = sel;
    const tr = sel && nodesTab ? E.invWrap.querySelector(`tr[data-row="${CSS.escape(sel)}"]`) : null;
    if (tr) { const head = E.invWrap.querySelector('thead'); scrollWithin(E.invWrap, tr, 'nearest', head ? head.offsetHeight : 0); }
  }
}
function buildInventory(nodesTab) {
  const I = S.inv;
  const tN = $('#inv-tab-nodes');
  const tE = $('#inv-tab-edges');
  tN.replaceChildren('Nodes ', h('span', { class: 'c', text: String(D.nodes.length) }));
  tE.replaceChildren('Relationships ', h('span', { class: 'c', text: String(D.edges.length) }));
  tN.setAttribute('aria-selected', String(nodesTab)); tN.tabIndex = nodesTab ? 0 : -1;
  tE.setAttribute('aria-selected', String(!nodesTab)); tE.tabIndex = nodesTab ? -1 : 0;
  if (E.invQ.value !== I.q) E.invQ.value = I.q;
  const active = document.activeElement;
  const keepFocus = active && active.dataset && active.dataset.invFilter;
  E.invFilters.replaceChildren(...invFilterControls());
  if (keepFocus) { const f = document.getElementById(keepFocus); if (f) f.focus(); }
  if (D.empty) { E.invWrap.replaceChildren(h('div', { class: 'doc-inner' }, unfinished())); E.invCount.textContent = ''; return; }
  const q = I.q.trim().toLowerCase();
  const cols = nodesTab ? NODE_COLS : EDGE_COLS;
  let rows;
  if (nodesTab) {
    rows = D.nodes.filter((n) => (I.kind === 'all' || n.kind === I.kind) && (I.status === 'all' || n.status === I.status) && (I.cat === 'all' || n.category === I.cat) && (!q || n.hay.includes(q)));
  } else {
    rows = D.edges.filter((e) => (I.type === 'all' || e.type === I.type) && (I.evidence === 'all' || e.tier === I.evidence) &&
      (!q || [D.byId.get(e.from).label, D.byId.get(e.to).label, e.type, e.label, TIER_ONE[e.tier], edgeLevel(e) || '', e.sources.join(' ')].join(' ').toLowerCase().includes(q)));
  }
  const [sk, sd] = I.sort[I.tab];
  const sc = cols.find((c) => c.key === sk) || cols[0];
  rows = rows.slice().sort((a, b) => cmp(sc, sd)(a, b) || (nodesTab ? byLabel(a, b) : 0));
  E.invCount.textContent = `Showing ${rows.length} of ${nodesTab ? D.nodes.length : D.edges.length}`;
  const thead = h('thead', null, h('tr', null, cols.map((c) => h('th', { scope: 'col', 'aria-sort': c.key === sk ? (sd > 0 ? 'ascending' : 'descending') : null },
    h('button', { type: 'button', class: 'sort', 'data-sort': c.key }, c.label, c.key === sk ? h('span', { class: 'ar', 'aria-hidden': 'true', text: sd > 0 ? '\u2191' : '\u2193' }) : null)))));
  const tbody = h('tbody', null, rows.length
    ? rows.map((r) => h('tr', nodesTab ? { class: 'row', 'data-row': r.id } : { class: 'row', 'data-row': r.from, 'data-edge': r.id, 'data-from': r.from, 'data-to': r.to },
      cols.map((c) => h('td', { class: c.num ? 'num' : null, 'data-label': c.label }, c.cell(r)))))
    : h('tr', { class: 'empty-row' }, h('td', { colspan: String(cols.length) }, 'Nothing matches. Clear the search or filters.')));
  E.invWrap.replaceChildren(h('table', { class: `inv-table${nodesTab ? '' : ' edges-t'}`, id: nodesTab ? 'node-table' : 'edge-table' }, thead, tbody));
}

/* ---------- directions ---------- */
function subnav(label, items) {
  return h('nav', { class: 'subnav', 'aria-label': label }, items.map(([id, text]) => h('button', { type: 'button', class: 'chip', 'data-jump': id, text })));
}
function renderDirections() {
  if (D.empty) { E.dirDoc.replaceChildren(h('div', { class: 'doc-inner' }, h('div', { class: 'sec' }, unfinished()))); return; }
  const dirs = D.directions;
  const rec = dirs[0];
  const secs = [];
  secs.push(h('section', { class: 'sec', id: 'dir-rec' }, h('h2', { text: 'Recommendation' }),
    rec ? h('div', { class: 'rec', 'data-direction': rec.id, 'data-rank': String(rec.rank) },
      h('p', { class: 'nm', text: rec.name }),
      h('div', { class: 'meta', text: `Ranked ${rec.rank} of ${dirs.length}${rec.stance ? ', ' + rec.stance.toLowerCase() : ''}` }),
      rec.decision ? h('p', null, rich(rec.decision)) : null) : h('p', { class: 'unk', text: 'No directions in the data.' })));
  if (dirs.length) {
    const ASPECTS = [['decision', 'Decision'], ['moves', 'Moves'], ['cost', 'Cost'], ['sources', 'Sources'], ['risk', 'Risks'], ['givesUp', 'Gives up'], ['ergonomics', 'Agent ergonomics'], ['prerequisites', 'Prerequisites']];
    secs.push(h('section', { class: 'sec', id: 'dir-compare' }, h('h2', { text: 'Compare directions' }),
      h('p', { class: 'cap', text: 'Ranked best first. Read across a row to compare one aspect. Each source ID opens its Evidence row; IDs are records, not independent proof.' }),
      h('div', { class: 'compare', vars: { '--n': String(dirs.length) } }, dirs.map((d, i) => h('article', { class: `dcol${i === 0 ? ' is-rec' : ''}`, 'data-direction': d.id, 'data-rank': String(d.rank), 'aria-label': `Rank ${d.rank}: ${d.name}` },
        h('div', { class: 'dcell dhead' }, h('span', { class: 'rk', text: i === 0 ? `Rank ${d.rank}, recommended` : `Rank ${d.rank}` }), h('span', { class: 'nm', text: d.name }), d.stance ? h('span', { class: 'stance', text: d.stance }) : null),
        ASPECTS.map(([k, label]) => h('div', { class: 'dcell' }, h('h4', { text: label }),
          k === 'sources' ? (d.sources.length ? srcIds(d.sources) : h('p', { class: 'unk', text: 'None cited' })) : Array.isArray(d[k]) ? (d[k].length ? h('ul', { class: 'list' }, d[k].map((s) => h('li', null, rich(s)))) : h('p', { class: 'unk', text: 'None listed' })) : d[k] ? h('p', null, rich(d[k])) : h('p', { class: 'unk', text: 'Not stated' }))))))));
  }
  const prods = D.nodes.filter((n) => n.kind === 'product').sort(byLabel);
  const usedTypes = TYPES_IN.filter((t) => D.edges.some((e) => e.type === t && (D.byId.get(e.from).kind === 'product') !== (D.byId.get(e.to).kind === 'product')));
  const hasSpread = prods.length > 0 && usedTypes.length > 0;
  if (hasSpread) {
    const cell = (p, t) => {
      const m = new Map();
      for (const l of p.links) if (l.e.type === t && l.other.kind !== 'product') { const prev = m.get(l.other.id); m.set(l.other.id, prev && TIERS.indexOf(prev) < TIERS.indexOf(l.e.tier) ? prev : l.e.tier); }
      return m;
    };
    const distinct = (t) => { const s = new Set(); prods.forEach((p) => cell(p, t).forEach((_, id) => s.add(id))); return s.size; };
    secs.push(h('section', { class: 'sec', id: 'dir-spread' }, h('h2', { text: 'Current spread' }),
      h('p', { class: 'cap', text: 'Which provider each product uses for each relationship type, derived from the map. Solid underline: provider read; dotted: documentary; dashed: inferred. None of these shows traffic or health.' }),
      h('div', { class: 'wide' }, h('table', { class: 'spread', id: 'spread-table' },
        h('thead', null, h('tr', null, h('th', { scope: 'col' }, h('span', { class: 'thl', text: 'Product' })), usedTypes.map((t) => h('th', { scope: 'col' }, h('span', { class: 'thl' }, sw(t), t))))),
        h('tbody', null, prods.map((p) => h('tr', null, h('th', { scope: 'row', class: 'rowh' }, h('button', { type: 'button', class: 'ref', 'data-goto': p.id, text: p.label })),
          usedTypes.map((t) => { const m = cell(p, t); return h('td', null, m.size ? [...m.entries()].map(([id, tier]) => h('span', { class: `prov${tier === 'live' ? '' : tier === 'config' ? ' cfg' : ' inf'}` }, h('button', { type: 'button', class: 'ref', 'data-goto': id, title: TIER_TITLE[tier], text: D.byId.get(id).label }))) : h('span', { class: 'unk', text: '\u2013' })); })))),
        h('tfoot', null, h('tr', null, h('td', { text: 'Distinct providers' }), usedTypes.map((t) => h('td', { text: String(distinct(t)) }))))))));
  }
  const provs = D.nodes.filter((n) => n.kind === 'provider' || n.kind === 'tool').sort((a, b) => b.productCount - a.productCount || byLabel(a, b));
  if (provs.length) {
    secs.push(h('section', { class: 'sec', id: 'dir-fit' }, h('h2', { text: 'Provider fit' }),
      h('p', { class: 'cap', text: 'Operating strengths and weaknesses for agents, ordered by how many products use each provider.' }),
      h('div', { class: 'wide' }, h('table', { id: 'fit-table' },
        h('thead', null, h('tr', null, ['Provider', 'Used by', 'Strengths', 'Weaknesses', 'Cost', 'Status'].map((t) => h('th', { scope: 'col' }, h('span', { class: 'thl', text: t }))))),
        h('tbody', null, provs.map((n) => h('tr', null,
          h('td', null, h('button', { type: 'button', class: 'ref cell-name', 'data-goto': n.id, text: n.label }), h('div', { class: 'cell-sub', text: `${KIND_ONE[n.kind]}, ${CAT_ONE[n.category]}` })),
          h('td', { class: 'num', text: plural(n.productCount, 'product') }),
          h('td', null, n.strengths.length ? h('ul', { class: 'list plus' }, n.strengths.map((s) => h('li', null, rich(s)))) : h('span', { class: 'unk', text: 'None recorded' })),
          h('td', null, n.weaknesses.length ? h('ul', { class: 'list minus' }, n.weaknesses.map((s) => h('li', null, rich(s)))) : h('span', { class: 'unk', text: 'None recorded' })),
          h('td', null, unknownText(n.cost)),
          h('td', { text: STATUS_ONE[n.status] }))))))));
  }
  secs.push(h('section', { class: 'sec', id: 'dir-alt' }, h('h2', { text: 'Unused options' }),
    h('p', { class: 'cap', text: 'Optional providers outside this fictional workspace. Source links the vendor page; IDs under it open their Evidence rows.' }),
    D.alternatives.length ? h('div', { class: 'wide' }, h('table', { id: 'alt-table' },
      h('thead', null, h('tr', null, ['Option', 'For', 'Why consider it', 'Tradeoff', 'Source'].map((t) => h('th', { scope: 'col' }, h('span', { class: 'thl', text: t }))))),
      h('tbody', null, D.alternatives.map((a) => h('tr', { 'data-alternative': a.name },
        h('td', { class: 'cell-name', text: a.name }), h('td', { text: a.use }), h('td', null, rich(a.why)), h('td', null, rich(a.tradeoff)),
        h('td', null, a.source ? h('a', { href: a.source, target: '_blank', rel: 'noopener noreferrer', text: new URL(a.source).hostname }) : a.sourceText || !a.sources.length ? h('span', { class: 'unk', text: a.sourceText || 'None' }) : null,
          a.sources.length ? srcIds(a.sources) : null)))))) : h('p', { class: 'unk', text: 'None listed.' })));
  if (dirs.length) {
    secs.push(h('section', { class: 'sec', id: 'dir-ranked' }, h('h2', { text: 'Ranked recommendation' }),
      h('p', { class: 'cap', text: 'The directions in order, best first, after the analysis above.' }),
      h('ol', { class: 'ranked', id: 'ranked-list' }, dirs.map((d) => h('li', { 'data-direction': d.id, 'data-rank': String(d.rank) },
        h('span', { class: 'rk', 'aria-hidden': 'true', text: String(d.rank) }),
        h('span', { class: 'nm' }, h('span', { class: 'sr-only', text: `Rank ${d.rank}: ` }), d.name),
        d.stance ? h('span', { class: 'stance', text: d.stance }) : null,
        d.decision ? h('p', null, rich(d.decision)) : null)))));
  }
  const nav = [['dir-rec', 'Recommendation'], dirs.length ? ['dir-compare', 'Compare'] : null, hasSpread ? ['dir-spread', 'Current spread'] : null, provs.length ? ['dir-fit', 'Provider fit'] : null, ['dir-alt', 'Unused options'], dirs.length ? ['dir-ranked', 'Ranked recommendation'] : null].filter(Boolean);
  E.dirDoc.replaceChildren(subnav('Directions sections', nav), h('div', { class: 'doc-inner' }, secs));
}

/* ---------- evidence ---------- */
function citedText(id) {
  const c = D.cited.get(id);
  if (!c) return '';
  const parts = [];
  if (c.nodes.length) parts.push(plural(c.nodes.length, 'node'));
  if (c.edges.length) parts.push(plural(c.edges.length, 'link'));
  if (c.findings.length) parts.push(plural(c.findings.length, 'finding'));
  if (c.directions.length) parts.push(plural(c.directions.length, 'direction'));
  if (c.alternatives.length) parts.push(plural(c.alternatives.length, 'unused option'));
  return parts.join(', ') || 'Not cited';
}
function sourceRows() {
  const q = S.srcQ.trim().toLowerCase();
  const rows = D.sources.filter((s) => (S.srcLevel === 'all' || s.level === S.srcLevel) && (!q || [s.id, s.label, s.level, s.result, s.date, s.url].join(' ').toLowerCase().includes(q)));
  return rows.length ? rows.map((s) => h('tr', { id: 'src-' + s.id, 'data-source': s.id },
    h('td', null, h('span', { class: 'cell-name', text: s.id })),
    h('td', { text: s.label }),
    h('td', null, h('span', { class: 'lvl-pill', text: s.level })),
    h('td', { text: s.date || '' }),
    h('td', { text: s.result }),
    h('td', null, s.url ? h('a', { href: s.url, target: '_blank', rel: 'noopener noreferrer', text: new URL(s.url).hostname }) : h('span', { class: 'unk', text: 'None' })),
    h('td', { class: 'num', text: citedText(s.id) }))) : [h('tr', { class: 'empty-row' }, h('td', { colspan: '7', text: 'No sources match.' }))];
}
function renderEvidence() {
  if (D.empty) { E.evDoc.replaceChildren(h('div', { class: 'doc-inner' }, h('div', { class: 'sec' }, unfinished()))); return; }
  const lv = Object.keys(LEVEL_RANK).map((l) => [l, l, count(D.sources, (s) => s.level === l)]).filter(([, , n]) => n);
  const secs = [
    h('section', { class: 'sec', id: 'ev-coverage' }, h('h2', { text: 'Coverage' }),
      D.meta.coverage ? h('p', { class: 'cap', css: 'color:var(--fg)', text: D.meta.coverage }) : null,
      D.meta.limits.length ? [h('h3', { class: 'h3', text: 'Limits' }), h('ul', { class: 'list' }, D.meta.limits.map((s) => h('li', null, rich(s))))] : null),
    h('section', { class: 'sec', id: 'ev-findings' }, h('h2', { text: 'Findings' }),
      D.findings.length ? D.findings.map((f) => h('div', { class: 'finding' }, h('h3', { text: f.title }), h('p', null, rich(f.detail)), srcIds(f.evidence))) : h('p', { class: 'unk', text: 'None recorded.' })),
    h('section', { class: 'sec', id: 'ev-gaps' }, h('h2', { text: 'Coverage gaps' }),
      h('p', { class: 'cap', text: 'What the research could not see, and the read that would close each gap.' }),
      D.gaps.length ? h('div', { class: 'wide' }, h('table', { id: 'gap-table' },
        h('thead', null, h('tr', null, ['Area', 'What is missing', 'Read needed'].map((t) => h('th', { scope: 'col' }, h('span', { class: 'thl', text: t }))))),
        h('tbody', null, D.gaps.map((g) => h('tr', null, h('td', { class: 'cell-name', text: g.area }), h('td', null, rich(g.detail)), h('td', { text: g.needed })))))) : h('p', { class: 'unk', text: 'None recorded.' })),
    h('section', { class: 'sec', id: 'ev-sources' }, h('h2', { text: `Sources (${D.sources.length})` }),
      h('p', { class: 'cap', text: 'Level says how a fact was seen. api, cli and dns are successful provider reads: positive proof of account or repository metadata, not traffic or health. config covers repository source or config, docs, and records of failed or unattributed probes; pricing is a public price page. Neither proves deployment. Cited by counts the records on this page that cite a source, not independent provider proof.' }),
      h('div', { class: 'toolbar', css: 'padding:0 0 10px;border:0' },
        h('label', null, h('span', { class: 'sr-only', text: 'Search sources' }), h('input', { class: 'input', id: 'src-q', type: 'search', placeholder: 'Search sources  /', autocomplete: 'off', spellcheck: 'false' })),
        h('div', { class: 'group', role: 'group', 'aria-label': 'Source level' }, [['all', 'All', D.sources.length], ...lv].map(([l, label, n]) => h('button', { type: 'button', class: 'chip', 'data-level': l, 'aria-pressed': String(S.srcLevel === l) }, label, h('span', { class: 'c', text: String(n) }))))),
      h('div', { class: 'wide' }, h('table', { id: 'source-table' },
        h('thead', null, h('tr', null, ['ID', 'Method', 'Level', 'Date', 'Observation', 'Link', 'Cited by'].map((t) => h('th', { scope: 'col' }, h('span', { class: 'thl', text: t }))))),
        h('tbody', { id: 'source-body' }, sourceRows())))),
  ];
  E.evDoc.replaceChildren(subnav('Evidence sections', [['ev-coverage', 'Coverage'], ['ev-findings', 'Findings'], ['ev-gaps', 'Gaps'], ['ev-sources', 'Sources']]), h('div', { class: 'doc-inner' }, secs));
}
function flashSource() {
  for (const r of $$('tr.is-flash', E.evDoc)) r.classList.remove('is-flash');
  if (!S.flash) return;
  const row = document.getElementById('src-' + S.flash);
  if (!row) return;
  row.classList.add('is-flash');
  const nav = E.evDoc.querySelector('.subnav');
  scrollWithin(E.evDoc, row, 'center', nav ? nav.offsetHeight : 0);
}

/* ---------- events ---------- */
function toggleFocus() {
  if (!S.selected) return;
  S.focus = !S.focus;
  S.scale = null;
  render();
  E.vp.scrollTop = 0;
  E.vp.scrollLeft = 0;
  announce(S.focus ? `Focused on ${D.byId.get(S.selected).label} and its neighbors.` : 'Showing the full map.');
}
function resetMap() {
  S.kinds = new Set(DEFAULT_KINDS);
  S.longTail = DEFAULT_LONGTAIL;
  S.statuses = new Set(STATUSES_IN);
  S.cat = 'all';
  S.types = new Set(TYPES_IN);
  S.evidence = new Set(TIERS);
  S.query = '';
  E.q.value = '';
  S.focus = false;
  S.scale = null;
  select(null);
}
function focusSearch() {
  const el = S.view === 'map' ? E.q : S.view === 'inventory' ? E.invQ : S.view === 'evidence' ? $('#src-q') : null;
  if (el) { el.focus(); el.select(); }
}
function toggleKeys(open) {
  const show = open == null ? E.keys.hidden : open;
  E.keys.hidden = !show;
  E.keysBtn.setAttribute('aria-expanded', String(show));
}

document.addEventListener('click', (ev) => {
  const t = ev.target.closest('button, a, tr.row');
  if (!t) return;
  if (t.matches('.tab')) { setView(t.dataset.view); return; }
  if (t.dataset.goto) {
    const id = t.dataset.goto;
    if (S.focus && S.selected !== id) S.focus = false;
    S.selected = D.byId.has(id) ? id : null;
    cursorId = S.selected;
    setView('map');
    const b = S.selected && E.nodes.querySelector(`[data-node="${CSS.escape(S.selected)}"]`);
    if (b) b.focus();
    if (S.selected) announce(`${D.byId.get(S.selected).label} selected on the map.`);
    return;
  }
  if (t.dataset.select) {
    const fromDetail = !!t.closest('#detail');
    select(t.dataset.select);
    if (fromDetail) { const title = E.dHead.querySelector('.d-title'); if (title) title.focus(); }
    return;
  }
  if (t.dataset.src) {
    S.flash = t.dataset.src;
    if (!evBuilt) { renderEvidence(); evBuilt = true; }
    S.srcQ = '';
    S.srcLevel = 'all';
    const qi = $('#src-q');
    if (qi) { qi.value = ''; $$('[data-level]', E.evDoc).forEach((c) => c.setAttribute('aria-pressed', String(c.dataset.level === 'all'))); $('#source-body').replaceChildren(...sourceRows()); }
    setView('evidence');
    return;
  }
  if (t.dataset.level) {
    S.srcLevel = t.dataset.level;
    $$('[data-level]', E.evDoc).forEach((c) => c.setAttribute('aria-pressed', String(c.dataset.level === S.srcLevel)));
    $('#source-body').replaceChildren(...sourceRows());
    return;
  }
  if (t.dataset.viewGo) { setView(t.dataset.viewGo); return; }
  if (t.dataset.jump) {
    const s = document.getElementById(t.dataset.jump);
    const box = s && s.closest('.doc');
    const nav = box && box.querySelector('.subnav');
    if (box) scrollWithin(box, s, 'start', nav ? nav.offsetHeight : 0);
    return;
  }
  if (t.dataset.longtail) { S.longTail = !S.longTail; render(); return; }
  if (t.dataset.kind) { const k = t.dataset.kind; if (S.kinds.has(k)) S.kinds.delete(k); else S.kinds.add(k); render(); return; }
  if (t.dataset.status) { const k = t.dataset.status; if (S.statuses.has(k)) S.statuses.delete(k); else S.statuses.add(k); render(); return; }
  if (t.id === 'filters-btn') { const open = t.closest('.toolbar').classList.toggle('is-open'); t.setAttribute('aria-expanded', String(open)); return; }
  if (t.dataset.type) { const k = t.dataset.type; if (S.types.has(k)) S.types.delete(k); else S.types.add(k); render(); return; }
  if (t.dataset.evidence) { const k = t.dataset.evidence; if (S.evidence.has(k)) S.evidence.delete(k); else S.evidence.add(k); render(); return; }
  if (t.dataset.inv) { S.inv.tab = t.dataset.inv; render(); return; }
  if (t.dataset.sort) {
    const cur = S.inv.sort[S.inv.tab];
    S.inv.sort[S.inv.tab] = cur[0] === t.dataset.sort ? [cur[0], -cur[1]] : [t.dataset.sort, 1];
    render();
    const b = E.invWrap.querySelector(`[data-sort="${t.dataset.sort}"]`);
    if (b) b.focus();
    return;
  }
  if (t.dataset.node) { select(t.dataset.node); return; }
  if (t.matches('tr.row')) { select(t.dataset.row); return; }
  const act = t.dataset.act;
  if (act === 'clear') { const id = S.selected; select(null); if (S.view === 'map' && id) { const b = E.nodes.querySelector(`[data-node="${CSS.escape(id)}"]`); if (b) b.focus(); } }
  else if (act === 'focus') toggleFocus();
  else if (act === 'inventory') { S.inv.tab = 'nodes'; setView('inventory'); }
  else if (act === 'reset') resetMap();
  else if (t === E.focusBtn) toggleFocus();
  else if (t.id === 'zoom-in') zoomTo(curScale * 1.2);
  else if (t.id === 'zoom-out') zoomTo(curScale / 1.2);
  else if (t.id === 'zoom-fit') fitMap();
  else if (t.id === 'reset-btn') resetMap();
  else if (t === E.keysBtn) toggleKeys();
});
document.addEventListener('change', (ev) => {
  const t = ev.target;
  if (t === E.catSel) { S.cat = t.value; render(); return; }
  if (t.dataset && t.dataset.invFilter) {
    const key = { 'inv-kind': 'kind', 'inv-status': 'status', 'inv-cat': 'cat', 'inv-type': 'type', 'inv-evidence': 'evidence' }[t.dataset.invFilter];
    S.inv[key] = t.value;
    render();
  }
});
document.addEventListener('input', (ev) => {
  const t = ev.target;
  if (t === E.q) { S.query = t.value; paint(); }
  else if (t === E.invQ) { S.inv.q = t.value; renderInventory(); }
  else if (t.id === 'src-q') { S.srcQ = t.value; $('#source-body').replaceChildren(...sourceRows()); }
});
E.q.addEventListener('keydown', (ev) => {
  if (ev.key !== 'Enter' || !LAY) return;
  const q = S.query.trim().toLowerCase();
  if (!q) return;
  const first = LAY.items.filter((it) => it.n.mhay.includes(q)).sort((a, b) => ['L', 'C', 'R'].indexOf(a.col) - ['L', 'C', 'R'].indexOf(b.col) || a.y - b.y)[0];
  if (!first) return;
  ev.preventDefault();
  select(first.id);
  moveCursor(first.id);
});
E.nodes.addEventListener('keydown', (ev) => {
  const b = ev.target.closest('.node');
  if (!b || !LAY) return;
  if (!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(ev.key)) return;
  ev.preventDefault();
  const it = LAY.byId.get(b.dataset.node);
  const next = it && neighborItem(it, ev.key);
  if (next) moveCursor(next.id);
});
E.nodes.addEventListener('mouseover', (ev) => {
  const b = ev.target.closest('.node');
  const id = b ? b.dataset.node : null;
  if (id !== S.hover) { S.hover = id; paint(); }
});
E.nodes.addEventListener('mouseleave', () => { if (S.hover) { S.hover = null; paint(); } });
E.nodes.addEventListener('focusin', (ev) => {
  const b = ev.target.closest('.node');
  if (!b) return;
  cursorId = b.dataset.node;
  for (const o of E.nodes.querySelectorAll('[tabindex="0"]')) if (o !== b) o.tabIndex = -1;
  b.tabIndex = 0;
  let visible = false;
  try { visible = b.matches(':focus-visible'); } catch (e) { visible = false; }
  if (visible) { S.hover = b.dataset.node; paint(); }
});
E.nodes.addEventListener('focusout', () => { if (S.hover) { S.hover = null; paint(); } });

let drag = null;
E.vp.addEventListener('pointerdown', (ev) => {
  if (ev.pointerType !== 'mouse' || ev.button !== 0 || ev.target.closest('.node, .map-empty')) return;
  drag = { x: ev.clientX, y: ev.clientY, sl: E.vp.scrollLeft, st: E.vp.scrollTop, moved: false, id: ev.pointerId };
});
E.vp.addEventListener('pointermove', (ev) => {
  if (!drag || ev.pointerId !== drag.id) return;
  const dx = ev.clientX - drag.x;
  const dy = ev.clientY - drag.y;
  if (!drag.moved && Math.abs(dx) + Math.abs(dy) > 4) { drag.moved = true; E.vp.classList.add('is-panning'); try { E.vp.setPointerCapture(drag.id); } catch (e) { /* capture is optional */ } }
  if (drag.moved) { E.vp.scrollLeft = drag.sl - dx; E.vp.scrollTop = drag.st - dy; }
});
const endDrag = (ev) => {
  if (!drag) return;
  const clicked = !drag.moved && ev.type === 'pointerup';
  E.vp.classList.remove('is-panning');
  drag = null;
  if (clicked && S.selected && !S.focus) select(null);
};
E.vp.addEventListener('pointerup', endDrag);
E.vp.addEventListener('pointercancel', endDrag);
E.vp.addEventListener('wheel', (ev) => {
  if (!ev.ctrlKey || !LAY) return;
  ev.preventDefault();
  const r = E.vp.getBoundingClientRect();
  zoomTo(curScale * Math.exp(-ev.deltaY * 0.0018), ev.clientX - r.left, ev.clientY - r.top);
}, { passive: false });

$('.tabs').addEventListener('keydown', (ev) => {
  if (ev.key !== 'ArrowRight' && ev.key !== 'ArrowLeft') return;
  const i = VIEWS.indexOf(S.view) + (ev.key === 'ArrowRight' ? 1 : -1);
  const v = VIEWS[(i + VIEWS.length) % VIEWS.length];
  setView(v);
  document.getElementById('tab-' + v).focus();
});
$('.tabs2').addEventListener('keydown', (ev) => {
  if (ev.key !== 'ArrowRight' && ev.key !== 'ArrowLeft') return;
  S.inv.tab = S.inv.tab === 'nodes' ? 'edges' : 'nodes';
  render();
  document.getElementById(S.inv.tab === 'nodes' ? 'inv-tab-nodes' : 'inv-tab-edges').focus();
});
document.addEventListener('focusin', (ev) => { const p = ev.target.closest && ev.target.closest('.pane'); if (p) setActivePane(p); });
document.addEventListener('pointerdown', (ev) => { const p = ev.target.closest && ev.target.closest('.pane'); if (p) setActivePane(p); });
document.addEventListener('keydown', (ev) => {
  const typing = ev.target.matches && ev.target.matches('input, select, textarea');
  if (ev.key === 'Escape') {
    if (!E.keys.hidden) { toggleKeys(false); E.keysBtn.focus(); return; }
    if (typing) { if (ev.target.value) { ev.target.value = ''; ev.target.dispatchEvent(new Event('input', { bubbles: true })); } else ev.target.blur(); return; }
    if (S.focus) { toggleFocus(); return; }
    if (S.selected) { const id = S.selected; select(null); const b = S.view === 'map' && E.nodes.querySelector(`[data-node="${CSS.escape(id)}"]`); if (b) b.focus(); }
    return;
  }
  if (typing || ev.metaKey || ev.ctrlKey || ev.altKey) return;
  if (ev.key >= '1' && ev.key <= '4') { setView(VIEWS[Number(ev.key) - 1]); return; }
  if (ev.key === '/') { ev.preventDefault(); focusSearch(); return; }
  if (ev.key === '?') { toggleKeys(); return; }
  if (S.view !== 'map') return;
  if (ev.key === 'f' || ev.key === 'F') { if (S.selected) toggleFocus(); }
  else if (ev.key === '+' || ev.key === '=') zoomTo(curScale * 1.2);
  else if (ev.key === '-' || ev.key === '_') zoomTo(curScale / 1.2);
  else if (ev.key === '0') fitMap();
});
window.addEventListener('popstate', () => { readHash(); render(); });
if (typeof ResizeObserver !== 'undefined') new ResizeObserver(() => { if (S.view === 'map' && S.scale == null) applyScale(); }).observe(E.vp);

/* ---------- boot ---------- */
E.barState.replaceChildren(D.empty
  ? h('span', { class: 'state-draft', text: D.error ? 'Data error' : 'Unfinished shell, no research data' })
  : `${D.meta.date || 'Undated'}, ${plural(D.nodes.length, 'node')}, ${plural(D.edges.length, 'relationship')}`);
buildMapControls();
if (D.empty) { $('#view-map .toolbar').hidden = true; E.legend.hidden = true; $('#view-inventory .toolbar').hidden = true; }
readHash();
render();
setActivePane(document.getElementById('view-' + S.view));
if (matchMedia('(max-width:760px)').matches && S.view === 'map' && !S.selected && LAY) requestAnimationFrame(() => { const center = LAY.items.find(i => i.col === 'C'); if (center) E.vp.scrollLeft = Math.max(0, center.x * curScale - (E.vp.clientWidth - center.w * curScale) / 2); });
if (D.warn.length) console.warn('infra-data integrity:', D.warn);
// Directory URLs avoid static hosts' index.html redirects; offline files still
// navigate to their actual sibling file. No server/API is needed in either mode.
const scopeHref=file=>file==='index.html'&&/^https?:$/.test(location.protocol)?'./':file;
{const review=document.querySelector('.review-link');if(review)review.href=scopeHref('index.html');}
for(const scopeSelect of [document.getElementById('topo-scope'),document.getElementById('topo-scope-compact')]){
 if(scopeSelect){scopeSelect.replaceChildren(...D.meta.scopes.filter(s=>/^(index|scope-[a-f0-9]{16})\.html$/.test(s.file)).map(s=>h('option',{value:s.file,text:s.label,selected:s.id===D.meta.scopeId})));scopeSelect.addEventListener('change',()=>{location.href=scopeHref(scopeSelect.value);});}
}
const scopeButton=document.getElementById('topo-scope-btn'),scopeDock=document.getElementById('topo-scope-dock');
if(scopeButton&&scopeDock){scopeButton.addEventListener('click',()=>{scopeDock.hidden=!scopeDock.hidden;scopeButton.setAttribute('aria-expanded',String(!scopeDock.hidden));if(!scopeDock.hidden)document.getElementById('topo-scope-compact').focus();});scopeDock.addEventListener('keydown',e=>{if(e.key==='Escape'){scopeDock.hidden=true;scopeButton.setAttribute('aria-expanded','false');scopeButton.focus();e.stopPropagation();}});}
document.querySelector('.bar-title').textContent=D.meta.title||'Topo';
window.__infraMap = { state: S, data: D, layout: () => LAY, warnings: D.warn };
})();
