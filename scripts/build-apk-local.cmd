@echo off
REM Build the Alux .apk on this machine without `tauri android build`,
REM which regenerates the Android sources (wiping the Alux patches) and
REM then fails on the symlink step.
REM
REM  1. vcvars64 + ANDROID_HOME/NDK + CARGO_HOME/CARGO_TARGET_DIR (space-free)
REM  2. cargo build the Rust lib for aarch64-linux-android
REM  3. copy .so + frontend into the Gradle project
REM  4. gradlew assembleDebug, skipping the Rust tasks (they need npm.bat)
setlocal
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set JAVA_HOME=C:\Users\l\AppData\Local\Java\jdk17
set ANDROID_HOME=C:\Users\l\AppData\Local\Android\Sdk
set ANDROID_SDK_ROOT=C:\Users\l\AppData\Local\Android\Sdk
set NDK_HOME=C:\Users\l\AppData\Local\Android\Sdk\ndk\26.1.10909125
set CARGO_HOME=D:\app\Web Browser\alux\.cargo
set CARGO_TARGET_DIR=D:\alux-target
set CARGO_PROFILE_DEV_DEBUG=0
set CARGO_PROFILE_DEV_STRIP=debuginfo

REM Tauri CLI normally wires these up; calling cargo directly needs them.
set NDKBIN=%NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\bin
set CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=%NDKBIN%\aarch64-linux-android24-clang.cmd
set CC_aarch64_linux_android=%NDKBIN%\aarch64-linux-android24-clang.cmd
set CXX_aarch64_linux_android=%NDKBIN%\aarch64-linux-android24-clang++.cmd
set AR_aarch64_linux_android=%NDKBIN%\llvm-ar.exe
set RANLIB_aarch64_linux_android=%NDKBIN%\llvm-ranlib.exe

cd /d "D:\app\Web Browser\alux"

echo == frontend ==
call npm run build || exit /b 1

echo == rust (aarch64-linux-android) ==
cargo build --package alux --manifest-path src-tauri\Cargo.toml --target aarch64-linux-android --features tauri/custom-protocol --lib || exit /b 1

set GEN=D:\app\Web Browser\alux\src-tauri\gen\android
echo == copy artifacts ==
copy /Y "D:\alux-target\aarch64-linux-android\debug\libalux_lib.so" "%GEN%\app\src\main\jniLibs\arm64-v8a\libalux_lib.so" >nul || exit /b 1
REM Wipe stale bundles: the assets dir otherwise keeps every previous build's
REM hashed index-*.js and the APK ships them all.
if exist "%GEN%\app\src\main\assets" rd /s /q "%GEN%\app\src\main\assets"
mkdir "%GEN%\app\src\main\assets"
xcopy /E /I /Y "D:\app\Web Browser\alux\dist\*" "%GEN%\app\src\main\assets\" >nul || exit /b 1

echo == gradle ==
REM Stale packaging output here produces a 47 MB APK with ~20 MB of orphan data.
if exist "%GEN%\app\build\outputs\apk" rd /s /q "%GEN%\app\build\outputs\apk"
cd /d "%GEN%"
call gradlew.bat assembleDebug --no-daemon -x rustBuildArmDebug -x rustBuildArm64Debug -x rustBuildX86Debug -x rustBuildX86_64Debug || exit /b 1
echo APK: %GEN%\app\build\outputs\apk\arm64\debug\app-arm64-debug.apk
