# Install Topo

Topo {{version}} is a local-first early preview. The collector and exporter are Rust;
Cloudflare serves the site and downloads. The browser renderer preserves the
approved map workspace. Private repositories and generated maps stay local.

## Supported downloads

| Computer | Target |
| --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Linux x86_64 | `x86_64-unknown-linux-gnu` |

Linux builds target Ubuntu 22.04's glibc compatibility floor. Alpine/musl,
Linux ARM64 and native Windows are not supported by this release. On Windows,
run the Linux version inside WSL2 and scan a checkout in that environment.

You need Git to collect a repository. You do not need Rust, Bun, Node, Go,
provider credentials or a model key. The installer also needs curl and
`sha256sum` or `shasum`.

## Installer

```sh
curl -fsSL https://topo.mistystep.io/install.sh -o install-topo.sh
# Inspect the script.
sh install-topo.sh
export PATH="$HOME/.local/bin:$PATH"
topo --version
```

The default destination is `$HOME/.local/bin/topo`. Set `TOPO_INSTALL_DIR` for a
different destination. The installer refuses to overwrite a pre-existing file
unless you explicitly pass `--force`. It does not use sudo or change your shell
configuration.

A checksum catches download corruption; it is not independent proof of the
publisher's identity. Downloads and checksums are delivered over HTTPS from the
same site. macOS downloads are not Apple-notarized in this preview; the
command-line installation path is the tested entry point.

## Manual install

Download the executable and its `.sha256` file for your platform:

- [macOS Apple Silicon executable](/releases/v{{version}}/topo-aarch64-apple-darwin) · [SHA256](/releases/v{{version}}/topo-aarch64-apple-darwin.sha256)
- [macOS Intel executable](/releases/v{{version}}/topo-x86_64-apple-darwin) · [SHA256](/releases/v{{version}}/topo-x86_64-apple-darwin.sha256)
- [Linux x86_64 executable](/releases/v{{version}}/topo-x86_64-unknown-linux-gnu) · [SHA256](/releases/v{{version}}/topo-x86_64-unknown-linux-gnu.sha256)

Compare the executable's SHA256 with the published file before making it
executable and placing it on PATH. The executable contains its own parser and
map assets; keep no build checkout or sidecar runtime files.

## Component notices

[Third-party notices](/third-party/) list the incorporated components and license
texts. The executable also embeds them: `topo --licenses > notices.html`. This
preview is distributed as a downloadable tool; its source repository remains
private and no open-source license is assigned to Topo itself.

## Update or uninstall

Read the current installer and run `sh install-topo.sh --force` to explicitly
replace your installed version. Releases are versioned; keep an earlier
executable if you need rollback. A tool rollback does not change your maps.

Uninstall by removing the executable from your selected install directory.
Generated maps are separate files and are not removed automatically.

[Make your first map](/docs/quickstart/)
