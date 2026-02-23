# Always run from repo root
$ErrorActionPreference = "Stop"

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

Write-Host "== Install Python dev requirements =="
python -m pip install -r requirements-dev.txt

Write-Host "== Rust tests =="
cargo test

Write-Host "== Build/install Python extension (maturin develop) =="
maturin develop --manifest-path crates/bindings/rnb_py/Cargo.toml

Write-Host "== Python tests =="
python -m pytest -q

Write-Host "== Registrar smoke test (Render) =="
$registrarUrl = $env:RINNOVO_REGISTRAR_URL
if ([string]::IsNullOrWhiteSpace($registrarUrl)) {
    Write-Host "RINNOVO_REGISTRAR_URL not set; skipping registrar smoke test."
} else {
    Write-Host "Pinging console session at $registrarUrl/v1/console/session"
    curl "$registrarUrl/v1/console/session"
}

Write-Host "== Conda daemon package build (optional) =="
$conda = Get-Command conda -ErrorAction SilentlyContinue
if ($null -eq $conda) {
    Write-Host "conda not found; skipping Conda daemon integration test."
} else {
    Write-Host "Building rinnovo-daemon Conda package (this may take a while)..."
    conda build conda/rinnovo-daemon
}

Write-Host "== Docker daemon image build (optional) =="
$docker = Get-Command docker -ErrorAction SilentlyContinue
if ($null -eq $docker) {
    Write-Host "docker not found; skipping Docker daemon integration test."
} else {
    Write-Host "Building daemon + engine image from docker/daemon.Dockerfile..."
    docker build -f docker/daemon.Dockerfile -t rinnovo-daemon-test .

    Write-Host "Running Docker daemon smoke test..."
    docker run --rm rinnovo-daemon-test
}
