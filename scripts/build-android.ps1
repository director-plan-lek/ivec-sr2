#requires -Version 7.0
<#
  Alux — สคริปต์เตรียมสภาพแวดล้อมและ build .apk
  ตรวจของที่ต้องมีก่อน แล้วค่อยสั่ง build (ไม่ดาวน์โหลด SDK ให้อัตโนมัติ)

  วิธีใช้:
    pwsh -File scripts/build-android.ps1            # ตรวจ env + build debug apk
    pwsh -File scripts/build-android.ps1 -CheckOnly # ตรวจอย่างเดียว
#>
param(
  [switch]$CheckOnly,
  [switch]$Release
)

$ErrorActionPreference = 'Stop'
$problems = @()

function Test-Tool([string]$name, [scriptblock]$probe) {
  try { $v = & $probe; Write-Host ("  [OK]   {0}: {1}" -f $name, $v) -ForegroundColor Green }
  catch { Write-Host ("  [MISS] {0}" -f $name) -ForegroundColor Red; $script:problems += $name }
}

Write-Host "`n== 1. ตรวจเครื่องมือ ==" -ForegroundColor Cyan
Test-Tool 'node'  { node --version }
Test-Tool 'npm'   { npm --version }
Test-Tool 'cargo' { cargo --version }

if (-not $env:JAVA_HOME) { $env:JAVA_HOME = 'C:\Program Files\Eclipse Adoptium\jdk-17.0.16.8-hotspot' }
Test-Tool 'java (JDK 17)' { & "$env:JAVA_HOME\bin\java.exe" -version 2>&1 | Select-Object -First 1 }

if (-not $env:ANDROID_HOME) { $env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk" }
if (Test-Path $env:ANDROID_HOME) { Write-Host "  [OK]   ANDROID_HOME: $env:ANDROID_HOME" -ForegroundColor Green }
else { Write-Host "  [MISS] ANDROID_HOME ($env:ANDROID_HOME)" -ForegroundColor Red; $problems += 'Android SDK' }

if (-not $env:NDK_HOME -and (Test-Path "$env:ANDROID_HOME\ndk")) {
  $env:NDK_HOME = (Get-ChildItem "$env:ANDROID_HOME\ndk" -Directory | Sort-Object Name -Descending | Select-Object -First 1).FullName
}
if ($env:NDK_HOME) { Write-Host "  [OK]   NDK: $env:NDK_HOME" -ForegroundColor Green }
else { Write-Host "  [MISS] Android NDK" -ForegroundColor Red; $problems += 'Android NDK' }

Write-Host "`n== 2. ตรวจ Rust targets ==" -ForegroundColor Cyan
$installed = (rustup target list --installed) -join ' '
foreach ($t in 'aarch64-linux-android','armv7-linux-androideabi','x86_64-linux-android') {
  if ($installed -match [regex]::Escape($t)) { Write-Host "  [OK]   $t" -ForegroundColor Green }
  else { Write-Host "  [MISS] $t  (rustup target add $t)" -ForegroundColor Yellow; $problems += "target $t" }
}

if ($problems.Count -gt 0) {
  Write-Host "`nยังขาด: $($problems -join ', ')" -ForegroundColor Yellow
  Write-Host "อ่านขั้นตอนติดตั้งที่ docs/ANDROID-PORT.md`n" -ForegroundColor Yellow
  if ($CheckOnly) { exit 1 }
}

if ($CheckOnly) { Write-Host "`nตรวจเสร็จ`n"; exit 0 }

Write-Host "`n== 3. ติดตั้ง Rust targets ที่ขาด ==" -ForegroundColor Cyan
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

if (-not (Test-Path 'src-tauri/gen/android')) {
  Write-Host "`n== 4. สร้างโปรเจกต์ Android ครั้งแรก ==" -ForegroundColor Cyan
  npx tauri android init
}

Write-Host "`n== 5. Build ==" -ForegroundColor Cyan
if ($Release) { npx tauri android build --apk }
else { npx tauri android build --apk --debug }

Write-Host "`nไฟล์ .apk อยู่ที่ src-tauri/gen/android/app/build/outputs/apk/" -ForegroundColor Green
