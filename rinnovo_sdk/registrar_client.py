from __future__ import annotations

import os
from dataclasses import dataclass
from typing import List, Optional

import httpx


def _base_url(override: Optional[str] = None) -> str:
    """
    Resolve the registrar base URL from an explicit override or the
    RINNOVO_REGISTRAR_URL environment variable.
    """
    url = override or os.getenv("RINNOVO_REGISTRAR_URL")
    if not url:
        raise RuntimeError(
            "RINNOVO_REGISTRAR_URL is not set; set it to your registrar base "
            "URL (e.g. https://rinnovo-re.onrender.com)."
        )
    return url.rstrip("/")


@dataclass
class ProfileSummary:
    id: str
    name: str
    default_workspace_id: str


@dataclass
class WorkspaceSummary:
    id: str
    name: str
    engine_id: Optional[str]


def login(registrar_url: Optional[str] = None, timeout: float = 5.0) -> ProfileSummary:
    """
    "Log in" to the registrar by checking connectivity and fetching the
    active profile.

    At this stage there is no user-level authentication; this simply
    verifies that the registrar is reachable and returns the first
    configured profile (typically `prof_default`).
    """
    base = _base_url(registrar_url)
    resp = httpx.get(f"{base}/v1/profiles", timeout=timeout)
    resp.raise_for_status()
    profiles = resp.json()
    if not profiles:
        raise RuntimeError("Registrar returned no profiles")

    p = profiles[0]
    return ProfileSummary(
        id=p["id"],
        name=p.get("name", p["id"]),
        default_workspace_id=p.get("default_workspace_id", ""),
    )


def list_workspaces(
    registrar_url: Optional[str] = None,
    profile_id: Optional[str] = None,
    timeout: float = 5.0,
) -> List[WorkspaceSummary]:
    """
    List workspaces for the given profile via the registrar.

    If `profile_id` is not provided, the current profile from `login()`
    is used.
    """
    base = _base_url(registrar_url)
    profile = login(base, timeout=timeout) if profile_id is None else None
    pid = profile_id or (profile.id if profile else None)
    if not pid:
        raise RuntimeError("Unable to resolve profile id for list_workspaces")

    resp = httpx.get(f"{base}/v1/profiles/{pid}/workspaces", timeout=timeout)
    resp.raise_for_status()
    data = resp.json()

    out: List[WorkspaceSummary] = []
    for w in data:
        out.append(
            WorkspaceSummary(
                id=w["id"],
                name=w.get("name", w["id"]),
                engine_id=w.get("engine_id"),
            )
        )
    return out

