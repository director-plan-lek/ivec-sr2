# Build Alux for Windows (.exe + installer)
# Usage:  pwsh -File scripts\build-windows.ps1
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "==> 1/3 dependencies" -ForegroundColor Cyan
if (-not (Test-Path "node_modules")) { npm install --ignore-scripts }

Write-Host "==> 2/3 frontend (TypeScript)" -ForegroundColor Cyan
npm run build

Write-Host "==> 3/3 Tauri release build" -ForegroundColor Cyan
npx tauri build

$release = Join-Path $root "src-tauri\target\release"
$bundle  = Join-Path $release "bundle\nsis"

Write-Host ""
Write-Host "==> done" -ForegroundColor Green
Get-ChildItem $release -Filter "alux.exe" -ErrorAction SilentlyContinue | ForEach-Object {
    "portable : {0}  ({1:N1} MB)" -f $_.FullName, ($_.Length / 1MB)
}
Get-ChildItem $bundle -Filter "*.exe" -ErrorAction SilentlyContinue | ForEach-Object {
    "installer: {0}  ({1:N1} MB)" -f $_.FullName, ($_.Length / 1MB)
}
