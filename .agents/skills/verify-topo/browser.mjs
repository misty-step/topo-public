#!/usr/bin/env node
// Browser drives for verify-topo. Invoked by ./verify-topo; usable directly:
//   node .agents/skills/verify-topo/browser.mjs map   INDEX_HTML OUT_DIR
//   node .agents/skills/verify-topo/browser.mjs home  INDEX_HTML OUT_DIR
//   node .agents/skills/verify-topo/browser.mjs site  BASE_URL   OUT_DIR
//   node .agents/skills/verify-topo/browser.mjs gate  HOST_URL   OUT_DIR
//   node .agents/skills/verify-topo/browser.mjs owner HOST_URL   OUT_DIR [--headed] [--password-env NAME]
// Every drive writes OUT_DIR/result.json ({ ok, checks[], ... }) and exits 0 only
// when every check passed. `owner` exits 2 when it stops at the owner gate.
import { createRequire } from 'node:module';
import { mkdir, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';

const repo = process.env.TOPO_VERIFY_REPO || process.cwd();
const { chromium } = createRequire(path.join(repo, 'package.json'))('playwright');
const [mode, target, out, ...flags] = process.argv.slice(2);
if (!mode || !target || !out) {
  console.error('usage: browser.mjs map|home|site|gate|owner TARGET OUT_DIR [flags]');
  process.exit(64);
}
await mkdir(out, { recursive: true });
const flag = name => flags.includes(name);
const option = name => { const i = flags.indexOf(name); return i >= 0 ? flags[i + 1] : undefined; };
const result = { mode, target, startedAt: new Date().toISOString(), checks: [] };
const check = (name, ok, detail = '') => {
  result.checks.push({ name, ok: Boolean(ok), detail: String(detail) });
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` - ${detail}` : ''}`);
  return Boolean(ok);
};
const shot = (page, name) => page.screenshot({ path: path.join(out, `${name}.png`), fullPage: false });
const aria = async (page, name) => writeFile(path.join(out, `${name}.aria.txt`), await page.locator('body').ariaSnapshot());
const DEVICES = [
  ['desktop', { width: 1440, height: 1000 }],
  ['phone', { width: 390, height: 844 }],
];
const launch = (headed = false) => chromium.launch({ headless: !headed, executablePath: process.env.CHROMIUM_EXECUTABLE || undefined });
const context = (browser, device, viewport, colorScheme = 'light') =>
  browser.newContext({ viewport, colorScheme, isMobile: device === 'phone', hasTouch: device === 'phone' });
const watchErrors = page => { const errors = []; page.on('pageerror', e => errors.push(e.message)); return errors; };

// Exported map (topo run / topo render): offline file:// workspace.
async function driveMap(browser) {
  const url = pathToFileURL(path.resolve(target)).href;
  for (const [device, viewport] of DEVICES) {
    for (const scheme of ['light', 'dark']) {
      const name = `map-${device}-${scheme}`;
      const ctx = await context(browser, device, viewport, scheme);
      const network = [];
      ctx.on('request', r => { if (!r.url().startsWith('file:') && !r.url().startsWith('data:')) network.push(r.url()); });
      const page = await ctx.newPage();
      const errors = watchErrors(page);
      await page.goto(url);
      check(`${name}: title names Topo`, /^Topo/.test(await page.title()), await page.title());
      check(`${name}: Views tablist`, await page.getByRole('tablist', { name: 'Views' }).isVisible());
      // Phones fold the scope picker behind a compact `Scope` button.
      if (device === 'phone') await page.getByRole('button', { name: 'Scope' }).click();
      const scope = page.getByRole('combobox', { name: 'Map scope' });
      const scopes = await scope.locator('option').allTextContents();
      check(`${name}: Map scope lists portfolio, repo and module scopes`,
        scopes.some(s => s.startsWith('Project')) && scopes.some(s => s.startsWith('Repo')) && scopes.some(s => s.startsWith('Module')), scopes.join(' | '));
      const nodes = page.getByRole('group', { name: 'Infrastructure map' }).getByRole('button');
      const count = await nodes.count();
      check(`${name}: map renders nodes`, count > 0, `${count} nodes`);
      if (device === 'desktop') {
        // The inspector starts on `Overview`; selecting must replace it with this node's label.
        const before = await page.locator('#detail-head .d-title').innerText().catch(() => '');
        const label = (await nodes.first().getAttribute('aria-label') || await nodes.first().innerText()).split('. ')[0];
        await nodes.first().click();
        await page.waitForFunction(l => document.querySelector('#detail-head .d-title')?.textContent === l, label, { timeout: 5000 }).catch(() => {});
        const title = await page.locator('#detail-head .d-title').innerText().catch(() => '');
        const selected = await page.locator('#detail').getAttribute('data-node');
        check(`${name}: selecting a node opens its inspector`, title === label && title !== before && Boolean(selected), `${before} -> ${title} (${selected})`);
        const body = await page.locator('#detail-body').innerText();
        check(`${name}: inspector has no literal null/undefined fragments`, !/\b(?:null|undefined)\b/.test(body));
        const search = page.getByRole('searchbox', { name: 'Search nodes' });
        const counted = async () => (await page.locator('#map-q-count').innerText().catch(() => '')).trim();
        await search.fill(process.env.TOPO_VERIFY_SEARCH || 'npm');
        await page.waitForFunction(() => /\d+ match/.test(document.querySelector('#map-q-count')?.textContent || ''));
        const hit = await counted();
        check(`${name}: search finds matching nodes`, /^[1-9]\d* match(es)?$/.test(hit), hit);
        await search.fill('verify-topo-no-such-node');
        await page.waitForFunction(() => /^0 matches$/.test((document.querySelector('#map-q-count')?.textContent || '').trim()));
        const miss = await counted();
        check(`${name}: search reports zero matches for an absent node`, miss === '0 matches', miss);
        await search.fill('');
        await page.getByRole('tab', { name: '2 Inventory' }).click();
        const rows = await page.getByRole('tabpanel', { name: 'Inventory table' }).getByRole('row').count();
        check(`${name}: Inventory view lists the nodes`, rows > 1, `${rows - 1} rows`);
        await page.getByRole('tab', { name: '4 Evidence' }).click();
        check(`${name}: Evidence view opens`, await page.locator('#view-evidence').isVisible());
        await page.getByRole('tab', { name: '1 Map' }).click();
        const repoScope = scopes.find(s => s.startsWith('Repo'));
        await Promise.all([page.waitForURL(/scope-[0-9a-f]+\.html/), scope.selectOption({ label: repoScope })]);
        const picked = await page.getByRole('combobox', { name: 'Map scope' }).locator('option:checked').innerText();
        check(`${name}: choosing a scope opens its sibling page`, picked === repoScope, `${path.basename(new URL(page.url()).pathname)} shows ${picked}`);
        await page.goto(url);
        await page.getByRole('button', { name: 'Fullscreen' }).first().click();
        const exit = page.getByRole('button', { name: 'Exit fullscreen' });
        const expanded = await exit.waitFor({ timeout: 5000 }).then(() => exit.getAttribute('aria-pressed')).catch(() => null);
        check(`${name}: Fullscreen expands (control becomes Exit fullscreen, pressed)`, expanded === 'true', String(expanded));
        // Let the browser finish entering fullscreen (native, or the in-page fallback) before Escape.
        await page.waitForFunction(() => document.fullscreenElement || document.querySelector('[data-fullscreen-active], .is-fullscreen'), null, { timeout: 5000 }).catch(() => {});
        await page.waitForTimeout(300);
        await page.keyboard.press('Escape');
        const restored = page.getByRole('button', { name: 'Fullscreen', exact: true }).first();
        const back = await restored.waitFor({ timeout: 5000 }).then(() => restored.getAttribute('aria-pressed')).catch(() => null);
        check(`${name}: Escape leaves fullscreen and keeps the map`, back === 'false'
          && await page.getByRole('tablist', { name: 'Views' }).isVisible(), String(back));
      }
      await shot(page, name);
      if (device === 'desktop' && scheme === 'light') await aria(page, name);
      check(`${name}: no page errors`, errors.length === 0, errors.join('; '));
      check(`${name}: offline - no network requests`, network.length === 0, network.slice(0, 3).join(' '));
      await ctx.close();
    }
  }
}

// Composed private home (topo-home), driven from disk with fictional inputs.
async function driveHome(browser) {
  const url = pathToFileURL(path.resolve(target)).href;
  for (const [device, viewport] of DEVICES) {
    const name = `home-${device}`;
    const ctx = await context(browser, device, viewport);
    const page = await ctx.newPage();
    const errors = watchErrors(page);
    await page.goto(url);
    check(`${name}: title is Topo`, (await page.title()) === 'Topo', await page.title());
    const projects = page.getByRole('link', { name: /^Projects \d+/ });
    check(`${name}: Projects destination present`, await projects.isVisible());
    check(`${name}: Repos coverage reads Partial`, /Partial/.test(await page.locator('main').innerText()));
    await shot(page, name);
    if (device === 'desktop') {
      await aria(page, name);
      await projects.click();
      await page.waitForLoadState('load');
      check(`${name}: Projects page opens from disk`, /projects\/index\.html$/.test(page.url()), path.basename(path.dirname(page.url())));
      await shot(page, `${name}-projects`);
      const sourceMap = page.getByRole('link', { name: 'Repo · partial source map' });
      check(`${name}: project row links to its source map`, await sourceMap.count() === 1);
      if (await sourceMap.count() === 1) {
        await Promise.all([page.waitForURL(/\/maps\/scope-[0-9a-f]+\.html/), sourceMap.click()]);
        check(`${name}: source map opens from the project row`, await page.getByRole('group', { name: 'Infrastructure map' }).getByRole('button').count() > 0,
          path.basename(new URL(page.url()).pathname));
        await shot(page, `${name}-map`);
      }
      await Promise.all([page.waitForURL(u => /\/index\.html$/.test(u.pathname) && !/\/(maps|projects)\//.test(u.pathname)),
        page.getByRole('link', { name: 'Topo home', exact: true }).click()]).catch(() => {});
      check(`${name}: Topo home link returns to the catalogue`, /\/index\.html$/.test(page.url()) && !/\/(maps|projects)\//.test(page.url()), page.url().split('/').slice(-2).join('/'));
    }
    check(`${name}: no page errors`, errors.length === 0, errors.join('; '));
    await ctx.close();
  }
}

// Public site: read-only GETs against production or a local Worker.
async function driveSite(browser) {
  const base = target.replace(/\/$/, '');
  const get = async p => { const r = await fetch(base + p, { redirect: 'manual' }); return { status: r.status, type: r.headers.get('content-type') || '', text: await r.text() }; };
  const health = await get('/health');
  const manifest = await get('/manifest.json');
  const install = await get('/install.sh');
  let h = {}, m = {};
  try { h = JSON.parse(health.text); m = JSON.parse(manifest.text); } catch {}
  result.identity = { health: h, manifest: m };
  check('site: /health is ok', health.status === 200 && h.status === 'ok', `${health.status} ${h.source || ''}`);
  check('site: /health source equals /manifest.json source_revision', h.source && h.source === m.source_revision, `${h.source} vs ${m.source_revision}`);
  const installVersion = (install.text.match(/^version=(\S+)/m) || [])[1];
  check('site: install.sh version equals manifest native_version', installVersion && installVersion === m.native_version, `${installVersion} vs ${m.native_version}`);
  const missing = await get('/verify-topo-missing-page');
  check('site: unknown path returns 404 page', missing.status === 404 && /Page not found/.test(missing.text), String(missing.status));
  const post = await fetch(base + '/', { method: 'POST' });
  check('site: POST is refused', post.status >= 400, String(post.status));
  for (const [device, viewport] of DEVICES) {
    for (const scheme of ['light', 'dark']) {
      const name = `site-${device}-${scheme}`;
      const ctx = await context(browser, device, viewport, scheme);
      const page = await ctx.newPage();
      const errors = watchErrors(page);
      await page.goto(base + '/');
      check(`${name}: home heading`, await page.getByRole('heading', { name: 'Your software, in view.' }).isVisible());
      check(`${name}: demo embed present`, await page.locator('iframe[title="Topo\'s fictional example map"]').count() === 1);
      await shot(page, `${name}-home`);
      if (scheme === 'light') {
        const nav = page.getByRole('navigation', { name: 'Main navigation' });
        await nav.getByRole('link', { name: 'Docs' }).click();
        check(`${name}: Docs opens`, /\/docs\/$/.test(page.url()));
        await page.goto(base + '/docs/quickstart/');
        check(`${name}: quickstart shows the install command`, /install\.sh/.test(await page.locator('article').innerText()));
        await shot(page, `${name}-quickstart`);
        await page.getByRole('navigation', { name: 'Main navigation' }).getByRole('link', { name: 'Changelog' }).click();
        check(`${name}: Changelog renders`, await page.locator('article h1').innerText().then(t => t === 'Changelog').catch(() => false));
        await page.goto(base + '/demo/');
        const entity = page.getByRole('button', { name: /^docs\.beacon\.example\./ });
        if (device === 'desktop') {
          await entity.click();
          check(`${name}: demo inspector opens docs.beacon.example`, (await page.locator('#detail-head .d-title').innerText()) === 'docs.beacon.example');
        } else {
          check(`${name}: demo map renders on phone`, await page.getByRole('button', { name: /^Beacon API\./ }).isVisible());
        }
        await shot(page, `${name}-demo`);
        if (device === 'desktop') await aria(page, `${name}-demo`);
      }
      check(`${name}: no page errors`, errors.length === 0, errors.join('; '));
      await ctx.close();
    }
  }
  // Slow network: home and demo still become usable on throttled Slow-3G-like conditions.
  const ctx = await context(browser, 'phone', DEVICES[1][1]);
  const page = await ctx.newPage();
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Network.emulateNetworkConditions', { offline: false, latency: 400, downloadThroughput: 50 * 1024, uploadThroughput: 20 * 1024 });
  const started = Date.now();
  await page.goto(base + '/demo/', { timeout: 120000 });
  const usable = await page.getByRole('button', { name: /^Beacon API\./ }).isVisible();
  check('site-slow-network: demo usable on throttled phone', usable, `${Date.now() - started} ms`);
  await shot(page, 'site-slow-network-demo');
  await ctx.close();
}

// Signed out: the private host must hand an anonymous visitor to the owner sign-in.
async function driveGate(browser) {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  await page.goto(target, { waitUntil: 'networkidle' });
  const landed = new URL(page.url());
  result.landed = landed.origin + landed.pathname;
  const host = new URL(target).hostname;
  check('gate: anonymous visitor is sent off the private host to sign in', landed.hostname !== host, result.landed);
  check('gate: no private home content is shown', !(await page.getByRole('link', { name: /^Projects \d+/ }).count()));
  const signInHost = process.env.TOPO_VERIFY_SIGN_IN_HOST;
  if (signInHost) check('gate: lands on the configured sign-in host', landed.hostname === signInHost, landed.hostname);
  check('gate: page offers a sign-in form or identity choice', await page.getByRole('textbox').count() > 0
    || await page.getByRole('button').count() > 0);
  await shot(page, 'gate-signed-out');
  await aria(page, 'gate-signed-out');
  // Wrong identity: a forged Access cookie must not reach assets.
  const forged = await fetch(target, { redirect: 'manual', headers: { cookie: 'CF_Authorization=forged.verify-topo.token' } });
  check('gate: forged session cookie is refused', forged.status !== 200, String(forged.status));
  const wrongToken = await fetch(target, { redirect: 'manual', headers: { 'CF-Access-Client-Id': 'verify-topo.access', 'CF-Access-Client-Secret': 'not-a-secret' } });
  check('gate: unknown service token is refused', wrongToken.status !== 200, String(wrongToken.status));
  await ctx.close();
}

// Owner sign-in: a real owner session on the private host, then an owner-only page.
async function driveOwner() {
  const owner = process.env.TOPO_VERIFY_OWNER_EMAIL;
  if (!owner) throw new Error('TOPO_VERIFY_OWNER_EMAIL is required for the owner drive');
  const profile = option('--profile') || process.env.TOPO_VERIFY_OWNER_PROFILE
    || path.join(process.env.HOME, '.cache/tmp/verify-topo/owner-profile');
  await mkdir(profile, { recursive: true, mode: 0o700 });
  const headed = flag('--headed');
  const passwordEnv = option('--password-env');
  const ctx = await chromium.launchPersistentContext(profile, { headless: !headed, viewport: { width: 1440, height: 1000 },
    executablePath: process.env.CHROMIUM_EXECUTABLE || undefined });
  const page = ctx.pages()[0] || await ctx.newPage();
  const host = new URL(target).hostname;
  await page.goto(target, { waitUntil: 'networkidle' });
  result.owner = owner;
  if (new URL(page.url()).hostname !== host) {
    await shot(page, 'owner-sign-in-page');
    const password = passwordEnv ? process.env[passwordEnv] : undefined;
    if (password) {
      await page.getByRole('textbox', { name: 'Email' }).fill(owner);
      await page.getByRole('textbox', { name: 'Password' }).fill(password);
      await page.getByRole('button', { name: 'Sign in' }).click();
      const proceed = page.getByRole('button', { name: 'Continue' });
      await Promise.race([page.waitForURL(u => new URL(u).hostname === host, { timeout: 60000 }),
        proceed.waitFor({ timeout: 60000 }).then(() => proceed.click())]).catch(() => {});
    } else if (headed) {
      console.log(`Waiting up to 10 minutes for ${owner} to finish signing in in the opened browser...`);
    } else {
      result.unproven = `stopped at ${new URL(page.url()).origin}/sign-in: no owner credential supplied `
        + '(pass --password-env NAME, or --headed for the owner to type it)';
      check('owner: session established', false, result.unproven);
      await ctx.close();
      return 2;
    }
    await page.waitForURL(u => new URL(u).hostname === host, { timeout: headed ? 600000 : 60000 }).catch(() => {});
  }
  check('owner: browser is on the private host', new URL(page.url()).hostname === host, new URL(page.url()).origin);
  const identity = await page.evaluate(async () => {
    const r = await fetch('/cdn-cgi/access/get-identity', { credentials: 'include' });
    return r.ok ? r.json() : { status: r.status };
  }).catch(e => ({ error: e.message }));
  result.identity = { email: identity.email, idp: identity.idp && { type: identity.idp.type }, status: identity.status };
  check('owner: Access identity is the real owner', identity.email === owner, identity.email || JSON.stringify(result.identity));
  const idpType = process.env.TOPO_VERIFY_OWNER_IDP_TYPE;
  if (idpType) check(`owner: signed in through the ${idpType} login method`, identity.idp && identity.idp.type === idpType, identity.idp && identity.idp.type);
  // Owner-only page. Record structure only; the private maps are personal content.
  check('owner: private home renders', (await page.title()) === 'Topo'
    && await page.getByRole('link', { name: /^Projects \d+/ }).isVisible(), await page.title());
  const manifest = await page.evaluate(async () => (await fetch('/manifest.json')).json()).catch(() => ({}));
  result.home = { schema: manifest.schema, generated_at: manifest.generated_at, source_revision: manifest.source_revision,
    scopes: Array.isArray(manifest.pages) ? manifest.pages.length : undefined };
  check('owner: private manifest is a topo.home.v1 bundle', manifest.schema === 'topo.home.v1', manifest.schema);
  await ctx.close();
  return 0;
}

const browser = mode === 'owner' ? null : await launch();
let code = 0;
try {
  if (mode === 'map') await driveMap(browser);
  else if (mode === 'home') await driveHome(browser);
  else if (mode === 'site') await driveSite(browser);
  else if (mode === 'gate') await driveGate(browser);
  else if (mode === 'owner') code = await driveOwner();
  else { console.error(`unknown mode ${mode}`); code = 64; }
} catch (error) {
  check(`${mode}: drive completed`, false, error.message.split('\n')[0]);
} finally {
  if (browser) await browser.close();
}
result.finishedAt = new Date().toISOString();
result.ok = result.checks.length > 0 && result.checks.every(c => c.ok);
await writeFile(path.join(out, 'result.json'), JSON.stringify(result, null, 2) + '\n');
process.exit(code || (result.ok ? 0 : 1));
