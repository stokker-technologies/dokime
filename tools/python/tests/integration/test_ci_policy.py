# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Select Required Native Platforms From Provider Event Boundaries"""

# === Imports === #

# Standard Library Imports
import json
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.ci import policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #
DEV = {"ref": "refs/heads/dev", "repository": {"full_name": policy.REPOSITORY}}

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "event,manual_ref,expected",
    [
        pytest.param(DEV, None, ("linux",), id="dev"),
        pytest.param(
            {**DEV, "ref": "refs/heads/main"}, None, policy.FULL_PLATFORMS, id="main"
        ),
        pytest.param(
            {**DEV, "ref": "refs/heads/release/0.1.0"},
            None,
            policy.FULL_PLATFORMS,
            id="release",
        ),
        pytest.param(
            {**DEV, "ref": "refs/tags/v0.1.0"}, None, policy.FULL_PLATFORMS, id="tag"
        ),
        pytest.param({**DEV, "ref": "dev"}, None, policy.FULL_PLATFORMS, id="raw-ref"),
        pytest.param(
            {**DEV, "ref": None}, None, policy.FULL_PLATFORMS, id="missing-ref"
        ),
        pytest.param(
            {
                "pull_request": {
                    "base": {"ref": "dev", "repo": {"full_name": policy.REPOSITORY}}
                }
            },
            None,
            ("linux",),
            id="pr-dev",
        ),
        pytest.param(
            {
                "pull_request": {
                    "base": {"ref": "main", "repo": {"full_name": policy.REPOSITORY}}
                }
            },
            None,
            policy.FULL_PLATFORMS,
            id="pr-main",
        ),
        pytest.param(
            {
                "pull_request": {
                    "base": {"ref": "dev", "repo": {"full_name": "other/dokime"}}
                }
            },
            None,
            policy.FULL_PLATFORMS,
            id="pr-foreign",
        ),
        pytest.param(
            {"ref": "refs/heads/dev"},
            None,
            policy.FULL_PLATFORMS,
            id="unknown-repository",
        ),
        pytest.param(
            {**DEV, "ref": "dev"}, "refs/heads/dev", ("linux",), id="manual-dev"
        ),
        pytest.param(
            {**DEV, "ref": "dev"},
            "refs/heads/main",
            policy.FULL_PLATFORMS,
            id="manual-main",
        ),
    ],
)
def test_platform_policy(
    tmp_path: Path,
    monkeypatch,
    event: dict,
    manual_ref: str | None,
    expected: tuple[str, ...],
) -> None:
    event_file = tmp_path / "event.json"
    event_file.write_text(json.dumps(event), encoding="utf-8")
    monkeypatch.setenv("GITHUB_EVENT_PATH", str(event_file))
    if manual_ref is not None:
        event_name = "workflow_dispatch"
    elif "pull_request" in event:
        event_name = "pull_request"
    else:
        event_name = "push"
    monkeypatch.setenv("GITHUB_EVENT_NAME", event_name)
    monkeypatch.setenv("GITHUB_REF", manual_ref or "refs/heads/dev")
    loaded = policy.current_event()
    if manual_ref is not None:
        assert loaded["ref"] == manual_ref
    else:
        assert loaded == event
    assert policy.required_platforms(loaded) == expected


# === Private Functions === #

# === Entry Point === #
