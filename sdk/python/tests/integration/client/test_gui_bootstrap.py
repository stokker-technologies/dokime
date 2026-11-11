# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Provision GUI Prerequisites Independently Of The CI Provider"""

# === Imports === #

# Standard Library Imports
import subprocess

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.ci import bootstrap

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.fixture
def host(monkeypatch: pytest.MonkeyPatch):
    commands = []
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr(bootstrap.sys, "platform", "linux")
    monkeypatch.setattr(bootstrap.os, "geteuid", lambda: 1001, raising=False)
    monkeypatch.setattr(bootstrap.shutil, "which", lambda name: "/usr/bin/" + name)
    monkeypatch.setattr(bootstrap, "_package_installed", lambda package: False)

    def run(command, **options):
        commands.append(command)
        assert options["stdin"] == subprocess.DEVNULL
        return subprocess.CompletedProcess(command, 0, stdout="")

    monkeypatch.setattr(bootstrap.subprocess, "run", run)
    monkeypatch.setattr(bootstrap.time, "sleep", lambda delay_s: None)
    return commands


@pytest.mark.parametrize("uid", [0, 1001])
@pytest.mark.parametrize("runtime,desktop", [(False, False), (True, True)])
def test_install_packages(monkeypatch: pytest.MonkeyPatch, host, uid: int, runtime: bool, desktop: bool) -> None:
    monkeypatch.setattr(bootstrap.os, "geteuid", lambda: uid)
    bootstrap.install_gui_packages(runtime=runtime, desktop=desktop)

    prefix = [] if uid == 0 else ["sudo", "-n"]
    selected = bootstrap.GUI_RUNTIME_PACKAGES if runtime else bootstrap.GUI_PACKAGES
    packages = (*selected, *(bootstrap.DESKTOP_PACKAGES if desktop else ()))
    apt = [*prefix, "apt-get", "-o", f"Acquire::Retries={bootstrap.APT_ACQUIRE_RETRIES}"]
    assert host == [
        [*apt, "-o", "APT::Update::Error-Mode=any", "update"],
        [*apt, "install", "-y", "--no-install-recommends", *packages],
    ]


def test_missing_privilege(monkeypatch: pytest.MonkeyPatch, host) -> None:
    monkeypatch.setattr(bootstrap.shutil, "which", lambda name: None if name == "sudo" else "/usr/bin/" + name)
    with pytest.raises(SystemExit):
        bootstrap.install_gui_packages()
    assert host == []


@pytest.mark.parametrize("state", ["foreign", "unavailable", "installed"])
def test_skip_installation(monkeypatch: pytest.MonkeyPatch, host, state: str) -> None:
    if state == "foreign":
        monkeypatch.setattr(bootstrap.sys, "platform", "win32")
    elif state == "unavailable":
        monkeypatch.setattr(bootstrap.shutil, "which", lambda name: None)
    else:
        monkeypatch.setattr(bootstrap, "_package_installed", lambda package: True)
    bootstrap.install_gui_packages()
    assert host == []


def test_sudo_refusal_preserves_diagnostics_after_retries(monkeypatch: pytest.MonkeyPatch, host) -> None:
    def refuse(command, **options):
        host.append(command)
        assert options["stdin"] == subprocess.DEVNULL
        return subprocess.CompletedProcess(command, 1, stdout="sudo: a password is required\n")

    monkeypatch.setattr(bootstrap.subprocess, "run", refuse)
    with pytest.raises(RuntimeError, match="sudo: a password is required"):
        bootstrap.install_gui_packages()
    assert host
    assert all(command[:3] == ["sudo", "-n", "apt-get"] and command[-1] == "update" for command in host)


# === Private Functions === #

# === Entry Point === #
