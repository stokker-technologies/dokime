# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Mark Local Dependency Overrides Unpublishable Locally And Refuse Them In CI"""

# === Imports === #

# Standard Library Imports
import json
import subprocess
from collections.abc import Callable
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools import _policy, release

# Type Checking Imports


# === Public Exports === #

# === Constants === #

CI_VARIABLES = ("CI", "GITHUB_ACTIONS", "GITLAB_CI")
TREE = "c" * 40

# === Types === #

# === Classes === #

# === Public Functions === #


def case_override_local(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    sibling = _resolved(tmp_path, monkeypatch, override=True)

    first = release.inputs(tmp_path / "project")
    second = release.inputs(tmp_path / "project")

    assert first == second
    override = first["local_overrides"][0]
    assert override["package"] == "harmos"
    assert override["manifest"] == sibling.as_posix()
    assert len(override["source"]) == 64
    sibling.write_text("[workspace]\n# Changed Dependency\n", encoding="utf-8")
    changed = release.inputs(tmp_path / "project")
    assert changed["local_overrides"][0]["source"] != override["source"]
    assert any(
        node["package"] == f"harmos@0.1.0|local:{sibling.as_posix()}"
        for node in first["graph"]
    )
    warning = capsys.readouterr().err
    assert warning.count("unpublished local dependency overrides") == 1
    assert f"harmos: {sibling.as_posix()}" in warning


def case_override_ci(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    _resolved(tmp_path, monkeypatch, override=True)
    monkeypatch.setenv("CI", "true")

    with pytest.raises(
        _policy.ToolError, match="unpublished local dependency override: harmos"
    ):
        release.inputs(tmp_path / "project")
    assert capsys.readouterr().err == ""


def case_published(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    _resolved(tmp_path, monkeypatch, override=False)
    monkeypatch.setenv("CI", "true")

    identity = release.inputs(tmp_path / "project")

    assert set(identity) == {"source_tree", "graph", "locks", "build_environment"}
    assert identity["source_tree"] == TREE
    assert {node["package"] for node in identity["graph"]} == {
        "dokime@0.1.0|path:Cargo.toml",
        "harmos@0.1.0|registry+https://github.com/rust-lang/crates.io-index",
    }
    assert capsys.readouterr().err == ""


# === Private Functions === #


def _resolved(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, *, override: bool
) -> Path:
    """Serve canned Git and Cargo metadata so no native tool runs."""
    project = tmp_path / "project"
    project.mkdir()
    (project / "Cargo.toml").write_text('[workspace]\nmembers = ["."]\n')
    sibling = (tmp_path / "harmos/Cargo.toml").resolve()
    sibling.parent.mkdir()
    sibling.write_text("[workspace]\n", encoding="utf-8")
    dependency = {
        "name": "harmos",
        "version": "0.1.0",
        "id": "harmos",
        "source": None
        if override
        else "registry+https://github.com/rust-lang/crates.io-index",
        "manifest_path": str(sibling),
    }
    metadata = {
        "packages": [
            {
                "name": "dokime",
                "version": "0.1.0",
                "id": "dokime",
                "source": None,
                "manifest_path": str(project / "Cargo.toml"),
            },
            dependency,
        ],
        "resolve": {
            "nodes": [
                {"id": "dokime", "features": [], "dependencies": ["harmos"]},
                {"id": "harmos", "features": [], "dependencies": []},
            ]
        },
    }
    outputs = {
        "rev-parse": TREE + "\n",
        "status": "",
        "metadata": json.dumps(metadata),
    }

    def run(
        command: list[str], **_: object
    ) -> subprocess.CompletedProcess[str | bytes]:
        if command == ["git", "rev-parse", "--show-toplevel"]:
            return subprocess.CompletedProcess(command, 0, stdout=str(sibling.parent))
        if command[1] == "ls-files":
            return subprocess.CompletedProcess(command, 0, stdout=b"Cargo.toml\0")
        return subprocess.CompletedProcess(command, 0, stdout=outputs[command[1]])

    monkeypatch.setattr(release.subprocess, "run", run)
    for name in CI_VARIABLES:
        monkeypatch.delenv(name, raising=False)
    release._warn_overrides.cache_clear()
    return sibling


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_override_local, id="override_local"),
        pytest.param(case_override_ci, id="override_ci"),
        pytest.param(case_published, id="published"),
    ],
)
def test_contract(
    scenario: Callable[..., None],
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scenario(tmp_path, monkeypatch, capsys)
