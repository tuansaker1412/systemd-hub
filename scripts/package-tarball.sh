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

ICON_SRC="${ICON_SRC:-data/icons/systemd-hub.svg}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY (build with cargo build --release first)" >&2
  exit 1
fi
if [[ ! -f "$ICON_SRC" ]]; then
  echo "error: icon not found at $ICON_SRC" >&2
  exit 1
fi

STAGE="systemd-hub-${VERSION}-${DISTRO}-${ARCH}"
STAGE_DIR="${OUT_DIR}/${STAGE}"

rm -rf "$STAGE_DIR"
mkdir -p \
  "$STAGE_DIR/bin" \
  "$STAGE_DIR/share/applications" \
  "$STAGE_DIR/share/icons/hicolor/scalable/apps" \
  "$STAGE_DIR/share/doc/systemd-hub"

install -m 755 "$BINARY" "$STAGE_DIR/bin/systemd-hub"
install -m 644 data/dev.systemdhub.SystemdHub.desktop \
  "$STAGE_DIR/share/applications/dev.systemdhub.SystemdHub.desktop"
install -m 644 "$ICON_SRC" \
  "$STAGE_DIR/share/icons/hicolor/scalable/apps/systemd-hub.svg"
install -m 644 README.md LICENSE "$STAGE_DIR/share/doc/systemd-hub/"

cat >"$STAGE_DIR/INSTALL.txt" <<EOF
Systemd Hub ${VERSION} (${DISTRO}, ${ARCH})

Runtime requirements:
  - Linux with systemd
  - GTK 4.12+
  - libadwaita 1.5+

Quick install (user-local):
  mkdir -p ~/.local/bin \\
           ~/.local/share/applications \\
           ~/.local/share/icons/hicolor/scalable/apps
  cp bin/systemd-hub ~/.local/bin/
  cp share/applications/dev.systemdhub.SystemdHub.desktop \\
     ~/.local/share/applications/
  cp share/icons/hicolor/scalable/apps/systemd-hub.svg \\
     ~/.local/share/icons/hicolor/scalable/apps/
  # optional: ensure ~/.local/bin is on PATH
  # optional: refresh menus/icons
  #   update-desktop-database ~/.local/share/applications 2>/dev/null || true
  #   gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true

System install:
  sudo install -Dm755 bin/systemd-hub /usr/local/bin/systemd-hub
  sudo install -Dm644 share/applications/dev.systemdhub.SystemdHub.desktop \\
    /usr/local/share/applications/dev.systemdhub.SystemdHub.desktop
  sudo install -Dm644 share/icons/hicolor/scalable/apps/systemd-hub.svg \\
    /usr/local/share/icons/hicolor/scalable/apps/systemd-hub.svg
  # optional: refresh menus/icons
  #   sudo update-desktop-database /usr/local/share/applications 2>/dev/null || true
  #   sudo gtk-update-icon-cache -f -t /usr/local/share/icons/hicolor 2>/dev/null || true

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
