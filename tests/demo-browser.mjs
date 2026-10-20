// US-005/011: exercise the emitted demo through browser controls, never controller internals.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { chromium, expect } from 'playwright/test';

const fixture = html => html.split('<script type="application/json" id="infra-data">')[1].split('</script>')[0];
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const light = page => page.evaluate(() => getComputedStyle(document.body).backgroundColor.match(/\d+/g).slice(0, 3).reduce((s, v) => s + +v, 0) > 382);
const state = page => page.evaluate(() => ({
  selected: document.querySelector('#detail').dataset.node,
  focus: document.querySelector('#focus-btn').getAttribute('aria-pressed'),
  query: document.querySelector('#map-q').value,
  scale: document.querySelector('#map-scale').style.transform,
  hash: location.hash,
}));

export async function verifyDemo(base, output) {
  await mkdir(output, { recursive: true });
  const reference = await readFile('design/reference.html', 'utf8');
  const response = await fetch(new URL('/demo/', base));
  assert.equal(response.status, 200);
  const html = await response.text();
  assert.equal(fixture(html), fixture(reference), 'Exact frozen fixture bytes');
  const data = JSON.parse(fixture(html));
  assert.equal(data.nodes.length, 43);
  assert.equal(data.edges.length, 69);
  const sample = data.nodes.find(n => n.label === 'docs.beacon.example');
  assert(sample, 'Original issue reproduction entity');
  const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_EXECUTABLE });
  const report = { base, manifest: await (await fetch(new URL('/manifest.json', base))).json(),
    browser: browser.version(), fixtureSha256: sha256(fixture(html)), demoSha256: sha256(html), cases: [] };
  try {
    for (const [device, viewport] of [['desktop', { width: 1440, height: 1000 }], ['phone', { width: 390, height: 844 }]]) {
      for (const colorScheme of ['light', 'dark']) {
        const name = `${device}-${colorScheme}`;
        const context = await browser.newContext({ viewport, colorScheme, isMobile: device === 'phone', hasTouch: device === 'phone' });
        const page = await context.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        let checks = 0;
        async function inspector(node) {
          await expect(page.locator('#detail')).toHaveAttribute('data-node', node.id);
          await expect(page.locator('#detail-head .d-title')).toHaveText(node.label);
          const body = page.locator('#detail-body');
          assert(!/\b(?:null|undefined|false)\b|nullnull/.test(await body.innerText()), `${name}: ${node.id} literal optional fragment`);
          assert.deepEqual(await body.evaluate(el => [...el.childNodes].filter(n => n.nodeType === Node.TEXT_NODE && n.textContent.trim()).map(n => n.textContent)), [], `${node.id}: stray top-level text`);
          const relationships = data.edges.filter(e => e.from === node.id || e.to === node.id).length;
          await expect(body.getByRole('heading', { name: `Relationships (${relationships})`, exact: true })).toBeVisible();
          checks++;
        }
        try {
          // US-014: anonymous public notes keep text, not private source links.
          assert.equal((await page.goto(new URL('/docs/', base).href)).status(), 200);
          await page.getByRole('navigation', { name: 'Main navigation', exact: true }).getByRole('link', { name: 'Changelog' }).click();
          await expect(page.locator('article h1')).toHaveText('Changelog');
          const notes = await page.locator('article').innerText();
          const privateLinks = await page.locator('article a').evaluateAll(links => links
            .map(link => link.href).filter(url => /^https:\/\/github\.com\/misty-step\/topo(?:[/?#]|$)/i.test(url)));
          assert.deepEqual(privateLinks, [], `${name}: no inaccessible source links`);
          assert.equal(await light(page), colorScheme === 'light', `${name}: changelog system palette`);
          assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `${name}: changelog fits`);
          assert.equal((await context.request.post(new URL('/changelog/', base).href)).status(), 405);
          assert.equal((await page.goto(new URL('/absent-public-note/', base).href)).status(), 404);
          await page.getByRole('link', { name: 'documentation', exact: true }).click();
          await page.getByRole('navigation', { name: 'Main navigation', exact: true }).getByRole('link', { name: 'Changelog' }).click();
          assert.equal(await page.locator('article').innerText(), notes, `${name}: notes recover unchanged`);
          await page.screenshot({ path: `${output}/${name}-changelog.png` });
          assert.equal((await page.goto(new URL('/demo/#inventory', base).href)).status(), 200);
          for (const node of data.nodes) {
            await page.locator('#inv-q').fill(node.label);
            await page.locator(`#node-table [data-select="${node.id}"]`).click();
            await inspector(node);
            await page.locator('#detail-body [data-goto]').click();
            await inspector(node);
            await page.locator('#detail-body [data-act="inventory"]').click();
            // On phones the inspector is a sheet over the table; dismiss it before choosing another row.
            await page.locator('#detail-head [data-act="clear"]').click();
          }
          await page.locator('#inv-q').fill(sample.label);
          await page.locator(`#node-table [data-select="${sample.id}"]`).click();
          await page.locator('#detail-body [data-goto]').click();
          await page.locator('#focus-btn').click();
          await expect(page.locator('#focus-btn')).toHaveAttribute('aria-pressed', 'true');
          await page.locator('#map-q').fill('beacon');
          await expect(page.locator('#map-q-count')).toContainText('match');
          await page.locator('#zoom-in').click();
          const retained = await state(page);
          // The system setting picks the palette; changing it keeps the open map's state.
          assert.equal(await light(page), colorScheme === 'light', `${name}: system palette`);
          await page.emulateMedia({ colorScheme: colorScheme === 'light' ? 'dark' : 'light' });
          assert.equal(await light(page), colorScheme !== 'light', `${name}: palette follows a system change`);
          assert.deepEqual(await state(page), retained, `${name}: theme change preserves state`);
          await page.emulateMedia({ colorScheme });
          const toggle = page.locator('[data-fullscreen-toggle]');
          const fullscreen = [];
          for (const mode of ['native', 'denied']) {
            if (mode === 'denied') await page.evaluate(() => {
              Element.prototype.requestFullscreen = () => Promise.reject(new DOMException('Deliberate browser-proof denial', 'NotAllowedError'));
            });
            await toggle.click();
            await expect(toggle).toHaveAttribute('aria-pressed', 'true');
            if (mode === 'native') await page.waitForFunction(() => !!document.fullscreenElement);
            else assert.equal(await page.evaluate(() => !!document.fullscreenElement), false);
            assert.deepEqual(await state(page), retained, `${name}/${mode}: entry preserves state`);
            await page.locator('#detail-body').evaluate(el => { el.scrollTop = 0; });
            await page.screenshot({ path: `${output}/${name}-${mode}.png` });
            if (mode === 'native') await toggle.click();
            else await page.keyboard.press('Escape');
            await expect(toggle).toHaveAttribute('aria-pressed', 'false');
            await page.waitForFunction(() => !document.fullscreenElement);
            assert.equal(await toggle.evaluate(el => document.activeElement === el), true);
            assert.deepEqual(await state(page), retained, `${name}/${mode}: exit preserves state`);
            fullscreen.push(mode);
          }
          await page.locator('#focus-btn').click();
          await expect(page.locator('#focus-btn')).toHaveAttribute('aria-pressed', 'false');
          const node = page.locator(`#map-nodes [data-node="${sample.id}"]`);
          await node.focus();
          await page.keyboard.press('ArrowDown');
          assert.notEqual(await page.evaluate(() => document.activeElement.dataset.node), sample.id);
          assert.equal((await state(page)).selected, sample.id, 'Traversal does not silently change selection');
          await page.locator('#detail-head [data-act="clear"]').click();
          await expect(page.locator('#detail')).toHaveAttribute('data-node', '');
          assert.equal(await node.evaluate(el => document.activeElement === el), true, 'Close restores selected-node focus');
          // Home embeds the same demo. Expansion must retain the iframe and isolate outside controls.
          await page.goto(base);
          assert.equal(await light(page), colorScheme === 'light', `${name}: home palette`);
          const frame = page.frameLocator('iframe');
          await frame.locator('#map-q').fill('beacon');
          await frame.locator('#map-q').press('Enter');
          const embeddedSelection = await frame.locator('#detail').getAttribute('data-node');
          assert(embeddedSelection);
          await expect(frame.locator('[data-fullscreen-toggle]')).toBeHidden();
          const hostToggle = page.locator('[data-fullscreen-toggle]');
          for (const mode of ['native', 'denied']) {
            if (mode === 'denied') await page.evaluate(() => {
              Element.prototype.requestFullscreen = () => Promise.reject(new DOMException('Deliberate browser-proof denial', 'NotAllowedError'));
            });
            await hostToggle.click();
            await expect(hostToggle).toHaveAttribute('aria-pressed', 'true');
            if (mode === 'native') await page.waitForFunction(() => !!document.fullscreenElement);
            assert.equal(await page.locator('.intro').evaluate(el => el.inert), true);
            await frame.locator('#map-q').focus();
            await frame.locator('#map-q').press('Escape');
            await expect(hostToggle).toHaveAttribute('aria-pressed', 'false');
            await expect(frame.locator('#detail')).toHaveAttribute('data-node', embeddedSelection);
            await expect(frame.locator('#map-q')).toHaveValue('beacon');
            assert.equal(await page.locator('.intro').evaluate(el => el.inert), false);
            assert.equal(await hostToggle.evaluate(el => document.activeElement === el), true);
          }
          assert.deepEqual(errors, [], 'No uncaught browser errors');
          report.cases.push({ name, inspectors: checks, fullscreen, embeddedFullscreen: ['native', 'denied'], errors });
        } catch (error) {
          await page.screenshot({ path: `${output}/${name}-failure.png` });
          throw error;
        } finally { await context.close(); }
      }
    }
    await writeFile(`${output}/results.json`, JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report));
    return report;
  } finally { await browser.close(); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert(process.argv[2], 'Usage: bun tests/demo-browser.mjs ORIGIN [OUTPUT_DIRECTORY]');
  await verifyDemo(process.argv[2], process.argv[3] || '.cache/browser-proof');
}
