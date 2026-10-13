# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The CI bootstrap branches on the host system and never guesses a package manager."""

# === Imports === #

# Standard Library Imports
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.ci import bootstrap

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PlatformStepTests(unittest.TestCase):
    def case_linux_asked(self):
        self.assertEqual(
            bootstrap.platform_steps("Linux", gui_packages=True),
            [bootstrap.install_gui_packages],
        )
        self.assertEqual(bootstrap.platform_steps("Linux", gui_packages=False), [])

    def case_windows_packages(self):
        for requested in (True, False):
            self.assertEqual(
                bootstrap.platform_steps("Windows", gui_packages=requested),
                [bootstrap.verify_visual_cpp_toolset],
            )

    def case_other_loudly(self):
        with self.assertRaises(SystemExit) as caught:
            bootstrap.platform_steps("Darwin", gui_packages=False)
        self.assertEqual(
            str(caught.exception),
            "bootstrap.py supports Linux and Windows CI hosts, not Darwin.",
        )

    def case_contract(self):
        for scenario in (
            "case_linux_asked",
            "case_windows_packages",
            "case_other_loudly",
        ):
            with self.subTest(scenario=scenario):
                type(self)(scenario).debug()


class GuiPackageTests(unittest.TestCase):
    def case_host_alone(self):
        with (
            patch.object(bootstrap.shutil, "which", return_value=None),
            patch.object(bootstrap, "_run") as run,
        ):
            bootstrap.install_gui_packages()
        run.assert_not_called()

    def case_apt_absent(self):
        for installed, expected in ((True, 0), (False, 2)):
            with (
                patch.object(bootstrap.shutil, "which", return_value="/usr/bin/dpkg"),
                patch.object(bootstrap.os, "geteuid", create=True, return_value=0),
                patch.object(bootstrap, "_package_installed", return_value=installed),
                patch.object(bootstrap, "_run") as run,
            ):
                bootstrap.install_gui_packages()
            self.assertEqual(run.call_count, expected)

    def case_debian_skipped(self):
        with (
            patch.object(bootstrap.shutil, "which", return_value="/usr/bin/dpkg"),
            patch.object(bootstrap.os, "geteuid", create=True, return_value=1000),
            patch.object(bootstrap, "_package_installed", return_value=False),
            patch.object(bootstrap, "_run") as run,
            self.assertRaises(SystemExit) as caught,
        ):
            bootstrap.install_gui_packages()
        run.assert_not_called()
        self.assertIn(bootstrap.GUI_MARKER_PACKAGE, str(caught.exception))

    def case_marker_packages(self):
        self.assertIn(bootstrap.GUI_MARKER_PACKAGE, bootstrap.GUI_PACKAGES)

    def case_contract(self):
        for scenario in (
            "case_host_alone",
            "case_apt_absent",
            "case_debian_skipped",
            "case_marker_packages",
        ):
            with self.subTest(scenario=scenario):
                type(self)(scenario).debug()


class ArgumentTests(unittest.TestCase):
    def case_defaults_host(self):
        arguments = bootstrap.parse_arguments([])
        self.assertFalse(arguments.gui_packages)
        self.assertEqual(arguments.tasks, [])

    def case_tasks_given(self):
        arguments = bootstrap.parse_arguments(
            ["--gui-packages", "sync", "build:cli:dev"]
        )
        self.assertTrue(arguments.gui_packages)
        self.assertEqual(arguments.tasks, ["sync", "build:cli:dev"])

    def case_contract(self):
        for scenario in ("case_defaults_host", "case_tasks_given"):
            with self.subTest(scenario=scenario):
                type(self)(scenario).debug()


# === Public Functions === #


@pytest.fixture(autouse=True)
def bootstrap_environment(monkeypatch):
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.delenv("GITLAB_CI", raising=False)


def test_hosted_linux_sudo(monkeypatch):
    monkeypatch.setenv("GITHUB_ACTIONS", "true")
    monkeypatch.setattr(
        bootstrap.shutil, "which", lambda command: f"/usr/bin/{command}"
    )
    monkeypatch.setattr(bootstrap.os, "geteuid", lambda: 1000, raising=False)
    monkeypatch.setattr(bootstrap, "_package_installed", lambda package: False)
    calls = []
    monkeypatch.setattr(bootstrap, "_run", calls.append)
    bootstrap.install_gui_packages()
    assert calls[0] == ["sudo", "-n", "apt-get", "update"]
    assert calls[1][:4] == ["sudo", "-n", "apt-get", "install"]


@pytest.mark.parametrize("revision", ["main", "a" * 39, "A" * 40])
def test_harmos_pin(tmp_path: Path, revision: str) -> None:
    (tmp_path / "Cargo.toml").write_text(
        '[workspace.dependencies]\nharmos = { git = "https://github.com/stokker-technologies/harmos.git", rev = "'
        + revision
        + '" }\n'
    )
    with pytest.raises(ValueError, match="exact declared public Git revision"):
        bootstrap.harmos_pin(tmp_path)


def test_harmos_consumer(tmp_path: Path, monkeypatch) -> None:
    (tmp_path / "Cargo.toml").write_text(
        "[workspace.dependencies]\n"
        + "".join(
            f'{name} = {{ git = "https://github.com/stokker-technologies/harmos.git", rev = "{"a" * 40}" }}\n'
            for name in ("harmos", "harmos-contract", "harmos-signal")
        )
    )
    monkeypatch.setattr(
        bootstrap.subprocess, "check_output", lambda *args, **kwargs: "pinned rustc"
    )
    calls = []
    monkeypatch.setattr(bootstrap, "_run", calls.append)
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert calls == []

    def install(command):
        calls.append(command)
        destination = Path(command[command.index("--root") + 1])
        binary = (
            destination
            / "bin"
            / ("cargo-harmos.exe" if bootstrap.os.name == "nt" else "cargo-harmos")
        )
        binary.parent.mkdir()
        binary.write_bytes(b"prepared canonical tool")

    monkeypatch.setattr(bootstrap, "_run", install)
    bootstrap.prepare_harmos(tmp_path, allow_install=True)
    assert len(calls) == 1
    bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert len(calls) == 1
    binary = next((tmp_path / ".cache/harmos-tool/bin").iterdir())
    binary.write_bytes(b"changed tool")
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert len(calls) == 1

    manifest = tmp_path / "Cargo.toml"
    lines = manifest.read_text().splitlines()
    manifest.write_text(
        "\n".join(
            line.replace("a" * 40, "b" * 40)
            if line.startswith("harmos-contract =")
            else line
            for line in lines
        )
    )
    with pytest.raises(ValueError, match="all Harmos dependencies"):
        bootstrap.harmos_pin(tmp_path)


# === Private Functions === #

# === Entry Point === #


@pytest.mark.parametrize(
    "contract", [PlatformStepTests, GuiPackageTests, ArgumentTests]
)
def test_bootstrap_contract(contract: type[unittest.TestCase]) -> None:
    contract("case_contract").debug()
