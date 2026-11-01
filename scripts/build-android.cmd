@echo off
setlocal
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set JAVA_HOME=C:\Users\l\AppData\Local\Java\jdk17
set ANDROID_HOME=C:\Users\l\AppData\Local\Android\Sdk
set ANDROID_SDK_ROOT=C:\Users\l\AppData\Local\Android\Sdk
set NDK_HOME=C:\Users\l\AppData\Local\Android\Sdk\ndk\26.1.10909125

REM tauri-plugin build scripts copy JS into the plugin crate inside CARGO_HOME;
REM %USERPROFILE%\.cargo is not writable for spawned build scripts here.
set CARGO_HOME=D:\app\Web Browser\alux\.cargo

REM The NDK linker wrapper is a .cmd that cannot handle spaces in the
REM linker-arguments response file path, so keep build output outside the
REM "Web Browser" directory.
set CARGO_TARGET_DIR=D:\alux-target

REM cl.exe fails with D8050 when emitting /Z7 debug records for the
REM vswhom-sys C++ build script; drop debug info for dev builds.
set CARGO_PROFILE_DEV_DEBUG=0
set CARGO_PROFILE_DEV_STRIP=debuginfo

cd /d "D:\app\Web Browser\alux"
npx tauri android build --apk --debug --target aarch64 --config tauri.android.conf.json
exit /b %ERRORLEVEL%
