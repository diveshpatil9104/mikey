#!/bin/sh
set -e

# Owlmic Linux AppImage Packaging Script

VERSION="0.1.0"
APP_DIR="target/AppDir"

echo "Building release binary..."
cargo build --release

rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"

cp target/release/owlmic "${APP_DIR}/usr/bin/owlmic"
chmod +x "${APP_DIR}/usr/bin/owlmic"

cat << 'EOF' > "${APP_DIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/owlmic" "$@"
EOF
chmod +x "${APP_DIR}/AppRun"

cat << 'EOF' > "${APP_DIR}/owlmic.desktop"
[Desktop Entry]
Type=Application
Name=Owlmic
Exec=owlmic
Icon=owlmic
Categories=AudioVideo;Audio;
EOF

# Create dummy icon if not present
touch "${APP_DIR}/owlmic.png"

echo "AppDir prepared at ${APP_DIR}."
if command -v appimagetool >/dev/null 2>&1; then
    appimagetool "${APP_DIR}" "target/Owlmic-${VERSION}-x86_64.AppImage"
    echo "Generated AppImage."
else
    echo "appimagetool not found on PATH. Run appimagetool target/AppDir to build AppImage."
fi
