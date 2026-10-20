# Your first map

Find the dependencies in one Git repository, then open an interactive map.
Everything is collected locally; nothing is uploaded.

## Install once

You need Git, curl, and a supported macOS or Linux computer.
[Platform details and manual downloads](/docs/install/).

```sh
curl -fsSL https://topo.mistystep.io/install.sh -o install-topo.sh
# Read the script before running it.
sh install-topo.sh
export PATH="$HOME/.local/bin:$PATH"
topo --version
```

No sudo, account, model key, Rust, Go, Bun or Node toolchain is needed.

## Make a map

From the root of a Git repository with at least one commit:

```sh
topo run --repo app=. --out "$HOME/topo-maps/app-001"
```

Open `$HOME/topo-maps/app-001/index.html` in your desktop browser. On macOS:

```sh
open "$HOME/topo-maps/app-001/index.html"
```

On Linux, use your file manager or `xdg-open`. Keeping output outside the source
checkout helps avoid accidentally committing the map with your application.

## Find one useful relationship

1. Choose your repository in **Map scope**, then a directory if useful.
2. Select a source file. Read its inspector's **from** and **to** relationships.
3. Click **Focus** to show its known neighborhood. Wires point from a consumer
   to its dependency.
4. Read the source receipt and collection limits before acting on the result.

Use **Fullscreen** to give the workspace more room. Try **Inventory** for a list
instead of a graph. [All map controls](/docs/using-the-map/).

A source import is not proof of a running connection. Every collection reports
partial relationship coverage; an unresolved target is not an absent dependency.

## Keep or share the result

Rerun into a new directory such as `app-002`. Existing output is never overwritten.
To share, keep the complete folder together and collect only intended inputs:
[share a map](/docs/sharing/). A focused view does not redact other scopes.

<details>
<summary>Map more than one repository</summary>

```sh
topo run \
  --repo api="$HOME/work/api" \
  --repo web="$HOME/work/web" \
  --out "$HOME/topo-maps/system-001"
```

The portfolio groups known dependencies and retains shared identities. It maps
only repositories you select, not an automatically discovered organization.
[Collection and scope guide](/docs/collecting/).

</details>

<details>
<summary>No repository handy, or only received a map?</summary>

[Explore the fictional example](/demo/) without installing. If someone sent you
a map folder, open its `index.html`; viewing it needs no Topo installation.

</details>
