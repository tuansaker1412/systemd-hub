#!/usr/bin/env bash
# Create a simple .rpm package for Fedora/RHEL-family systems.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
# RPM forbids '-' in version; replace with '.'
RPM_VERSION="${VERSION//-/.}"
RELEASE="${RELEASE:-1}"
DISTRO="${DISTRO:-fedora}"
ARCH="${ARCH:-x86_64}"
BINARY="${BINARY:-target/release/systemd-hub}"
OUT_DIR="${OUT_DIR:-dist}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found at $BINARY" >&2
  exit 1
fi

if ! command -v rpmbuild >/dev/null 2>&1; then
  echo "error: rpmbuild not found (install rpm-build)" >&2
  exit 1
fi

WORK="${OUT_DIR}/rpmbuild"
rm -rf "$WORK"
mkdir -p "$WORK"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Source layout consumed by the spec %install section
SRC_DIR="${WORK}/SOURCES/systemd-hub-${RPM_VERSION}"
mkdir -p "$SRC_DIR/bin" "$SRC_DIR/share/applications" "$SRC_DIR/share/doc/systemd-hub"
install -m 755 "$BINARY" "$SRC_DIR/bin/systemd-hub"
install -m 644 data/dev.systemdhub.SystemdHub.desktop \
  "$SRC_DIR/share/applications/dev.systemdhub.SystemdHub.desktop"
install -m 644 README.md LICENSE "$SRC_DIR/share/doc/systemd-hub/"
tar -C "$WORK/SOURCES" -czf "$WORK/SOURCES/systemd-hub-${RPM_VERSION}.tar.gz" \
  "systemd-hub-${RPM_VERSION}"

cat >"$WORK/SPECS/systemd-hub.spec" <<EOF
Name:           systemd-hub
Version:        ${RPM_VERSION}
Release:        ${RELEASE}%{?dist}
Summary:        Native Linux systemd service manager
License:        GPL-3.0-or-later
URL:            https://github.com/systemdhub/systemd-hub
Source0:        %{name}-%{version}.tar.gz
BuildArch:      ${ARCH}

Requires:       gtk4 libadwaita

%description
Systemd Hub is a GTK 4 / libadwaita desktop app for managing
systemd services over D-Bus (start/stop/restart, logs, dashboard).
Built against ${DISTRO}.

%prep
%setup -q

%build
# Prebuilt binary from CI

%install
install -Dm755 bin/systemd-hub %{buildroot}%{_bindir}/systemd-hub
install -Dm644 share/applications/dev.systemdhub.SystemdHub.desktop \\
  %{buildroot}%{_datadir}/applications/dev.systemdhub.SystemdHub.desktop
install -Dm644 share/doc/systemd-hub/README.md \\
  %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 share/doc/systemd-hub/LICENSE \\
  %{buildroot}%{_docdir}/%{name}/LICENSE

%files
%{_bindir}/systemd-hub
%{_datadir}/applications/dev.systemdhub.SystemdHub.desktop
%{_docdir}/%{name}/README.md
%{_docdir}/%{name}/LICENSE

%changelog
* $(date '+%a %b %d %Y') Systemd Hub Contributors <noreply@systemdhub.dev> - ${RPM_VERSION}-${RELEASE}
- Automated release package
EOF

rpmbuild \
  --define "_topdir ${WORK}" \
  --define "_build_id_links none" \
  -bb "$WORK/SPECS/systemd-hub.spec"

mkdir -p "$OUT_DIR"
RPM_FILE="$(find "$WORK/RPMS" -type f -name '*.rpm' | head -1)"
if [[ -z "$RPM_FILE" ]]; then
  echo "error: rpmbuild did not produce an rpm" >&2
  exit 1
fi

BASE="$(basename "$RPM_FILE")"
# Include distro label for multi-target releases
DEST="${OUT_DIR}/${BASE%.rpm}.${DISTRO}.rpm"
# If basename already unique, still copy with clear name
DEST="${OUT_DIR}/systemd-hub-${RPM_VERSION}-${RELEASE}.${DISTRO}.${ARCH}.rpm"
cp "$RPM_FILE" "$DEST"

echo "Created $DEST"
echo "$DEST"
