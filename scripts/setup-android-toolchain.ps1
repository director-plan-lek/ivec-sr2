#requires -Version 7.0
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$root    = "$env:LOCALAPPDATA\AluxAndroid"
$jdkDir  = "$env:LOCALAPPDATA\Java"
$sdkDir  = "$env:LOCALAPPDATA\Android\Sdk"
New-Item -ItemType Directory -Force -Path $root, $jdkDir, $sdkDir | Out-Null

function Get-File([string]$url, [string]$dest) {
  if (Test-Path $dest) { Write-Host "  cached: $dest"; return }
  Write-Host "  downloading $url"
  curl.exe -sSL --retry 3 --max-time 1800 -o $dest $url
  if (-not (Test-Path $dest)) { throw "download failed: $url" }
  Write-Host ("  got {0:N1} MB" -f ((Get-Item $dest).Length / 1MB))
}

Write-Host '== JDK 17 (Temurin) ==' -ForegroundColor Cyan
$jdkZip = "$root\jdk17.zip"
Get-File 'https://api.adoptium.net/v3/binary/latest/17/ga/windows/x64/jdk/hotspot/normal/eclipse' $jdkZip
$jdkOut = "$jdkDir\jdk17"
if (-not (Test-Path $jdkOut)) {
  New-Item -ItemType Directory -Force -Path $jdkOut | Out-Null
  tar.exe -xf $jdkZip -C $jdkOut
  # zip มีโฟลเดอร์ชั้นเดียว (jdk-17.0.x) — ย้ายขึ้นมาให้ JAVA_HOME ตรง
  $inner = Get-ChildItem $jdkOut -Directory | Select-Object -First 1
  if ($inner -and -not (Test-Path "$jdkOut\bin\java.exe")) {
    Get-ChildItem $inner.FullName -Force | Move-Item -Destination $jdkOut -Force
    Remove-Item $inner.FullName -Recurse -Force
  }
}
if (Test-Path "$jdkOut\bin\java.exe") { Write-Host "  JAVA_HOME=$jdkOut" -ForegroundColor Green }
else { throw 'JDK layout unexpected' }

Write-Host '== Android command-line tools ==' -ForegroundColor Cyan
$cmdToolsZip = "$root\cmdtools.zip"
Get-File 'https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip' $cmdToolsZip
$cmdToolsDir = "$sdkDir\cmdline-tools"
if (-not (Test-Path "$cmdToolsDir\latest\bin\sdkmanager.bat")) {
  New-Item -ItemType Directory -Force -Path $cmdToolsDir | Out-Null
  tar.exe -xf $cmdToolsZip -C $cmdToolsDir
  if (Test-Path "$cmdToolsDir\cmdline-tools") { Move-Item "$cmdToolsDir\cmdline-tools" "$cmdToolsDir\latest" -Force }
}
Write-Host ("  sdkmanager: " + (Test-Path "$cmdToolsDir\latest\bin\sdkmanager.bat")) -ForegroundColor Green

Write-Host '== sdkmanager packages ==' -ForegroundColor Cyan
$env:JAVA_HOME = $jdkOut
$env:ANDROID_HOME = $sdkDir
$env:ANDROID_SDK_ROOT = $sdkDir
$sm = "$cmdToolsDir\latest\bin\sdkmanager.bat"
$yes = ("y`n" * 20)
$yes | & $sm --sdk_root=$sdkDir --licenses 2>&1 | Select-Object -Last 3
& $sm --sdk_root=$sdkDir 'platform-tools' 'platforms;android-34' 'build-tools;34.0.0' 'ndk;26.1.10909125' 2>&1 | Select-Object -Last 8

Write-Host '== result ==' -ForegroundColor Cyan
foreach ($p in "$sdkDir\platform-tools\adb.exe", "$sdkDir\ndk\26.1.10909125\ndk-build.cmd") {
  Write-Host ("  {0}: {1}" -f $p, (Test-Path $p))
}
Get-ChildItem "$sdkDir\ndk" -ErrorAction SilentlyContinue | Select-Object Name
