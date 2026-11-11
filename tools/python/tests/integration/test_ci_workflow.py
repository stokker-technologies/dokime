# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove Task Transport, Required Gates And Compiler Cache Compatibility."""

# === Imports === #

# Standard Library Imports
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools import policy
from dokime_tools.ci import native, transport, workflow
from dokime_tools.release import registry

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class WorkflowTests(unittest.TestCase):
    def test_release_mode_exports_routing_and_target_revision(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "output"
            for mode, sha in (("tag", "a" * 40), ("prepare", ""), ("skip", "")):
                with (
                    self.subTest(mode=mode),
                    patch.dict(os.environ, {"GITHUB_OUTPUT": str(output)}, clear=True),
                    patch("dokime_tools.ci.review.github_release_mode", return_value={"mode": mode, "sha": sha}),
                ):
                    output.write_text("", encoding="utf-8")
                    self.assertEqual(workflow.main(["release-mode"]), 0)
                    self.assertEqual(output.read_text(encoding="utf-8"), f"mode={mode}\nsha={sha}\n")

    def test_nightly_report_replaces_only_its_publication_results(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            release = {"body": "| CI Run | [7](https://example.test/run) |\n"}
            environment = {
                "GITHUB_RUN_ID": "7",
                "GITHUB_REPOSITORY": "example/dokime",
                "RUN_URL": "https://example.test/run",
                "NEEDS": json.dumps({"bench": {"result": "failure"}}),
            }

            def edit(command, **_):
                release["body"] = Path(command[-1]).read_text(encoding="utf-8")

            with (
                patch.dict(os.environ, environment, clear=True),
                patch.object(workflow.policy, "root", return_value=root),
                patch.object(workflow.transport, "receive"),
                patch.object(workflow.transport, "send"),
                patch("dokime_tools.ci.review.request", return_value=release),
                patch.object(workflow.subprocess, "run", side_effect=edit) as publish,
            ):
                workflow.main(["nightly-report"])
                workflow.main(["nightly-report"])
                self.assertEqual(release["body"].count("<!-- nightly-results -->"), 1)
                self.assertEqual(publish.call_count, 2)
                release["body"] = "| CI Run | [8](https://example.test/later) |\n"
                with self.assertRaises(ValueError):
                    workflow.main(["nightly-report"])
                self.assertEqual(publish.call_count, 2)

    def test_required_rejects_failed_cancelled_and_unexpected_skips(self):
        for result in ("failure", "cancelled", "skipped"):
            environment = {"RESULTS": json.dumps({"test": {"result": result}})}
            with patch.dict(os.environ, environment, clear=True), self.assertRaises(ValueError):
                workflow.main(["required"])

    def test_required_allows_only_event_excluded_skips(self):
        results = {"build": {"result": "success"}, "package": {"result": "skipped"}}
        environment = {"RESULTS": json.dumps(results), "OPTIONAL_JOBS": '["package"]'}
        with patch.dict(os.environ, environment, clear=True):
            self.assertEqual(workflow.main(["required"]), 0)
        results["package"]["result"] = "failure"
        environment["RESULTS"] = json.dumps(results)
        with patch.dict(os.environ, environment, clear=True), self.assertRaises(ValueError):
            workflow.main(["required"])

    def test_transport_expands_producer_identity(self):
        output = {"archive": "target/ci/package-{profile}-{platform}.tar.zst", "paths": ["target/package/{profile}/{target}"]}
        environment = {"CI_OUTPUTS": json.dumps([output]), "CI_PLATFORM": "unix"}
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(transport, "target", return_value="linux-x86_64"),
            patch.object(transport.artifacts, "pack") as pack,
        ):
            transport.send("release")
        self.assertEqual(pack.call_args.args[1], Path("target/ci/package-release-unix.tar.zst"))
        self.assertEqual(pack.call_args.args[2], output["paths"])
        self.assertEqual(pack.call_args.args[3], "release")

    def test_transport_extracts_downloads_only_when_requested(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / "product.tar.zst"
            archive.touch()
            (root / "report.json").touch()
            with patch.dict(os.environ, {}, clear=True), patch.object(transport.artifacts, "extract") as extract:
                transport.receive()
                extract.assert_not_called()
            with (
                patch.dict(os.environ, {"CI_TRANSPORT": "true"}, clear=True),
                patch.object(transport.Path, "glob", return_value=[archive]),
                patch.object(transport.artifacts, "extract") as extract,
            ):
                transport.receive()
                extract.assert_called_once_with(Path.cwd(), archive)

    def test_compiler_cache_ignores_unrelated_tools_but_tracks_rust_and_crates(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in ("mise.lock", "Cargo.lock", "uv.lock", "crates/dokime-gui/pnpm-lock.yaml"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("original", encoding="utf-8")
            environment = {
                "RUNNER_OS": "Linux",
                "RUNNER_ARCH": "X64",
                "COMPILER_CACHE_BACKEND": "disk",
                "CI_CACHE_PROFILE": "release",
                "RUST_TOOLCHAIN": "1.98.0",
            }
            original = native.default_caches(root, environment)
            (root / "mise.lock").write_text("other tool changed", encoding="utf-8")
            tools = native.default_caches(root, environment)
            self.assertEqual(original["sccache"], tools["sccache"])
            self.assertNotEqual(original["mise"]["key"], tools["mise"]["key"])
            (root / "Cargo.lock").write_text("crate changed", encoding="utf-8")
            crates = native.default_caches(root, environment)["sccache"]
            self.assertNotEqual(crates["key"], original["sccache"]["key"])
            self.assertEqual(crates["restore-keys"], original["sccache"]["restore-keys"])
            rust = native.default_caches(root, environment | {"RUST_TOOLCHAIN": "1.99.0"})["sccache"]
            self.assertNotEqual(rust["restore-keys"], crates["restore-keys"])
            tests = native.default_caches(root, environment | {"CI_CACHE_PROFILE": "test"})["sccache"]
            self.assertNotEqual(tests["restore-keys"], crates["restore-keys"])

    def test_release_rehearsal_uses_moved_evidence_and_requires_green_jobs(self):
        head = "a" * 40
        merged = "b" * 40
        review = {"head": head, "number": 7}
        pull = {"number": 7, "head": {"sha": head}}
        run = {
            "id": 9,
            "head_sha": head,
            "pull_requests": [{"number": 7}],
            "display_title": "CI / Validation",
            "status": "completed",
            "conclusion": "success",
        }
        jobs = [{"name": "CI / Required", "conclusion": "success"}, {"name": registry.REHEARSAL_CHECK, "conclusion": "success"}]

        def pages(settings, endpoint, key=None):
            return [run] if "workflows/ci.yml" in endpoint else jobs

        with (
            patch.object(registry, "document", return_value=pull),
            patch.object(registry, "pages", side_effect=pages),
            patch.object(registry, "_same_tree") as tree,
        ):
            registry.rehearsed(Path.cwd(), {"provider": "github"}, review, merged)
            tree.assert_called_once_with(Path.cwd(), head, merged)
            jobs[1]["conclusion"] = "failure"
            with self.assertRaises(policy.ToolError):
                registry.rehearsed(Path.cwd(), {"provider": "github"}, review, merged)

    def test_publication_guard_restores_mutating_token_after_read_guard(self):
        observed = []
        environment = {"RELEASE_TARGET_SHA": "a" * 40, "CI_READ_TOKEN": "read", "GH_TOKEN": "release"}
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(workflow.runpy, "run_path", side_effect=lambda *args, **kwargs: observed.append(os.environ["GH_TOKEN"])),
            patch("dokime_tools.release.run.main", return_value=0),
        ):
            workflow.publication_guard()
            self.assertEqual(os.environ["GH_TOKEN"], "release")
        self.assertEqual(observed, ["read"])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
