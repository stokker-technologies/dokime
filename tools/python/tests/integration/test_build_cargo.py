# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Prove Focused Cargo Dispatch Without Invoking A Compiler."""

# === Imports === #

# Standard Library Imports
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from environment import isolate_environment

from dokime_tools.build import cargo, lint

# Third Party Imports
# Local Imports
from dokime_tools.ci import tasks

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class FocusedCargoTests(unittest.TestCase):
    def test_dispatches_only_requested_operation(self):
        directory = Path.cwd()
        operations = ("build", "check", "test", "clippy", "run", "bench")
        for operation in operations:
            with self.subTest(operation=operation):
                arguments = ["--locked", "-p", "example"]
                if operation == "test":
                    arguments += ["--no-run", "--no-fail-fast"]
                events = []
                with (
                    patch.object(
                        cargo.subprocess,
                        "run",
                        side_effect=lambda *args, events=events, **kwargs: events.append((args, kwargs)) or SimpleNamespace(returncode=7),
                    ),
                ):
                    status = cargo.focused(directory, operation, arguments)
                self.assertEqual(status, 7)
                self.assertEqual(len(events), 1)
                command, options = events[0]
                self.assertEqual(command, (["cargo", operation, *arguments],))
                self.assertEqual(options["cwd"], directory)
                self.assertEqual(options["env"]["CARGO_BUILD_WARNINGS"], "deny")

    def test_rejects_target_redirection_before_dispatch(self):
        with (
            patch.object(cargo.subprocess, "run") as run,
            self.assertRaises(ValueError),
        ):
            cargo.focused(Path.cwd(), "build", ["--target-dir", "elsewhere"])
        run.assert_not_called()


class DesktopLintTests(unittest.TestCase):
    def test_prepares_missing_frontend_before_linting(self):
        with tempfile.TemporaryDirectory() as temporary:
            web = Path(temporary) / "web"
            arguments = ["--locked", "--workspace", "--profile", "dev"]
            with patch.object(lint.frontend, "WEB_ROOT", web), patch.object(lint.cargo, "clippy") as clippy:
                clippy.side_effect = lambda *args: self.assertTrue((web / "dist/index.html").is_file())
                lint.desktop(arguments)
            clippy.assert_called_once_with(lint.frontend.ROOT, arguments)

    def test_preserves_existing_frontend(self):
        with tempfile.TemporaryDirectory() as temporary:
            web = Path(temporary)
            dist = web / "dist"
            dist.mkdir()
            index = dist / "index.html"
            index.write_text("existing frontend", encoding="utf-8")
            with patch.object(lint.frontend, "WEB_ROOT", web), patch.object(lint.cargo, "clippy"):
                lint.desktop(["--locked"])
            self.assertEqual(index.read_text(encoding="utf-8"), "existing frontend")


# === Public Functions === #


def setUpModule():
    isolate_environment()


# === Private Functions === #

# === Entry Point === #


class TaskSelectionTests(unittest.TestCase):
    def test_focused_build_and_check_dispatch_only_the_package(self):
        for check in (False, True):
            with self.subTest(check=check), patch.object(tasks.cargo, "focused", return_value=7) as focused:
                result = tasks.main(["build", "--package", "example", "--profile", "release", *(["--check"] if check else [])])
                self.assertEqual(result, 7)
                focused.assert_called_once_with(
                    Path.cwd(), "check" if check else "build", ["--locked", "--package", "example", "--profile", "release"]
                )

    def test_focused_test_preserves_target_and_exact_name(self):
        for target in ("lib", "integration"):
            with self.subTest(target=target), patch.object(tasks.cargo, "focused", return_value=0) as focused:
                self.assertEqual(tasks.main(["test", "--package", "example", "--target", target, "--exact", "suite::one test"]), 0)
                selected = ["--lib"] if target == "lib" else ["--test", target]
                focused.assert_called_once_with(
                    Path.cwd(), "test", ["--locked", "--package", "example", *selected, "suite::one test", "--", "--exact"]
                )

    def test_language_layer_and_tooling_selections_use_existing_leaves(self):
        cases = [
            (
                ["test", "--language", "python", "--layer", "integration", "--project", "tools/python"],
                ["test:python:integration", "--", "--profile", "dev", "--project", "tools/python"],
            ),
            (["build", "--artifact", "tests"], ["build:rust:tests", "--", "--profile", "dev"]),
            (["check", "--repository"], ["check:repository", "--"]),
            (
                ["bench", "--language", "rust", "--layer", "unit", "--binary", "example", "--select", "filter"],
                ["bench:rust:unit", "--", "--binary", "example", "--select", "filter"],
            ),
        ]
        for arguments, command in cases:
            with (
                self.subTest(arguments=arguments),
                patch.object(tasks.subprocess, "run", return_value=SimpleNamespace(returncode=9)) as run,
            ):
                self.assertEqual(tasks.main(arguments), 9)
                self.assertEqual(run.call_args.args[0], ["mise", "run", *command])
