#!/bin/sh
# Linux: compila FontCraft con Rust y deja en dist/
#   - fontcraft_<versión>_<arquitectura>.deb   (Ubuntu 22.04+, Debian 12+, Mint 21+…: doble clic o apt)
#   - fontcraft-<versión>.<arquitectura>.rpm    (Fedora 38+, openSUSE Leap 15.5+, RHEL 9+…)
#   - FontCraft-<versión>-linux-<arquitectura>.tar.gz (programa, iconos, archivos de escritorio e install.sh)
#
# Uso:
#   sh compilar-linux.sh           para este equipo (x64, ARM64 o ARMv7, lo detecta solo)
#   sh compilar-linux.sh arm64     Linux ARM de 64 bits (Raspberry Pi 4/5 con sistema de 64 bits, servidores ARM)
#   sh compilar-linux.sh armhf     Linux ARM de 32 bits (ARMv7: Raspberry Pi con sistema de 32 bits)
#   sh compilar-linux.sh x64       PC de 64 bits (Intel/AMD)
# Para otra arquitectura que la del equipo usa "cross" (con Docker o Podman), que trae lo necesario;
# Cross.toml le dice qué bibliotecas de Ubuntu añadir.
#
# Paquetes de Ubuntu que necesita para compilar en el propio equipo (una sola vez):
#   build-essential pkg-config libgtk-3-dev libxdo-dev libayatana-appindicator3-dev
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null 2>&1 || { echo "No encuentro Rust (cargo). Instálalo desde https://rustup.rs"; exit 1; }
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)

HOST=$(uname -m)
case "${1:-}" in
  "") TARGET="" ;;
  x64|amd64|x86_64) TARGET=x86_64-unknown-linux-gnu ;;
  arm64|aarch64) TARGET=aarch64-unknown-linux-gnu ;;
  armhf|armv7|arm) TARGET=armv7-unknown-linux-gnueabihf ;;
  *) echo "No conozco \"$1\". Usa: x64, arm64 o armhf."; exit 1 ;;
esac
# Lo que pide el equipo mismo se compila directamente.
case "$HOST:$TARGET" in
  x86_64:x86_64-unknown-linux-gnu|aarch64:aarch64-unknown-linux-gnu|armv7l:armv7-unknown-linux-gnueabihf) TARGET="" ;;
esac

if [ -z "$TARGET" ]; then
  ARCH="$HOST"
  cargo build --release
  OUT=target/release
  DEB_TARGET=""
else
  ARCH=$(echo "$TARGET" | cut -d- -f1)
  command -v cross >/dev/null 2>&1 || cargo install cross
  cross build --release --target "$TARGET"
  OUT="target/$TARGET/release"
  DEB_TARGET="--target $TARGET"
fi

mkdir -p dist
# .deb (cargo-deb se instala la primera vez)
command -v cargo-deb >/dev/null 2>&1 || cargo install cargo-deb
# shellcheck disable=SC2086
cargo deb --no-build $DEB_TARGET --output dist/
# .rpm para Fedora, openSUSE, RHEL y derivadas (cargo-generate-rpm se instala la primera vez)
command -v cargo-generate-rpm >/dev/null 2>&1 || cargo install cargo-generate-rpm
# shellcheck disable=SC2086
cargo generate-rpm $DEB_TARGET --output dist/
# .tar.gz con un instalador para el usuario
T="dist/FontCraft-$VERSION-linux-$ARCH"
rm -rf "$T"
mkdir -p "$T"
cp "$OUT/FontCraft" "$T/fontcraft"
cp packaging/linux/fontcraft.desktop packaging/linux/fontcraft-library.xml "$T/"
cp assets/icons/png/fontcraft-256.png assets/icons/png/fontcraft-library-256.png "$T/"
cp packaging/linux/install.sh "$T/"
tar -czf "$T.tar.gz" -C dist "FontCraft-$VERSION-linux-$ARCH"
rm -rf "$T"
echo "Listo: carpeta dist ($ARCH)"
