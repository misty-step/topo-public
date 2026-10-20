# Source browser

The source browser is the map a user opens from disk after `topo run`. It is a
self-contained workspace with no backend and no network. In it the user selects
a node to inspect its evidence, searches, switches between Map, Inventory,
Directions and Evidence views, moves between portfolio, repository and module
scopes, and expands to fullscreen.

## Sub-features

- `map-open`: `index.html` opens from `file://` with the `Views` tablist and the `Map scope` picker.
- `map-inspect`: selecting a node replaces the `Overview` inspector with that node's label, without literal `null` or `undefined` fragments.
- `map-search`: `Search nodes` reports a non-zero match count for a present node and `0 matches` for an absent one.
- `map-views`: tab `2 Inventory` lists node rows. Tab `4 Evidence` opens the evidence document.
- `map-scopes`: choosing `Repo · example/beacon` in `Map scope` opens that scope's sibling `scope-*.html`.
- `map-fullscreen`: `Fullscreen` becomes a pressed `Exit fullscreen`, and Escape restores an unpressed `Fullscreen`.
- `map-phone`: at 390px the scope picker sits behind the `Scope` button and the map still renders.
- `map-dark`: the system dark setting restyles the map with no errors.
- `map-offline`: no request leaves `file://`.

## How to get to it (user POV)

- Open `topo-map/index.html` in a browser after `topo run`, as the quickstart tells users.
- Open any `scope-*.html` directly, or reach it through `Map scope`.
- Keyboard: `1` to `4` switch views, arrow keys move between nodes, `F` focuses, Escape clears.

## Driving it with verify-topo

Preconditions:

- `.agents/skills/verify-topo/verify-topo cli` has written `$TOPO_VERIFY_RUN/map/` in the same run.
- Playwright is installed (`.agents/skills/verify-topo/verify-topo setup`).

- **Open.** Run `.agents/skills/verify-topo/verify-topo browse`. For each of desktop and phone, in light and dark, it opens `map/index.html` from `file://`. The title starts with `Topo`, the `Views` tablist is visible, and `Map scope` lists Project, Repo and Module scopes.
- **Inspect.** On desktop it clicks the first button in group `Infrastructure map`. The inspector title (`#detail-head .d-title`) must change from `Overview` to that node's label, `#detail` must carry its `data-node`, and `#detail-body` must have no `null` or `undefined`.
- **Search.** It fills `Search nodes` with `npm`. `#map-q-count` must read `N match` or `N matches` with N of at least 1. Then it searches `verify-topo-no-such-node`, which must read `0 matches`. Override the first query with `TOPO_VERIFY_SEARCH`.
- **Views.** It clicks tab `2 Inventory`. Tabpanel `Inventory table` must have data rows. Then it clicks tab `4 Evidence`, and `#view-evidence` must be visible.
- **Scopes.** It selects the `Repo · …` option in `Map scope`. The page must navigate to a `scope-*.html` where that option is selected.
- **Fullscreen.** It clicks `Fullscreen`. The control must become `Exit fullscreen` with `aria-pressed="true"`. After Escape it must return to `Fullscreen` with `aria-pressed="false"`, and the tablist must remain.
- **Phone, dark and offline.** Each phone pass clicks `Scope` before reading the picker. Every pass asserts there were no page errors and no request outside `file:` or `data:`.
- **Proof.** `evidence/browse/result.json`, `map-<device>-<scheme>.png`, and `map-desktop-light.aria.txt`.

## Gotchas

- The scope picker is invisible at phone width until `Scope` is pressed. A hidden-element query returns no options. That is not a bug.
- Choosing a scope is a full page navigation to a sibling file. Wait for the URL, not for `load` on the old page.
- The fixture is small: 3 nodes at portfolio scope. Search, inventory and inspector checks prove behaviour, not scale. For the 43-entity specimen, use `tests/demo-browser.mjs` (see public-site).
- On a 390px phone the `Evidence` tab label is clipped beside `Scope` (it shows "Evidenc"). The tab still works.
