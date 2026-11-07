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
    monkeypatch.setattr(bootstrap, "_run", commands.append)
    return commands


@pytest.mark.parametrize("uid", [0, 1001])
@pytest.mark.parametrize("runtime,desktop", [(False, False), (True, True)])
def test_install_packages(monkeypatch: pytest.MonkeyPatch, host, uid: int, runtime: bool, desktop: bool) -> None:
    monkeypatch.setattr(bootstrap.os, "geteuid", lambda: uid)
    bootstrap.install_gui_packages(runtime=runtime, desktop=desktop)

    prefix = [] if uid == 0 else ["sudo", "-n"]
    selected = bootstrap.GUI_RUNTIME_PACKAGES if runtime else bootstrap.GUI_PACKAGES
    packages = (*selected, *(bootstrap.DESKTOP_PACKAGES if desktop else ()))
    assert host == [
        [*prefix, "apt-get", "update"],
        [*prefix, "apt-get", "install", "-y", "--no-install-recommends", *packages],
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


def test_sudo_refusal(monkeypatch: pytest.MonkeyPatch, host) -> None:
    def refuse(command):
        host.append(command)
        raise subprocess.CalledProcessError(1, command)

    monkeypatch.setattr(bootstrap, "_run", refuse)
    with pytest.raises(subprocess.CalledProcessError):
        bootstrap.install_gui_packages()
    assert host == [["sudo", "-n", "apt-get", "update"]]


# === Private Functions === #

# === Entry Point === #
