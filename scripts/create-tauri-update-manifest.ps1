param(
  [string]$InstallerUrl,
  [string]$SignaturePath,
  [string]$OutFile = "release/latest.json",
  [string]$Target = "windows-x86_64",
  [string]$Notes = "Dentista v1 Professional update"
)

$ErrorActionPreference = "Stop"

if (-not $InstallerUrl) {
  throw "InstallerUrl is required."
}

if (-not $SignaturePath -or -not (Test-Path -LiteralPath $SignaturePath)) {
  throw "SignaturePath must point to the generated .sig file."
}

$configPath = Join-Path $PSScriptRoot "..\src-tauri\tauri.conf.json"
$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
$signature = (Get-Content -LiteralPath $SignaturePath -Raw).Trim()
$manifest = [ordered]@{
  version = $config.version
  notes = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString("o")
  platforms = [ordered]@{
    $Target = [ordered]@{
      signature = $signature
      url = $InstallerUrl
    }
  }
}

$outDir = Split-Path -Parent $OutFile
if ($outDir -and -not (Test-Path -LiteralPath $outDir)) {
  New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}

$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutFile -Encoding UTF8
Write-Host "Updater manifest written to $OutFile"
