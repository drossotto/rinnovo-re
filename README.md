# Rinnovo Representation Engine

Rinnovo RE is a local‑first representation engine for biological data built
around the Rinnovo Binary (RNB) format. It provides a compact on‑disk
container, a runtime, a small HTTP engine, a registrar, and a
browser console so you can inspect and query artifacts end‑to‑end.

The focus of this repository is a minimal, understandable core: clear
segment types, explicit kernels, and thin interfaces that can be embedded in
larger tools and workflows.

## What’s in this repo?

- **RNB format & runtime** – Rust crates for the `.rnb` container, object and
  bio‑level kernels, and an HTTP engine (`crates/`).
- **Python registrar** – a FastAPI service that tracks profiles, workspaces,
  and engines (`python/registrar`).
- **Web console** – static pages for the engine console, install flow, and
  experimental playgrounds (including an AlphaFold viewer) in `web/`.
- **Docs & specs** – architecture notes, kernel overview, and file‑format
  specification in `docs/` and `site/`.

## Getting started

For an end‑to‑end walkthrough using the registrar, engine, and Python SDK,
see `docs/roommate_quickstart.md`.

Common entry points from the repo root:

- Start the web console for local development: `make dev-web`
- Run core tests and docs checks: `make test-all`
- Build the Python bindings: `make build-py`

## Usage and License

Use of this repository is governed by the accompanying `LICENSE` file. In
summary:

- You may use, view, and modify this code for personal, educational, or
  internal business purposes where such use is appropriate and lawful.
- Any commercial use, redistribution, or offering of this code or derivative
  works requires prior written permission from the copyright holder.
- You may not use any part of this repository (including source code,
  documentation, issues, or other content) to train or improve artificial
  intelligence systems, large language models, or similar machine‑learning
  models.
- You may not scrape, crawl, or otherwise systematically harvest content from
  this repository for inclusion in datasets, search indexes, or AI training
  corpora.

If you are unsure whether a particular use is permitted, treat it as not
permitted unless you have obtained explicit written permission in advance.

See `LICENSE` for the complete terms.
