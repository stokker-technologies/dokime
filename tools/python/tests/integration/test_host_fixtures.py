# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Mechanical Test Host Fixtures."""

# === Imports === #

# Standard Library Imports
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.build.rust.tests
from dokime_tools.build import cargo, cli_fixtures, desktop_fixtures, host_fixtures, lint, paths
from dokime_tools.prepare import generated

# Type Checking Imports

# === Public Exports === #


# === Constants === #


# === Types === #

# === Classes === #


class HostFixtureTests(unittest.TestCase):
    def test_cpp_cli_fixture_runs_and_stages_the_release_plugin(self) -> None:
        for platform, extension in (("posix", ""), ("nt", ".exe")):
            temporary_directory = tempfile.TemporaryDirectory()
            root = Path(temporary_directory.name)
            try:
                bundle = root / "target/cp/release/dokime-cpp-fixture.tar.gz"
                bundle.parent.mkdir(parents=True)
                bundle.write_bytes(b"archive")
                bundle.with_name(f"{bundle.name}.sha256").write_text("a" * 64, encoding="utf-8")
                output = root / "target/build/dev/native"
                (output / "bin").mkdir(parents=True)
                (output / "bin" / f"dokime{extension}").write_bytes(b"product cli")
                placements = []

                def run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                    if command[1] == "new":
                        plugin = Path(command[2]) / "assets/plugin" / f"dokime-plugin{extension}"
                        plugin.parent.mkdir(parents=True)
                        plugin.write_bytes(b"prepared plugin")
                    if Path(command[1]).name == "run_plugin.py":
                        self.assertEqual(Path(command[2]).read_bytes(), b"prepared plugin")
                        placements.append(command[-1])
                    return subprocess.CompletedProcess(command, 0, stdout="", stderr="")

                with (
                    self.subTest(platform=platform),
                    patch.object(cli_fixtures.configuration, "ROOT", root),
                    patch.object(cli_fixtures.paths, "output", return_value=output),
                    patch.object(cli_fixtures, "os", SimpleNamespace(name=platform)),
                    patch.object(cli_fixtures.subprocess, "run", side_effect=run),
                    patch.object(cli_fixtures.archive, "stamp"),
                ):
                    cli_fixtures.build("dev", "cpp")
                self.assertEqual(placements, ["managed", "remote"])
                self.assertEqual((output / "fixtures/cli/cpp" / f"dokime-plugin{extension}").read_bytes(), b"prepared plugin")
            finally:
                temporary_directory.cleanup()

    def test_rust_cli_fixture_keeps_extensionless_executable_separate_from_source(self) -> None:
        for platform, extension in (("posix", ""), ("nt", ".exe")):
            temporary_directory = tempfile.TemporaryDirectory()
            root = Path(temporary_directory.name)
            try:
                executable = root / "target/debug" / f"rust{extension}"
                executable.parent.mkdir(parents=True)
                executable.write_bytes(b"prepared plugin")
                executable.chmod(0o755)
                output = root / "target/build/dev/native"
                (output / "bin").mkdir(parents=True)
                (output / "bin" / f"dokime{extension}").write_bytes(b"product cli")

                def run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                    if command[1] == "new":
                        project = Path(command[2])
                        project.mkdir()
                        (project / "dokime.toml").write_text("[plugin]\n", encoding="utf-8")
                    return subprocess.CompletedProcess(command, 0, stdout="", stderr="")

                patches = (
                    patch.object(cli_fixtures.configuration, "ROOT", root),
                    patch.object(cli_fixtures.paths, "output", return_value=output),
                    patch.object(cli_fixtures, "os", SimpleNamespace(name=platform)),
                    patch.object(cli_fixtures.subprocess, "run", side_effect=run),
                    patch.object(cli_fixtures.archive, "stamp"),
                )
                for pending in patches:
                    pending.start()
                try:
                    cli_fixtures.build("dev", "rust")
                finally:
                    for pending in reversed(patches):
                        pending.stop()

                fixture = output / "fixtures/cli" / executable.name
                source = output / "fixtures/cli/rust-source/dokime.toml"
                self.assertEqual(fixture.read_bytes(), executable.read_bytes())
                self.assertTrue(source.is_file())
                if os.name != "nt":
                    self.assertEqual(fixture.stat().st_mode & 0o777, executable.stat().st_mode & 0o777)
            finally:
                temporary_directory.cleanup()

    def test_selected_rust_fixture_skips_unrelated_producers(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        root = Path(temporary_directory.name)
        try:
            report = root / "target/generated/report/report.html"
            report.parent.mkdir(parents=True)
            report.write_bytes(b"report")
            with (
                patch.object(host_fixtures.configuration, "ROOT", root),
                patch.object(host_fixtures.generated, "prepare") as prepare,
                patch.object(host_fixtures.subprocess, "run") as run,
                patch.object(host_fixtures.shutil, "copy2") as copy,
            ):
                host_fixtures.build("dev", "rust", "dokime-rust-plugin")
                prepare.assert_not_called()
                run.assert_called_once()
                command = run.call_args.args[0]
                self.assertEqual(command[command.index("--bin") + 1], "dokime-rust-plugin")
                self.assertNotIn("--bins", command)
                extension = ".exe" if os.name == "nt" else ""
                copy.assert_called_once_with(
                    root / "target/debug" / f"dokime-rust-plugin{extension}",
                    paths.output(root, "build", "dev") / "fixtures/host" / f"dokime-rust-plugin{extension}",
                )
        finally:
            temporary_directory.cleanup()

    def test_report_generation_failure_stops_native_consumers(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            for consume in (
                lambda: host_fixtures.build("dev", "rust"),
                lambda: dokime_tools.build.rust.tests.build("dev"),
                lambda: lint.desktop([]),
            ):
                with (
                    self.subTest(consumer=consume),
                    patch.object(host_fixtures.configuration, "ROOT", Path(temporary_directory.name)),
                    patch.object(generated, "prepare", side_effect=ValueError("generation failed")),
                    patch.object(host_fixtures.subprocess, "run") as run,
                    patch.object(cargo, "clippy") as clippy,
                    patch.object(cargo, "archive") as archive,
                ):
                    self.assertRaises(ValueError, consume)
                    run.assert_not_called()
                    clippy.assert_not_called()
                    archive.assert_not_called()
        finally:
            temporary_directory.cleanup()

    def test_desktop_bundle_failure_keeps_tool_diagnostics(self) -> None:
        diagnostic = "fixture stderr\n"
        refusal = subprocess.CompletedProcess(["dokime"], 3, stderr=diagnostic)
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                patch.object(desktop_fixtures.configuration, "ROOT", Path(temporary)),
                patch.object(desktop_fixtures.paths, "target", return_value="windows-x86_64"),
                patch.object(desktop_fixtures.subprocess, "run", return_value=refusal),
                patch.object(Path, "is_file", return_value=True),
                patch.object(desktop_fixtures.shutil, "copy2"),
                patch.object(desktop_fixtures.sys, "stderr", new_callable=io.StringIO) as stderr,
                self.assertRaises(subprocess.CalledProcessError) as failure,
            ):
                desktop_fixtures.build("release")
        finally:
            temporary_directory.cleanup()
        self.assertEqual(failure.exception.returncode, refusal.returncode)
        self.assertEqual(stderr.getvalue(), diagnostic)

    def test_explicit_producers_build_only_the_selected_language(self) -> None:
        for language in ("rust", "python", "csharp", "cpp", "none"):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                root = Path(temporary)
                (root / "Cargo.toml").write_text(
                    '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n',
                    encoding="utf-8",
                )
                fixture = root / "library/sdk/python/tests/fixtures/conformance/__init__.py"
                fixture.parent.mkdir(parents=True)
                fixture.write_bytes(b"fixture source")
                product = paths.output(root, "build", "release") / "bin"
                product.mkdir(parents=True)
                extension = ".exe" if os.name == "nt" else ""
                for name in ("dokime-host", "plugin-obc"):
                    (product / f"{name}{extension}").write_bytes(b"product")
                report = root / "target/generated/report/report.html"
                report.parent.mkdir(parents=True)
                report.write_bytes(b"report")
                with (
                    self.subTest(language=language),
                    patch.object(host_fixtures.configuration, "ROOT", root),
                    patch.object(host_fixtures.generated, "prepare") as prepare,
                    patch.object(host_fixtures.subprocess, "run") as run,
                    patch.object(host_fixtures.shutil, "copy2"),
                    patch.object(host_fixtures.archive, "stamp") as stamp,
                ):
                    run.return_value.stdout = json.dumps(
                        {
                            "packages": [
                                {
                                    "name": "harmos",
                                    "source": "git+https://example.test/library#fixture-revision",
                                    "manifest_path": str(root / "library/sdk/rust/Cargo.toml"),
                                }
                            ]
                        }
                    )
                    host_fixtures.build("release", language)
                    if language == "rust":
                        prepare.assert_called_once_with()
                    else:
                        prepare.assert_not_called()
                    commands = [call.args[0] for call in run.call_args_list]
                    self.assertEqual(
                        {command[2] for command in commands if command[1] == "-C"},
                        set() if language in ("none", "rust") else {f"sdk/{language}"},
                    )
                    stamp.assert_not_called()
                    if language == "cpp":
                        self.assertEqual(commands[0][4:], ["build", "--", "--preset", "release", "--target", "dokime_client"])
                        self.assertEqual(commands[1][4:], ["package", "--", "--preset", "release"])
                        self.assertEqual(commands[2][4:8], ["consumer", "--", "--preset", "release"])
                        self.assertEqual(Path(commands[2][-1]), root / "sdk/tests/fixtures/cpp/thermal")
                    if language == "csharp":
                        self.assertEqual([command[4] for command in commands], ["build", "package", "publish:system"])
                    if language == "rust":
                        staged = paths.output(root, "build", "release") / "fixtures/host/python/conformance/__init__.py"
                        self.assertEqual(staged.read_bytes(), fixture.read_bytes())
                        reference = next(command for command in commands if command[:2] == ["cargo", "install"])
                        self.assertEqual(reference[reference.index("--git") + 1], "https://example.test/library")
                        self.assertEqual(reference[reference.index("--rev") + 1], "fixture-revision")
                        self.assertEqual(reference[reference.index("--profile") + 1], "release")
                        self.assertNotIn("--force", reference)
                        self.assertEqual(reference[reference.index("--target-dir") + 1], str(root / "target"))
                        consumer = commands[0]
                        selected = [consumer[index + 1] for index, value in enumerate(consumer) if value == "--bin"]
                        self.assertEqual(selected, list(host_fixtures.RUST_FIXTURES))
                        self.assertNotIn("--bins", consumer)
                        for command in commands:
                            if command[:3] == ["mise", "run", "build"]:
                                self.assertEqual(command[command.index("--profile") + 1], "release")
                        cached = Path(reference[reference.index("--root") + 1]) / "bin" / f"harmos-conformance{extension}"
                        cached.parent.mkdir(parents=True)
                        cached.write_bytes(b"cached reference")
                        run.reset_mock()
                        host_fixtures.build("release", language)
                        self.assertFalse(any(call.args[0][:2] == ["cargo", "install"] for call in run.call_args_list))
                        dev_product = paths.output(root, "build", "dev") / "bin"
                        dev_product.mkdir(parents=True)
                        for name in ("dokime-host", "plugin-obc"):
                            (dev_product / f"{name}{extension}").write_bytes(b"dev product")
                        run.reset_mock()
                        host_fixtures.build("dev", language)
                        self.assertTrue(any(call.args[0][:2] == ["cargo", "install"] for call in run.call_args_list))
            finally:
                temporary_directory.cleanup()

    def test_local_default_builds_present_toolchains_and_preserves_inventory(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            (root / "Cargo.toml").write_text(
                '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n', encoding="utf-8"
            )
            fixture = root / "library/sdk/python/tests/fixtures/conformance/__init__.py"
            fixture.parent.mkdir(parents=True)
            fixture.write_bytes(b"fixture source")
            output = paths.output(root, "build", "dev") / "fixtures/host"
            output.mkdir(parents=True)
            (output / "fixture").write_bytes(b"fixture")
            stamp = output.parent / "stamp.json"
            stamp.write_text('{"files":{"existing":"digest"},"metadata":{"owner":"tests"}}')
            product = output.parents[1] / "bin"
            product.mkdir()
            extension = ".exe" if os.name == "nt" else ""
            for name in ("dokime-host", "plugin-obc"):
                (product / f"{name}{extension}").write_bytes(b"product")
            report = root / "target/generated/report/report.html"
            report.parent.mkdir(parents=True)
            report.write_bytes(b"report")
            with (
                patch.object(host_fixtures.configuration, "ROOT", root),
                patch.object(host_fixtures.generated, "prepare"),
                patch.object(host_fixtures.subprocess, "run") as run,
                patch.object(host_fixtures.shutil, "copy2"),
                patch.object(host_fixtures.shutil, "which", side_effect=lambda name: name if name == "dotnet" else None),
                patch.object(host_fixtures.archive, "stamp") as inventory,
            ):
                run.return_value.stdout = json.dumps(
                    {
                        "packages": [
                            {
                                "name": "harmos",
                                "source": "git+https://example.test/library#fixture-revision",
                                "manifest_path": str(root / "library/sdk/rust/Cargo.toml"),
                            }
                        ]
                    }
                )
                host_fixtures.build("dev")
                self.assertEqual({call.args[0][2] for call in run.call_args_list if call.args[0][1] == "-C"}, {"sdk/csharp", "sdk/python"})
                expected = [
                    "existing",
                    (output / "fixture").relative_to(root).as_posix(),
                    (output / "python/conformance/__init__.py").relative_to(root).as_posix(),
                ]
                self.assertEqual(inventory.call_args.args[2], expected)
                self.assertEqual(inventory.call_args.args[3], {"owner": "tests"})
                run.reset_mock()
                host_fixtures.build("dev", "rust")
                self.assertEqual({call.args[0][2] for call in run.call_args_list if call.args[0][1] == "-C"}, set())
                self.assertEqual(inventory.call_args.args[2], expected)
        finally:
            temporary_directory.cleanup()

    def test_explicit_producer_failure_is_not_skipped(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                patch.object(host_fixtures.configuration, "ROOT", Path(temporary)),
                patch.object(host_fixtures.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "mise")),
                self.assertRaises(subprocess.CalledProcessError),
            ):
                host_fixtures.build("dev", "csharp")
        finally:
            temporary_directory.cleanup()

    def test_desktop_selects_requested_cli_bytes_and_refuses_missing_profile(self) -> None:
        extension = ".exe" if os.name == "nt" else ""
        for profile in ("dev", "release"):
            temporary_directory = tempfile.TemporaryDirectory()
            try:
                root = Path(temporary_directory.name)
                for compiler_profile in ("dev", "release"):
                    executable = paths.output(root, "build", compiler_profile) / "bin" / f"dokime{extension}"
                    executable.parent.mkdir(parents=True)
                    executable.write_bytes(compiler_profile.encode())
                output = paths.output(root, "build", profile)
                fixtures = output / "fixtures/host"
                fixtures.mkdir(parents=True)
                for name in ("dokime-rust-thermal", "plugin-obc"):
                    (fixtures / f"{name}{extension}").write_bytes(b"plugin")
                (output / "bin/plugin-obc.dokime").write_bytes(b"product bundle")
                with (
                    self.subTest(profile=profile),
                    patch.object(desktop_fixtures.configuration, "ROOT", root),
                    patch.object(desktop_fixtures.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, stderr="")) as run,
                    patch.object(desktop_fixtures.archive, "stamp"),
                ):
                    desktop_fixtures.build(profile)
                    self.assertEqual((output / "fixtures/desktop" / f"dokime{extension}").read_bytes(), profile.encode())
                    self.assertEqual((output / "fixtures/desktop/plugin-obc.dokime").read_bytes(), b"product bundle")
                    self.assertEqual(len(run.call_args_list), 1)
                    self.assertEqual(run.call_args.args[0][0], str(output / "bin" / f"dokime{extension}"))
                    (output / "bin" / f"dokime{extension}").unlink()
                    self.assertRaises(ValueError, desktop_fixtures.build, profile)
            finally:
                temporary_directory.cleanup()

    def test_host_selects_requested_rust_bytes_and_refuses_missing_profile(self) -> None:
        extension = ".exe" if os.name == "nt" else ""
        for profile in ("dev", "release"):
            temporary_directory = tempfile.TemporaryDirectory()
            try:
                root = Path(temporary_directory.name)
                names = (
                    "dokime-rust-thermal",
                    "dokime-rust-execution",
                    "dokime-rust-boundary",
                    "dokime-rust-secrets",
                    "dokime-rust-capture",
                    "dokime-rust-history",
                    "dokime-rust-plugin",
                    "dokime-host",
                    "plugin-obc",
                )
                for compiler_profile in ("debug", "release"):
                    directory = root / "target" / compiler_profile
                    directory.mkdir(parents=True)
                    for name in names:
                        (directory / f"{name}{extension}").write_bytes(compiler_profile.encode())
                    (directory / f"content_fixture{extension}").write_bytes(compiler_profile.encode())
                (root / "Cargo.toml").write_text(
                    '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n'
                )
                product = paths.output(root, "build", profile) / "bin"
                product.mkdir(parents=True)
                for name in ("dokime-host", "plugin-obc"):
                    (product / f"{name}{extension}").write_bytes(profile.encode())
                report = root / "target/generated/report/report.html"
                report.parent.mkdir(parents=True)
                report.write_bytes(b"report")
                reference = (
                    root / "target/reference-host" / paths.target() / profile / "fixture-revision/bin" / f"harmos-conformance{extension}"
                )
                reference.parent.mkdir(parents=True)
                reference.write_bytes(b"reference")
                fixture = root / "library/sdk/python/tests/fixtures/conformance/__init__.py"
                fixture.parent.mkdir(parents=True)
                fixture.write_bytes(b"source")
                metadata = {
                    "packages": [
                        {
                            "name": "harmos",
                            "source": "git+https://example.test/library#fixture-revision",
                            "manifest_path": str(root / "library/sdk/rust/Cargo.toml"),
                        }
                    ]
                }
                with (
                    self.subTest(profile=profile),
                    patch.object(host_fixtures.configuration, "ROOT", root),
                    patch.object(host_fixtures.generated, "prepare"),
                    patch.object(
                        host_fixtures.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, stdout=json.dumps(metadata))
                    ) as run,
                ):
                    host_fixtures.build(profile, "rust")
                    selected = "debug" if profile == "dev" else profile
                    output = paths.output(root, "build", profile) / "fixtures/host"
                    for name in (*names, "content_fixture"):
                        expected = profile if name in ("dokime-host", "plugin-obc") else selected
                        self.assertEqual((output / f"{name}{extension}").read_bytes(), expected.encode())
                    commands = [call.args[0] for call in run.call_args_list if call.args[0][:2] != ["cargo", "metadata"]]
                    for command in commands:
                        self.assertEqual(command[command.index("--profile") + 1], profile)
                    self.assertEqual(run.call_args_list[0].kwargs["env"]["CARGO_TARGET_DIR"], str(root / "target"))
                    (root / "target" / selected / f"dokime-rust-thermal{extension}").unlink()
                    self.assertRaises(FileNotFoundError, host_fixtures.build, profile, "rust")
            finally:
                temporary_directory.cleanup()


# === Public Functions === #


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
