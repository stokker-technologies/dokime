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
import io
import subprocess
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from environment import isolate_environment

from dokime_tools.build import cargo, lint
from dokime_tools.build.rust import __main__ as product

# Third Party Imports
# Local Imports
from dokime_tools.ci import tasks
from dokime_tools.prepare import generated

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class GeneratedContractTests(unittest.TestCase):
    def test_overlapping_preparations_read_their_own_compiler_records(self):
        records = []
        running = False

        def compile_contracts(*args, **kwargs):
            nonlocal running
            output = kwargs["stdout"]
            output.write("inner" if running else "outer")
            output.flush()
            if not running:
                running = True
                generated.prepare()

        with (
            patch.object(generated.subprocess, "run", side_effect=compile_contracts),
            patch.object(
                generated, "generate", side_effect=lambda messages, **kwargs: records.append(messages.read_text(encoding="utf-8"))
            ),
        ):
            generated.prepare()
        self.assertEqual(records, ["inner", "outer"])


class ProductCompilerRecordTests(unittest.TestCase):
    def test_publishes_only_successful_compiler_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            messages = root / "product/compiler.json"
            messages.parent.mkdir()
            messages.write_text("previous successful build", encoding="utf-8")
            web = root / "gui"
            (web / "dist").mkdir(parents=True)

            def compile_product(*args, **kwargs):
                kwargs["stdout"].write("replacement build")
                self.assertEqual(messages.read_text(encoding="utf-8"), "previous successful build")
                if fail:
                    raise subprocess.CalledProcessError(1, args[0])

            with (
                patch.object(product.paths, "output", return_value=root),
                patch.object(product.configuration, "features", return_value=[]),
                patch.object(product.frontend, "WEB_ROOT", web),
                patch.object(product.subprocess, "run", side_effect=compile_product),
                patch.object(product.outputs, "executables", return_value={}),
                patch.object(product.plugins, "prepare", return_value=[]),
                patch.object(product.cargo, "record_target"),
            ):
                # Retain The Last Successful Record And The Failed Diagnostics
                fail = True
                with self.assertRaises(subprocess.CalledProcessError):
                    product.build("dev")
                self.assertEqual(messages.read_text(encoding="utf-8"), "previous successful build")
                pending = messages.with_name("compiler.pending.json")
                self.assertEqual(pending.read_text(encoding="utf-8"), "replacement build")

                # Publish The Completed Replacement Only After Cargo Succeeds
                fail = False
                product.build("dev")
                self.assertEqual(messages.read_text(encoding="utf-8"), "replacement build")
                self.assertFalse(pending.exists())


class FocusedCargoTests(unittest.TestCase):
    def test_focused_test_passes_features_before_harness_arguments(self):
        arguments = [
            "test",
            "--package",
            "dokime-runtime",
            "--features",
            "testing",
            "--target",
            "integration",
            "--exact",
            "config::settings_backends",
        ]
        with patch.object(tasks.transport, "receive"), patch.object(tasks.cargo, "focused", return_value=0) as focused:
            self.assertEqual(tasks.main(arguments), 0)
        focused.assert_called_once_with(
            Path.cwd(),
            "test",
            [
                "--locked",
                "--package",
                "dokime-runtime",
                "--features",
                "testing",
                "--test",
                "integration",
                "config::settings_backends",
                "--",
                "--exact",
            ],
        )

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

    def test_focused_features_reach_cargo_unchanged(self):
        # Select A Focused Build Or Test With Defaults Enabled
        for operation in ("build", "test"):
            for features in ("testing,fft", "testing fft"):
                arguments = [operation, "--package", "example", "--features", features]
                suffix = ["--profile", "dev"] if operation == "build" else ["--lib"]
                if operation == "test":
                    arguments += ["--target", "lib"]
                with self.subTest(operation=operation, features=features), patch.object(tasks.cargo, "focused", return_value=0) as focused:
                    self.assertEqual(tasks.main(arguments), 0)
                    focused.assert_called_once_with(
                        Path.cwd(), operation, ["--locked", "--package", "example", "--features", features, *suffix]
                    )

    def test_features_require_a_focused_package(self):
        # Reject Features Before Dispatching An Unfocused Task
        for operation in ("build", "test"):
            diagnostic = io.StringIO()
            with (
                self.subTest(operation=operation),
                patch.object(tasks.cargo, "focused") as focused,
                redirect_stderr(diagnostic),
                self.assertRaises(SystemExit) as rejected,
            ):
                tasks.main([operation, "--features", "testing"])
            self.assertEqual(rejected.exception.code, 2)
            focused.assert_not_called()

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
