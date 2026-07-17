#!/usr/bin/env bash
# Build a portable AppImage (GTK 4 + libadwaita) via linuxdeploy.
#
# Requirements:
#   - prebuilt release binary (cargo build --release)
#   - curl/wget, file, patchelf, desktop-file-utils
#   - GTK 4 / libadwaita runtime + devel pieces used by linuxdeploy-plugin-gtk
#   - gobject-introspection, librsvg, gdk-pixbuf loaders
#
# Notes:
#   - AppImages need FUSE or APPIMAGE_EXTRACT_AND_RUN=1 (set automatically).
#   - Built against the host glibc; prefer Ubuntu 24.04 for a reasonable baseline.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
ARCH="${ARCH:-x86_64}"
BINARY="${BINARY:-target/release/systemd-hub}"
OUT_DIR="${OUT_DIR:-dist}"
APPDIR="${APPDIR:-${OUT_DIR}/AppDir}"
TOOLS_DIR="${TOOLS_DIR:-${OUT_DIR}/appimage-tools}"
DESKTOP_SRC="${DESKTOP_SRC:-data/dev.systemdhub.SystemdHub.desktop}"
ICON_SRC="${ICON_SRC:-data/icons/systemd-hub.svg}"

# Resolve paths relative to repo root
[[ "$BINARY" = /* ]] || BINARY="${ROOT}/${BINARY}"
[[ "$DESKTOP_SRC" = /* ]] || DESKTOP_SRC="${ROOT}/${DESKTOP_SRC}"
[[ "$ICON_SRC" = /* ]] || ICON_SRC="${ROOT}/${ICON_SRC}"
[[ "$OUT_DIR" = /* ]] || OUT_DIR="${ROOT}/${OUT_DIR}"
[[ "$APPDIR" = /* ]] || APPDIR="${OUT_DIR}/AppDir"
[[ "$TOOLS_DIR" = /* ]] || TOOLS_DIR="${OUT_DIR}/appimage-tools"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY (build with cargo build --release first)" >&2
  exit 1
fi
if [[ ! -f "$DESKTOP_SRC" ]]; then
  echo "error: desktop file not found: $DESKTOP_SRC" >&2
  exit 1
fi
if [[ ! -f "$ICON_SRC" ]]; then
  echo "error: icon not found: $ICON_SRC" >&2
  exit 1
fi

mkdir -p "$OUT_DIR" "$TOOLS_DIR"
rm -rf "$APPDIR"
mkdir -p "$APPDIR"

# linuxdeploy AppImages cannot mount FUSE in most CI containers
export APPIMAGE_EXTRACT_AND_RUN=1
export DEPLOY_GTK_VERSION="${DEPLOY_GTK_VERSION:-4}"
export ARCH

download() {
  local url="$1"
  local dest="$2"
  if [[ -f "$dest" ]]; then
    return 0
  fi
  echo "Downloading $(basename "$dest")..."
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "$dest" "$url"
  else
    wget -q -O "$dest" "$url"
  fi
}

LINUXDEPLOY="${TOOLS_DIR}/linuxdeploy-${ARCH}.AppImage"
PLUGIN_GTK="${TOOLS_DIR}/linuxdeploy-plugin-gtk.sh"
PLUGIN_APPIMAGE="${TOOLS_DIR}/linuxdeploy-plugin-appimage-${ARCH}.AppImage"

download \
  "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${ARCH}.AppImage" \
  "$LINUXDEPLOY"
download \
  "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh" \
  "$PLUGIN_GTK"
download \
  "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-${ARCH}.AppImage" \
  "$PLUGIN_APPIMAGE"

chmod +x "$LINUXDEPLOY" "$PLUGIN_GTK" "$PLUGIN_APPIMAGE"

# Put plugins next to linuxdeploy so it auto-discovers them
# (or on PATH — linuxdeploy looks for linuxdeploy-plugin-*).
ln -sfn "$(realpath "$PLUGIN_GTK")" "${TOOLS_DIR}/linuxdeploy-plugin-gtk"
ln -sfn "$(realpath "$PLUGIN_APPIMAGE")" "${TOOLS_DIR}/linuxdeploy-plugin-appimage"
export PATH="${TOOLS_DIR}:${PATH}"
export LINUXDEPLOY="$LINUXDEPLOY"

# Desktop entry for AppImage: Icon name must match icon file basename.
DESKTOP_APPIMAGE="${TOOLS_DIR}/systemd-hub.desktop"
sed \
  -e 's|^Exec=.*|Exec=systemd-hub|' \
  -e 's|^Icon=.*|Icon=systemd-hub|' \
  "$DESKTOP_SRC" >"$DESKTOP_APPIMAGE"
# Ensure StartupWMClass-friendly Name stays readable
grep -q '^Name=' "$DESKTOP_APPIMAGE"

bundle_extra_resources() {
  local appdir="$1"
  mkdir -p "$appdir/usr/share/icons" "$appdir/usr/share"

  if [[ -d /usr/share/libadwaita-1 ]]; then
    echo "Bundling /usr/share/libadwaita-1"
    cp -a /usr/share/libadwaita-1 "$appdir/usr/share/"
  fi

  # Symbolic icons used by libadwaita / GTK widgets
  for theme in Adwaita hicolor; do
    if [[ -d "/usr/share/icons/${theme}" ]]; then
      echo "Bundling icon theme: ${theme}"
      mkdir -p "$appdir/usr/share/icons/${theme}"
      # Merge so app-specific icons already placed by linuxdeploy are kept
      cp -a "/usr/share/icons/${theme}/." "$appdir/usr/share/icons/${theme}/" || true
    fi
  done

  if [[ -d "$appdir/usr/share/glib-2.0/schemas" ]] && command -v glib-compile-schemas >/dev/null 2>&1; then
    glib-compile-schemas "$appdir/usr/share/glib-2.0/schemas" || true
  fi
}

echo "Populating AppDir with linuxdeploy (GTK${DEPLOY_GTK_VERSION})..."
# Pass 1: deploy binary + GTK runtime resources into AppDir (no AppImage yet).
(
  cd "$OUT_DIR"
  "$LINUXDEPLOY" \
    --appdir "$APPDIR" \
    --executable "$BINARY" \
    --desktop-file "$DESKTOP_APPIMAGE" \
    --icon-file "$ICON_SRC" \
    --plugin gtk
)

bundle_extra_resources "$APPDIR"

echo "Creating AppImage..."
# Pass 2: pack AppDir into AppImage.
# Prefer LDAI_OUTPUT / LINUXDEPLOY_OUTPUT_VERSION (current linuxdeploy API).
(
  cd "$OUT_DIR"
  env \
    LDAI_OUTPUT="systemd-hub-${VERSION}-${ARCH}.AppImage" \
    OUTPUT="systemd-hub-${VERSION}-${ARCH}.AppImage" \
    LINUXDEPLOY_OUTPUT_VERSION="${VERSION}" \
    "$LINUXDEPLOY" \
    --appdir "$APPDIR" \
    --output appimage
)

# Normalize artifact name (linuxdeploy naming can vary by desktop Name=)
shopt -s nullglob
candidates=(
  "${OUT_DIR}/systemd-hub-${VERSION}-${ARCH}.AppImage"
  "${OUT_DIR}"/*"${ARCH}".AppImage
  "${OUT_DIR}"/*.AppImage
)
APPIMAGE=""
for f in "${candidates[@]}"; do
  if [[ -f "$f" ]]; then
    APPIMAGE="$f"
    break
  fi
done

if [[ -z "$APPIMAGE" ]]; then
  echo "error: AppImage was not produced" >&2
  ls -laR "$OUT_DIR" >&2 || true
  exit 1
fi

FINAL="${OUT_DIR}/systemd-hub-${VERSION}-${ARCH}.AppImage"
if [[ "$(realpath "$APPIMAGE")" != "$(realpath "$FINAL")" ]]; then
  mv -f "$APPIMAGE" "$FINAL"
fi
chmod +x "$FINAL"

echo "Created $FINAL"
echo "$FINAL"
