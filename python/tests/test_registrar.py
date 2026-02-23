from __future__ import annotations

from python.registrar.app import (
    app,
    DEFAULT_PROFILE_ID,
    DEFAULT_WORKSPACE_ID,
)
from fastapi.testclient import TestClient


def test_register_engine_heartbeat_and_session_flow() -> None:
    client = TestClient(app)

    # Initially, no engines but a default profile exists.
    resp_profiles = client.get("/v1/profiles")
    assert resp_profiles.status_code == 200
    profiles = resp_profiles.json()
    assert any(p["id"] == DEFAULT_PROFILE_ID for p in profiles)

    # Register a new engine.
    resp_reg = client.post(
        "/v1/engines/register",
        json={
            "name": "local-dev",
            "kind": "local",
            "endpoint_url": "http://127.0.0.1:8787",
            "version": "0.1.0",
            "capabilities": ["rnb:v1"],
        },
    )
    assert resp_reg.status_code == 200
    data = resp_reg.json()
    engine_id = data["engine_id"]
    heartbeat_token = data["heartbeat_token"]
    assert data["profile_id"] == DEFAULT_PROFILE_ID

    # Heartbeat with correct token should succeed.
    resp_hb = client.post(
        f"/v1/engines/{engine_id}/heartbeat",
        headers={"X-Engine-Token": heartbeat_token},
        json={"status": "online"},
    )
    assert resp_hb.status_code == 200

    # Listing engines for the default profile should include our engine.
    resp_engines = client.get(f"/v1/profiles/{DEFAULT_PROFILE_ID}/engines")
    assert resp_engines.status_code == 200
    engines = resp_engines.json()
    assert any(e["id"] == engine_id for e in engines)

    # Console session should now surface the default profile, workspace,
    # and the registered engine.
    resp_session = client.get("/v1/console/session")
    assert resp_session.status_code == 200
    sess = resp_session.json()
    assert sess["profile"]["id"] == DEFAULT_PROFILE_ID
    assert sess["workspace"]["id"] == DEFAULT_WORKSPACE_ID
    assert sess["engine"] is not None
    assert sess["engine"]["id"] == engine_id
    assert isinstance(sess["engine_session_token"], str)
    assert sess["engine_session_token"]


def test_heartbeat_rejects_invalid_token() -> None:
    client = TestClient(app)

    # Register an engine to obtain a valid id.
    resp_reg = client.post(
        "/v1/engines/register",
        json={
            "name": "local-dev",
            "kind": "local",
            "endpoint_url": "http://127.0.0.1:8787",
            "version": "0.1.0",
            "capabilities": [],
        },
    )
    assert resp_reg.status_code == 200
    engine_id = resp_reg.json()["engine_id"]

    # Heartbeat with an invalid token should be rejected.
    resp_hb = client.post(
        f"/v1/engines/{engine_id}/heartbeat",
        headers={"X-Engine-Token": "invalid"},
        json={"status": "online"},
    )
    assert resp_hb.status_code == 401
