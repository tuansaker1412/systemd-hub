#!/usr/bin/env bash
# Create a simple .deb package for Debian/Ubuntu-family systems.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
DISTRO="${DISTRO:-ubuntu}"
ARCH_DEB="${ARCH_DEB:-amd64}"
BINARY="${BINARY:-target/release/systemd-hub}"
OUT_DIR="${OUT_DIR:-dist}"
MAINTAINER="${MAINTAINER:-Systemd Hub Contributors <noreply@systemdhub.dev>}"

ICON_SRC="${ICON_SRC:-data/icons/systemd-hub.svg}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY" >&2
  exit 1
fi
if [[ ! -f "$ICON_SRC" ]]; then
  echo "error: icon not found at $ICON_SRC" >&2
  exit 1
fi

PKG_ROOT="${OUT_DIR}/deb/systemd-hub_${VERSION}_${ARCH_DEB}"
rm -rf "$PKG_ROOT"
mkdir -p \
  "$PKG_ROOT/DEBIAN" \
  "$PKG_ROOT/usr/bin" \
  "$PKG_ROOT/usr/share/applications" \
  "$PKG_ROOT/usr/share/icons/hicolor/scalable/apps" \
  "$PKG_ROOT/usr/share/doc/systemd-hub"

install -m 755 "$BINARY" "$PKG_ROOT/usr/bin/systemd-hub"
install -m 644 data/dev.systemdhub.SystemdHub.desktop \
  "$PKG_ROOT/usr/share/applications/dev.systemdhub.SystemdHub.desktop"
install -m 644 "$ICON_SRC" \
  "$PKG_ROOT/usr/share/icons/hicolor/scalable/apps/systemd-hub.svg"
install -m 644 README.md "$PKG_ROOT/usr/share/doc/systemd-hub/README.md"
install -m 644 LICENSE "$PKG_ROOT/usr/share/doc/systemd-hub/copyright"

# Refresh desktop/icon caches after install/remove (best-effort).
cat >"$PKG_ROOT/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
EOF
cat >"$PKG_ROOT/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
fi
EOF
chmod 755 "$PKG_ROOT/DEBIAN/postinst" "$PKG_ROOT/DEBIAN/postrm"

# Installed-Size in KiB
INSTALLED_SIZE="$(du -sk "$PKG_ROOT/usr" | cut -f1)"

cat >"$PKG_ROOT/DEBIAN/control" <<EOF
Package: systemd-hub
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH_DEB}
Maintainer: ${MAINTAINER}
Installed-Size: ${INSTALLED_SIZE}
Depends: libgtk-4-1, libadwaita-1-0, libc6
Recommends: systemd, systemd-sysv
Description: Native Linux systemd service manager
 Systemd Hub is a GTK 4 / libadwaita desktop app for managing
 systemd services over D-Bus (start/stop/restart, logs, dashboard).
 Built against ${DISTRO}.
EOF

mkdir -p "$OUT_DIR"
DEB_NAME="systemd-hub_${VERSION}_${ARCH_DEB}.${DISTRO}.deb"
dpkg-deb --build --root-owner-group "$PKG_ROOT" "${OUT_DIR}/${DEB_NAME}"

echo "Created ${OUT_DIR}/${DEB_NAME}"
echo "${OUT_DIR}/${DEB_NAME}"
