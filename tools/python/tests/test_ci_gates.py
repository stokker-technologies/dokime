# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Prove Development Selection And Release Publication Boundaries."""

# === Imports === #

# Standard Library Imports
import copy
import hashlib
import itertools
import json
import os
import runpy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.ci import impact, simulate
from dokime_tools.release import edge, pins, policy, registry

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[3]

# === Types === #

# === Classes === #


class ImpactTests(unittest.TestCase):
    def test_dev_and_review(self):
        plan = simulate.PLAN["impact"]
        full = impact.select(plan, {"Cargo.toml"})
        development = impact.scope(plan, full, "package")
        self.assertTrue({"package:rust", "package:python", "build:rust:tests"} <= development)
        self.assertFalse(any(job.startswith(("test:", "release:", "bench:", "prepare:")) for job in development))
        self.assertTrue(any(job.startswith("test:") for job in impact.scope(plan, full)))
        for job in development:
            self.assertTrue(set(plan["dependencies"][job]) <= development)
        action = runpy.run_path(str(ROOT / ".github/actions/plan-impact/run.py"))
        self.assertEqual(action["scope"](plan, action["select"](plan, {"Cargo.toml"}), "package"), development)
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        authored = next(line.removeprefix("        plan: ") for line in workflow.splitlines() if line.startswith("        plan: "))
        hosted_plan = json.loads(json.loads(authored))
        self.assertEqual(impact.scope(hosted_plan, impact.select(hosted_plan, {"Cargo.toml"}), "package"), development)
        print("Dev Impact:", ", ".join(sorted(development)))
        print("PR Impact:", ", ".join(sorted(full)))

    def test_guidance_scope(self):
        plan = simulate.PLAN["impact"]
        selected = impact.select(plan, {"CONTRIBUTING.md"})
        self.assertEqual(impact.scope(plan, selected, "package"), set())

    def test_conservative_scope(self):
        plan = {"dependencies": {"package:rust": ["producer"], "producer": []}, "rules": []}
        self.assertEqual(impact.close(plan["dependencies"], {"unknown"}), set(plan["dependencies"]))
        self.assertEqual(impact.scope(plan, {"package:rust"}, "package"), {"package:rust", "producer"})
        with self.assertRaises(ValueError):
            impact.scope(plan, set(), "typo")


class ReleaseChecksTests(unittest.TestCase):
    def setUp(self):
        self.guard = runpy.run_path(str(ROOT / ".github/actions/review-release/checks.py"))
        self.sha = "a" * 40
        self.ci_run = {
            "id": 7,
            "check_suite_id": 9,
            "head_sha": self.sha,
            "head_branch": "dev",
            "event": "push",
            "status": "completed",
            "conclusion": "success",
        }
        self.gate = {
            "id": 12,
            "name": "CI / Required",
            "app": {"slug": "github-actions", "id": 8},
            "check_suite": {"id": 9},
            "status": "completed",
            "conclusion": "success",
        }
        self.runs = [self.ci_run]
        self.checks = [self.gate]
        self.rules = []
        self.statuses = []

    def pages(self, endpoint, key=None):
        if endpoint.startswith("actions/"):
            return self.runs
        if "/check-runs?" in endpoint:
            return self.checks
        if endpoint == "rules/branches/main":
            return self.rules
        if endpoint.endswith("/statuses"):
            return self.statuses
        raise AssertionError(endpoint)

    def test_exact_commit(self):
        self.assertTrue(self.guard["green"](self.sha, self.pages))
        self.ci_run["head_sha"] = "b" * 40
        self.assertFalse(self.guard["green"](self.sha, self.pages))
        self.ci_run["head_sha"] = self.sha
        self.gate["check_suite"]["id"] += 1
        self.assertFalse(self.guard["green"](self.sha, self.pages))

    def test_latest_failure(self):
        self.runs.append({**self.ci_run, "id": self.ci_run["id"] + 1, "conclusion": "failure"})
        with self.assertRaises(ValueError):
            self.guard["green"](self.sha, self.pages)
        self.runs[-1]["status"] = "in_progress"
        self.assertFalse(self.guard["green"](self.sha, self.pages))

    def test_required_contexts(self):
        self.rules = [{"type": "required_status_checks", "parameters": {"required_status_checks": [{"context": "security"}]}}]
        self.assertFalse(self.guard["green"](self.sha, self.pages))
        self.statuses = [{"id": 1, "context": "security", "state": "success"}]
        self.assertTrue(self.guard["green"](self.sha, self.pages))
        self.statuses.append({"id": 2, "context": "security", "state": "failure"})
        with self.assertRaises(ValueError):
            self.guard["green"](self.sha, self.pages)
        self.rules[0]["parameters"]["required_status_checks"][0]["integration_id"] = 4
        self.statuses = []
        self.checks.append({**self.gate, "name": "security", "id": 13, "app": {"id": 5}})
        self.assertFalse(self.guard["green"](self.sha, self.pages))
        self.checks[-1]["app"]["id"] = 4
        self.assertTrue(self.guard["green"](self.sha, self.pages))
        self.checks[-1]["conclusion"] = "neutral"
        with self.assertRaises(ValueError):
            self.guard["green"](self.sha, self.pages)

    def test_wait_and_timeout(self):
        self.runs = []

        def completed(_):
            self.runs = [self.ci_run]

        self.guard["wait"](self.sha, self.pages, sleep=completed, clock=lambda: 0)
        self.runs = []
        times = iter([0, 100000])
        with self.assertRaises(ValueError):
            self.guard["wait"](self.sha, self.pages, sleep=lambda _: None, clock=lambda: next(times))


class PinTests(unittest.TestCase):
    def test_release_and_nightly(self):
        packages = [
            {"name": "direct", "source": "git+https://example.test/direct#123"},
            {"name": "transitive", "source": "git+https://example.test/transitive#456"},
        ]
        with patch.object(policy, "execute", return_value=json.dumps({"packages": packages})), patch.dict(os.environ, {}, clear=True):
            with self.assertRaises(policy.ToolError):
                pins.check(ROOT)
            self.assertFalse(pins.check(ROOT, warn=True))
        with patch.object(policy, "execute", return_value=json.dumps({"packages": [{"name": "local", "source": None}]})):
            self.assertTrue(pins.check(ROOT))

    def test_nightly_skips(self):
        with (
            patch.object(pins, "check", return_value=False) as check,
            patch.dict(os.environ, {"GITHUB_WORKFLOW_REF": "owner/repo/.github/workflows/nightly.yml@refs/heads/dev"}, clear=True),
            patch.object(registry.storage, "preflight") as preflight,
        ):
            registry.rehearse_crates(ROOT)
            self.assertTrue(check.call_args.kwargs["warn"])
            preflight.assert_not_called()


class EdgeTests(unittest.TestCase):
    def test_atomic_replacement(self):
        for failing_upload, missing_release in itertools.product((None, "linux.zip", "windows.zip", "SHA256SUMS"), (False, True)):
            with self.subTest(failing_upload=failing_upload, missing_release=missing_release), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                files = []
                for name in ("linux.zip", "windows.zip", "SHA256SUMS"):
                    data = name.encode()
                    (directory / name).write_bytes(data)
                    files.append({"name": name, "path": name, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()})
                release = {} if missing_release else {"id": 17, "draft": False, "body": "", "tag_name": edge.TAG}
                assets = [{"id": 1, "name": "obsolete.zip", "size": 3}]
                tag_moves = []
                published = []

                def document(
                    settings,
                    endpoint,
                    method="GET",
                    body=None,
                    *,
                    release=release,
                    assets=assets,
                    files=files,
                    tag_moves=tag_moves,
                    published=published,
                    missing_release=missing_release,
                    **kwargs,
                ):
                    if endpoint.startswith("actions/"):
                        return {"workflow_runs": []}
                    if endpoint == "releases/tags/edge":
                        return None if not release or release["draft"] else copy.deepcopy(release)
                    if endpoint == "releases" and method == "POST":
                        self.assertEqual(body["tag_name"], "edge")
                        self.assertTrue(body["draft"])
                        release.update({"id": 17, **body})
                        return copy.deepcopy(release)
                    if endpoint == "git/ref/tags/edge":
                        return None if missing_release else {"object": {"sha": "b" * 40}}
                    if (endpoint == "git/refs/tags/edge" and method == "PATCH") or (endpoint == "git/refs" and method == "POST"):
                        self.assertTrue(release["draft"])
                        self.assertEqual({item["name"] for item in assets}, {item["name"] for item in files})
                        if method == "POST":
                            self.assertEqual(body["ref"], "refs/tags/edge")
                        tag_moves.append(body["sha"])
                        return {}
                    if endpoint == "releases/17" and method == "PATCH":
                        if body.get("draft") is False:
                            published.append(copy.deepcopy(assets))
                        release.update(body)
                        return copy.deepcopy(release)
                    raise AssertionError((endpoint, method))

                failure = [failing_upload]

                def request(settings, endpoint, method, data=None, *, release=release, assets=assets, failure=failure, **kwargs):
                    self.assertTrue(release["draft"], "asset mutation exposed a partial public channel")
                    if method == "DELETE":
                        assets.clear()
                        return
                    name = endpoint.split("name=")[1]
                    if name == failure[0]:
                        raise policy.ToolError("upload interrupted")
                    assets.append(
                        {
                            "id": len(assets) + 2,
                            "name": name,
                            "size": len(data),
                            "state": "uploaded",
                            "digest": "sha256:" + hashlib.sha256(data).hexdigest(),
                        }
                    )

                def pages(settings, endpoint, *, release=release, assets=assets, **kwargs):
                    if endpoint == "releases":
                        return [copy.deepcopy(release)] if release else []
                    return copy.deepcopy(assets)

                with (
                    patch.object(registry, "document", side_effect=document),
                    patch.object(registry, "pages", side_effect=pages),
                    patch.object(registry, "request", side_effect=request),
                ):
                    if failing_upload:
                        with self.assertRaises(policy.ToolError):
                            edge._github(directory, {}, files, "a" * 40, "7", "notes")
                        self.assertTrue(release["draft"])
                        self.assertEqual(tag_moves, [])
                        self.assertEqual(published, [])
                        failure[0] = None
                    edge._github(directory, {}, files, "a" * 40, "7", "notes")
                self.assertFalse(release["draft"])
                self.assertEqual(tag_moves, ["a" * 40])
                self.assertEqual(len(published), 1)
                self.assertEqual({item["name"] for item in published[0]}, {item["name"] for item in files})


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
