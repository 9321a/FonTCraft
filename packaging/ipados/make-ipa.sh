#!/bin/sh
# Builds dist/FontCraft-<version>-ipados.ipa: the iPadOS / iOS application package (.ipa)
# for iPad and iPhone (ARM64), ready for installation via AltStore, Sideloadly, TrollStore,
# Apple Configurator, Xcode or MDM.
#
# Usage: packaging/ipados/make-ipa.sh [path/to/binary]
set -e
cd "$(dirname "$0")/../.."
ROOT="$(pwd)"

BIN="$1"
if [ -z "$BIN" ]; then
  if [ -f "ipados/target/aarch64-apple-ios/release/FontCraft" ]; then
    BIN="ipados/target/aarch64-apple-ios/release/FontCraft"
  elif [ -f "target/aarch64-apple-ios/release/FontCraft" ]; then
    BIN="target/aarch64-apple-ios/release/FontCraft"
  else
    echo "Compilando binario de lanzamiento para iPadOS (aarch64-apple-ios)..."
    (cd ipados && cargo build --release --target aarch64-apple-ios)
    BIN="ipados/target/aarch64-apple-ios/release/FontCraft"
  fi
fi

[ -f "$BIN" ] || { echo "No se encontró el binario: $BIN"; exit 1; }

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
STAGE=$(mktemp -d "${TMPDIR:-/tmp}/fontcraft-ipa.XXXXXX")
PAYLOAD="$STAGE/Payload"
APP="$PAYLOAD/FontCraft.app"
mkdir -p "$APP"

echo "Preparando paquete FontCraft.app para iPadOS..."
cp "$BIN" "$APP/FontCraft"
chmod 755 "$APP/FontCraft"

# Info.plist
cp packaging/ipados/Info.plist "$APP/Info.plist"
if command -v /usr/libexec/PlistBuddy >/dev/null 2>&1; then
  /usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$APP/Info.plist" 2>/dev/null || true
  /usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$APP/Info.plist" 2>/dev/null || true
  plutil -lint "$APP/Info.plist" >/dev/null 2>&1 || true
fi

# PkgInfo
printf "APPL????" > "$APP/PkgInfo"

# Idiomas (.lproj)
for L in en es fr de it pt nl sv da nb fi pl cs tr ro hu el ru uk id vi zh-Hans zh-Hant ja ko; do
  mkdir -p "$APP/$L.lproj"
  printf '"CFBundleDisplayName" = "FontCraft";\n"CFBundleName" = "FontCraft";\n' > "$APP/$L.lproj/InfoPlist.strings"
done

# Iconos de la aplicación para iPad y iPhone
ICONS_DIR="assets/icons/png"
if [ -d "$ICONS_DIR" ]; then
  [ -f "$ICONS_DIR/fontcraft-120.png" ] && cp "$ICONS_DIR/fontcraft-120.png" "$APP/AppIcon60x60@2x.png"
  [ -f "$ICONS_DIR/fontcraft-152.png" ] && cp "$ICONS_DIR/fontcraft-152.png" "$APP/AppIcon76x76@2x~ipad.png"
  [ -f "$ICONS_DIR/fontcraft-167.png" ] && cp "$ICONS_DIR/fontcraft-167.png" "$APP/AppIcon83.5x83.5@2x~ipad.png"
  [ -f "$ICONS_DIR/fontcraft-1024.png" ] && cp "$ICONS_DIR/fontcraft-1024.png" "$APP/AppIcon1024.png"
fi

# Generar perfil de configuración para iPadOS (.mobileconfig) para inyección del instalador de fuentes
UUID_PAYLOAD="B8D97D2E-73E5-4E89-B0F3-$(date +%s | cut -c1-12)"
UUID_PROFILE="C3A19F84-29E1-4C5D-97D8-$(date +%s | cut -c1-12)"
cat <<EOF > "$APP/FontCraft-Fonts.mobileconfig"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>PayloadDisplayName</key>
	<string>FontCraft Fonts Installer</string>
	<key>PayloadDescription</key>
	<string>Instalador de fuentes para iPadOS de FontCraft para Procreate, Canva, Affinity, Pages y suites de diseño.</string>
	<key>PayloadIdentifier</key>
	<string>app.fontcraft.fonts.profile</string>
	<key>PayloadOrganization</key>
	<string>FontCraft</string>
	<key>PayloadRemovalDisallowed</key>
	<false/>
	<key>PayloadType</key>
	<string>Configuration</string>
	<key>PayloadUUID</key>
	<string>$UUID_PROFILE</string>
	<key>PayloadVersion</key>
	<integer>1</integer>
	<key>PayloadContent</key>
	<array>
		<dict>
			<key>PayloadType</key>
			<string>com.apple.font</string>
			<key>PayloadVersion</key>
			<integer>1</integer>
			<key>PayloadIdentifier</key>
			<string>app.fontcraft.font.payload</string>
			<key>PayloadUUID</key>
			<string>$UUID_PAYLOAD</string>
			<key>PayloadDisplayName</key>
			<string>FontCraft Managed Fonts</string>
		</dict>
	</array>
</dict>
</plist>
EOF

# Firma ad-hoc para iPadOS
if command -v codesign >/dev/null 2>&1; then
  codesign --force --deep -s - "$APP" >/dev/null 2>&1 || true
fi

mkdir -p "$ROOT/dist"
IPA_NAME="FontCraft-$VERSION-ipados.ipa"
IPA_PATH="$ROOT/dist/$IPA_NAME"
IPA_GENERIC="$ROOT/dist/FontCraft-ipados.ipa"

rm -f "$IPA_PATH" "$IPA_GENERIC"
(cd "$STAGE" && zip -qry "$IPA_PATH" Payload)
cp "$IPA_PATH" "$IPA_GENERIC"
cp "$APP/FontCraft-Fonts.mobileconfig" "$ROOT/dist/FontCraft-Fonts.mobileconfig"

rm -rf "$STAGE"

echo ""
echo "Listo: dist/$IPA_NAME"
echo "  - Archivo IPA: dist/$IPA_NAME (y dist/FontCraft-ipados.ipa)"
echo "  - Perfil de fuentes iPadOS: dist/FontCraft-Fonts.mobileconfig"
echo ""
echo "Métodos de instalación en iPad / iPadOS:"
echo "  1. AltStore / Sideloadly / TrollStore: arrastra dist/$IPA_NAME para instalar directamente."
echo "  2. Xcode / Apple Configurator: Window › Devices and Simulators › Instalar app."
echo "  3. Perfil de fuentes: abre dist/FontCraft-Fonts.mobileconfig en Safari/Archivos e instálalo en Ajustes › Perfil descargado."
