# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prove Changed-Path Selection Preserves Test Ownership And Consumer Coverage."""

# === Imports === #

# Standard Library Imports
import io
import json
import tempfile
import tomllib
import unittest
from contextlib import nullcontext, redirect_stdout
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.ci import tasks
from dokime_tools.test import focused
from dokime_tools.test import run as archived

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ChangedPathsTests(unittest.TestCase):
    def test_story_edits_and_source_edits_keep_their_dependency_boundaries(self):
        with tempfile.TemporaryDirectory() as temporary:
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
            self.assertEqual(commands, [["--locked", "--package", "engine", "--features", "engine/testing", "--test", "integration"]])
            commands = focused.selections(root, metadata, ["engine/tests/support/fixture.rs"], None)
            self.assertEqual([command[2] for command in commands], ["engine"])
            self.assertIn("system", commands[0])

            # A Source Edit Reaches Renamed And Transitive Consumers, Excluding Unrelated Packages.
            commands = focused.selections(root, metadata, ["messages/src/lib.rs"], "integration")
            self.assertEqual(len(commands), 1)
            self.assertEqual(
                [commands[0][index + 1] for index, value in enumerate(commands[0][:-1]) if value == "--package"],
                ["client", "engine", "messages"],
            )
            self.assertTrue(all(command[-2:] == ["--test", "integration"] for command in commands))
            self.assertIn("client/testing,engine/testing,messages/testing", commands[0])
            self.assertEqual(focused.selections(root, metadata, ["messages"], "integration"), commands)

            # A Concurrent Source Edit Broadens An Already Selected Story Back To All Its Tests.
            commands = focused.selections(root, metadata, ["engine/tests/integration/save.rs", "messages/src/lib.rs"], None)
            self.assertEqual(len(commands), 1)
            self.assertEqual(commands[0][-1], "--tests")
            self.assertEqual(focused.selections(root, metadata, ["engine/tests/integration/deleted.rs"], None)[0][-1], "integration")

            # Workspace Changes Cover Every Package; Unknown Languages And Empty Selections Fail Explicitly.
            commands = focused.selections(root, metadata, ["Cargo.lock"], "unit")
            self.assertEqual(len(commands), 1)
            self.assertEqual(
                {commands[0][index + 1] for index, value in enumerate(commands[0][:-1]) if value == "--package"},
                {"messages", "engine", "client", "independent"},
            )
            commands = focused.selections(
                root, metadata, ["engine/tests/integration/save.rs"], None, ["engine/runtime", "independent/unused"]
            )
            self.assertIn("engine/runtime,engine/testing", commands[0])
            for paths, layer in (
                (["docs/guide.md"], None),
                (["../outside.rs"], None),
                (["engine/benches/large.rs"], None),
                (["engine/tests/integration/save.rs"], "system"),
            ):
                with self.subTest(paths=paths), self.assertRaises(ValueError):
                    focused.selections(root, metadata, paths, layer)

    def test_focused_dispatch_runs_the_story_filter_and_plan_never_compiles(self):
        with patch.object(tasks.transport, "receive"), patch.object(tasks.cargo, "focused", return_value=7) as execute:
            for selector in ("--story", "--select"):
                self.assertEqual(tasks.main(["test", "--package", "example", "--target", "integration", selector, "save::"]), 7)
                self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--test", "integration", "save::"])
            execute.reset_mock()
            with redirect_stdout(io.StringIO()) as output:
                self.assertEqual(tasks.main(["test", "--package", "example", "--story", "save::", "--no-run", "--timings", "--plan"]), 0)
            execute.assert_not_called()
            self.assertIn("--test integration --no-run save::", output.getvalue())
            self.assertEqual(tasks.main(["test", "--package", "example"]), 7)
            self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--tests"])
            self.assertEqual(tasks.main(["test", "--package", "example", "--layer", "system", "--story", "save::"]), 7)
            self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--test", "system", "save::"])
            execute.reset_mock()
            with patch.object(archived, "test", return_value=7) as archive:
                self.assertEqual(
                    tasks.main(["test", "--package", "example", "--target", "integration", "--artifact", "tests", "--exact", "save"]), 7
                )
                self.assertEqual(archive.call_args.args[-1], ["-E", "binary_id(=example::integration)", "--", "--exact", "save"])
            execute.assert_not_called()

    def test_filterset_reaches_source_and_archived_story_selection(self):
        expression = "not (test(native_one) or test(native_two))"
        arguments = ["test", "--package", "example", "--target", "system", "--filterset", expression]
        with patch.object(tasks.transport, "receive"), patch.object(tasks.cargo, "focused", return_value=7) as execute:
            self.assertEqual(tasks.main(arguments), 7)
            self.assertEqual(execute.call_args.args[-1], ["--locked", "--package", "example", "--test", "system", "-E", expression])
            with patch.object(archived.archive, "install_wheels"), patch.object(archived, "test", return_value=9) as run:
                self.assertEqual(tasks.main([*arguments, "--artifact", "tests"]), 9)
                self.assertEqual(run.call_args.args[-1], ["-E", "binary_id(=example::system)", "-E", expression])

    def test_native_runner_keeps_deadlines_exact_selection_and_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "tools").mkdir()
            (root / "tools/project.toml").write_text(
                "[conventions.testing.deadlines]\nunit=120\nintegration=300\nsystem=600\nacceptance=1800\n"
            )
            observed = []

            def execute(command, **kwargs):
                if command[1] == "metadata":
                    return SimpleNamespace(
                        stdout=json.dumps({"packages": [{"name": "example", "targets": [{"kind": ["test"], "name": "integration"}]}]})
                    )
                configuration = Path(command[command.index("--config-file") + 1])
                profile = tomllib.loads(configuration.read_text())["profile"]["project"]
                observed.append((command, kwargs, profile))
                junit = configuration.parent / "store/project/junit.xml"
                junit.parent.mkdir(parents=True)
                junit.write_text('<testsuite><testcase name="save"/></testsuite>')
                return SimpleNamespace(returncode=7)

            with (
                patch.object(focused.desktop_bench, "credentials", side_effect=lambda environment: nullcontext(environment)) as credentials,
                patch.object(focused.subprocess, "run", side_effect=execute),
                patch.object(
                    focused.cargo, "compiler_environment", return_value={"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER": "custom-linker"}
                ) as compiler,
            ):
                self.assertEqual(focused.run(root, ["--package", "example", "--test", "integration", "save", "--", "--exact"]), 7)
                self.assertEqual(focused.run(root, ["--package", "example", "--tests", "--no-run"]), 7)
            self.assertEqual(credentials.call_count, 2)
            self.assertEqual(compiler.call_count, 2)
            compiler.assert_called_with("test")
            command, options, profile = observed[0]
            self.assertEqual(command[-3:], ["--", "--exact", "save"])
            self.assertEqual(command[command.index("--no-tests") + 1], "fail")
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


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
