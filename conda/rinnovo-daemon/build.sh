#!/usr/bin/env bash
set -euxo pipefail

# Build the rnb_agent binary in release mode.
cargo build --release -p rnb_agent

# Install into the Conda prefix under the public-facing name rnb_daemon.
mkdir -p "${PREFIX}/bin"
cp "target/release/rnb_agent" "${PREFIX}/bin/rnb_daemon"
