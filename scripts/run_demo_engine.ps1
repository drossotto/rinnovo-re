# Run a local rnb_engine_http instance with sensible defaults
# and registrar wiring so a human user can see the engine appear
# in the remote registrar.

param(
    [int]$Port = 8787
)

$ErrorActionPreference = "Stop"

# Always run from repo root (script is already in scripts/, so go up one level)
Set-Location (Split-Path $PSScriptRoot -Parent)

# Optional: load .env if present (reuse the pattern from test.ps1)
if (Test-Path ".env") {
    Write-Host "== Loading .env into environment =="
    $lines = Get-Content ".env"
    foreach ($line in $lines) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $trimmed = $line.Trim()
        if ($trimmed.StartsWith("#")) { continue }

        $parts = $trimmed.Split("=", 2, [System.StringSplitOptions]::None)
        if ($parts.Length -ne 2) { continue }

        $key = $parts[0].Trim()
        $value = $parts[1].Trim()
        if ($key) {
            Set-Item -Path ("Env:" + $key) -Value $value
        }
    }
}

# Default registrar URL if not provided.
if ([string]::IsNullOrWhiteSpace($env:RINNOVO_REGISTRAR_URL)) {
    $env:RINNOVO_REGISTRAR_URL = "https://registrar.rinnovotech.com"
}

# Configure engine port and endpoint URL.
$env:RINNOVO_ENGINE_PORT = $Port
if ([string]::IsNullOrWhiteSpace($env:RINNOVO_ENGINE_ENDPOINT_URL)) {
    $env:RINNOVO_ENGINE_ENDPOINT_URL = "http://127.0.0.1:$Port"
}

if ([string]::IsNullOrWhiteSpace($env:RINNOVO_ENGINE_NAME)) {
    $env:RINNOVO_ENGINE_NAME = "local-dev"
}

if ([string]::IsNullOrWhiteSpace($env:RINNOVO_ENGINE_KIND)) {
    $env:RINNOVO_ENGINE_KIND = "local"
}

Write-Host "== Starting rnb_engine_http =="
Write-Host "Registrar: $($env:RINNOVO_REGISTRAR_URL)"
Write-Host "Engine endpoint: $($env:RINNOVO_ENGINE_ENDPOINT_URL)"
Write-Host "Engine port: $Port"

cargo run -p rnb_engine_http

