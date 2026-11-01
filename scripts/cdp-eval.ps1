param(
  [string]$Expression = "document.title",
  [string]$TargetUrlMatch = "tauri.localhost",
  [int]$WaitMs = 0,
  [string]$ScreenshotPath = ""
)
$ErrorActionPreference = 'Stop'

function Get-Target([string]$match) {
  $targets = Invoke-RestMethod -Uri 'http://127.0.0.1:9222/json' -TimeoutSec 15
  if ($match -eq '*') { return $targets | Select-Object -Last 1 }
  return $targets | Where-Object { $_.url -like "*$match*" } | Select-Object -First 1
}

function Invoke-Cdp([string]$wsUrl, [hashtable]$payload, [int]$timeoutSec = 60) {
  $ws = New-Object System.Net.WebSockets.ClientWebSocket
  $ct = [System.Threading.CancellationToken]::None
  $ws.ConnectAsync([Uri]$wsUrl, $ct).Wait($timeoutSec * 1000) | Out-Null
  $json = $payload | ConvertTo-Json -Depth 10 -Compress
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
  $seg = New-Object System.ArraySegment[byte] -ArgumentList @(,$bytes)
  $ws.SendAsync($seg, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, $ct).Wait() | Out-Null
  $buffer = New-Object byte[] 262144
  $sb = New-Object System.Text.StringBuilder
  while ($true) {
    $recvSeg = New-Object System.ArraySegment[byte] -ArgumentList @(,$buffer)
    $res = $ws.ReceiveAsync($recvSeg, $ct)
    $res.Wait($timeoutSec * 1000) | Out-Null
    [void]$sb.Append([System.Text.Encoding]::UTF8.GetString($buffer, 0, $res.Result.Count))
    if ($res.Result.EndOfMessage) { break }
  }
  $ws.Dispose()
  return $sb.ToString()
}

$t = Get-Target $TargetUrlMatch
if (-not $t) { Write-Output "TARGET NOT FOUND ($TargetUrlMatch)"; exit 1 }
Write-Output "target: $($t.title) $($t.url)"

$payload = @{ id = 1; method = 'Runtime.evaluate'; params = @{ expression = $Expression; returnByValue = $true; awaitPromise = $true } }
$raw = Invoke-Cdp $t.webSocketDebuggerUrl $payload
$obj = $raw | ConvertFrom-Json
if ($obj.result.result.value -ne $null) { Write-Output ("RESULT: " + ($obj.result.result.value | ConvertTo-Json -Compress -Depth 6)) }
elseif ($obj.result.exceptionDetails) { Write-Output ("EXCEPTION: " + ($obj.result.exceptionDetails | ConvertTo-Json -Compress -Depth 6)) }
else { Write-Output ("RAW: " + $raw) }

if ($WaitMs -gt 0) { Start-Sleep -Milliseconds $WaitMs }

if ($ScreenshotPath) {
  $shot = Invoke-Cdp $t.webSocketDebuggerUrl @{ id = 2; method = 'Page.captureScreenshot'; params = @{ format = 'png' } }
  $img = ($shot | ConvertFrom-Json).result.data
  [System.IO.File]::WriteAllBytes($ScreenshotPath, [Convert]::FromBase64String($img))
  Write-Output "screenshot: $ScreenshotPath"
}
