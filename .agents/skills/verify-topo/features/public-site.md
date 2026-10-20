# Public site

https://topo.mistystep.io is where a new user learns about Topo, tries the
fictional demo, reads docs, installs the CLI and checks the changelog. It is a
static Rust Worker. It answers only GET and HEAD, and reports its identity at
`/health` and `/manifest.json`.

## Sub-features

- `site-home`: heading `Your software, in view.` with the embedded fictional demo (`iframe` titled `Topo's fictional example map`).
- `site-docs`: `Docs` in `Main navigation` opens `/docs/`. `/docs/quickstart/` shows the `install.sh` command.
- `site-changelog`: `Changelog` opens a page whose article heading is `Changelog`.
- `site-demo`: `/demo/` renders the fictional map. Selecting `docs.beacon.example` opens its inspector.
- `site-identity`: `/health` is `ok`, and its `source` equals `/manifest.json` `source_revision`. The `install.sh` `version` equals `native_version`.
- `site-refusals`: an unknown path returns the 404 `Page not found` page, and POST is refused.
- `site-phone-dark`: home, docs and demo render at 390px and in dark mode without page errors.
- `site-slow-network`: the demo is usable on a throttled phone connection.

## How to get to it (user POV)

- Visit https://topo.mistystep.io and use `Main navigation`: `Demo`, `Docs`, `Install`, `Changelog`.
- Follow `Map your repo` to the quickstart, or `Open map` to the demo.

## Driving it with verify-topo

Preconditions:

- The site answers (`.agents/skills/verify-topo/verify-topo doctor`).
- To drive a local Worker instead, pass its base URL, for example `.agents/skills/verify-topo/verify-topo site http://127.0.0.1:8798`.

- **Identity.** Run `.agents/skills/verify-topo/verify-topo site`. It fetches `/health`, `/manifest.json` and `/install.sh`, then checks status `ok`, matching source revisions, and the installer version against `native_version`.
- **Refusals.** It fetches `/verify-topo-missing-page`, which must return 404 and `Page not found`. A POST to `/` must return 4xx (405 today).
- **Home.** On desktop and phone, in light and dark, it opens `/`. The heading `Your software, in view.` and the demo iframe must be present.
- **Docs and changelog.** In light mode it clicks `Docs` (URL `/docs/`), opens `/docs/quickstart/` (article mentions `install.sh`), and clicks `Changelog` (article `h1` is `Changelog`).
- **Demo.** It opens `/demo/`. On desktop it clicks button `docs.beacon.example. …`, and the inspector title must be `docs.beacon.example`. On a phone, button `Beacon API. …` must be visible.
- **Slow network.** It throttles a phone context through CDP (400 ms latency, 50 KiB/s) and opens `/demo/`. `Beacon API` must become visible.
- **Exhaustive demo.** When the viewer or demo changed, also run `bun tests/demo-browser.mjs https://topo.mistystep.io $TOPO_VERIFY_RUN/evidence/demo-browser` from the repository root. It sweeps every inspector, both palettes, and native and denied fullscreen. CI also runs it against the bundled Worker. Against a live site, its `changelog system palette` assertion can race the page load after the `Changelog` click; if only that assertion fails, check the palette directly before calling it a regression.
- **Proof.** `evidence/site/result.json` (includes the health and manifest identities), `site-<device>-<scheme>-home.png`, the `-quickstart` and `-demo` screenshots, `site-desktop-light-demo.aria.txt`, and `site-slow-network-demo.png`.

## Gotchas

- A deploy changes `/health` `source` first. Compare it with the merged commit you expect before calling a release verified.
- `/install.sh` follows the native release, not every code deploy. The `version` in the script lags `source_revision` by design.
- The demo is frozen fictional data (43 entities, 69 edges). Never expect real infrastructure there. Real names on the public site are a privacy bug.
- The site is a static Worker with no backend, so "backend down" means the Worker or Cloudflare is down. `/health` and the doctor catch that. There is no partial-backend state to drive.
