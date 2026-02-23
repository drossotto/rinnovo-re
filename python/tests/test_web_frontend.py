import pathlib


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_index_routes_to_landing_and_console():
    html = read("web/index.html")

    # Shadow DOM host element and simple hash-based router must exist.
    assert "<rinnovo-root" in html
    assert "hash === \"#console\"" in html or "hash === '#console'" in html

    # Router should reference both landing and console pages.
    assert "pages/landing.html" in html
    assert "pages/console.html" in html


def test_landing_open_console_links_use_console_route():
    html = read("web/pages/landing.html")

    # Landing page should be present and mention Rinnovo.
    assert "Rinnovo Representation Engine" in html

    # Primary actions must link to the console route.
    assert "#console" in html

    # Console page referenced by the route must exist.
    console_path = REPO_ROOT / "web" / "pages" / "console.html"
    assert console_path.is_file()


def test_console_has_sidebar_and_theme_controls():
    html = read("web/pages/console.html")

    # Console header and title.
    assert "Rinnovo Representation Engine" in html

    # Sidebar toggle and theme toggle must be present.
    assert 'id="sidebar-toggle"' in html
    assert "toggleSidebar" in html
    assert "toggleTheme" in html

    # Layout should contain the core panels we expect to evolve.
    for section_id in ("workspace", "objects", "views"):
        assert f'id="{section_id}"' in html

