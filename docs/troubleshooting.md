# Troubleshooting

Find the part that failed, fix its access or input, and retry into a fresh output
folder. A failed read never establishes that a dependency does not exist.

## Install and launch

| Symptom | Next action |
| --- | --- |
| `topo: command not found` | Add your selected install directory to PATH; default is `$HOME/.local/bin` |
| Installer says destination exists | Keep the existing file, choose `TOPO_INSTALL_DIR`, or explicitly reinstall with `--force` |
| Download checksum mismatch | Nothing is installed; retry over HTTPS and compare against the published checksum |
| Unsupported platform | Check [supported downloads](/docs/install/); native Windows, Linux ARM64 and Alpine/musl are not available |
| Linux dynamic-loader/glibc error | Use Ubuntu 22.04+ compatible glibc; the GNU build is not a musl executable |
| macOS blocks a downloaded file | This preview is not Apple-notarized; use the documented command-line install path, subject to your organization's policy |

<details>
<summary>Find the repository root</summary>

```sh
git rev-parse --show-toplevel
```

Use that directory with `--repo app=PATH`. Do not collect a nested source directory
as if it were a repository; select its module scope after collecting the Git root.
A newly initialized repository needs at least one commit.

</details>

## Collection and queries

| Result | What it means and what to do |
| --- | --- |
| `Output already exists` | The previous run is retained; choose a new output directory |
| `Expected namespace=path` | Use `--repo app=/path`, with the value quoted when it contains spaces |
| `Git … failed; no absence established` | Check Git/access/root/HEAD; no successful enumeration was established |
| Read, syntax or budget error | Inspect the collection's `errors` and `limits`, repair support/access/size, then recollect |
| Unresolved alias | The collector cannot establish that target; read its receipt and [known gaps](/docs/limitations/) |
| Sparse map in another language | Unsupported source is unknown, not a dependency-free project |
| `Unknown scope` or out-of-scope entity | List scopes and use IDs from this actual run; boundary context depends on the selected scope |
| `Graph revision mismatch` | You requested a different run; inspect the revision rather than silently accepting another |
| `Graph revision does not match its content` | The graph bytes changed; restore the original bundle or recollect, not a fabricated checksum |

## Browser and sharing

If the page is a mail/drive preview, download and extract the complete folder.
Open `index.html`, not a screenshot. Keep sibling scope files together.

Search highlights; it is not a graph filter. If nothing is visible, reset kind,
status, category and relationship filters, then check the collection evidence.
Use Inventory for an accessible list.

**Fullscreen** uses the browser's fullscreen mode when available. When it is
unsupported or denied, the same button expands the map within the page. Exit
with the button or Escape. Browser chrome may remain visible in the fallback.
Older HTML does not gain new controls automatically; use the current CLI's
`render` command into a new directory to upgrade views without recollection.

<details>
<summary>Rendering without access to the original source</summary>

```sh
topo render --graph /path/to/received-map/graph.json \
  --out "$HOME/topo-maps/received-rendered"
```

This uses the received graph's exact revision; its evidence is still dated.
If the graph is damaged, obtain the intact original rather than edit it.

</details>

Native iPhone Safari and managed-browser policies are not covered by desktop
Chromium checks. Prefer the full-page demo or Inventory if an embedded graph
is difficult to navigate. [Privacy and sharing](/docs/privacy/) still apply.
