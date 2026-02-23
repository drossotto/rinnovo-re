from __future__ import annotations

from pathlib import Path
from typing import Optional

import typer

from .config import (
    Config,
    ProfileConfig,
    WorkspaceConfig,
    load_config,
    save_config,
    active_profile,
)


app = typer.Typer(help="Rinnovo SDK CLI")
profile_app = typer.Typer(help="Manage Rinnovo profiles")
workspace_app = typer.Typer(help="Manage workspaces within a profile")

app.add_typer(profile_app, name="profile")
app.add_typer(workspace_app, name="workspace")


@profile_app.command("list")
def profile_list() -> None:
    """
    List configured profiles and the active one.
    """
    cfg = load_config()
    if not cfg.profiles:
        typer.echo("No profiles configured.")
        return

    for name, p in cfg.profiles.items():
        marker = "*" if cfg.current_profile == name else " "
        typer.echo(f"{marker} {name} -> {p.engine_url}")


@profile_app.command("create")
def profile_create(
    name: str = typer.Argument(..., help="Profile name, e.g. 'default'"),
    engine_url: str = typer.Option(
        "http://localhost:8787", "--engine-url", help="Engine base URL"
    ),
    api_token: Optional[str] = typer.Option(
        None, "--api-token", help="API token or bearer token for the engine"
    ),
) -> None:
    """
    Create or update a profile pointing at an engine URL.
    """
    cfg = load_config()

    profile = ProfileConfig(
        name=name,
        engine_url=engine_url,
        api_token=api_token,
        default_workspace=cfg.profiles.get(name, ProfileConfig(name, engine_url)).default_workspace,  # type: ignore[arg-type]
        workspaces=cfg.profiles.get(name, ProfileConfig(name, engine_url)).workspaces,  # type: ignore[arg-type]
    )

    cfg.profiles[name] = profile
    if cfg.current_profile is None:
        cfg.current_profile = name

    save_config(cfg)
    typer.echo(f"Profile '{name}' set to engine {engine_url}")


@profile_app.command("use")
def profile_use(name: str) -> None:
    """
    Set the active profile.
    """
    cfg = load_config()
    if name not in cfg.profiles:
        raise typer.Exit(code=1)

    cfg.current_profile = name
    save_config(cfg)
    typer.echo(f"Current profile set to '{name}'")


@workspace_app.command("list")
def workspace_list() -> None:
    """
    List workspaces for the active profile.
    """
    cfg = load_config()
    profile = active_profile(cfg)
    if profile is None:
        typer.echo("No active profile. Use 'rinnovo profile create' or 'rinnovo profile use'.")
        raise typer.Exit(code=1)

    if not profile.workspaces:
        typer.echo(f"No workspaces configured for profile '{profile.name}'.")
        return

    for name, ws in profile.workspaces.items():
        marker = "*" if profile.default_workspace == name else " "
        typer.echo(f"{marker} {name} (id={ws.id}) -> {ws.engine_url}")


@workspace_app.command("add")
def workspace_add(
    name: str = typer.Argument(..., help="Workspace name, e.g. 'bgc_catalog_v1'"),
    workspace_id: Optional[str] = typer.Option(
        None, "--id", help="Engine workspace identifier (defaults to name)"
    ),
    engine_url: Optional[str] = typer.Option(
        None, "--engine-url", help="Override engine URL for this workspace"
    ),
    api_token: Optional[str] = typer.Option(
        None, "--api-token", help="Override API token for this workspace"
    ),
) -> None:
    """
    Register a workspace under the active profile.
    """
    cfg = load_config()
    profile = active_profile(cfg)
    if profile is None:
        typer.echo("No active profile. Use 'rinnovo profile create' or 'rinnovo profile use'.")
        raise typer.Exit(code=1)

    ws = WorkspaceConfig(
        id=workspace_id or name,
        name=name,
        engine_url=engine_url or profile.engine_url,
        api_token=api_token or profile.api_token,
    )

    profile.workspaces[name] = ws
    if profile.default_workspace is None:
        profile.default_workspace = name

    cfg.profiles[profile.name] = profile
    save_config(cfg)
    typer.echo(f"Workspace '{name}' registered for profile '{profile.name}'.")


@workspace_app.command("use")
def workspace_use(name: str) -> None:
    """
    Set the default workspace for the active profile.
    """
    cfg = load_config()
    profile = active_profile(cfg)
    if profile is None:
        typer.echo("No active profile. Use 'rinnovo profile create' or 'rinnovo profile use'.")
        raise typer.Exit(code=1)

    if name not in profile.workspaces:
        typer.echo(f"Workspace '{name}' not found in profile '{profile.name}'.")
        raise typer.Exit(code=1)

    profile.default_workspace = name
    cfg.profiles[profile.name] = profile
    save_config(cfg)
    typer.echo(f"Default workspace for profile '{profile.name}' set to '{name}'.")


if __name__ == "__main__":
    app()

