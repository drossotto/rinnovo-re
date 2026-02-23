from __future__ import annotations

from types import SimpleNamespace

import pytest

import rinnovo_sdk.registrar_client as rc


class _FakeResponse:
    def __init__(self, json_obj):
        self._json = json_obj
        self.status_code = 200

    def raise_for_status(self) -> None:  # pragma: no cover - trivial
        return

    def json(self):
        return self._json


def test_login_parses_profile(monkeypatch):
    def fake_get(url, timeout=5.0):
        assert url.endswith("/v1/profiles")
        return _FakeResponse(
            [
                {
                    "id": "prof_default",
                    "name": "Default",
                    "default_workspace_id": "ws_default",
                }
            ]
        )

    monkeypatch.setenv("RINNOVO_REGISTRAR_URL", "https://example.com")
    monkeypatch.setattr(rc, "httpx", SimpleNamespace(get=fake_get))

    profile = rc.login()
    assert profile.id == "prof_default"
    assert profile.name == "Default"
    assert profile.default_workspace_id == "ws_default"


def test_list_workspaces_uses_login_when_profile_missing(monkeypatch):
    calls = []

    def fake_get(url, timeout=5.0):
        calls.append(url)
        if url.endswith("/v1/profiles"):
            return _FakeResponse(
                [
                    {
                        "id": "prof_default",
                        "name": "Default",
                        "default_workspace_id": "ws_default",
                    }
                ]
            )
        elif url.endswith("/v1/profiles/prof_default/workspaces"):
            return _FakeResponse(
                [
                    {"id": "ws_default", "name": "Local Workspace", "engine_id": None},
                    {"id": "ws_other", "name": "Other", "engine_id": "eng_1"},
                ]
            )
        else:
            pytest.fail(f"unexpected URL {url}")

    monkeypatch.setenv("RINNOVO_REGISTRAR_URL", "https://example.com")
    monkeypatch.setattr(rc, "httpx", SimpleNamespace(get=fake_get))

    workspaces = rc.list_workspaces()
    assert [w.id for w in workspaces] == ["ws_default", "ws_other"]
    assert workspaces[0].engine_id is None
    assert workspaces[1].engine_id == "eng_1"
    assert any(url.endswith("/v1/profiles") for url in calls)
    assert any(url.endswith("/v1/profiles/prof_default/workspaces") for url in calls)

