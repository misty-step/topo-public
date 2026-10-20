import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';
const require = createRequire(import.meta.url);
const { Miniflare, Response: MFResponse } = createRequire(require.resolve('wrangler/package.json'))('miniflare');
const directory = resolve('private-cloudflare/build');
const modules = Object.fromEntries(await Promise.all((await readdir(directory))
  .filter(name => name.endsWith('.js') || name.endsWith('.wasm'))
  .map(async name => [name, { type: name.endsWith('.wasm') ? 'wasm' : 'esm', contents: await readFile(join(directory, name)) }])));
// Ephemeral local test signing material, never a provider credential or persisted key.
const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
const issuer = 'https://example.cloudflareaccess.com';
const audience = 'example-audience';
const jwk = { ...publicKey.export({ format: 'jwk' }), alg: 'RS256', use: 'sig', kid: 'fixture' };
const claims = { iss: issuer, aud: audience, type: 'app', exp: Math.floor(Date.now() / 1000) + 600 };
function token(values = claims, corrupt = false) {
  const body = [Buffer.from(JSON.stringify({ alg: 'RS256', kid: 'fixture', typ: 'JWT' })).toString('base64url'), Buffer.from(JSON.stringify(values)).toString('base64url')].join('.');
  const signature = sign('RSA-SHA256', Buffer.from(body), privateKey);
  if (corrupt) signature[0] ^= 1;
  return body + '.' + signature.toString('base64url');
}
const cases = [
  { name: 'explicit-host-valid-access', status: 200, host: 'maps.example.test', assertion: token(), assets: 1 },
  { name: 'missing-host', status: 503, configuredHost: null },
  { name: 'empty-host', status: 403, configuredHost: '' },
  { name: 'alternate-host', status: 403, host: 'other.example.test', assertion: token() },
  { name: 'missing-access', status: 403 },
  { name: 'wrong-audience', status: 403, assertion: token({ ...claims, aud: 'other-audience' }) },
  { name: 'expired-access', status: 403, assertion: token({ ...claims, exp: 1 }) },
  { name: 'wrong-signature', status: 403, assertion: token(claims, true) },
  { name: 'unconfigured-issuer', status: 403, assertion: token({ ...claims, iss: 'https://other.cloudflareaccess.com' }) },

  { name: 'missing-configured-issuer', status: 403, missingIssuer: true, assertion: token() },
  { name: 'missing-configured-audience', status: 403, missingAudience: true, assertion: token() },
  { name: 'empty-configured-issuer', status: 403, configuredIssuer: '', assertion: token() },
  { name: 'empty-configured-audience', status: 403, configuredAudience: '', assertion: token() },
  { name: 'whitespace-audience', status: 403, configuredAudience: ' qa ', assertion: token() },
  { name: 'invalid-configured-issuer', status: 403, configuredIssuer: 'https://example.invalid', assertion: token() },
  { name: 'http-host', status: 403, scheme: 'http', assertion: token() },
  { name: 'nondefault-port', status: 403, host: 'maps.example.test:8443', assertion: token() },
  { name: 'post-method', status: 405, method: 'POST', assertion: token() },
  { name: 'unsigned-redirect-forbidden', status: 403, path: '/maps' },
  { name: 'signed-redirect', status: 307, path: '/maps', assertion: token() },
  { name: 'future-nbf', status: 403, assertion: token({...claims, nbf: claims.exp + 100}) },
  { name: 'service-token-type-rejected', status: 403, assertion: token({...claims, type: 'service'}) },
  { name: 'array-audience-accepted', status: 200, assertion: token({...claims, aud: ['unrelated', audience]}), assets: 1 },
  { name: 'array-audience-rejected', status: 403, assertion: token({...claims, aud: ['unrelated']}) },
  { name: 'malformed-jwt', status: 403, assertion: 'not.a.jwt' },
  { name: 'oversized-jwt', status: 403, assertion: 'x'.repeat(32769) },
];
const results = [];
for (const item of cases) {
  let assets = 0, keys = 0;
  const host = item.configuredHost === undefined ? 'maps.example.test' : item.configuredHost;
  const env = {
    ...(item.missingIssuer ? {} : { ACCESS_TEAM_DOMAIN: { type: 'text', value: item.configuredIssuer ?? issuer } }), ...(item.missingAudience ? {} : { ACCESS_AUD: { type: 'text', value: item.configuredAudience ?? audience } }),
    ...(host === null ? {} : { PRIVATE_HOST: { type: 'text', value: host } }),
    ASSETS: { type: 'fetcher', handler: async () => { assets++; return new MFResponse('Fictional private map'); } },
  };
  const mf = new Miniflare({ workers: [{ config: { name: 'private-host-proof', manifest: { mainModule: 'index.js', modules }, compatibilityDate: '2026-10-09', env },
    dev: { outboundService: { type: 'fetcher', handler: async request => {
      assert.equal(request.url, issuer + '/cdn-cgi/access/certs'); keys++;
      return MFResponse.json({ keys: [jwk] });
    } } } } ] });
  try {
    await mf.ready;
    const response = await mf.dispatchFetch((item.scheme ?? 'https') + '://' + (item.host ?? 'maps.example.test') + (item.path ?? '/maps/'), { redirect: 'manual', method: item.method ?? 'GET', headers: item.assertion ? { 'Cf-Access-Jwt-Assertion': item.assertion } : {} });
    const body = await response.text();
    assert.equal(response.status, item.status, item.name);
    assert.equal(assets, item.assets ?? 0, item.name + ': assets');
    assert.equal(response.headers.get('Cache-Control'), 'private, no-store');
    if (response.status !== 200) assert(!body.includes('Fictional private map'));
    results.push({ name: item.name, status: response.status, assets, keys });
  } finally { await mf.dispose(); }
}
const proof = { head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), moduleSha256: createHash('sha256').update(await readFile(join(directory, 'index.js'))).digest('hex'), wasmSha256: createHash('sha256').update(await readFile(join(directory, 'index_bg.wasm'))).digest('hex'), cases: results };
await writeFile(resolve('.cache/private-worker-proof.json'), JSON.stringify(proof, null, 2) + '\n');
console.log(JSON.stringify({ passed: results.length, deniedBeforeAssets: results.filter(x => x.status !== 200 && x.assets === 0).length }));
