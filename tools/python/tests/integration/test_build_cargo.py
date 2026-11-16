# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Prove Focused Build Dispatch and Compiler Input Freshness."

# === Imports === #

# Standard Library Imports
import io
import shlex
import shutil
import subprocess
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from types import SimpleNamespace
from typing import Any
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.build.rust.__main__
from dokime_tools.build import cargo, organization
from dokime_tools.ci import tasks
from dokime_tools.test import archive

from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ProductCompilerRecordTests(unittest.TestCase):
    def test_publishes_only_successful_compiler_output(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            messages = root / "product/compiler.json"
            messages.parent.mkdir()
            messages.write_text("previous successful build", encoding="utf-8")
            web = root / "gui"
            (web / "dist").mkdir(parents=True)

            def compile_product(*args: Any, **kwargs: Any) -> None:
                # The Plugin Bundle Follows a Successful Compile; It Writes No Compiler Record.
                if args[0][1:3] == ["plugin", "bundle"]:
                    return
                kwargs["stdout"].write("replacement build")
                self.assertEqual(messages.read_text(encoding="utf-8"), "previous successful build")
                if fail:
                    raise subprocess.CalledProcessError(1, args[0])

            with (
                patch.object(dokime_tools.build.rust.__main__.paths, "output", return_value=root),
                patch.object(dokime_tools.build.rust.__main__.configuration, "ROOT", root),
                patch.object(dokime_tools.build.rust.__main__.subprocess, "run", side_effect=compile_product),
                patch.object(dokime_tools.build.rust.__main__.outputs, "executables", return_value={}),
                patch.object(dokime_tools.build.rust.__main__.cargo, "record_target"),
            ):
                # Retain The Last Successful Record And The Failed Diagnostics
                fail = True
                with self.assertRaises(subprocess.CalledProcessError):
                    dokime_tools.build.rust.__main__.build("dev")
                self.assertEqual(messages.read_text(encoding="utf-8"), "previous successful build")
                pending = messages.with_name("compiler.pending.json")
                self.assertEqual(pending.read_text(encoding="utf-8"), "replacement build")

                # Publish The Completed Replacement Only After Cargo Succeeds
                fail = False
                exported = dokime_tools.build.rust.__main__.build("dev")
                self.assertEqual(messages.read_text(encoding="utf-8"), "replacement build")
                self.assertFalse(pending.exists())
                self.assertEqual(set(exported), {"bin/plugin-obc.dokime", "product/compiler.json", "product/target.txt"})
        finally:
            directory_directory.cleanup()


class CompilerEnvironmentTests(unittest.TestCase):
    def test_archive_timings_and_linker_options_compose(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            destination = root / "tests/archive.tar.zst"
            for extra in ([], ["--timings"], ["--timings=html"]):
                with (
                    self.subTest(extra=extra),
                    patch.object(archive, "location", return_value=destination),
                    patch.object(archive, "require"),
                    patch.object(archive, "prune"),
                    patch.object(
                        cargo, "compiler_environment", return_value={"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER": "custom-linker"}
                    ),
                    patch.object(cargo.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run,
                ):
                    self.assertEqual(cargo.archive(root, "dev", extra), destination)
                    command = run.call_args.args[0]
                    timings = []
                    for timings_argument in command:
                        if not (timings_argument == "--timings" or timings_argument.startswith("--timings=")):
                            continue
                        timings.append(timings_argument)

                    self.assertEqual(timings, extra or ["--timings"])
                    self.assertEqual(run.call_args.kwargs["env"]["CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER"], "custom-linker")
        finally:
            temporary_directory.cleanup()

    def test_development_and_test_selectors_share_the_linker(self) -> None:
        directory = Path.cwd()
        for operation in ("build", "check", "test", "rustc", "clippy"):
            with (
                self.subTest(operation=operation),
                patch.object(cargo, "os", SimpleNamespace(name="nt", environ={})),
                patch.object(cargo.subprocess, "run") as run,
            ):
                cargo.focused(directory, operation, ["--locked", "-p", "example"])
                selected = run.call_args.kwargs["env"]
                self.assertEqual(selected["CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER"], "rust-lld")
                self.assertEqual(selected["CARGO_BUILD_WARNINGS"], "deny")

    def test_release_bench_linux_and_explicit_overrides_retain_their_environment(self) -> None:
        for platform, profile in (("nt", "release"), ("nt", "bench"), ("posix", "dev"), ("posix", "test")):
            source = {"RUSTFLAGS": "--cfg example"}
            with self.subTest(platform=platform, profile=profile), patch.object(cargo.os, "name", platform):
                self.assertEqual(cargo.compiler_environment(profile, source=source), source)
        source = {"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER": "link.exe"}
        with patch.object(cargo.os, "name", "nt"):
            self.assertEqual(cargo.compiler_environment("dev", source=source), source)
        self.assertEqual(source, {"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER": "link.exe"})


class IconInputTests(unittest.TestCase):
    def test_icon_generation_does_not_rewrite_identical_compiler_inputs(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            brand = root / "organization"
            (brand / "icons").mkdir(parents=True)
            (brand / "icons/icon.png").write_bytes(b"source fixture")
            destination = root / "crates/dokime-gui/src-tauri/icons"
            destination.mkdir(parents=True)
            small = b"ic08" + (13).to_bytes(4, "big") + b"small"
            large = b"ic09" + (13).to_bytes(4, "big") + b"large"
            header = b"icns" + (8 + len(small) + len(large)).to_bytes(4, "big")
            outputs = {"icon.ico": b"icon fixture", "64x64.png": b"png fixture", "icon.icns": header + large + small}

            def render(command: list[str], **options: Any) -> None:
                directory = Path(command[command.index("--output") + 1])
                for name, content in outputs.items():
                    (directory / name).write_bytes(content)

            with (
                patch.object(organization.configuration, "ROOT", root),
                patch.object(organization, "BRAND", brand),
                patch.object(organization.subprocess, "run", side_effect=render),
                patch.object(shutil, "copyfile", wraps=shutil.copyfile) as copies,
            ):
                organization.generate_icons()
                self.assertEqual(copies.call_count, len(outputs))
                self.assertEqual((destination / "icon.icns").read_bytes(), header + small + large)
                outputs["icon.icns"] = header + small + large
                organization.generate_icons()
                self.assertEqual(copies.call_count, len(outputs))
                outputs["icon.ico"] = b"changed icon fixture"
                organization.generate_icons()
                self.assertEqual(copies.call_count, len(outputs) + 1)
                self.assertEqual((destination / "icon.ico").read_bytes(), outputs["icon.ico"])
                for invalid in (b"icns", b"icns" + (16).to_bytes(4, "big") + b"ic08" + bytes(4)):
                    outputs["icon.icns"] = invalid
                    with self.assertRaises(ValueError):
                        organization.generate_icons()
        finally:
            temporary_directory.cleanup()


class FocusedCargoTests(unittest.TestCase):
    def test_focused_test_passes_features_before_harness_arguments(self) -> None:
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

    def test_dispatches_only_requested_operation(self) -> None:
        directory = Path.cwd()
        operations = ("build", "check", "clippy", "run", "bench", "rustc")
        for operation in operations:
            with self.subTest(operation=operation):
                arguments = ["--locked", "-p", "example"]
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

    def test_rejects_target_redirection_before_dispatch(self) -> None:
        with (
            patch.object(cargo.subprocess, "run") as run,
            self.assertRaises(ValueError),
        ):
            cargo.focused(Path.cwd(), "build", ["--target-dir", "elsewhere"])
        run.assert_not_called()

    def test_default_build_plan_has_no_side_effects(self) -> None:
        output = io.StringIO()
        with (
            patch.object(tasks.transport, "receive") as receive,
            patch.object(tasks.transport, "send") as send,
            patch.object(subprocess, "run") as execute,
            redirect_stdout(output),
        ):
            result = tasks.main(["build", "--plan"])
        command = shlex.split(output.getvalue())
        output.close()
        self.assertEqual(result, 0)
        self.assertEqual(command, ["mise", "run", "build:product", "--", "--profile", "dev"])
        receive.assert_not_called()
        send.assert_not_called()
        execute.assert_not_called()


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #


class TaskSelectionTests(unittest.TestCase):
    def test_focused_build_and_check_dispatch_only_the_package(self) -> None:
        for check in (False, True):
            with self.subTest(check=check), patch.object(tasks.cargo, "focused", return_value=7) as focused:
                result = tasks.main(["build", "--package", "example", "--profile", "release", *(["--check"] if check else [])])
                self.assertEqual(result, 7)
                focused.assert_called_once_with(
                    Path.cwd(), "check" if check else "build", ["--locked", "--package", "example", "--profile", "release"]
                )

    def test_native_compiler_arguments_pass_through_without_lint(self) -> None:
        arguments = ["rustc", "--locked", "--package", "example", "--bin", "example", "--", "-C", "linker=rust-lld"]
        with patch.object(tasks.cargo, "focused", return_value=3) as focused:
            self.assertEqual(tasks.main(["build", "--cargo", *arguments]), 3)
        focused.assert_called_once_with(Path.cwd(), "rustc", arguments[1:])

    def test_native_compiler_selection_requires_a_package_and_excludes_lint(self) -> None:
        for arguments in (["clippy", "-p", "example"], ["build", "--workspace"], ["rustc"]):
            with self.subTest(arguments=arguments), patch.object(tasks.cargo, "focused") as focused, redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):
                    tasks.main(["build", "--cargo", *arguments])
                focused.assert_not_called()

    def test_timings_precede_test_harness_arguments(self) -> None:
        with patch.object(tasks.cargo, "focused", return_value=0) as focused:
            self.assertEqual(tasks.main(["test", "--package", "example", "--target", "integration", "--exact", "one", "--timings"]), 0)
        focused.assert_called_once_with(
            Path.cwd(), "test", ["--timings", "--locked", "--package", "example", "--test", "integration", "one", "--", "--exact"]
        )

    def test_timings_reject_non_compiling_operations(self) -> None:
        with patch.object(tasks.subprocess, "run") as run, redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            tasks.main(["format", "--timings"])
        run.assert_not_called()

    def test_focused_test_preserves_target_and_exact_name(self) -> None:
        for target in ("lib", "integration"):
            with self.subTest(target=target), patch.object(tasks.cargo, "focused", return_value=0) as focused:
                self.assertEqual(tasks.main(["test", "--package", "example", "--target", target, "--exact", "suite::one test"]), 0)
                selected = ["--lib"] if target == "lib" else ["--test", target]
                focused.assert_called_once_with(
                    Path.cwd(), "test", ["--locked", "--package", "example", *selected, "suite::one test", "--", "--exact"]
                )

    def test_focused_features_reach_cargo_unchanged(self) -> None:
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

    def test_features_require_a_focused_package(self) -> None:
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

    def test_language_layer_and_tooling_selections_use_existing_leaves(self) -> None:
        cases = [
            (
                ["test", "--language", "python", "--layer", "integration", "--project", "tools/python"],
                ["test:python:integration", "--", "--profile", "dev", "--project", "tools/python"],
            ),
            (["build", "--artifact", "tests"], ["build:rust:tests", "--", "--profile", "dev"]),
            (["build", "--artifact", "tests", "--timings"], ["build:rust:tests", "--", "--profile", "dev", "--timings"]),
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
