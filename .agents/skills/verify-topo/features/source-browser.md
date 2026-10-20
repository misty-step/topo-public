# Source browser

The source browser is the map a user opens from disk after `topo run`. It is a
self-contained workspace with no backend and no network. In it the user selects
a node to inspect its evidence, searches, switches between Map, Inventory,
Directions and Evidence views, moves between portfolio, repository and module
scopes, and expands to fullscreen.

## Sub-features

- `map-open`: `index.html` opens from `file://` with the `Views` tablist and the `Map scope` picker.
- `map-inspect`: selecting a node fills the `Details` inspector without literal `null` or `undefined` fragments.
- `map-search`: `Search nodes` narrows the map and reports a match count.
- `map-views`: tab `2 Inventory` lists node rows. Tab `4 Evidence` opens the evidence document.
- `map-scopes`: choosing `Repo · beacon` in `Map scope` opens that scope's sibling `scope-*.html`.
- `map-fullscreen`: `Fullscreen` expands the workspace, and Escape returns.
- `map-phone`: at 390px the scope picker sits behind the `Scope` button and the map still renders.
- `map-dark`: the system dark setting restyles the map with no errors.
- `map-offline`: no request leaves `file://`.

## How to get to it (user POV)

- Open `topo-map/index.html` in a browser after `topo run`, as the quickstart tells users.
- Open any `scope-*.html` directly, or reach it through `Map scope`.
- Keyboard: `1` to `4` switch views, arrow keys move between nodes, `F` focuses, Escape clears.

## Driving it with verify-topo

Preconditions:

- `verify-topo cli` has written `$TOPO_VERIFY_RUN/map/` in the same run.
- Playwright is installed (`verify-topo setup`).

- **Open.** Run `verify-topo browse`. For each of desktop and phone, in light and dark, it opens `map/index.html` from `file://`. The title starts with `Topo`, the `Views` tablist is visible, and `Map scope` lists Project, Repo and Module scopes.
- **Inspect.** On desktop it clicks the first button in group `Infrastructure map`. The inspector title (`#detail-head .d-title`) must be non-empty, and `#detail-body` must have no `null` or `undefined`.
- **Search.** It fills `Search nodes` with `npm`. `#map-q-count` must report matches. Override the query with `TOPO_VERIFY_SEARCH`.
- **Views.** It clicks tab `2 Inventory`. Tabpanel `Inventory table` must have data rows. Then it clicks tab `4 Evidence`, and `#view-evidence` must be visible.
- **Scopes.** It selects `Repo · beacon` in `Map scope`. The page must navigate to a `scope-*.html` where that option is selected.
- **Fullscreen.** It clicks `Fullscreen`, then presses Escape. The workspace must still show its tablist.
- **Phone, dark and offline.** Each phone pass clicks `Scope` before reading the picker. Every pass asserts there were no page errors and no request outside `file:` or `data:`.
- **Proof.** `evidence/browse/result.json`, `map-<device>-<scheme>.png`, and `map-desktop-light.aria.txt`.

## Gotchas

- The scope picker is invisible at phone width until `Scope` is pressed. A hidden-element query returns no options. That is not a bug.
- Choosing a scope is a full page navigation to a sibling file. Wait for the URL, not for `load` on the old page.
- The fixture is small: 3 nodes at portfolio scope. Search, inventory and inspector checks prove behaviour, not scale. For the 43-entity specimen, use `tests/demo-browser.mjs` (see public-site).
- On a 390px phone the `Evidence` tab label is clipped beside `Scope` (it shows "Evidenc"). The tab still works.
