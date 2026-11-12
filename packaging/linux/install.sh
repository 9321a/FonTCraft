#!/bin/sh
# Instala FontCraft solo para este usuario (sin permisos de administrador): programa, menú de
# aplicaciones, iconos y el tipo de archivo .fcl. Se ejecuta desde la carpeta del .tar.gz.
set -e
cd "$(dirname "$0")"
D="$HOME/.local"
mkdir -p "$D/bin" "$D/share/applications" "$D/share/icons/hicolor/256x256/apps" "$D/share/icons/hicolor/256x256/mimetypes" "$D/share/mime/packages"
cp fontcraft "$D/bin/fontcraft"
chmod 755 "$D/bin/fontcraft"
cp fontcraft.desktop "$D/share/applications/fontcraft.desktop"
cp fontcraft-256.png "$D/share/icons/hicolor/256x256/apps/fontcraft.png"
cp fontcraft-library-256.png "$D/share/icons/hicolor/256x256/mimetypes/application-x-fontcraft-library.png"
cp fontcraft-library.xml "$D/share/mime/packages/fontcraft-library.xml"
if command -v update-mime-database >/dev/null 2>&1; then update-mime-database "$D/share/mime"; fi
if command -v update-desktop-database >/dev/null 2>&1; then update-desktop-database "$D/share/applications"; fi
echo "FontCraft está instalado. Ábrelo desde el menú de aplicaciones (o con: ~/.local/bin/fontcraft)."
