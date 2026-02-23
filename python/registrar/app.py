from __future__ import annotations

import os
import secrets
from datetime import datetime, timezone
from typing import List, Optional

from fastapi import Depends, FastAPI, Header, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from sqlalchemy import (
    JSON,
    Column,
    DateTime,
    ForeignKey,
    String,
    create_engine,
    select,
)
from sqlalchemy.orm import Session, declarative_base, relationship, sessionmaker


app = FastAPI(title="Rinnovo Registrar", version="0.1.0")

# --- CORS configuration ------------------------------------------------------

_cors_origins_raw = os.getenv("CORS_ALLOW_ORIGINS")
if _cors_origins_raw:
    _cors_origins = [
        origin.strip()
        for origin in _cors_origins_raw.split(",")
        if origin.strip()
    ]
else:
    # Development-friendly default; tighten via CORS_ALLOW_ORIGINS in prod.
    _cors_origins = ["*"]

app.add_middleware(
    CORSMiddleware,
    allow_origins=_cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# --- Database setup -----------------------------------------------------------


DATABASE_URL = os.getenv("DATABASE_URL")
if not DATABASE_URL:
    # For local development and tests, fall back to a small SQLite file.
    DATABASE_URL = "sqlite:///./registrar.db"

engine = create_engine(DATABASE_URL, future=True)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine, future=True)
Base = declarative_base()


class Profile(Base):
    __tablename__ = "profiles"

    id = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    default_workspace_id = Column(String, nullable=True)

    workspaces = relationship("Workspace", back_populates="profile")
    engines = relationship("Engine", back_populates="profile")


class Engine(Base):
    __tablename__ = "engines"

    id = Column(String, primary_key=True)
    profile_id = Column(String, ForeignKey("profiles.id"), nullable=False)
    name = Column(String, nullable=False)
    kind = Column(String, nullable=False)
    endpoint_url = Column(String, nullable=False)
    version = Column(String, nullable=False)
    capabilities = Column(JSON, nullable=False)
    status = Column(String, nullable=False)
    last_seen_at = Column(DateTime(timezone=True), nullable=False)
    heartbeat_token = Column(String, nullable=False)

    profile = relationship("Profile", back_populates="engines")


class Workspace(Base):
    __tablename__ = "workspaces"

    id = Column(String, primary_key=True)
    profile_id = Column(String, ForeignKey("profiles.id"), nullable=False)
    name = Column(String, nullable=False)
    engine_id = Column(String, ForeignKey("engines.id"), nullable=True)
    kind = Column(String, nullable=False, default="local_engine")
    artifact_ref = Column(String, nullable=True)
    remote_ref = Column(String, nullable=True)

    profile = relationship("Profile", back_populates="workspaces")


Base.metadata.create_all(bind=engine)


def get_db() -> Session:
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


# Default identifiers used for the initial stub profile/workspace.
DEFAULT_PROFILE_ID = "prof_default"
DEFAULT_WORKSPACE_ID = "ws_default"


def ensure_default_profile_and_workspace(db: Session) -> Profile:
    profile = db.get(Profile, DEFAULT_PROFILE_ID)
    if profile is None:
        profile = Profile(
            id=DEFAULT_PROFILE_ID,
            name="Default",
            default_workspace_id=DEFAULT_WORKSPACE_ID,
        )
        db.add(profile)
        db.commit()
        db.refresh(profile)

    # Ensure a default workspace exists for this profile.
    stmt = select(Workspace).where(
        Workspace.id == DEFAULT_WORKSPACE_ID,
        Workspace.profile_id == profile.id,
    )
    ws = db.execute(stmt).scalar_one_or_none()
    if ws is None:
        ws = Workspace(
            id=DEFAULT_WORKSPACE_ID,
            profile_id=profile.id,
            name="Local Workspace",
            engine_id=None,
            kind="local_engine",
        )
        db.add(ws)
        db.commit()

    return profile


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


def engine_to_info(e: Engine) -> EngineInfo:
    return EngineInfo(
        id=e.id,
        profile_id=e.profile_id,
        name=e.name,
        kind=e.kind,
        endpoint_url=e.endpoint_url,
        version=e.version,
        capabilities=list(e.capabilities or []),
        status=e.status,
        last_seen_at=e.last_seen_at,
    )


# --- Engine registration + heartbeat -----------------------------------------


@app.post("/v1/engines/register", response_model=EngineRegisterResponse)
def register_engine(
    payload: EngineRegisterRequest,
    db: Session = Depends(get_db),
) -> EngineRegisterResponse:
    """
    Register an engine instance under the default profile.

    In the future, `profile_token` will be used to look up the profile; for
    now we always attach engines to a stub 'prof_default'.
    """
    profile = ensure_default_profile_and_workspace(db)

    # Random ID to avoid collisions when multiple engines register within
    # the same second (e.g. in tests or rapid restarts).
    engine_id = f"eng_{secrets.token_hex(8)}"
    heartbeat_token = secrets.token_hex(16)

    inst = Engine(
        id=engine_id,
        profile_id=profile.id,
        name=payload.name,
        kind=payload.kind,
        endpoint_url=payload.endpoint_url,
        version=payload.version,
        capabilities=list(payload.capabilities),
        status="online",
        last_seen_at=datetime.now(timezone.utc),
        heartbeat_token=heartbeat_token,
    )
    db.add(inst)
    db.commit()

    return EngineRegisterResponse(
        engine_id=engine_id,
        heartbeat_token=heartbeat_token,
        profile_id=profile.id,
    )


@app.post("/v1/engines/{engine_id}/heartbeat")
def engine_heartbeat(
    engine_id: str,
    payload: EngineHeartbeatRequest,
    x_engine_token: str = Header(..., alias="X-Engine-Token"),
    db: Session = Depends(get_db),
) -> None:
    inst = db.get(Engine, engine_id)
    if inst is None:
        raise HTTPException(status_code=404, detail="engine not found")

    if x_engine_token != inst.heartbeat_token:
        raise HTTPException(status_code=401, detail="invalid engine token")

    if payload.status:
        inst.status = payload.status
    inst.last_seen_at = datetime.now(timezone.utc)
    db.add(inst)
    db.commit()


@app.get("/v1/profiles", response_model=List[ProfileInfo])
def list_profiles(db: Session = Depends(get_db)) -> List[ProfileInfo]:
    # Ensure the default profile/workspace exists.
    profile = ensure_default_profile_and_workspace(db)
    return [
        ProfileInfo(
            id=profile.id,
            name=profile.name,
            default_workspace_id=profile.default_workspace_id or DEFAULT_WORKSPACE_ID,
        )
    ]


@app.get("/v1/profiles/{profile_id}/engines", response_model=List[EngineInfo])
def list_engines(profile_id: str, db: Session = Depends(get_db)) -> List[EngineInfo]:
    profile = db.get(Profile, profile_id)
    if profile is None:
        raise HTTPException(status_code=404, detail="profile not found")

    stmt = select(Engine).where(Engine.profile_id == profile_id)
    engines = db.execute(stmt).scalars().all()

    return [engine_to_info(e) for e in engines]


@app.get("/v1/profiles/{profile_id}/workspaces", response_model=List[WorkspaceInfo])
def list_workspaces(
    profile_id: str,
    db: Session = Depends(get_db),
) -> List[WorkspaceInfo]:
    profile = db.get(Profile, profile_id)
    if profile is None:
        raise HTTPException(status_code=404, detail="profile not found")

    stmt = select(Workspace).where(Workspace.profile_id == profile_id)
    workspaces = db.execute(stmt).scalars().all()

    return [
        WorkspaceInfo(id=w.id, name=w.name, engine_id=w.engine_id) for w in workspaces
    ]


@app.get("/v1/console/session", response_model=ConsoleSessionResponse)
def console_session(
    profile_id: Optional[str] = None,
    db: Session = Depends(get_db),
) -> ConsoleSessionResponse:
    """
    Minimal console bootstrap: return a single default profile + workspace and,
    if any engine is registered, use it as the target engine.
    """
    profile = ensure_default_profile_and_workspace(db)
    if profile_id is not None and profile_id != profile.id:
        raise HTTPException(status_code=404, detail="profile not found")

    # Resolve workspace, defaulting to the known stub if present.
    stmt_ws = select(Workspace).where(
        Workspace.id == (profile.default_workspace_id or DEFAULT_WORKSPACE_ID),
        Workspace.profile_id == profile.id,
    )
    ws = db.execute(stmt_ws).scalar_one_or_none()
    if ws is None:
        ws = Workspace(
            id=DEFAULT_WORKSPACE_ID,
            profile_id=profile.id,
            name="Local Workspace",
            engine_id=None,
            kind="local_engine",
        )
        db.add(ws)
        db.commit()

    workspace_info = WorkspaceInfo(id=ws.id, name=ws.name, engine_id=ws.engine_id)

    # Prefer the most recently seen engine, if any.
    stmt_eng = (
        select(Engine)
        .where(Engine.profile_id == profile.id)
        .order_by(Engine.last_seen_at.desc())
    )
    latest = db.execute(stmt_eng).scalars().first()

    engine_info: Optional[EngineInfo] = None
    engine_token: Optional[str] = None
    if latest is not None:
        engine_info = engine_to_info(latest)
        engine_token = secrets.token_hex(16)  # placeholder session token
        workspace_info.engine_id = latest.id

    return ConsoleSessionResponse(
        profile=ProfileInfo(
            id=profile.id,
            name=profile.name,
            default_workspace_id=profile.default_workspace_id or DEFAULT_WORKSPACE_ID,
        ),
        workspace=workspace_info,
        engine=engine_info,
        engine_session_token=engine_token,
    )


# To run locally:
#   uvicorn python.registrar.app:app --reload
