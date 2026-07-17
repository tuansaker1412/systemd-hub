#!/usr/bin/env bash
# Create a portable release tarball for a given distro label.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
DISTRO="${DISTRO:?DISTRO is required (e.g. ubuntu-24.04, fedora-41)}"
ARCH="${ARCH:-x86_64}"
BINARY="${BINARY:-target/release/systemd-hub}"
OUT_DIR="${OUT_DIR:-dist}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY (build with cargo build --release first)" >&2
  exit 1
fi

STAGE="systemd-hub-${VERSION}-${DISTRO}-${ARCH}"
STAGE_DIR="${OUT_DIR}/${STAGE}"

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/bin" "$STAGE_DIR/share/applications" "$STAGE_DIR/share/doc/systemd-hub"

install -m 755 "$BINARY" "$STAGE_DIR/bin/systemd-hub"
install -m 644 data/dev.systemdhub.SystemdHub.desktop \
  "$STAGE_DIR/share/applications/dev.systemdhub.SystemdHub.desktop"
install -m 644 README.md LICENSE "$STAGE_DIR/share/doc/systemd-hub/"

cat >"$STAGE_DIR/INSTALL.txt" <<EOF
Systemd Hub ${VERSION} (${DISTRO}, ${ARCH})

Runtime requirements:
  - Linux with systemd
  - GTK 4.12+
  - libadwaita 1.5+

Quick install (user-local):
  mkdir -p ~/.local/bin ~/.local/share/applications
  cp bin/systemd-hub ~/.local/bin/
  cp share/applications/dev.systemdhub.SystemdHub.desktop ~/.local/share/applications/
  # optional: ensure ~/.local/bin is on PATH

System install:
  sudo install -Dm755 bin/systemd-hub /usr/local/bin/systemd-hub
  sudo install -Dm644 share/applications/dev.systemdhub.SystemdHub.desktop \\
    /usr/local/share/applications/dev.systemdhub.SystemdHub.desktop

Debian/Ubuntu runtime packages (if needed):
  sudo apt install libgtk-4-1 libadwaita-1-0

Fedora runtime packages (if needed):
  sudo dnf install gtk4 libadwaita
EOF

mkdir -p "$OUT_DIR"
TARBALL="${OUT_DIR}/${STAGE}.tar.gz"
tar -C "$OUT_DIR" -czf "$TARBALL" "$STAGE"

echo "Created $TARBALL"
echo "$TARBALL"
