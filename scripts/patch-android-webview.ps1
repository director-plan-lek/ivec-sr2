#requires -Version 7.0
<#
  Re-applies the Alux Android patches after `npx tauri android init`.

  Tauri generates the files below with "DO NOT MODIFY" and they lack two things a
  real browser needs:

    RustWebViewClient.kt — forwards every URL to Rust, so Android deep links
                           (intent://, market://, tel:) become net::ERR_UNKNOWN_URL_SCHEME.
    RustWebView.kt       — never enables third-party cookies, which Android blocks by
                           default for targetSdk >= 30, breaking Google sign-in
                           (net::ERR_CONNECTION_ABORTED on accounts.google.com).

  The patched copies live in scripts/android/.
#>
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$genDir = 'src-tauri\gen\android\app\src\main\java\com\alux\browser'

$files = @(
  @{ Name = 'generated\RustWebViewClient.kt'; Template = 'android\RustWebViewClient.kt' },
  @{ Name = 'generated\RustWebView.kt';       Template = 'android\RustWebView.kt' },
  @{ Name = 'MainActivity.kt';                Template = 'android\MainActivity.kt' }
)

if (-not (Test-Path (Join-Path $root $genDir))) {
  Write-Host "generated android sources not found — run 'npx tauri android init' first" -ForegroundColor Yellow
  exit 1
}

foreach ($f in $files) {
  $template = Join-Path $PSScriptRoot $f.Template
  $target = Join-Path $root (Join-Path $genDir $f.Name)

  if (-not (Test-Path $template)) {
    Write-Host "missing template: $template" -ForegroundColor Red
    exit 1
  }
  if (-not (Test-Path $target)) {
    Write-Host "missing generated file: $target" -ForegroundColor Red
    exit 1
  }

  if ((Get-Content $target -Raw) -match 'Alux patch') {
    Write-Host ("already patched: {0}" -f $f.Name) -ForegroundColor Green
    continue
  }

  Copy-Item $template $target -Force
  Write-Host ("patched: {0}" -f $f.Name) -ForegroundColor Green
}
