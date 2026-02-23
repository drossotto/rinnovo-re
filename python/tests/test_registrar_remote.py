from __future__ import annotations

import os

import httpx
import pytest


@pytest.mark.integration
def test_remote_console_session_shape():
    url = os.getenv("RINNOVO_REGISTRAR_URL")
    if not url:
        pytest.skip("RINNOVO_REGISTRAR_URL not set; skipping remote registrar test")

    base = url.rstrip("/")
    resp = httpx.get(f"{base}/v1/console/session", timeout=5.0)
    resp.raise_for_status()

    data = resp.json()
    assert "profile" in data
    assert "workspace" in data
    # Engine may or may not be present depending on whether an engine has
    # registered, but the field should exist.
    assert "engine" in data
    assert "engine_session_token" in data

