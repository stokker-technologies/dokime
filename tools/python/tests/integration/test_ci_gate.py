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
from collections.abc import Callable
from pathlib import Path
from copy import deepcopy
import subprocess

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
    assert len(calls) == 6
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
    ],
)
def test_gate_contract(
    scenario: Callable[..., None], arguments: tuple, tmp_path: Path, monkeypatch
) -> None:
    scenario(tmp_path, monkeypatch, *arguments)


# === Entry Point === #
