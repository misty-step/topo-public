# Private home

The private home is a catalogue composed by `topo-home` from a project-state
export and map bundles. It leads with typed Infra maps, then Projects and Repos,
with freshness and partial-coverage labels. The owner reads it on their
self-hosted private host after signing in. The same files also work from
disk.

## Sub-features

- `home-compose`: `topo-home --state DIR --maps DIR --out NEW_DIR` writes `index.html`, `catalogue.json`, `manifest.json`, `maps/` and `projects/`.
- `home-open`: the catalogue titled `Topo` shows `Projects N` and Repos `Partial` coverage.
- `home-projects`: `Projects` opens `projects/index.html`. A project row's `Repo · partial source map` opens its map under `maps/`, and that page's `Topo home` link returns to the catalogue, from disk and when hosted.
- `home-phone`: the catalogue renders at phone width without errors.
- `home-deployed`: the deployed bundle is a `topo.home.v1` manifest with a generation time and source revision.

## How to get to it (user POV)

- The owner opens their private host (`vars.PRIVATE_HOST`) and signs in (see sign-in).
- An operator composes one locally with `topo-home` and opens `index.html` from disk.

## Driving it with verify-topo

Preconditions:

- `.agents/skills/verify-topo/verify-topo cli` ran in this `TOPO_VERIFY_RUN`, so `map/` and `state-out/` exist.
- `cargo` can build `topo-home` from this checkout.

- **Compose.** Run `.agents/skills/verify-topo/verify-topo browse`. After the map drive, it builds `target/debug/topo-home` and composes `$TOPO_VERIFY_RUN/home` from the fictional state and map. The step must print `Private catalogue composed`.
- **Open.** On desktop and phone, it opens `home/index.html`. The title must be `Topo`, link `Projects 1` must be visible, and the main region must read `Partial`.
- **Navigate.** On desktop it clicks `Projects 1`. The URL must end `projects/index.html`. It then clicks the `example/beacon` row's `Repo · partial source map`. A `maps/scope-*.html` page must open with map nodes. Finally it clicks `Topo home`, which must return to the catalogue `index.html`.
- **Deployed bundle.** With `TOPO_VERIFY_PRIVATE_SETTINGS` set, `.agents/skills/verify-topo/verify-topo doctor --live` reads `/manifest.json` from the private host with the publishing service token. Expect schema `topo.home.v1`, `generated_at`, and `source_revision`. The owner drive in sign-in re-reads it inside the owner's session.
- **Proof.** `evidence/browse-home/result.json`, `home-desktop.png`, `home-desktop-projects.png`, `home-desktop-map.png`, `home-phone.png`, and `home-desktop.aria.txt`.

## Gotchas

- The fixture has no infrastructure snapshot, so the local home shows no Infra tiles. Infra tiles need a `topo.infrastructure.v1` input, which is private. Do not compose one from real data for verification.
- The deployed home is only as fresh as the operator's last `topo-regenerate` and publish. An old `generated_at` points at the operator's publishing job, not the Worker.
- Never screenshot or commit the deployed home. It lists private repositories and infrastructure.
- A service-token read proves the bundle is served, not that the owner can sign in.
