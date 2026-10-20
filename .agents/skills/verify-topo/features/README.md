# Topo verification map

This directory is the maintained source for verifying Topo's user-facing
behaviour. Read this index before driving, then follow the matching feature file.

## Baseline preconditions

- `verify-topo setup` has run in this checkout.
- `TOPO_VERIFY_RUN` names a fresh directory for this run.
- `verify-topo doctor` prints no `FAIL`. For sign-in work, `doctor --live` also
  prints no `FAIL`.
- The public site is https://topo.mistystep.io. A private deployment is
  described by `TOPO_VERIFY_PRIVATE_SETTINGS` (the settings JSON from
  `docs/deployment.md`), and its owner by `TOPO_VERIFY_OWNER_EMAIL`. Without
  them, the private checks report `SKIP` with that reason.
- Write journeys use only the fictional `beacon` fixture that `verify-topo cli`
  creates. Never run them against real repositories or private inventories.

## Driving conventions

- Use `verify-topo <command>`. Each command drives its whole journey and records
  evidence. To drive a single step by hand, copy the command from the transcript.
- Browser handles are accessible roles and names, such as tab `2 Inventory`,
  combobox `Map scope`, searchbox `Search nodes` and button `Fullscreen`. A
  few viewer internals (`#detail-head .d-title`, `#map-q-count`) have no
  accessible name.
- Phones (390x844) fold the scope picker behind a `Scope` button. Desktop is
  1440x1000.
- The CLI prints JSON on stdout and diagnostics on stderr. Assert exit codes,
  not just output.

## Proof and skip reporting

- CLI proof is `cli/transcript.txt`, with command, exit, stdout and stderr.
- UI proof is `result.json`, plus a screenshot and an ARIA snapshot for each
  driven surface.
- Owner proof is the session's Access identity plus the private home rendering.
  Anything less is reported as `UNPROVEN`, with the access that is missing.
- Report a feature you could not reach with the command you ran and the
  precondition that failed. Do not count a different entry point as coverage.

## Feature entry contract

Each feature file has an H1 and a one-paragraph summary, then four H2s in this
order: `Sub-features`, `How to get to it (user POV)`, `Driving it with
verify-topo` (starting with `Preconditions:`), and `Gotchas`.

## Features

- [CLI](./cli.md): install, collect, scopes, query, render, project state, and
  the refusals that must not become empty successes.
- [Source browser](./source-browser.md): the exported offline map, with
  inspector, search, inventory, evidence, scopes, fullscreen, phone width,
  dark mode and no network.
- [Private home](./private-home.md): `topo-home` composition from disk, and what
  the deployed owner-only home must show.
- [Public site](./public-site.md): home, docs, quickstart, changelog, demo,
  install script, health and manifest, 404, slow network, phone width.
- [Sign-in](./sign-in.md): the signed-out gate, wrong identity, the owner
  session, and the owner-only page of a self-hosted private deployment.
