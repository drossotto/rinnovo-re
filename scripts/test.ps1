# Always run from repo root
$ErrorActionPreference = "Stop"

Write-Host "== Rust tests =="
cargo test

Write-Host "== Build/install Python extension (maturin develop) =="
maturin develop --manifest-path crates/bindings/rnb_py/Cargo.toml

Write-Host "== Python tests =="
python -m pytest -q