# Compilar FontCraft / Building FontCraft

Rust compila cada binario en (o para) su sistema. Las bibliotecas propias de Linux (GTK), Windows y
Android solo están en esos sistemas, así que cada binario final se arma allí: a mano con los archivos de
abajo, o automáticamente en GitHub con `.github/workflows/binaries.yml`.

## Sistemas compatibles (de los últimos cinco años, Intel/AMD y ARM64)

| Sistema | Versión mínima | Procesadores |
|---|---|---|
| macOS | 12 Monterey (2021) | Intel y Apple silicon (app universal) |
| Windows | 10 22H2 (2022) y 11 | x64 y ARM64 |
| Linux | Ubuntu 22.04, Debian 12, Mint 21, Fedora 38, openSUSE Leap 15.5, RHEL 9 (o más nuevos) | x64 y ARM64 (ARMv7 solo a mano) |
| Android | 12 (2021) | ARM64 y x86_64 |

Linux tiene ahora también paquete `.rpm` (Fedora, openSUSE, RHEL y derivadas) además de `.deb` y `.tar.gz`.

## A mano (con Rust instalado: https://rustup.rs)

| Sistema | Archivo | Resultado en `dist/` |
|---|---|---|
| macOS (Intel, Apple silicon o universal) | doble clic en `Compilar FontCraft.command` y elegir 1-4 | `FontCraft.app` y `FontCraft-…-macos-<arq>.zip` |
| Windows x64 | doble clic en `compilar-windows.bat` (en un PC x64) o `compilar-windows.bat x64` | `…-windows-x64-portable.zip`, `…-windows-x64-setup.exe` |
| Windows ARM64 | doble clic en `compilar-windows.bat` (en un PC ARM) o `compilar-windows.bat arm64` | `…-windows-arm64-portable.zip`, `…-windows-arm64-setup.exe` |
| Linux x64 / ARM64 / ARMv7 | `sh compilar-linux.sh` (para el propio equipo) | `fontcraft_…_<arq>.deb`, `fontcraft-…<arq>.rpm`, `FontCraft-…-linux-<arq>.tar.gz` |
| Linux ARM desde otro PC | `sh compilar-linux.sh arm64` o `sh compilar-linux.sh armhf` (o `x64`) | lo mismo, para esa arquitectura |
| Android | `sh compilar-android.sh` (en Linux) | `FontCraft-…-android.apk` |

### ARM64 en cada sistema (resumen)

| Sistema | Cómo compilar para ARM64 |
|---|---|
| macOS (Apple silicon: M1, M2, M3, M4…) | `Compilar FontCraft.command` → opción **2** (solo ARM64) o **4** (universal: Intel + ARM64). Se puede hacer desde un Mac Intel. Sin preguntar: `FONTCRAFT_ARCH=arm64 ./"Compilar FontCraft.command"` |
| Windows en ARM (Snapdragon…) | en el PC ARM: doble clic en `compilar-windows.bat`; desde un PC x64: `compilar-windows.bat arm64` |
| Linux ARM64 (Raspberry Pi 4/5 de 64 bits, servidores ARM) | en el equipo ARM: `sh compilar-linux.sh`; desde un PC x64: `sh compilar-linux.sh arm64` (con Docker o Podman) |
| Linux ARM de 32 bits (ARMv7) | `sh compilar-linux.sh armhf` |
| Android 12+ (teléfonos y tabletas ARM64, equipos x86_64) | en Linux: `sh compilar-android.sh` (un solo `.apk` para todos) |

### macOS
- Opción 1 compila para el procesador del Mac que lo ejecuta; 2 para Apple silicon; 3 para Intel; 4 crea una
  app universal que funciona en los dos (es la recomendable para compartir).

### Windows (x64 y ARM64)
- El instalador necesita Inno Setup 6 (https://jrsoftware.org/isdl.php); sin él se crea solo la portable.
- El instalador de ARM64 solo se instala en Windows en ARM; el de x64, en PCs Intel/AMD (y también corre
  emulado en Windows en ARM, aunque es mejor el de ARM64).
- Compilar ARM64 desde un PC x64 pide, en Visual Studio Build Tools, el componente
  "MSVC … ARM64 build tools".

### Linux (x64, ARM64 de 64 bits, ARMv7 de 32 bits)
- En el propio equipo, una vez: `build-essential pkg-config libgtk-3-dev libxdo-dev libayatana-appindicator3-dev`.
- Para otra arquitectura (por ejemplo, armar el de Raspberry Pi desde un PC) se usa `cross`, que funciona
  con Docker o Podman; `Cross.toml` le indica las bibliotecas de Ubuntu de esa arquitectura.
- El `.tar.gz` trae `install.sh`, que instala solo para el usuario, sin administrador.

### Android (desde Linux)
- Necesita el SDK y el NDK de Android (Android Studio › SDK Manager › SDK Tools › NDK, o `sdkmanager`),
  Java 17, y la variable `ANDROID_HOME` (y `ANDROID_NDK_ROOT` si el NDK no está dentro del SDK).
- El script añade a Rust los procesadores de Android, instala `cargo-apk` la primera vez y arma un solo
  `.apk` para teléfonos y tabletas ARM64 y equipos y emuladores x86_64 (Android 12 o posterior).
- Se firma con la clave de depuración; para publicar define `CARGO_APK_RELEASE_KEYSTORE` y
  `CARGO_APK_RELEASE_KEYSTORE_PASSWORD`.
- En Android, FontCraft muestra las fuentes del sistema (`/system/fonts`), las bibliotecas, carpetas y
  Google Fonts. No hay icono en la barra de menús, atajo global ni activación de fuentes para otras apps
  (Android no lo permite), y la carpeta de una biblioteca se escribe o pega.

## Automático (GitHub Actions)

El proyecto trae dos flujos en `.github/workflows/`:

| Flujo | Cuándo se ejecuta | Qué hace |
|---|---|---|
| **FontCraft check** (`check.yml`) | en cada cambio subido a la rama `main` y en cada *pull request* | pasa las pruebas en macOS, Windows y Linux, y comprueba que compila para Android |
| **FontCraft binaries** (`binaries.yml`) | a mano, o al crear una etiqueta `v…` (por ejemplo `v0.1.0`) | genera todos los paquetes (tabla de abajo); con una etiqueta los publica en una *Release* con `SHA256SUMS.txt` |

Paquetes que genera **FontCraft binaries**:

| Sistema | Procesadores | Archivos | Máquina de GitHub |
|---|---|---|---|
| macOS 12+ | Intel + Apple silicon (universal) | `FontCraft-…-macos-x86_64+arm64.zip` | macOS |
| Windows 10 22H2 / 11 | x64 y ARM64 | `…-windows-<arq>-setup.exe`, `…-windows-<arq>-portable.zip` | Windows x64 (ARM64 se compila desde ella) |
| Linux (Ubuntu 22.04+, Debian 12+, Fedora 38+…) | x64 y ARM64 | `.deb`, `.rpm`, `.tar.gz` | Ubuntu 22.04 x64 y Ubuntu 22.04 ARM |
| Linux 32 bits | ARMv7 (Raspberry Pi con sistema de 32 bits) | `.deb`, `.rpm`, `.tar.gz` | Ubuntu x64 con `cross` (si falla, no detiene lo demás) |
| Android 12+ | ARM64 y x86_64 | `FontCraft-…-android.apk` | Ubuntu 24.04 |

### Primera vez

1. Crea una cuenta en https://github.com y un repositorio nuevo (por ejemplo `fontcraft`).
2. Sube el **contenido de la carpeta `app`** (no la carpeta del proyecto entero): el archivo `.gitignore`
   ya deja fuera `target/`, `dist/` y `android/target/`, así que se suben unos 3 MB. Con GitHub Desktop:
   *File › Add local repository…* › la carpeta `app` › *Publish repository*.
3. En el repositorio: **Actions** › acepta activar los flujos si lo pregunta.

### Generar los paquetes

- **A mano:** Actions › *FontCraft binaries* › **Run workflow**. Al terminar (unos 20–30 minutos), los
  archivos están al final de la página de esa ejecución, en *Artifacts* (se guardan 30 días).
- **Publicar una versión:** sube el número en `Cargo.toml` (`version = "0.2.0"`), y crea la etiqueta
  `v0.2.0` (GitHub Desktop: *History* › clic derecho en el último cambio › *Create Tag…* › *Push*). Se
  genera todo y queda publicado en **Releases**.

### Notas

- La máquina Linux ARM (`ubuntu-22.04-arm`) la ofrece GitHub a los repositorios públicos; en uno privado,
  si no está disponible para tu plan, el trabajo de Linux ARM64 se queda esperando: cancélalo o haz el
  repositorio público. Windows ARM64 no depende de eso (se compila desde la máquina x64).
- Firmas: la app de macOS lleva firma local (ad hoc), el instalador de Windows va sin firmar y el `.apk`
  con la clave de depuración. Para repartirlos sin avisos hacen falta un Developer ID de Apple, un
  certificado de firma de código para Windows y una clave de Android, que se guardarían en el
  repositorio como *Secrets* (nunca dentro de los archivos).

## Comprobado en este Mac (Intel)

Se compiló aquí la app universal de macOS (`lipo -archs` → `x86_64 arm64`) y arranca en este Mac.

`cargo check` pasa sin errores ni avisos para Windows x64 y ARM64 (`x86_64/aarch64-pc-windows-msvc`),
Linux x64, ARM64 y ARMv7 (`x86_64/aarch64-unknown-linux-gnu`, `armv7-unknown-linux-gnueabihf`) y Android
ARM64 (`aarch64-linux-android`, paquete `android/`). En Linux y Android se comprueba sin la parte de red,
porque su cifrado necesita los compiladores de esos sistemas. Enlazar los binarios finales requiere cada
sistema (o el flujo de GitHub).

## Qué hace cada paquete

- **Instalador de Windows**: instala en Archivos de programa (o solo para el usuario), acceso en el menú
  Inicio (y en el escritorio si se elige), asocia los `.fcl` con su icono y se desinstala desde
  Configuración. En español y otros 6 idiomas.
- **Portable de Windows**: `fontcraft.exe` + `portable.txt`: guarda todo en `FontCraftData` dentro de su
  carpeta (ideal para una memoria USB).
- **.deb y .rpm de Linux**: programa en `/usr/bin`, entrada del menú, iconos y tipo de archivo `.fcl`.
- Datos: macOS `~/Library/Application Support/FontCraft`, Windows `%APPDATA%\FontCraft`, Linux
  `~/.local/share/FontCraft`, Android la carpeta privada de la app (o la carpeta portable).

## Iconos

Los iconos de la app y de los archivos `.fcl` salen de los dibujos de `assets/icons/source` (claro y oscuro).
Para volver a generarlos tras cambiar un dibujo: `cargo run -- --make-icons assets/icons` (macOS `.icns`,
Windows `.ico`, PNG de Linux) y `cargo run -- --make-android-icons android/res` (Android, con icono adaptable
y versión nocturna).

## Notas

- macOS: firma local (ad hoc). Para repartirla sin avisos hace falta Developer ID y notarización.
- Windows: SmartScreen puede avisar con un instalador sin firma de código (`signtool` + certificado).
