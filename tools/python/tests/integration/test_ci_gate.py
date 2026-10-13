# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Validate Required Native Jobs And Integration Trees"""

# === Imports === #

# Standard Library Imports
import re
import subprocess
from collections.abc import Callable
from copy import deepcopy
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.ci import gate, route

# Type Checking Imports

# === Public Exports === #

# === Constants === #
EVENT = {
    "pull_request": {
        "base": {"ref": "dev", "repo": {"full_name": gate.REPOSITORY}},
        "head": {
            "ref": "feature",
            "sha": "a" * 40,
            "repo": {"full_name": gate.REPOSITORY},
        },
    }
}

# === Types === #

# === Classes === #

# === Public Functions === #


def case_required_results(tmp_path: Path, monkeypatch, target, result) -> None:
    event = deepcopy(EVENT)
    event["pull_request"]["base"]["ref"] = target
    event["pull_request"]["head"]["ref"] = (
        "release/1.2.3" if target == "main" else "feature"
    )
    artifact = "ci_artifacts_catalog"
    monkeypatch.setattr(route, "version", lambda root: "1.2.3")
    needs = {"native": {"result": "success"}, artifact: {"result": result}}
    if result == "success":
        gate.verify(event, needs)
    else:
        with pytest.raises(ValueError, match="did not pass"):
            gate.verify(event, needs)


def case_foreign_promotion(tmp_path: Path, monkeypatch) -> None:
    event = deepcopy(EVENT)
    event["pull_request"]["base"]["ref"] = "main"
    event["pull_request"]["head"].update(
        {"ref": "dev", "repo": {"full_name": "other/dokime"}}
    )
    with pytest.raises(ValueError, match="configured repository"):
        gate.verify(event, {})


def case_target_advancement(tmp_path: Path, monkeypatch, changed) -> None:
    calls = []
    monkeypatch.setattr(
        gate,
        "job_id",
        lambda role, platform, **kwargs: calls.append((role, platform, kwargs)),
    )
    monkeypatch.setattr(gate, "_request", lambda path: {"commit": {"sha": "b" * 40}})
    monkeypatch.setattr(gate.subprocess, "run", lambda *args, **kwargs: None)
    monkeypatch.setattr(
        gate.subprocess,
        "check_output",
        lambda command, **kwargs: (
            "tested\n" if command[1] == "rev-parse" or not changed else "different\n"
        ),
    )
    if changed:
        with pytest.raises(ValueError, match="target advancement"):
            gate.verify_tree(EVENT)
    else:
        gate.verify_tree(EVENT)
    assert len(calls) == len(gate.ROLES) * len(gate.required_platforms(EVENT))
    assert all(kwargs == {"completed": True} for _, _, kwargs in calls)


def case_merge_conflict(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setattr(gate, "job_id", lambda *args, **kwargs: "1")
    monkeypatch.setattr(gate, "_request", lambda path: {"commit": {"sha": "b" * 40}})
    monkeypatch.setattr(gate.subprocess, "run", lambda *args, **kwargs: None)

    def conflict(command, **kwargs):
        raise subprocess.CalledProcessError(1, command)

    monkeypatch.setattr(gate.subprocess, "check_output", conflict)
    with pytest.raises(subprocess.CalledProcessError):
        gate.verify_tree(EVENT)


def case_native_roles(tmp_path: Path, monkeypatch, target: str) -> None:
    event = {
        "ref": f"refs/heads/{target}",
        "repository": {"full_name": gate.REPOSITORY},
    }
    calls = []
    monkeypatch.setattr(
        gate,
        "job_id",
        lambda role, platform, **kwargs: calls.append((role, platform, kwargs)),
    )
    monkeypatch.setenv("GITHUB_SHA", "a" * 40)
    monkeypatch.setattr(
        gate.subprocess, "check_output", lambda *args, **kwargs: "a" * 40 + "\n"
    )
    gate.verify_tree(event)
    platforms = ("linux",) if target == "dev" else ("linux", "windows")
    assert calls == [
        (role, platform, {"completed": True})
        for platform in platforms
        for role in (
            ("test:system:javascript",) + gate.ROLES
            if platform == "windows"
            else gate.ROLES
        )
    ]


def case_platform_failure(
    tmp_path: Path, monkeypatch, platform: str, failed: str
) -> None:
    event = {
        "ref": "refs/heads/dev" if platform == "linux" else "refs/heads/main",
        "repository": {"full_name": gate.REPOSITORY},
    }

    def job(role, selected, **kwargs):
        if platform == "linux":
            assert selected == "linux"
        if selected == platform and role == failed:
            raise ValueError(f"{platform} validation failed")
        return "1"

    monkeypatch.setattr(gate, "job_id", job)
    with pytest.raises(ValueError, match=f"{platform} validation failed"):
        gate.verify_tree(event)


def case_workflow_policy(tmp_path: Path, monkeypatch) -> None:
    root = Path(__file__).resolve().parents[4]
    advisory = (root / ".github/workflows/dev-windows.yml").read_text()
    required = (root / ".github/workflows/ci.yml").read_text()
    platform = (root / ".github/workflows/platform.yml").read_text()
    assert "uses: ./.github/workflows/platform.yml" in advisory
    assert "os: windows" in advisory
    assert "needs: policy" in advisory
    assert "native_windows" not in required
    assert "continue-on-error" not in advisory + platform
    assert "if: always()\n        uses: actions/upload-artifact@" in platform
    for job in ("check_fast", "test"):
        block = re.split(
            r"\n  [a-z_]+:\n", platform.split(f"\n  {job}:\n", 1)[1], maxsplit=1
        )[0]
        route = next(line for line in block.splitlines() if "runs-on:" in line)
        assert "github.event_name == 'push'" in route
        assert "github.repository == 'stokker-technologies/dokime'" in route
        assert "github.ref == 'refs/heads/dev'" in route
        assert "&& 'stokker-local-linux-x64'" in route
        assert "self-hosted" not in route and "fromJSON" not in route
        assert "'ubuntu-24.04'" in route and "'windows-2022'" in route
    build = platform.split("\n  build_release:\n", 1)[1].split(
        "\n  test_integration_javascript:\n", 1
    )[0]
    assert "stokker-local-linux-x64" not in build


# === Private Functions === #


@pytest.mark.parametrize(
    "scenario,arguments",
    [
        pytest.param(
            case_required_results, ("dev", "success"), id="required_results-0"
        ),
        pytest.param(
            case_required_results, ("dev", "failure"), id="required_results-1"
        ),
        pytest.param(
            case_required_results, ("dev", "cancelled"), id="required_results-2"
        ),
        pytest.param(
            case_required_results, ("dev", "skipped"), id="required_results-3"
        ),
        pytest.param(case_required_results, ("dev", None), id="required_results-4"),
        pytest.param(
            case_required_results, ("main", "success"), id="required_results-5"
        ),
        pytest.param(
            case_required_results, ("main", "failure"), id="required_results-6"
        ),
        pytest.param(
            case_required_results, ("main", "cancelled"), id="required_results-7"
        ),
        pytest.param(
            case_required_results, ("main", "skipped"), id="required_results-8"
        ),
        pytest.param(case_required_results, ("main", None), id="required_results-9"),
        pytest.param(case_foreign_promotion, (), id="foreign_promotion-10"),
        pytest.param(case_target_advancement, (False,), id="target_advancement-11"),
        pytest.param(case_target_advancement, (True,), id="target_advancement-12"),
        pytest.param(case_merge_conflict, (), id="merge_conflict-13"),
        pytest.param(case_native_roles, ("dev",), id="native_roles-dev"),
        pytest.param(case_native_roles, ("main",), id="native_roles-main"),
        pytest.param(case_platform_failure, ("linux", "check:fast"), id="linux-check"),
        pytest.param(
            case_platform_failure, ("linux", "build:release"), id="linux-build"
        ),
        pytest.param(case_platform_failure, ("linux", "test"), id="linux-test"),
        pytest.param(
            case_platform_failure, ("windows", "check:fast"), id="windows-check"
        ),
        pytest.param(
            case_platform_failure,
            ("windows", "test:system:javascript"),
            id="windows-installed-desktop",
        ),
        pytest.param(case_workflow_policy, (), id="workflow-policy"),
    ],
)
def test_gate_contract(
    scenario: Callable[..., None], arguments: tuple, tmp_path: Path, monkeypatch
) -> None:
    scenario(tmp_path, monkeypatch, *arguments)


# === Entry Point === #
