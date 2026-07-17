#!/usr/bin/env bash
# Create a simple .rpm package for Fedora/RHEL-family systems.
#
# Packages a prebuilt binary (no in-spec compile). Requires rpmbuild.
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

mkdir -p "$OUT_DIR"
# Absolute _topdir is required: relative paths break %prep on modern rpm (Fedora)
# because the build script cds into BUILD and re-expands relative topdir.
OUT_DIR_ABS="$(cd "$OUT_DIR" && pwd)"
WORK="${OUT_DIR_ABS}/rpmbuild"
rm -rf "$WORK"
mkdir -p "$WORK"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Stage prebuilt payload as Source files (no %setup / source tarball unpack).
install -m 755 "$BINARY" "$WORK/SOURCES/systemd-hub"
install -m 644 data/dev.systemdhub.SystemdHub.desktop \
  "$WORK/SOURCES/dev.systemdhub.SystemdHub.desktop"
install -m 644 README.md "$WORK/SOURCES/README.md"
install -m 644 LICENSE "$WORK/SOURCES/LICENSE"

cat >"$WORK/SPECS/systemd-hub.spec" <<EOF
Name:           systemd-hub
Version:        ${RPM_VERSION}
Release:        ${RELEASE}%{?dist}
Summary:        Native Linux systemd service manager
License:        GPL-3.0-or-later
URL:            https://github.com/tuansaker1412/systemd-hub
Source0:        systemd-hub
Source1:        dev.systemdhub.SystemdHub.desktop
Source2:        README.md
Source3:        LICENSE
BuildArch:      ${ARCH}

Requires:       gtk4 libadwaita

%description
Systemd Hub is a GTK 4 / libadwaita desktop app for managing
systemd services over D-Bus (start/stop/restart, logs, dashboard).
Built against ${DISTRO}.

# Prebuilt binary packaging: nothing to unpack or compile.
# Avoid %setup so relative/builddir layout differences across rpm
# versions (e.g. Fedora rpm 4.20+) cannot break %prep.
%prep
:

%build
:

%install
rm -rf %{buildroot}
install -Dm755 %{SOURCE0} %{buildroot}%{_bindir}/systemd-hub
install -Dm644 %{SOURCE1} \\
  %{buildroot}%{_datadir}/applications/dev.systemdhub.SystemdHub.desktop
install -Dm644 %{SOURCE2} %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 %{SOURCE3} %{buildroot}%{_docdir}/%{name}/LICENSE

%files
%{_bindir}/systemd-hub
%{_datadir}/applications/dev.systemdhub.SystemdHub.desktop
%{_docdir}/%{name}/README.md
%{_docdir}/%{name}/LICENSE

%changelog
* $(date '+%a %b %d %Y') Ngọc Tuấn <https://github.com/tuansaker1412> - ${RPM_VERSION}-${RELEASE}
- Automated release package
EOF

rpmbuild \
  --define "_topdir ${WORK}" \
  --define "_build_id_links none" \
  -bb "$WORK/SPECS/systemd-hub.spec"

RPM_FILE="$(find "$WORK/RPMS" -type f -name '*.rpm' | head -1)"
if [[ -z "$RPM_FILE" ]]; then
  echo "error: rpmbuild did not produce an rpm" >&2
  exit 1
fi

DEST="${OUT_DIR_ABS}/systemd-hub-${RPM_VERSION}-${RELEASE}.${DISTRO}.${ARCH}.rpm"
cp "$RPM_FILE" "$DEST"

echo "Created $DEST"
echo "$DEST"
