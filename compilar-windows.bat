@echo off
rem Windows: compila FontCraft con Rust y crea en la carpeta dist:
rem   - FontCraft-<version>-windows-<arq>-portable.zip  (aplicacion portable)
rem   - FontCraft-<version>-windows-<arq>-setup.exe     (instalador, si Inno Setup 6 esta instalado)
rem
rem Uso (doble clic = para este equipo):
rem   compilar-windows.bat          detecta el procesador: x64 (Intel/AMD) o ARM64 (Snapdragon, etc.)
rem   compilar-windows.bat x64      Windows de 64 bits Intel/AMD
rem   compilar-windows.bat arm64    Windows en ARM (ARM64); desde un PC x64 hacen falta las
rem                                 "herramientas de compilacion ARM64" de Visual Studio Build Tools
cd /d "%~dp0"
where cargo >nul 2>nul || (echo No encuentro Rust ^(cargo^). Instalalo desde https://rustup.rs & pause & exit /b 1)
rem The version (GitHub Actions passes the one in Cargo.toml).
if not defined FONTCRAFT_VERSION set FONTCRAFT_VERSION=0.1.0

set ARCH=%1
if "%ARCH%"=="" (
  if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (set ARCH=arm64) else (set ARCH=x64)
)
if /i "%ARCH%"=="arm64" (
  set TARGET=aarch64-pc-windows-msvc
) else if /i "%ARCH%"=="x64" (
  set TARGET=x86_64-pc-windows-msvc
) else (
  echo No conozco "%ARCH%". Usa: x64 o arm64.
  pause
  exit /b 1
)
echo Compilando FontCraft para Windows %ARCH% ^(%TARGET%^)...
rustup target add %TARGET%
cargo build --release --target %TARGET% || (echo La compilacion fallo. & pause & exit /b 1)
set EXE=target\%TARGET%\release\fontcraft.exe

rem Portable: el programa, sus iconos y portable.txt (guarda todo junto al programa).
set P=dist\FontCraft-portable-%ARCH%
if exist %P% rmdir /s /q %P%
mkdir %P%
copy /y %EXE% %P%\ >nul
copy /y assets\icons\fontcraft.ico %P%\ >nul
copy /y assets\icons\fontcraft-library.ico %P%\ >nul
copy /y packaging\windows\LEEME-PORTABLE.txt %P%\ >nul
echo FontCraft portable > %P%\portable.txt
powershell -NoProfile -Command "Compress-Archive -Force -Path '%P%\*' -DestinationPath 'dist\FontCraft-%FONTCRAFT_VERSION%-windows-%ARCH%-portable.zip'"

rem Instalador (Inno Setup 6: https://jrsoftware.org/isdl.php).
set ISCC="%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
if not exist %ISCC% set ISCC="%ProgramFiles%\Inno Setup 6\ISCC.exe"
if exist %ISCC% (
  %ISCC% /DArch=%ARCH% /DExeDir=..\..\target\%TARGET%\release packaging\windows\fontcraft.iss || (echo El instalador fallo. & pause & exit /b 1)
) else (
  echo Inno Setup 6 no esta instalado: se creo solo la version portable.
)
echo Listo: carpeta dist ^(%ARCH%^)
pause
