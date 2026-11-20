#!/bin/sh
# Builds dist/FontCraft.app: the program, its icon, the .fcl document icon, and the Info.plist
# that tells macOS FontCraft owns .fcl libraries (shown as single documents with their own icon).
# Usage: packaging/macos/make-app.sh [path/to/fontcraft binary]   (default: a release build)
set -e
cd "$(dirname "$0")/../.."
BIN="$1"
if [ -z "$BIN" ]; then
  cargo build --release
  BIN=target/release/FontCraft
fi
"$BIN" --make-icons assets/icons
# The bundle is put together and signed in a temporary folder: in a folder iCloud keeps (such as
# Documents) macOS adds Finder information to it while it is being signed, which codesign refuses.
STAGE=$(mktemp -d "${TMPDIR:-/tmp}/fontcraft-app.XXXXXX")
APP="$STAGE/FontCraft.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/fontcraft"
cp packaging/macos/Info.plist "$APP/Contents/Info.plist"
cp assets/icons/fontcraft.icns "$APP/Contents/Resources/AppIcon.icns"
cp assets/icons/fontcraft.icns "$APP/Contents/Resources/fontcraft.icns"
cp assets/icons/fontcraft-library.icns "$APP/Contents/Resources/"
# The version shown by macOS (CFBundleShortVersionString, which RightFont also reads as its own
# version: +[RFGlobals appVersion] 0x1000e95f0, string 0x100296281) is the one in Cargo.toml, so
# every release built by GitHub Actions carries its own number.
CARGO_VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $CARGO_VERSION" "$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $CARGO_VERSION" "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
# One .lproj folder for each language FontCraft speaks (CFBundleLocalizations): macOS gives the
# system's own panels (Print, Open, Save, the font and colour panels) in the language of the system
# only when the app says it has that language; without them they came up in English.
for L in $(/usr/libexec/PlistBuddy -c "Print CFBundleLocalizations" "$APP/Contents/Info.plist" | sed -e '1d' -e '$d' | tr -d ' '); do
  mkdir -p "$APP/Contents/Resources/$L.lproj"
  printf '"CFBundleDisplayName" = "FontCraft";\n"CFBundleName" = "FontCraft";\n' > "$APP/Contents/Resources/$L.lproj/InfoPlist.strings"
done
xattr -cr "$APP"
# Ad hoc signature so the bundle runs on this Mac (a Developer ID signature is needed to hand it out).
# Its designated requirement is the app's identifier, not the code's hash: macOS keeps the
# Accessibility and Automation permissions it was given across rebuilds (with a bare ad hoc
# signature every build looked like a new app, asked again and was listed twice).
ID=$(/usr/libexec/PlistBuddy -c "Print CFBundleIdentifier" "$APP/Contents/Info.plist")
codesign --force --deep -s - -i "$ID" -r="designated => identifier \"$ID\"" "$APP"
codesign --verify --deep --strict "$APP"
codesign -d -r- "$APP" 2>&1 | grep designated
VERSION=$(/usr/libexec/PlistBuddy -c "Print CFBundleShortVersionString" "$APP/Contents/Info.plist")
ARCH=$(lipo -archs "$APP/Contents/MacOS/fontcraft" | tr ' ' '+')
mkdir -p dist
rm -rf dist/FontCraft.app
ditto "$APP" dist/FontCraft.app
touch dist/FontCraft.app
ditto -c -k --keepParent "$APP" "dist/FontCraft-$VERSION-macos-$ARCH.zip"
rm -rf "$STAGE"
echo "Made dist/FontCraft.app — open it once (or copy it to Applications) so macOS learns the .fcl icon."
