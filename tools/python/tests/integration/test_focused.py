# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Prove Changed-Path Selection Preserves Test Ownership and Consumer Coverage."

# === Imports === #

# Standard Library Imports
import io
import json
import tempfile
import tomllib
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
import dokime_tools.test
import dokime_tools.test.archive
import dokime_tools.test.credentials
import dokime_tools.test.scratch
from dokime_tools.ci import tasks
from dokime_tools.test import focused, lanes

from .environment import Resource

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ChangedPathsTests(unittest.TestCase):
    def test_story_edits_and_source_edits_keep_their_dependency_boundaries(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary).resolve()
            packages = []
            for name, dependency in (("messages", None), ("engine", "messages"), ("client", "engine"), ("independent", None)):
                package = root / name
                packages.append(
                    {
                        "name": name,
                        "manifest_path": str(package / "Cargo.toml"),
                        "dependencies": [{"name": "renamed", "path": str(root / dependency)}] if dependency else [],
                        "targets": [
                            {"name": name, "kind": ["lib"], "test": True, "src_path": str(package / "src/lib.rs")},
                            {
                                "name": "integration",
                                "kind": ["test"],
                                "test": True,
                                "src_path": str(package / "tests/integration/main.rs"),
                                "required-features": ["testing"],
                            },
                            {"name": "system", "kind": ["test"], "test": True, "src_path": str(package / "tests/system/main.rs")},
                        ],
                    }
                )
            metadata = {"packages": packages}

            # A Story Changes Only Its Harness; Shared Fixtures Select All Owning Harnesses.
            commands = focused.selections(root, metadata, ["engine/tests/integration/save.rs"], None)
            self.assertEqual(commands, [["--locked", "--package", "engine", "--test", "integration"]])
            commands = focused.selections(root, metadata, ["engine/tests/support/fixture.rs"], None)
            self.assertEqual([command[2] for command in commands], ["engine"])
            self.assertIn("system", commands[0])

            # A Source Edit Reaches Renamed and Transitive Consumers, Excluding Unrelated Packages.
            commands = focused.selections(root, metadata, ["messages/src/lib.rs"], "integration")
            self.assertEqual(len(commands), 1)
            self.assertEqual(
                [commands[0][index + 1] for index, value in enumerate(commands[0][:-1]) if value == "--package"],
                ["client", "engine", "messages"],
            )
            self.assertTrue(all(command[-2:] == ["--test", "integration"] for command in commands))
            self.assertEqual(focused.selections(root, metadata, ["messages"], "integration"), commands)

            # A Concurrent Source Edit Broadens an Already Selected Story Back to All Its Tests.
            commands = focused.selections(root, metadata, ["engine/tests/integration/save.rs", "messages/src/lib.rs"], None)
            self.assertEqual(len(commands), 1)
            self.assertEqual(commands[0][-1], "--tests")
            self.assertEqual(focused.selections(root, metadata, ["engine/tests/integration/deleted.rs"], None)[0][-1], "integration")

            # Workspace Changes Cover Every Package; Unknown Languages and Empty Selections Fail Explicitly.
            commands = focused.selections(root, metadata, ["Cargo.lock"], "unit")
            self.assertEqual(len(commands), 1)
            self.assertEqual(
                {commands[0][index + 1] for index, value in enumerate(commands[0][:-1]) if value == "--package"},
                {"messages", "engine", "client", "independent"},
            )
            for paths, layer in (
                (["docs/guide.md"], None),
                (["../outside.rs"], None),
                (["engine/benches/large.rs"], None),
                (["engine/tests/integration/save.rs"], "system"),
            ):
                with self.subTest(paths=paths), self.assertRaises(ValueError):
                    focused.selections(root, metadata, paths, layer)
        finally:
            temporary_directory.cleanup()

    def test_changed_paths_and_direct_targets_share_feature_inference(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary_directory.name).resolve()
            (root / "tools").mkdir()
            (root / "tools/project.toml").write_text("", encoding="utf-8")
            targets = [
                {"name": "engine", "kind": ["lib"], "test": True, "src_path": str(root / "engine/src/lib.rs")},
                {
                    "name": "integration",
                    "kind": ["test"],
                    "test": True,
                    "src_path": str(root / "engine/tests/integration/main.rs"),
                    "required-features": ["testing"],
                },
                {
                    "name": "system",
                    "kind": ["test"],
                    "test": True,
                    "src_path": str(root / "engine/tests/system/main.rs"),
                    "required-features": ["system-testing"],
                },
                {"name": "engine-tool", "kind": ["bin"], "test": True, "required-features": ["command"]},
                {"name": "engine-example", "kind": ["example"], "test": True, "required-features": ["examples"]},
            ]
            metadata = {
                "packages": [
                    {"name": "engine", "manifest_path": str(root / "engine/Cargo.toml"), "dependencies": [], "targets": targets},
                    {
                        "name": "independent",
                        "manifest_path": str(root / "independent/Cargo.toml"),
                        "dependencies": [],
                        "targets": [{"name": "integration", "kind": ["test"], "test": True, "required-features": ["unused"]}],
                    },
                ]
            }
            cases = [
                (["--test", "integration"], "engine/testing"),
                (["--tests"], "engine/command,engine/system-testing,engine/testing"),
                (["--lib"], None),
                (["--bin", "engine-tool"], "engine/command"),
                (["--bin", "engine-tool", "--test", "integration"], "engine/command,engine/testing"),
            ]
            for selectors, features in cases:
                output = io.StringIO()
                with (
                    self.subTest(selectors=selectors),
                    patch.object(focused.subprocess, "run", return_value=SimpleNamespace(stdout=json.dumps(metadata))) as execute,
                    redirect_stdout(output),
                ):
                    status = focused.run(root, ["--locked", "--package", "engine", *selectors, "--no-run"], plan=True)
                self.assertEqual(status, 0)
                command = output.getvalue().split()
                self.assertEqual(command.count("--features"), int(features is not None))
                if features is not None:
                    self.assertEqual(command[command.index("--features") + 1], features)
                self.assertTrue(all(call.args[0][1] == "metadata" for call in execute.call_args_list))

            for path, layer, features in (
                ("engine/tests/integration/main.rs", None, "engine/testing"),
                ("engine/src/lib.rs", "integration", "engine/testing"),
                ("engine/src/lib.rs", None, "engine/command,engine/system-testing,engine/testing"),
            ):
                output = io.StringIO()
                with (
                    self.subTest(path=path, layer=layer),
                    patch.object(focused.subprocess, "run", return_value=SimpleNamespace(stdout=json.dumps(metadata))) as execute,
                    patch.object(focused.cargo, "focused") as compile,
                    redirect_stdout(output),
                ):
                    status = focused.changed(root, [path], layer, plan=True, no_run=True, timings=False)
                self.assertEqual(status, 0)
                compile.assert_not_called()
                command = output.getvalue().split()
                self.assertIn("--no-run", command)
                self.assertEqual(command.count("--features"), 1)
                self.assertEqual(command[command.index("--features") + 1], features)
                self.assertTrue(all(call.args[0][1] == "metadata" for call in execute.call_args_list))
        finally:
            temporary_directory.cleanup()

    def test_focused_dispatch_runs_the_story_filter_and_plan_never_compiles(self) -> None:
        output = io.StringIO()
        with (
            patch.object(tasks.transport, "receive"),
            patch.object(tasks.cargo, "focused", return_value=7) as execute,
            patch.object(lanes, "run", return_value=7) as archive,
            redirect_stdout(output),
        ):
            for selector in ("--story", "--select"):
                self.assertEqual(tasks.main(["test", "--package", "example", "--target", "integration", selector, "save::"]), 7)
                self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--test", "integration", "save::"])
            execute.reset_mock()
            self.assertEqual(tasks.main(["test", "--package", "example", "--story", "save::", "--no-run", "--timings", "--plan"]), 0)
            execute.assert_not_called()
            self.assertEqual(tasks.main(["test", "--package", "example"]), 7)
            self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--tests"])
            self.assertEqual(tasks.main(["test", "--package", "example", "--layer", "system", "--story", "save::"]), 7)
            self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--test", "system", "save::"])
            execute.reset_mock()
            self.assertEqual(
                tasks.main(["test", "--package", "example", "--target", "integration", "--artifact", "tests", "--exact", "save"]), 7
            )
            self.assertEqual(archive.call_args.args[-1], ["-E", "binary_id(=example::integration)", "--", "--exact", "save"])
            execute.assert_not_called()

    def test_changed_paths_reject_filtersets_before_launching_cargo(self) -> None:
        output = io.StringIO()
        with patch.object(focused, "changed") as execute, redirect_stderr(output), self.assertRaises(SystemExit) as failure:
            tasks.main(["test", "--changed", "Cargo.toml", "--filterset", "test(record)"])
        self.assertEqual(failure.exception.code, 2)
        self.assertIn("Remove Explicit Package, Target, Feature and Filter Selectors", output.getvalue())
        execute.assert_not_called()

    def test_filterset_reaches_source_and_archived_story_selection(self) -> None:
        expression = "not (test(native_one) or test(native_two))"
        arguments = ["test", "--package", "example", "--target", "system", "--filterset", expression]
        with (
            patch.object(tasks.transport, "receive"),
            patch.object(tasks.cargo, "focused", return_value=7) as execute,
            patch.object(dokime_tools.test.archive, "install_wheels"),
            patch.object(lanes, "run", return_value=9) as run,
        ):
            self.assertEqual(tasks.main(arguments), 7)
            self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--test", "system", "-E", expression])
            self.assertEqual(tasks.main([*arguments, "--artifact", "tests"]), 9)
            self.assertEqual(run.call_args.args[-1], ["-E", "binary_id(=example::system)", "-E", expression])

    def test_native_runner_keeps_deadlines_exact_selection_and_failure(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            (root / "tools").mkdir()
            (root / "tools/project.toml").write_text(
                "[conventions.testing.deadlines]\nunit=120\nintegration=300\nsystem=600\nacceptance=1800\n"
            )
            observed = []

            def execute(command: list[str], **kwargs: Any) -> SimpleNamespace:
                if command[1] == "metadata":
                    return SimpleNamespace(
                        stdout=json.dumps(
                            {
                                "packages": [
                                    {
                                        "name": "example",
                                        "targets": [
                                            {
                                                "kind": ["test"],
                                                "test": True,
                                                "name": "integration",
                                                "required-features": ["testing"],
                                            }
                                        ],
                                    }
                                ]
                            }
                        )
                    )
                configuration = Path(command[command.index("--config-file") + 1])
                profile = tomllib.loads(configuration.read_text())["profile"]["project"]
                observed.append((command, kwargs, profile))
                junit = configuration.parent / "store/project/junit.xml"
                junit.parent.mkdir(parents=True)
                junit.write_text('<testsuite><testcase name="save"/></testsuite>')
                return SimpleNamespace(returncode=7)

            with (
                patch.object(
                    dokime_tools.test.credentials, "Store", side_effect=lambda environment: Resource(environment)
                ) as store_factory,
                patch.object(focused.subprocess, "run", side_effect=execute),
                patch.object(
                    focused.cargo, "compiler_environment", return_value={"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER": "custom-linker"}
                ) as compiler,
            ):
                self.assertEqual(focused.run(root, ["--package", "example", "--test", "integration", "save", "--", "--exact"]), 7)
                self.assertEqual(focused.run(root, ["--package", "example", "--tests", "--no-run"]), 7)
            self.assertEqual(store_factory.call_count, 2)
            self.assertEqual(compiler.call_count, 2)
            compiler.assert_called_with("test")
            for command, _, _ in observed:
                self.assertEqual(command.count("--features"), 1)
                self.assertEqual(command[command.index("--features") + 1], "example/testing")
            command, options, profile = observed[0]
            self.assertLess(command.index("--features"), command.index("--"))
            self.assertEqual(command[-3:], ["--", "--exact", "save"])
            self.assertEqual(command[command.index("--no-tests") + 1], "fail")
            self.assertNotIn("--no-tests", observed[1][0])
            self.assertEqual(options["env"]["CARGO_BUILD_WARNINGS"], "deny")
            self.assertEqual(options["env"]["CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER"], "custom-linker")
            self.assertFalse(profile["fail-fast"])
            guards = profile["overrides"]
            self.assertTrue(
                any(
                    "integration" not in guard["filter"] and "kind(test)" in guard["filter"] and guard["slow-timeout"]["period"] == "300s"
                    for guard in guards
                )
            )
            self.assertEqual((root / "target/test-reports/focused/junit.xml").read_text(), '<testsuite><testcase name="save"/></testsuite>')
        finally:
            temporary_directory.cleanup()

    def test_native_report_survives_cleanup_failure(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary_directory.name)
            (root / "tools").mkdir()
            (root / "tools/project.toml").write_text("", encoding="utf-8")
            resource = Resource({})
            native = '<testsuite><testcase name="save"><failure/></testcase></testsuite>'

            def execute(command: list[str], **options: Any) -> SimpleNamespace:
                if command[1] == "metadata":
                    return SimpleNamespace(stdout=json.dumps({"packages": []}))
                configuration = Path(command[command.index("--config-file") + 1])
                junit = configuration.parent / "store/project/junit.xml"
                junit.parent.mkdir(parents=True)
                junit.write_text(native, encoding="utf-8")
                return SimpleNamespace(returncode=1)

            with (
                patch.object(dokime_tools.test.scratch, "Root", return_value=resource),
                patch.object(resource, "close", side_effect=RuntimeError("cleanup failed")) as close,
                patch.object(focused.subprocess, "run", side_effect=execute),
            ):
                self.assertRaises(RuntimeError, focused.run, root, ["--package", "example", "--lib"])
                close.assert_called_once_with()
            self.assertEqual((root / "target/test-reports/focused/junit.xml").read_text(encoding="utf-8"), native)
        finally:
            temporary_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
