#!/bin/sh
set -e

# Owlmic Linux .deb Packaging Script
# Generates a standard Debian/Ubuntu package with desktop autostart & firewall rule.

VERSION="0.1.0"
ARCH="amd64"
PACKAGE="owlmic"
PKG_DIR="target/debian/${PACKAGE}_${VERSION}_${ARCH}"

echo "Building release binary..."
cargo build --release

echo "Setting up package structure..."
rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"

# Copy binary
cp target/release/owlmic "${PKG_DIR}/usr/bin/owlmic"
chmod +x "${PKG_DIR}/usr/bin/owlmic"

# Desktop entry
cat << 'EOF' > "${PKG_DIR}/usr/share/applications/owlmic.desktop"
[Desktop Entry]
Type=Application
Name=Owlmic
Comment=Use your Android phone as a mic and webcam
Exec=/usr/bin/owlmic
Terminal=false
Categories=AudioVideo;Audio;
EOF

# Package metadata
cat << EOF > "${PKG_DIR}/DEBIAN/control"
Package: ${PACKAGE}
Version: ${VERSION}
Section: sound
Priority: optional
Architecture: ${ARCH}
Maintainer: Owlmic Contributors <owlmic@example.com>
Description: Turn your Android phone into a high-quality mic and webcam for PC.
 Lightweight tray application communicating over USB, Bluetooth, or Wi-Fi.
EOF

# Post-install script (firewall rule for ufw if active)
cat << 'EOF' > "${PKG_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v ufw >/dev/null 2>&1; then
    ufw allow 7653/tcp comment 'Owlmic TCP streaming' >/dev/null 2>&1 || true
    ufw allow 7654/udp comment 'Owlmic UDP discovery' >/dev/null 2>&1 || true
fi
exit 0
EOF
chmod +x "${PKG_DIR}/DEBIAN/postinst"

# Build .deb
dpkg-deb --build "${PKG_DIR}" "target/${PACKAGE}_${VERSION}_${ARCH}.deb"
echo "Generated target/${PACKAGE}_${VERSION}_${ARCH}.deb successfully."
