#!/bin/sh
# macOS: compila FontCraft para iPadOS / iOS con Rust y deja en dist/
#   FontCraft-<versión>-ipados.ipa  (iPad y iPhone ARM64; iPadOS 15 o posterior)
#   FontCraft-Fonts.mobileconfig     (perfil de instalación de fuentes en iPadOS)
#
# Métodos de instalación:
#   - AltStore, Sideloadly o TrollStore: instala el archivo .ipa directamente en el iPad.
#   - Xcode o Apple Configurator: Devices and Simulators › Instalar app.
#   - Perfil .mobileconfig: abre en Safari/Archivos e instala en Ajustes › Perfiles.
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

command -v cargo >/dev/null 2>&1 || { echo "No encuentro Rust (cargo). Instálalo desde https://rustup.rs"; exit 1; }

echo "Comprobando objetivo aarch64-apple-ios para iPadOS…"
rustup target add aarch64-apple-ios >/dev/null 2>&1 || true

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
echo "Compilando FontCraft $VERSION para iPadOS (aarch64-apple-ios) en modo release…"
(cd ipados && cargo build --release --target aarch64-apple-ios)

echo "Empaquetando instalador .ipa para iPadOS…"
sh packaging/ipados/make-ipa.sh ipados/target/aarch64-apple-ios/release/FontCraft

echo ""
echo "Listo: dist/FontCraft-$VERSION-ipados.ipa"
