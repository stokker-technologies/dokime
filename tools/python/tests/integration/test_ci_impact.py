# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove Impact Selection, Complete Comparisons and Required Results Deterministically."""

# === Imports === #

# Standard Library Imports
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

# isort: split

# Third Party Imports
# isort: split

# Local Imports
from dokime_tools.ci import impact, required
from dokime_tools.test import policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ImpactTests(unittest.TestCase):
    def test_path_order_and_duplicates_preserve_the_plan(self) -> None:
        cases = (
            ["README.md"],
            ["sdk/rust/README.md"],
            ["docs/assets/navigation.svg"],
            ["openspec/changes/example/tasks.md", "openspec/config.yaml"],
            ["sites/docs/pnpm-lock.yaml"],
            ["sdk/python/src/dokime_sdk/client.py"],
            ["sdk/python/uv.lock"],
            ["sdk/csharp/src/Dokime/Client.cs"],
            ["sdk/csharp/src/Dokime/packages.lock.json"],
            ["sdk/cpp/include/dokime/client.hpp"],
            ["sdk/cpp/vcpkg.json"],
            ["crates/dokime-gui/src/routes/index.tsx"],
            ["crates/dokime-gui/pnpm-lock.yaml"],
            ["crates/dokime-runtime/src/lib.rs"],
            ["sdk/rust/src/lib.rs"],
            ["sdk/rust/tests/consumer/Cargo.lock"],
            ["sdk/rust/tests/fixtures/capture.rs"],
            ["crates/dokime-runtime/tests/system/fixtures.rs"],
            ["sdk/tests/fixtures/csharp/SystemPlugin/Plugin.cs"],
            ["sdk/tests/fixtures/cpp/main.cpp"],
            ["proto/dokime/client.proto"],
            ["protocol/desktop.toml"],
            ["target-descriptors.bin"],
            ["Cargo.lock"],
            ["uv.lock", "mise.lock"],
            ["rust-toolchain.toml"],
            ["mise.toml"],
            ["tools/python/src/dokime_tools/ci/tasks.py"],
            [".github/workflows/validate-test.yml"],
            [".github/actions/setup/action.yml"],
            ["new-component/input.dat"],
            ["docs/index.md", "sdk/cpp/src/client.cpp"],
            ["sdk/csharp/src/Dokime/Client.cs", "sdk/python/src/dokime_sdk/client.py"],
        )
        for paths in cases:
            with self.subTest(paths=paths):
                classes = impact.classify(paths)
                result = impact.plan(classes, "review")
                reversed_result = impact.plan(impact.classify([*reversed(paths), *paths]), "review")
                self.assertEqual(result, reversed_result)

    def test_shared_frontend_inputs_select_gui_and_docs_without_other_languages(self) -> None:
        paths = (
            "packages/plots/src/CartesianPlot.tsx",
            "packages/document/package.json",
            "packages/report/report.html",
            "package.json",
            "pnpm-workspace.yaml",
            "pnpm-lock.yaml",
            "biome.json",
            "tsconfig.json",
            "tools/config/vite.base.ts",
        )
        for path in paths:
            with self.subTest(path=path):
                self.assertEqual(impact.classify([path]), frozenset({"gui", "docs"}))
                result = impact.plan(impact.classify([path]), "review")
                self.assertTrue(result["build_docs"])
                pushed = impact.plan(impact.classify([path]), "push")
                self.assertFalse(pushed["test"])

    def test_package_jobs_keep_each_platform_profile_and_artifact_contract(self) -> None:
        for event in ("review", "nightly"):
            rows = impact.plan(impact.classify(["packages/plots/src/axis.ts"]), event)["test_matrix"]["include"]
            packages = [row for row in rows if row["work"].startswith("Packages / ")]
            for name in ("plots", "document", "report"):
                selected = [row for row in packages if row["work"] == f"Packages / {name.title()}"]
                self.assertEqual({row["platform"] for row in selected}, {"unix", "windows"})
                for row in selected:
                    self.assertEqual(row["command"], f"test:typescript:integration -- --prepared --project packages/{name} --select tests")
                    self.assertEqual(json.loads(row["bootstrap"]), [])
                    for profile in ("dev", "release"):
                        archives = json.loads(row["archives"].replace("{profile}", profile).replace("{platform}", row["platform"]))
                        self.assertEqual(archives, [f"target/ci/build-clients-{profile}-{row['platform']}.tar.zst"])

    def test_event_eligibility_and_complete_validation(self) -> None:
        for classes in (impact.classify(["README.md"]), impact.classify(["unknown"])):
            for event in ("full",) if classes == frozenset({"docs"}) else ("release", "nightly", "full"):
                with self.subTest(event=event, classes=classes):
                    self.assertEqual(set(impact.plan(classes, event)["selected"]), set(impact.JOBS))
            for event in ("closed", "draft"):
                self.assertEqual(impact.plan(classes, event)["selected"], [])
        self.assertEqual(impact.plan(impact.classify(["README.md"]), "push")["selected"], ["build_docs"])
        for paths in (None, [], ["../outside"], ["/absolute"], ["crates\\runtime\\lib.rs"]):
            self.assertEqual(impact.classify(paths), frozenset({"all"}))

    def test_documentation_only_selects_no_product_jobs_for_each_event(self) -> None:
        documentation = (
            "README.md",
            "sdk/rust/README.md",
            "sdk/python/README.md",
            "sdk/csharp/README.md",
            "sdk/cpp/README.md",
            "openspec/config.yaml",
            "sites/docs/package.json",
            "sites/docs/pnpm-lock.yaml",
            "sites/other/index.html",
            "assets/screenshots/desktop.png",
            "tools/python/src/dokime_tools/docs/image.py",
            "tools/python/src/dokime_tools/ci/impact.py",
        )
        cases = [[path] for path in documentation] + [list(documentation)]
        for event in ("review", "push", "release", "nightly"):
            for paths in cases:
                with self.subTest(event=event, paths=paths):
                    result = impact.plan(impact.classify(paths), event)
                    self.assertEqual(result["selected"], ["build_docs"])
                    mixed = impact.plan(impact.classify([*paths, "crates/dokime-runtime/src/lib.rs"]), event)
                    self.assertTrue(mixed["build_product"])
                    self.assertTrue(mixed["check"])
                    self.assertEqual(mixed["test"], event != "push")

    def test_event_metadata_selects_full_release_and_safe_fallback(self) -> None:
        cases = (
            ("push", {}, "push"),
            ("workflow_dispatch", {}, "full"),
            ("pull_request", {}, "review"),
            ("pull_request", {"pull_request": None}, "review"),
            ("pull_request", {"pull_request": {"head": None}}, "review"),
            ("pull_request", {"pull_request": {"draft": True}}, "draft"),
            ("pull_request", {"pull_request": {"head": {"ref": "release/v1.2.3"}}}, "release"),
            ("pull_request", {"action": "closed", "pull_request": {"draft": True}}, "closed"),
        )
        for name, event, expected in cases:
            with self.subTest(name=name, event=event):
                self.assertEqual(impact._event(name, event), expected)

    def test_matrices_keep_platforms_and_each_artifact_producer(self) -> None:
        for path in ("sdk/python/uv.lock", "sdk/csharp/mise.toml", "sdk/cpp/vcpkg.json", "crates/dokime-gui/package.json", "Cargo.lock"):
            result = impact.plan(impact.classify([path]), "review")
            rows = result["test_matrix"]["include"]
            for language in {row["language"] for row in rows} - {"typescript"}:
                platforms = {row["platform"] for row in rows if row["language"] == language}
                self.assertEqual(platforms, {"unix", "windows"})
            for row in rows:
                for archive in json.loads(row["archives"]):
                    if "build-product-python-" in archive:
                        self.assertTrue(result["build_python"])
                    if "build-cli-" in archive or "build-clients-" in archive:
                        self.assertTrue(result["build_product"])
                    if "build-tests-" in archive:
                        self.assertTrue(result["build_tests"])

    def test_diff_decoder_keeps_deletions_and_both_rename_sides(self) -> None:
        paths = impact.diff_paths(b"R100\0sdk/python/old.py\0sdk/cpp/new.cpp\0D\0README.md\0A\0file with space.py\0")
        self.assertEqual(paths, ["sdk/python/old.py", "sdk/cpp/new.cpp", "README.md", "file with space.py"])
        for data in (b"", b"M\0README.md", b"R100\0old\0", b"X-unknown\0file\0", b"D\0\0", b"M\0\xff\0"):
            with self.subTest(data=data), self.assertRaises((ValueError, UnicodeError)):
                impact.diff_paths(data)

    def test_comparison_adapter_uses_merge_base_or_exact_push_pair(self) -> None:
        base, head, ancestor = "a" * 40, "b" * 40, "c" * 40
        for name, event, expected_base in (
            ("pull_request", {"pull_request": {"base": {"sha": base}, "head": {"sha": head}}}, ancestor),
            ("push", {"before": base, "after": head}, base),
        ):

            def git(directory: Path, *arguments: str) -> bytes:
                if arguments[0] == "merge-base":
                    return ancestor.encode() + b"\n"
                if arguments[0] == "diff":
                    return b"D\0README.md\0R050\0sdk/python/old.py\0sdk/cpp/new.cpp\0"
                return b""

            with self.subTest(name=name), patch.object(impact, "_git", side_effect=git) as command:
                paths = impact.changed_paths(event, name, Path("checkout"))
            self.assertEqual(paths, ["README.md", "sdk/python/old.py", "sdk/cpp/new.cpp"])
            self.assertEqual(command.call_args.args[-3:], (expected_base, head, "--"))
            commands = [call.args[1] for call in command.call_args_list]
            self.assertEqual(commands.count("merge-base"), int(name == "pull_request"))

    def test_comparison_failures_select_every_eligible_job(self) -> None:
        event = {"before": "a" * 40, "after": "b" * 40}
        for failure in (OSError("unavailable"), subprocess.CalledProcessError(1, ["git"]), ValueError("truncated")):
            with patch.object(impact, "_git", side_effect=failure):
                self.assertIsNone(impact.changed_paths(event, "push", Path("checkout")))
        for event in ({}, {"before": "0" * 40, "after": "b" * 40}, {"before": "missing", "after": "b" * 40}):
            with patch.object(impact, "_git") as command:
                self.assertIsNone(impact.changed_paths(event, "push", Path("checkout")))
                command.assert_not_called()

    def test_cli_outputs_are_json_and_missing_event_data_falls_back(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            output = Path(temporary.name) / "output"
            environment = {"GITHUB_EVENT_NAME": "push", "GITHUB_OUTPUT": str(output)}
            with patch.dict(os.environ, environment, clear=True):
                self.assertEqual(impact.main([]), 0)
            outputs = {key: json.loads(value) for key, value in (line.split("=", 1) for line in output.read_text().splitlines())}
            self.assertEqual(outputs, impact.plan(impact.classify(None), "push"))
        finally:
            temporary.cleanup()

    def test_release_review_still_compares_paths_before_selecting_product_work(self) -> None:
        with (
            patch.dict(os.environ, {"GITHUB_EVENT_NAME": "pull_request"}, clear=True),
            patch.object(impact, "_event", return_value="release"),
            patch.object(impact, "changed_paths", return_value=["README.md"]) as compare,
        ):
            self.assertEqual(impact.main([]), 0)
            compare.assert_called_once_with({}, "pull_request", Path.cwd())

    def test_nightly_uses_the_product_tag_base_for_docs_and_subsequent_product_changes(self) -> None:
        base, head = "a" * 40, "b" * 40
        environment = {"GITHUB_SHA": head}
        for paths, product in ((["docs/guide.md"], False), (["docs/guide.md", "sdk/rust/src/lib.rs"], True)):
            with (
                self.subTest(paths=paths),
                patch.dict(os.environ, environment, clear=True),
                patch.object(impact, "changed_paths", return_value=paths) as compare,
                patch.object(impact, "plan", wraps=impact.plan) as plan,
            ):
                self.assertEqual(impact.main(["--event", "nightly", "--base", base]), 0)
                compare.assert_called_once_with({"before": base, "after": head}, "push", Path.cwd())
                plan.assert_called_once_with(impact.classify(paths), "nightly")
            selected = impact.plan(impact.classify(paths), "nightly")
            self.assertEqual(selected["product"], product)
            self.assertTrue(selected["build_docs"])

    def test_required_result_table_rejects_unplanned_or_missing_proof(self) -> None:
        plan = impact.plan(impact.classify(["sdk/cpp/src/plugin.cpp"]), "review")
        baseline: dict[str, Any] = {name: {"result": "success" if plan[name] else "skipped"} for name in impact.JOBS}
        baseline["impact"] = {"result": "success", "outputs": {key: json.dumps(plan[key]) for key in ("selected", "skipped")}}
        self.assertEqual(required.failures(baseline), [])
        for name, selected in (("check", True), ("package", False)):
            for result in ("success", "skipped", "failure", "cancelled", "", None):
                results = {**baseline, name: {"result": result}}
                expected = [] if result == "success" or result == "skipped" and not selected else [name]
                with self.subTest(name=name, result=result):
                    self.assertEqual(required.failures(results), expected)
            results = dict(baseline)
            del results[name]
            self.assertEqual(required.failures(results), [name])
        for planner in ({}, {"result": "failure"}, {"result": "cancelled"}, {"result": "skipped"}, {"result": "success", "outputs": {}}):
            self.assertEqual(required.failures({**baseline, "impact": planner}), ["impact"])
        for selected, skipped in (([], []), (["check"], list(impact.JOBS)), (["unknown"], []), (None, [])):
            planner = {"result": "success", "outputs": {"selected": json.dumps(selected), "skipped": json.dumps(skipped)}}
            self.assertEqual(required.failures({**baseline, "impact": planner}), ["impact"])

    def test_expected_lanes_are_independent_of_task_names(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            directory = Path(temporary.name)
            (directory / "tools").mkdir()
            path = directory / "tools/project.toml"
            path.write_text('[conventions.testing]\nexpected_lanes = ["rust:unit", "python:system"]\n')
            self.assertEqual(policy.lanes(directory), [("unit", "rust"), ("system", "python")])
            for declaration in ('["rust:unit", "rust:unit"]', '["unknown:unit"]', '"rust:unit"'):
                path.write_text("[conventions.testing]\nexpected_lanes = " + declaration)
                with self.assertRaises(policy.TestError):
                    policy.lanes(directory)
        finally:
            temporary.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
