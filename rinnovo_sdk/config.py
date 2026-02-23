from __future__ import annotations

from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Dict, Optional, Any
import json


CONFIG_DIR_NAME = ".rinnovo"
CONFIG_FILE_NAME = "config.json"


def _default_config_path() -> Path:
    """
    Return the default location for the SDK config file.

    For now this is `${HOME}/.rinnovo/config.json` on all platforms.
    """
    home = Path.home()
    return home / CONFIG_DIR_NAME / CONFIG_FILE_NAME


@dataclass
class WorkspaceConfig:
    """
    Lightweight workspace reference for the console and SDK.

    This does not enforce any engine semantics yet; it simply records
    how to reach a workspace (by ID) on a given engine URL.
    """

    id: str
    name: str
    engine_url: str
    api_token: Optional[str] = None


@dataclass
class ProfileConfig:
    """
    User profile that groups engine connection details and workspaces.

    Multiple profiles allow connecting to different engines (e.g.
    local vs. remote) without rewriting config.
    """

    name: str
    engine_url: str
    api_token: Optional[str] = None
    default_workspace: Optional[str] = None
    workspaces: Dict[str, WorkspaceConfig] = field(default_factory=dict)


@dataclass
class Config:
    """
    Top-level SDK and console configuration.
    """

    profiles: Dict[str, ProfileConfig] = field(default_factory=dict)
    current_profile: Optional[str] = None

    @classmethod
    def empty(cls) -> "Config":
        return cls()


def load_config(path: Optional[Path] = None) -> Config:
    """
    Load configuration from disk, returning an empty Config if the file
    does not exist or cannot be parsed.
    """
    cfg_path = path or _default_config_path()
    if not cfg_path.exists():
        return Config.empty()

    try:
        raw = json.loads(cfg_path.read_text(encoding="utf-8"))
    except Exception:
        # On any parse error, fall back to an in-memory empty config.
        return Config.empty()

    profiles: Dict[str, ProfileConfig] = {}
    for name, pdata in (raw.get("profiles") or {}).items():
        workspaces: Dict[str, WorkspaceConfig] = {}
        for wname, wdata in (pdata.get("workspaces") or {}).items():
            workspaces[wname] = WorkspaceConfig(
                id=wdata.get("id", wname),
                name=wdata.get("name", wname),
                engine_url=wdata.get("engine_url", pdata.get("engine_url", "")),
                api_token=wdata.get("api_token") or pdata.get("api_token"),
            )

        profiles[name] = ProfileConfig(
            name=name,
            engine_url=pdata.get("engine_url", ""),
            api_token=pdata.get("api_token"),
            default_workspace=pdata.get("default_workspace"),
            workspaces=workspaces,
        )

    return Config(
        profiles=profiles,
        current_profile=raw.get("current_profile"),
    )


def save_config(cfg: Config, path: Optional[Path] = None) -> None:
    """
    Persist configuration to disk, creating the parent directory if needed.
    """
    cfg_path = path or _default_config_path()
    cfg_path.parent.mkdir(parents=True, exist_ok=True)

    # Convert dataclasses into plain dicts for JSON.
    profiles_dict: Dict[str, Any] = {}
    for name, profile in cfg.profiles.items():
        pdata = asdict(profile)
        # Flatten workspace objects into nested dicts.
        workspaces = {
            wname: asdict(wcfg) for wname, wcfg in profile.workspaces.items()
        }
        pdata["workspaces"] = workspaces
        profiles_dict[name] = pdata

    raw = {
        "profiles": profiles_dict,
        "current_profile": cfg.current_profile,
    }
    cfg_path.write_text(json.dumps(raw, indent=2, sort_keys=True), encoding="utf-8")


def active_profile(cfg: Optional[Config] = None) -> Optional[ProfileConfig]:
    """
    Return the currently selected profile, if any.
    """
    cfg = cfg or load_config()
    if not cfg.current_profile:
        return None
    return cfg.profiles.get(cfg.current_profile)

