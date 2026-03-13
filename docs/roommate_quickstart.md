# Roommate Quickstart: First Real User Flow

This document describes the smallest end‑to‑end path for a human user
to interact with the Rinnovo Representation Engine today.

The goal is:

- start a local engine that registers with the hosted registrar
- see that engine in the registrar API
- open an artifact from Python via the SDK

## 1. Prerequisites

- Rust toolchain (stable) installed
- Python 3.11 with a virtualenv (`.venv`) created
- This repo cloned locally and `maturin develop` run at least once
  so that the `rinnovo` extension is installed into the virtualenv
- The registrar deployed and reachable at:
  `https://registrar.rinnovotech.com`

Environment variable shortcut (optional, but recommended) in a
root‑level `.env` file:

```text
RINNOVO_REGISTRAR_URL=https://registrar.rinnovotech.com
RINNOVO_ENGINE_ENDPOINT_URL=http://127.0.0.1:8787
RINNOVO_ENGINE_NAME=local-dev
RINNOVO_ENGINE_KIND=local
```

## 2. Start a local engine bound to the registrar

From the repo root in PowerShell (with `.venv` activated):

```powershell
.\scripts\run_demo_engine.ps1
```

This will:

- load `.env` if present
- default `RINNOVO_REGISTRAR_URL` to
  `https://registrar.rinnovotech.com` if not set
- start `rnb_engine_http` on port `8787` (override with `-Port 8790`)
- register the engine with the registrar and begin heartbeating

You should see a line such as:

```text
rnb_engine_http listening on 0.0.0.0:8787
```

In another terminal, confirm that the registrar sees the engine:

```powershell
curl "$env:RINNOVO_REGISTRAR_URL/v1/profiles/prof_default/engines"
```

The response should contain a single `local-dev` engine with status
`online`.

## 3. Create a tiny demo artifact

In the same repo (with `.venv` active):

```powershell
.\.venv\Scripts\python.exe -c `
  "import pathlib, rinnovo; p = pathlib.Path('demo_empty.rnb'); `
   rinnovo.write_empty(str(p)); print('wrote', p)"
```

This produces a minimal but fully valid `.rnb` artifact on disk.

## 4. Explore from Python via the SDK

Launch a Python REPL or notebook using the virtualenv:

```powershell
.\.venv\Scripts\python.exe
```

Then:

```python
import os
from pathlib import Path

from rinnovo_sdk import Artifact, login, list_workspaces

os.environ["RINNOVO_REGISTRAR_URL"] = "https://registrar.rinnovotech.com"

profile = login()
print(profile)

workspaces = list_workspaces(profile_id=profile.id)
print(workspaces)

art = Artifact.open(Path("demo_empty.rnb"))
print("Header:", art.header)
print("Required segments:", art.required_segments)
```

At this stage the artifact is structurally minimal (no biology‑specific
tables yet), but the full path from:

registrar → engine → SDK → artifact

is exercised and ready for deeper bio‑level kernels in later slices.

