#!/bin/sh
# Doble clic en el Finder: compila FontCraft con Rust (cargo) y deja la app lista en la carpeta dist.
# Double-click in Finder: builds FontCraft with Rust (cargo) and leaves the app in the dist folder.
#
# Al abrirlo pregunta para qué Mac compilar:
#   1) Este Mac (lo que tenga: Intel o Apple silicon)
#   2) Apple silicon (ARM64: M1, M2, M3, M4…)
#   3) Intel (x64)
#   4) Universal (Intel + Apple silicon en una sola app)
# Sin preguntar, desde la Terminal: FONTCRAFT_ARCH=arm64 (o x64, universal, native) ./"Compilar FontCraft.command"
cd "$(dirname "$0")" || exit 1
export PATH="$HOME/.cargo/bin:$PATH"
# macOS 12 Monterey (2021) or newer: systems from the last five years, Intel and Apple silicon.
export MACOSX_DEPLOYMENT_TARGET=12.0
if ! command -v cargo >/dev/null 2>&1; then
  echo "No encuentro Rust (cargo). Instálalo desde https://rustup.rs y vuelve a abrir este archivo."
  read -r _
  exit 1
fi

fail() {
  echo ""
  echo "$1 Pulsa Intro para cerrar."
  read -r _
  exit 1
}

CHOICE="$FONTCRAFT_ARCH"
if [ -z "$CHOICE" ]; then
  echo "¿Para qué Mac compilar FontCraft?"
  echo "  1) Este Mac ($(uname -m))"
  echo "  2) Apple silicon (ARM64)"
  echo "  3) Intel (x64)"
  echo "  4) Universal (Intel + Apple silicon)"
  printf "Elige 1-4 [1]: "
  read -r n
  case "$n" in
    2) CHOICE=arm64 ;;
    3) CHOICE=x64 ;;
    4) CHOICE=universal ;;
    *) CHOICE=native ;;
  esac
fi

echo "Compilando FontCraft ($CHOICE) con $(cargo --version)…"
case "$CHOICE" in
  native)
    cargo build --release || fail "La compilación falló (el mensaje de arriba dice por qué)."
    BIN=target/release/FontCraft
    ;;
  arm64|x64)
    if [ "$CHOICE" = arm64 ]; then T=aarch64-apple-darwin; else T=x86_64-apple-darwin; fi
    rustup target add "$T" || fail "No se pudo preparar Rust para $T."
    cargo build --release --target "$T" || fail "La compilación falló (el mensaje de arriba dice por qué)."
    BIN="target/$T/release/FontCraft"
    ;;
  universal)
    rustup target add aarch64-apple-darwin x86_64-apple-darwin || fail "No se pudo preparar Rust para las dos arquitecturas."
    cargo build --release --target aarch64-apple-darwin || fail "La compilación para Apple silicon falló."
    cargo build --release --target x86_64-apple-darwin || fail "La compilación para Intel falló."
    lipo -create target/aarch64-apple-darwin/release/FontCraft target/x86_64-apple-darwin/release/FontCraft -output target/fontcraft-universal
    BIN=target/fontcraft-universal
    ;;
  *) fail "No conozco \"$CHOICE\". Usa: native, arm64, x64 o universal." ;;
esac

sh packaging/macos/make-app.sh "$BIN" || fail "No se pudo armar la app."
echo ""
echo "Listo: dist/FontCraft.app (y un .zip para compartir) en $(pwd)/dist — arquitecturas: $(lipo -archs "$BIN")"
[ -z "$FONTCRAFT_NO_OPEN" ] && open dist
