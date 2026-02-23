from __future__ import annotations

import secrets
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from typing import Dict, List, Optional

from fastapi import FastAPI, Header, HTTPException
from pydantic import BaseModel


app = FastAPI(title="Rinnovo Registrar", version="0.1.0")


# --- In-memory storage --------------------------------------------------------


@dataclass
class EngineInstance:
    id: str
    profile_id: str
    name: str
    kind: str
    endpoint_url: str
    version: str
    capabilities: List[str]
    status: str
    last_seen_at: datetime
    heartbeat_token: str


ENGINES: Dict[str, EngineInstance] = {}

# For now we stub a single default profile/workspace.
DEFAULT_PROFILE_ID = "prof_default"
DEFAULT_WORKSPACE_ID = "ws_default"


# --- Pydantic schemas ---------------------------------------------------------


class EngineRegisterRequest(BaseModel):
    profile_token: Optional[str] = None
    name: str
    kind: str = "local"  # "local" | "remote" | "managed"
    endpoint_url: str
    version: str
    capabilities: List[str] = []


class EngineRegisterResponse(BaseModel):
    engine_id: str
    heartbeat_token: str
    profile_id: str


class EngineHeartbeatRequest(BaseModel):
    status: Optional[str] = None  # "online" | "offline" | "unknown"
    load: Optional[dict] = None


class EngineInfo(BaseModel):
    id: str
    profile_id: str
    name: str
    kind: str
    endpoint_url: str
    version: str
    capabilities: List[str]
    status: str
    last_seen_at: datetime


class ProfileInfo(BaseModel):
    id: str
    name: str
    default_workspace_id: str


class WorkspaceInfo(BaseModel):
    id: str
    name: str
    engine_id: Optional[str] = None


class ConsoleSessionResponse(BaseModel):
    profile: ProfileInfo
    workspace: WorkspaceInfo
    engine: Optional[EngineInfo] = None
    engine_session_token: Optional[str] = None


# --- Engine registration + heartbeat -----------------------------------------


@app.post("/v1/engines/register", response_model=EngineRegisterResponse)
def register_engine(payload: EngineRegisterRequest) -> EngineRegisterResponse:
    """
    Register an engine instance under the default profile.

    In the future, `profile_token` will be used to look up the profile; for
    now we always attach engines to a stub 'prof_default'.
    """
    engine_id = f"eng_{len(ENGINES) + 1}"
    heartbeat_token = secrets.token_hex(16)

    inst = EngineInstance(
        id=engine_id,
        profile_id=DEFAULT_PROFILE_ID,
        name=payload.name,
        kind=payload.kind,
        endpoint_url=payload.endpoint_url,
        version=payload.version,
        capabilities=list(payload.capabilities),
        status="online",
        last_seen_at=datetime.now(timezone.utc),
        heartbeat_token=heartbeat_token,
    )
    ENGINES[engine_id] = inst

    return EngineRegisterResponse(
        engine_id=engine_id,
        heartbeat_token=heartbeat_token,
        profile_id=DEFAULT_PROFILE_ID,
    )


@app.post("/v1/engines/{engine_id}/heartbeat")
def engine_heartbeat(
    engine_id: str,
    payload: EngineHeartbeatRequest,
    x_engine_token: str = Header(..., alias="X-Engine-Token"),
) -> None:
    inst = ENGINES.get(engine_id)
    if inst is None:
        raise HTTPException(status_code=404, detail="engine not found")

    if x_engine_token != inst.heartbeat_token:
        raise HTTPException(status_code=401, detail="invalid engine token")

    if payload.status:
        inst.status = payload.status
    inst.last_seen_at = datetime.now(timezone.utc)
    ENGINES[engine_id] = inst


@app.get("/v1/profiles", response_model=List[ProfileInfo])
def list_profiles() -> List[ProfileInfo]:
    # For now, always return a single default profile.
    return [
        ProfileInfo(
            id=DEFAULT_PROFILE_ID,
            name="Default",
            default_workspace_id=DEFAULT_WORKSPACE_ID,
        )
    ]


@app.get("/v1/profiles/{profile_id}/engines", response_model=List[EngineInfo])
def list_engines(profile_id: str) -> List[EngineInfo]:
    if profile_id != DEFAULT_PROFILE_ID:
        raise HTTPException(status_code=404, detail="profile not found")

    out: List[EngineInfo] = []
    for inst in ENGINES.values():
        if inst.profile_id == profile_id:
            out.append(EngineInfo(**asdict(inst)))
    return out


@app.get("/v1/console/session", response_model=ConsoleSessionResponse)
def console_session(profile_id: Optional[str] = None) -> ConsoleSessionResponse:
    """
    Minimal console bootstrap: return a single default profile + workspace and,
    if any engine is registered, use it as the target engine.
    """
    pid = profile_id or DEFAULT_PROFILE_ID
    if pid != DEFAULT_PROFILE_ID:
        raise HTTPException(status_code=404, detail="profile not found")

    profile = ProfileInfo(
        id=DEFAULT_PROFILE_ID,
        name="Default",
        default_workspace_id=DEFAULT_WORKSPACE_ID,
    )
    workspace = WorkspaceInfo(
        id=DEFAULT_WORKSPACE_ID,
        name="Local Workspace",
        engine_id=None,
    )

    engine_info: Optional[EngineInfo] = None
    engine_token: Optional[str] = None
    # Prefer the most recently seen engine, if any.
    if ENGINES:
        latest = max(ENGINES.values(), key=lambda e: e.last_seen_at)
        engine_info = EngineInfo(**asdict(latest))
        engine_token = secrets.token_hex(16)  # placeholder session token
        workspace.engine_id = latest.id

    return ConsoleSessionResponse(
        profile=profile,
        workspace=workspace,
        engine=engine_info,
        engine_session_token=engine_token,
    )


# To run locally:
#   uvicorn registrar.app:app --reload

