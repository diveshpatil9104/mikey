#!/bin/sh
set -e

# Mikey Linux AppImage Packaging Script

VERSION="0.1.0"
APP_DIR="target/AppDir"

echo "Building release binary..."
cargo build --release

rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"

cp target/release/mikey "${APP_DIR}/usr/bin/mikey"
chmod +x "${APP_DIR}/usr/bin/mikey"

cat << 'EOF' > "${APP_DIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/mikey" "$@"
EOF
chmod +x "${APP_DIR}/AppRun"

cat << 'EOF' > "${APP_DIR}/mikey.desktop"
[Desktop Entry]
Type=Application
Name=Mikey
Exec=mikey
Icon=mikey
Categories=AudioVideo;Audio;
EOF

# Create dummy icon if not present
touch "${APP_DIR}/mikey.png"

echo "AppDir prepared at ${APP_DIR}."
if command -v appimagetool >/dev/null 2>&1; then
    appimagetool "${APP_DIR}" "target/Mikey-${VERSION}-x86_64.AppImage"
    echo "Generated AppImage."
else
    echo "appimagetool not found on PATH. Run appimagetool target/AppDir to build AppImage."
fi
