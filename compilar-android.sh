#!/bin/sh
# Linux (x64 o ARM64): compila FontCraft para Android con Rust y deja en dist/
#   FontCraft-<versión>-android.apk  (teléfonos y tabletas ARM64, y equipos y emuladores x86_64; Android 12 o posterior)
#
# Necesita (una vez):
#   - Rust (https://rustup.rs)
#   - Android SDK y NDK, por ejemplo con Android Studio (SDK Manager › SDK Tools › NDK) o con
#     sdkmanager "platforms;android-35" "build-tools;35.0.0" "ndk;26.3.11579264"
#   - Java (JDK 17), para firmar el .apk
#   - Variables: ANDROID_HOME (carpeta del SDK) y, si no está dentro del SDK, ANDROID_NDK_ROOT
#
# Firma: sin más datos se firma con la clave de depuración (para instalar y probar). Para publicar,
# antes de compilar define CARGO_APK_RELEASE_KEYSTORE y CARGO_APK_RELEASE_KEYSTORE_PASSWORD.
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null 2>&1 || { echo "No encuentro Rust (cargo). Instálalo desde https://rustup.rs"; exit 1; }

# El SDK y el NDK de Android.
ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
[ -d "$ANDROID_HOME" ] || { echo "No encuentro el SDK de Android. Define ANDROID_HOME (ahora: $ANDROID_HOME)."; exit 1; }
# Without ANDROID_NDK_ROOT, or with one that doesn't exist: the NDK inside the SDK (26.3, the one
# FontCraft is built with, when it's there; else the newest).
if [ -z "$ANDROID_NDK_ROOT" ] || [ ! -d "$ANDROID_NDK_ROOT" ]; then
  if [ -d "$ANDROID_HOME/ndk/26.3.11579264" ]; then
    ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/26.3.11579264"
  else
    ANDROID_NDK_ROOT=$(ls -d "$ANDROID_HOME"/ndk/* 2>/dev/null | sort -V | tail -1)
  fi
fi
[ -d "$ANDROID_NDK_ROOT" ] || { echo "No encuentro el NDK de Android. Instálalo con el SDK Manager o define ANDROID_NDK_ROOT."; exit 1; }
export ANDROID_HOME ANDROID_SDK_ROOT="$ANDROID_HOME" ANDROID_NDK_ROOT ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
echo "SDK: $ANDROID_HOME"
echo "NDK: $ANDROID_NDK_ROOT"

# Los procesadores de Android para Rust y la herramienta que arma el .apk.
rustup target add aarch64-linux-android x86_64-linux-android
command -v cargo-apk >/dev/null 2>&1 || cargo install cargo-apk

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
# The key the .apk is signed with: the one given in CARGO_APK_RELEASE_KEYSTORE (and its password),
# else this computer's Android debug key, made the first time (as Android Studio makes it).
if [ -z "$CARGO_APK_RELEASE_KEYSTORE" ]; then
  KS="$HOME/.android/debug.keystore"
  if [ ! -f "$KS" ]; then
    mkdir -p "$HOME/.android"
    keytool -genkeypair -v -keystore "$KS" -storepass android -keypass android -alias androiddebugkey \
      -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US" >/dev/null
  fi
  export CARGO_APK_RELEASE_KEYSTORE="$KS"
  export CARGO_APK_RELEASE_KEYSTORE_PASSWORD="android"
fi
command -v keytool >/dev/null 2>&1 || [ -f "$CARGO_APK_RELEASE_KEYSTORE" ] || { echo "Falta Java (keytool) para firmar el .apk."; exit 1; }
(cd android && cargo apk build --release)
mkdir -p dist
cp android/target/release/apk/FontCraft.apk "dist/FontCraft-$VERSION-android.apk"
echo ""
echo "Listo: dist/FontCraft-$VERSION-android.apk"
echo "Para instalarlo en un teléfono conectado por USB (con depuración USB activada):"
echo "  $ANDROID_HOME/platform-tools/adb install -r dist/FontCraft-$VERSION-android.apk"
