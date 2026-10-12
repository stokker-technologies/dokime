# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Preserve Migrated Tooling Contracts"""

# === Imports === #

# Standard Library Imports

from copy import deepcopy
import json
from pathlib import Path
import shutil


# Third Party Imports

import pytest


# Local Imports

from dokime_tools import frontend, installer


# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_report_freshness() -> None:
    assert [
        frontend.is_stale(previous, current, exists)
        for previous, current, exists in [
            (None, "a", True),
            ("a", "a", True),
            ("a", "a", False),
            ("a", "b", True),
        ]
    ] == [True, False, True, True]
    assert (
        frontend.REPORT
        == frontend.WEB_ROOT.parent / "dokime-app/dist-report/report.html"
    )
    assert frontend.STAMP == frontend.ROOT / "target/report-template/inputs.sha256"
    assert frontend.STAMP.parent != frontend.REPORT.parent


def case_installer_modes() -> None:
    for platform, profile, signed in [
        ("linux", "release", False),
        ("win32", "release", False),
        ("win32", "release", True),
        ("linux", "dev", False),
        ("win32", "dev", False),
    ]:
        args = installer.build_args(platform, profile, signed)
        assert args[args.index("--bundles") + 1] == (
            "deb,rpm" if platform == "linux" else "nsis"
        )
        assert ("--no-sign" in args) == (not signed)
        assert ("--debug" in args) == (profile == "dev")
        assert "--profile" not in args and "--release" not in args
        if not signed:
            config = json.loads(args[6])
            assert config["bundle"]["createUpdaterArtifacts"] is False
            if profile == "dev":
                assert (
                    config["build"]["beforeBuildCommand"]
                    == "pnpm run build && mise run tauri:resources:dev"
                )
            else:
                assert "build" not in config
    with pytest.raises(ValueError):
        installer.build_args("linux", "other", False)


def case_installer_config(tmp_path: Path) -> None:
    source = frontend.ROOT / installer.TAURI
    for relative in [
        "tauri.conf.json",
        "tauri.linux.conf.json",
        "windows/installer-hooks.nsh",
        "icons/icon.ico",
    ]:
        path = tmp_path / installer.TAURI / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / relative, path)
    installer.check(tmp_path)
    path = tmp_path / installer.TAURI / "tauri.conf.json"
    original = json.loads(path.read_text())
    variants = []
    for binaries in [[], ["binaries/dokime", "binaries/cargo-dokime"]]:
        config = deepcopy(original)
        config["bundle"]["externalBin"] = binaries
        variants.append(config)
    config = deepcopy(original)
    del config["bundle"]["windows"]
    variants.append(config)
    config = deepcopy(original)
    config["bundle"]["windows"]["nsis"]["installerHooks"] = "windows/other.nsh"
    variants.append(config)
    for resource in ["resources/runtime", "resources/plugins"]:
        config = deepcopy(original)
        config["bundle"]["resources"].append(resource)
        variants.append(config)
    config = deepcopy(original)
    config["bundle"]["linux"] = {
        "deb": {"files": {"/usr/share/dokime/plugins/retired.dokime": "retired"}}
    }
    variants.append(config)
    for config in variants:
        path.write_text(json.dumps(config))
        with pytest.raises(ValueError):
            installer.check(tmp_path)


# === Private Functions === #


# === Entry Point === #


@pytest.mark.parametrize("contract", ["modes", "configuration"])
def test_installer_contract(contract: str, tmp_path: Path) -> None:
    if contract == "modes":
        case_installer_modes()
    else:
        case_installer_config(tmp_path)
