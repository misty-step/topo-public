---
name: verify-topo
description: Drive Topo end to end the way its users do. Covers the installed CLI (install, collect, scopes, query, render, state), the exported map browser opened from disk, the composed private home, the public site at topo.mistystep.io, and owner sign-in to a self-hosted private maps deployment. Use it to prove a Topo change works, to check a deployment, or when something about Topo looks wrong.
---

# Verify Topo

Topo is a native Rust CLI. It writes offline HTML maps plus a JSON graph. The
public site (https://topo.mistystep.io) serves the installer, docs and a
fictional demo. An operator can also self-host an owner-only private maps
Worker (`topo-maps`) behind Cloudflare Access, configured as in
[docs/deployment.md](../../../docs/deployment.md). Everything here runs from a
shell. One helper drives every journey:

```sh
.agents/skills/verify-topo/verify-topo <command>
```

Run it from the repository root, or from anywhere by path. It finds the checkout
from its own location. Browser drives use Playwright from this checkout's locked
`node_modules`, through `browser.mjs` in the same directory.

## Launch

There is no long-lived server to start. The CLI is short-lived. Every drive uses
its own run directory, browser context and fixture.

1. Once per checkout, run `.agents/skills/verify-topo/verify-topo setup`. It runs
   `bun install --frozen-lockfile` and `bunx --no-install playwright install chromium`.
   Set `CHROMIUM_EXECUTABLE` to use an already-installed Chromium instead.
2. Pick a run directory: `export TOPO_VERIFY_RUN=~/.cache/tmp/verify-topo/$(date -u +%Y%m%dT%H%M%SZ)`.
   The default is `~/.cache/tmp/verify-topo/default`. Two concurrent runs must use
   different directories. The helper marks a run directory with
   `.verify-topo-run`. It refuses a directory that is non-empty and unmarked.
   It also refuses your home, the runs root `~/.cache/tmp/verify-topo`, the
   checkout, and anything containing the checkout or the owner profile,
   because cleanup empties the run directory. A symlinked run directory is
   resolved to its target. Nothing else is shared between runs except the owner
   browser profile (see sign-in).
3. Ready means `.agents/skills/verify-topo/verify-topo doctor` prints no `FAIL`.

The CLI journey installs the published executable from the public site into
`$TOPO_VERIFY_RUN/bin` (`TOPO_INSTALL_DIR`, never `~/.local/bin`). To prove an
unmerged change, use `cli --source`. It drives `target/debug/topo`,
built with `cargo build --locked -j 2 --bin topo --bin topo-home`.
`browse` always builds `topo-home` from this checkout, because the installer does
not ship it.

## Doctor

```sh
.agents/skills/verify-topo/verify-topo doctor
# Private deployment checks: point at your settings JSON and name the owner.
export TOPO_VERIFY_PRIVATE_SETTINGS=/path/outside/checkout/private-settings.json
export TOPO_VERIFY_OWNER_EMAIL=owner@example.test
# Inject a read-only Cloudflare API token and the publishing service token
# ({"client_id":…,"client_secret":…}) from your secret manager, never inline:
CLOUDFLARE_API_TOKEN=… TOPO_MAPS_ACCESS_CREDENTIAL=… \
  .agents/skills/verify-topo/verify-topo doctor --live
```

Doctor is read-only. Without `--live` it checks:

- local tools and Playwright;
- the checkout revision;
- that public `/health` is `ok` and agrees with `/manifest.json`, and whether
  the deployed source is in this checkout's history;
- with `TOPO_VERIFY_PRIVATE_SETTINGS` set, that the private host
  (`vars.PRIVATE_HOST`) sends anonymous visitors to `vars.ACCESS_TEAM_DOMAIN`.
  Without it, every private check reports `SKIP` with the reason.

`--live` adds the owner-identity configuration:

- the Access application for the private host has the audience `vars.ACCESS_AUD`;
- its allow rules admit exactly the address set in `TOPO_VERIFY_ALLOWED_EMAILS`.
  That is a comma-separated list of addresses or sha256 digests of their
  lowercase form; the default is the owner only, and the owner must be in the
  set. Digests of email addresses are guessable, so never commit a real list.
  It fails on any everyone, domain or group rule, on any `require` or
  `exclude` condition or `deny` policy, on a
  non-service-token non-identity rule, and on any bypass policy;
- it allows exactly one login method, named `TOPO_VERIFY_OWNER_IDP` when that
  is set. An empty list, which admits every method, fails;
- the most recent owner login to that host in the Access log.

It also reads the private manifest with the publishing service token. That proves
the deployed bundle exists. It is not owner proof. Run doctor first, and again
after any failed drive.

## Drive

| Command | Journey | Feature file |
| --- | --- | --- |
| `cli [--source]` | install, collect, scopes, query, render, state, and every refusal | [cli](features/cli.md) |
| `browse` | exported map from disk (desktop/phone, light/dark, offline), composed home | [source-browser](features/source-browser.md), [private-home](features/private-home.md) |
| `site [BASE_URL]` | public site, docs, demo, changelog, health, 404, slow network | [public-site](features/public-site.md) |
| `sign-in [--headed] [--password-env NAME]` | signed-out gate, wrong identity, owner session, owner-only page | [sign-in](features/sign-in.md) |
| `all [--source] [--live] [--headed] [--password-env NAME]` | doctor, cli, browse, site, sign-in in order, each flag passed to its step | all |

Run each command as `.agents/skills/verify-topo/verify-topo <command>`.
The helper is not on `PATH`. Each command prints one `PASS`, `FAIL`, `SKIP`, `INFO` or `UNPROVEN` line per
check. It exits non-zero if anything did not pass. `browse` needs the map that
`cli` wrote in the same `TOPO_VERIFY_RUN`. Read [features/README.md](features/README.md)
before claiming coverage. A proof covers only the entry points that were driven.

## Evidence

Public journeys write to `$TOPO_VERIFY_RUN/evidence/`. Sign-in writes to
`$TOPO_VERIFY_RUN/private-evidence/` instead:

- `cli/transcript.txt`: every command, its exit code, the head of stdout, and
  stderr. Full `cli/<step>.stdout` and `.stderr` sit beside it.
- `browse/`, `browse-home/`, `site/`, and `private-evidence/sign-in/` with `owner/`: each has a
  `result.json` (every check with pass/fail and detail), PNG screenshots per
  device and colour scheme, and `*.aria.txt` accessibility snapshots.

Proof standards:

- Drive the user path: the published installer, real flags, real clicks on
  accessible names. Do not use internal setters or test-only endpoints.
- Capture the action and the state it produced. A CLI step records its command
  and output. A map check selects a node and then reads the inspector.
- Check side effects, such as files written (`index.html`, `graph.json`,
  `manifest.json`), revisions kept across `render`, and ignored `.env` bytes
  absent from outputs.
- Failure states must fail loudly. Every refusal step asserts exit 1 and its
  message.
- Owner proof is the Access identity of the browser session plus the owner-only
  page. An HTTP 200 for a service token, a redirect, or a sent email is not owner
  proof.
- Private maps are personal content. The owner drive records only identity,
  manifest schema, timestamps and counts. Never screenshot or commit private map
  pages. Committed evidence goes under `docs/evidence/verify-topo/<date>/`.
- This repository is public. Commit only `evidence/` and printed output from
  runs without private settings. With `TOPO_VERIFY_PRIVATE_SETTINGS` set, the
  printed output of `doctor`, `sign-in` and `all` names the operator's Access
  team, private host and owner. Never commit that output, or
  `private-evidence/`. Replace
  absolute home paths with `~` before committing, and keep operator proof in
  the operator's own private records.

## Cleanup

```sh
.agents/skills/verify-topo/verify-topo cleanup            # keeps $TOPO_VERIFY_RUN/evidence
.agents/skills/verify-topo/verify-topo cleanup --profile  # also drops the owner browser profile
```

Cleanup removes the installed binary, fixture repositories, maps, the home and
scratch files in `$TOPO_VERIFY_RUN`. It never removes `evidence/` or
`private-evidence/`. Copy the sign-in proof in `private-evidence/` to the
operator's private records before you delete the run directory. Every
browser the helper starts closes when its drive ends. No server is started, so
there is nothing to kill. The owner profile at
`~/.cache/tmp/verify-topo/owner-profile` holds an Access session for the
application's session duration. Keep it to re-prove within that window. Set `TOPO_VERIFY_OWNER_PROFILE` to use another
profile directory. `cleanup --profile` removes only a directory under
`~/.cache/tmp/verify-topo` or one named `*profile*`. Remove it with `--profile` when done.

## Limits

- Production is read-only: GET and HEAD to the public site, one refused POST,
  and an owner sign-in, which creates only an Access session. Never deploy or
  roll back from this skill.
- Write journeys (`run`, `render`, `state`, `topo-home`) use the fictional
  fixture under `$TOPO_VERIFY_RUN`. Never point them at real private inputs.
- [`tests/demo-browser.mjs`](../../../tests/demo-browser.mjs) is the exhaustive
  demo sweep: all 43 inspectors, fullscreen, both palettes. Run it with
  `bun tests/demo-browser.mjs https://topo.mistystep.io $TOPO_VERIFY_RUN/evidence/demo-browser`
  when the demo or viewer changes. CI's full proof also runs it against the
  bundled Worker. Against a live site, its `changelog system palette`
  assertion can race the page load.
