#!/bin/sh
# Topo installer: downloads a native, versioned executable and verifies SHA256.
# No sudo, package managers, project execution, accounts or telemetry.
set -eu
umask 077
version=@@VERSION@@
base="https://topo.mistystep.io/releases/v$version"
force=false
case "${1:-}" in --force) force=true ;; '') ;; *) printf '%s\n' 'Usage: sh install.sh [--force]' >&2; exit 1 ;; esac
case "$(uname -s)/$(uname -m)" in
 Darwin/arm64) target=aarch64-apple-darwin ;;
 Darwin/x86_64) target=x86_64-apple-darwin ;;
 Linux/x86_64) target=x86_64-unknown-linux-gnu ;;
 *) printf '%s\n' 'Unsupported platform. This release supports macOS Apple Silicon/Intel and Linux x86_64 (including WSL2). See https://topo.mistystep.io/docs/install/' >&2; exit 1 ;;
esac
command -v curl >/dev/null 2>&1 || { printf '%s\n' 'curl is required to download Topo.' >&2; exit 1; }
command -v git >/dev/null 2>&1 || { printf '%s\n' 'Git is required to scan a repository. Install Git, then rerun this installer.' >&2; exit 1; }
if command -v sha256sum >/dev/null 2>&1; then hasher=sha256sum
elif command -v shasum >/dev/null 2>&1; then hasher=shasum
else printf '%s\n' 'sha256sum or shasum is required to verify the download.' >&2; exit 1
fi
dest="${TOPO_INSTALL_DIR:-$HOME/.local/bin}"
if [ -e "$dest/topo" ] || [ -L "$dest/topo" ]; then
 [ "$force" = true ] || { printf 'Already exists: %s/topo. Use --force to explicitly replace it, or choose TOPO_INSTALL_DIR.\n' "$dest" >&2; exit 1; }
fi
scratch="${TMPDIR:-$HOME/.cache/tmp}"
mkdir -p "$scratch"
tmp=$(mktemp -d "$scratch/topo-install.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
curl --proto '=https' --tlsv1.2 -fsS "$base/topo-$target" -o "$tmp/topo"
curl --proto '=https' --tlsv1.2 -fsS "$base/topo-$target.sha256" -o "$tmp/checksum"
read -r expected ignored < "$tmp/checksum"
case "$expected" in *[!a-f0-9]*|'') printf '%s\n' 'Invalid checksum response; nothing installed.' >&2; exit 1 ;; esac
[ "${#expected}" -eq 64 ] || { printf '%s\n' 'Invalid checksum length; nothing installed.' >&2; exit 1; }
if [ "$hasher" = sha256sum ]; then actual=$(sha256sum "$tmp/topo")
else actual=$(shasum -a 256 "$tmp/topo")
fi
actual=${actual%% *}
[ "$actual" = "$expected" ] || { printf '%s\n' 'Download checksum mismatch; nothing installed. Retry or report the problem.' >&2; exit 1; }
mkdir -p "$dest"
# Stage beside the destination, so replacement is atomic on the same filesystem.
staged=$(mktemp "$dest/.topo-install.XXXXXX")
trap 'rm -f "${staged:-}"; rm -rf "$tmp"' EXIT HUP INT TERM
cp "$tmp/topo" "$staged"
chmod 755 "$staged"
"$staged" --version
mv -f "$staged" "$dest/topo"
printf '\nInstalled %s/topo\n' "$dest"
case ":$PATH:" in *":$dest:"*) ;; *) printf 'Add it to your shell PATH:\n  export PATH="%s:$PATH"\n' "$dest" ;; esac
printf '\nIn a Git repository:\n  topo run --repo app=. --out topo-map\nThen open topo-map/index.html.\n'
