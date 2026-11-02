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
import json
import os
import runpy
import tempfile
import unittest
from datetime import UTC, datetime, timedelta, timezone
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.ci import impact, simulate
from dokime_tools.release import dev_channel, pins, policy, registry

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
    def test_nonstable_tags(self):
        action = runpy.run_path(str(ROOT / ".github/actions/review-release/run.py"))
        for tag in ("v1.2.3-dev.20261102120000.g01234567", "dev-channel", "v1.2.3-rc.1"):
            for environment in ({"GITHUB_REF": "refs/tags/" + tag}, {"CI_COMMIT_TAG": tag}):
                with self.subTest(environment=environment), patch.dict(os.environ, environment, clear=True):
                    with self.assertRaises(ValueError):
                        action["lookup"]()
                    with self.assertRaises(policy.ToolError):
                        registry.require_tag(ROOT, tag)

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


class ChannelTests(unittest.TestCase):
    def test_atomic_replacement(self):
        for failing_upload in (None, "linux.zip", "windows.zip", "SHA256SUMS"):
            with self.subTest(failing_upload=failing_upload), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                files = []
                for name in ("linux.zip", "windows.zip", "SHA256SUMS"):
                    data = name.encode()
                    (directory / name).write_bytes(data)
                    files.append({"name": name, "path": name, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()})
                release = {"id": 17, "draft": False, "body": "", "tag_name": dev_channel.TAG}
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
                    **kwargs,
                ):
                    if endpoint.startswith("actions/"):
                        return {"workflow_runs": []}
                    if endpoint == "releases/tags/dev-channel":
                        return None if release["draft"] else copy.deepcopy(release)
                    if endpoint == "git/ref/tags/dev-channel":
                        return {"object": {"sha": "b" * 40}}
                    if endpoint == "git/refs/tags/dev-channel" and method == "PATCH":
                        self.assertTrue(release["draft"])
                        self.assertEqual({item["name"] for item in assets}, {item["name"] for item in files})
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
                    return [copy.deepcopy(release)] if endpoint == "releases" else copy.deepcopy(assets)

                with (
                    patch.object(registry, "document", side_effect=document),
                    patch.object(registry, "pages", side_effect=pages),
                    patch.object(registry, "request", side_effect=request),
                    patch.object(dev_channel, "_github_immutable"),
                ):
                    if failing_upload:
                        with self.assertRaises(policy.ToolError):
                            dev_channel._github(directory, {}, files, "a" * 40, "7", "notes", tag="unused")
                        self.assertTrue(release["draft"])
                        self.assertEqual(tag_moves, [])
                        self.assertEqual(published, [])
                        failure[0] = None
                    dev_channel._github(directory, {}, files, "a" * 40, "7", "notes", tag="unused")
                self.assertFalse(release["draft"])
                self.assertEqual(tag_moves, ["a" * 40])
                self.assertEqual(len(published), 1)
                self.assertEqual({item["name"] for item in published[0]}, {item["name"] for item in files})

    def test_tag_formatting(self):
        sha = "0123456789abcdef" * 2 + "01234567"
        local = datetime(2026, 1, 1, 0, 30, 5, tzinfo=timezone(timedelta(hours=2)))
        self.assertEqual(dev_channel._dev_tag("12.3.0", sha, local), "v12.3.0-dev.20251231223005.g01234567")
        self.assertEqual(dev_channel._dev_tag("0.7.42", sha, local.astimezone(UTC)), "v0.7.42-dev.20251231223005.g01234567")
        for version, revision, timestamp in (
            ("01.2.3", sha, local),
            ("1.2.3-rc.1", sha, local),
            ("1.2.3", "abc", local),
            ("1.2.3", sha, local.replace(tzinfo=None)),
        ):
            with self.subTest(version=version, revision=revision, timestamp=timestamp), self.assertRaises(policy.ToolError):
                dev_channel._dev_tag(version, revision, timestamp)

    def test_owned_retention(self):
        releases = [
            {
                "id": index,
                "tag_name": f"v{version}-dev.2026110{index}120000.g01234567",
                "prerelease": True,
                "draft": False,
                "body": dev_channel.WORKFLOW_MARKER,
                "author": {"login": "github-actions[bot]", "type": "Bot"},
            }
            for index, version in ((1, "2.0.0"), (2, "1.0.0"), (3, "1.0.0"), (4, "0.1.0"))
        ]
        foreign = []
        for changes in (
            {"tag_name": "v1.2.3"},
            {"tag_name": "dev-channel"},
            {"tag_name": "v1.2.3-dev.20261301120000.g01234567"},
            {"tag_name": "v1.2.3-dev.20261101120000.g01234567-extra"},
            {"tag_name": "v01.2.3-dev.20261101120000.g01234567"},
            {"prerelease": False},
            {"draft": True},
            {"body": "foreign"},
            {"author": {"login": "some-user", "type": "User"}},
        ):
            foreign.append({**releases[0], **changes, "id": 100 + len(foreign)})
        unordered = foreign + list(reversed(releases))
        self.assertEqual([item["id"] for item in dev_channel._retired(unordered, 2)], [2, 1])
        self.assertEqual([item["id"] for item in dev_channel._retired(unordered, 1)], [3, 2, 1])
        self.assertEqual(dev_channel._retired(unordered, len(releases)), [])
        self.assertEqual(dev_channel._retired(foreign, 1), [])
        for keep in (0, -1, True, "2", None):
            with self.subTest(keep=keep), self.assertRaises(policy.ToolError):
                dev_channel._retention({"dev_release_retention": keep})

    def test_immutable_upload(self):
        for fault in (None, "upload", "digest", "missing-digest", "inventory", "state", "staged"):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                files = []
                for name in ("linux.zip", "windows.zip", "SHA256SUMS"):
                    data = name.encode()
                    (directory / name).write_bytes(data)
                    files.append({"name": name, "path": name, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()})
                if fault == "staged":
                    (directory / "windows.zip").write_bytes(b"corrupted")
                tag = "v1.2.3-dev.20261102120000.g01234567"
                sha = "01234567" + "a" * 32
                release = {}
                assets = []
                operations = []

                def document(settings, endpoint, method="GET", body=None, **kwargs):
                    operations.append((endpoint, method, body))
                    if endpoint == f"git/ref/tags/{tag}":
                        return None
                    if endpoint == "git/refs":
                        self.assertEqual(body, {"ref": "refs/tags/" + tag, "sha": sha})
                        return {}
                    if endpoint == "releases" and method == "POST":
                        self.assertEqual(body["target_commitish"], sha)
                        self.assertEqual(body["tag_name"], tag)
                        self.assertEqual(body["name"], tag)
                        self.assertTrue(body["draft"])
                        self.assertTrue(body["prerelease"])
                        self.assertEqual(body["make_latest"], "false")
                        release.update({**body, "id": 18})
                        return copy.deepcopy(release)
                    if endpoint == "releases/18" and method == "PATCH":
                        self.assertEqual({item["name"] for item in assets}, {item["name"] for item in files})
                        self.assertEqual(body["make_latest"], "false")
                        release.update(body)
                        return copy.deepcopy(release)
                    raise AssertionError((endpoint, method))

                def request(settings, endpoint, method, data=None, **kwargs):
                    self.assertTrue(release["draft"])
                    self.assertEqual(method, "POST")
                    if fault == "upload":
                        raise policy.ToolError("interrupted")
                    name = endpoint.split("name=")[1]
                    assets.append(
                        {
                            "name": name,
                            "size": len(data),
                            "state": "new" if fault == "state" else "uploaded",
                            "digest": None
                            if fault == "missing-digest"
                            else "sha256:" + ("bad" if fault == "digest" else hashlib.sha256(data).hexdigest()),
                        }
                    )

                def pages(settings, endpoint, **kwargs):
                    if endpoint == "releases":
                        return []
                    self.assertEqual(endpoint, "releases/18/assets")
                    return copy.deepcopy(assets[:-1] if fault == "inventory" else assets)

                with (
                    patch.object(registry, "document", side_effect=document),
                    patch.object(registry, "request", side_effect=request),
                    patch.object(registry, "pages", side_effect=pages),
                ):
                    if fault:
                        with self.assertRaises(policy.ToolError):
                            dev_channel._github_immutable(directory, {}, files, sha, tag, "notes")
                        self.assertTrue(release["draft"])
                        self.assertFalse(any(method == "PATCH" for _, method, _ in operations))
                    else:
                        dev_channel._github_immutable(directory, {}, files, sha, tag, "notes")
                        self.assertFalse(release["draft"])
                        self.assertTrue(release["prerelease"])

    def test_tag_collision(self):
        tag = "v1.2.3-dev.20261102120000.g01234567"
        for reference, releases in (
            ({"object": {"sha": "b" * 40}}, []),
            (None, [{"tag_name": tag, "draft": True}]),
            (None, [{"tag_name": tag, "draft": False}]),
        ):
            with (
                self.subTest(reference=reference, releases=releases),
                patch.object(registry, "document", return_value=reference) as document,
                patch.object(registry, "pages", return_value=releases),
                patch.object(registry, "request") as request,
            ):
                with self.assertRaises(policy.ToolError):
                    dev_channel._github_immutable(ROOT, {}, [], "a" * 40, tag, "notes")
                self.assertEqual(document.call_count, 1)
                request.assert_not_called()

    def test_publish_order(self):
        tag = "v1.2.3-dev.20261102120000.g01234567"
        older = {
            "id": 8,
            "tag_name": "v2.0.0-dev.20261101120000.g01234567",
            "prerelease": True,
            "draft": False,
            "body": dev_channel.WORKFLOW_MARKER,
            "author": {"login": "github-actions[bot]", "type": "Bot"},
        }
        newer = {**older, "id": 9, "tag_name": tag}
        operations = []

        def document(settings, endpoint, method="GET", body=None, **kwargs):
            operations.append((endpoint, method, body))
            return {"workflow_runs": []} if endpoint.startswith("actions/") else {}

        def pages(settings, endpoint, **kwargs):
            return [older, newer, {"id": 10, "tag_name": "v1.2.3", "prerelease": False}] if endpoint == "releases" else []

        def immutable(*args, **kwargs):
            operations.append(("immutable", "publish", None))

        def request(settings, endpoint, method, **kwargs):
            operations.append((endpoint, method, None))

        with (
            patch.object(dev_channel, "_github_release", return_value={"id": 17, "body": ""}),
            patch.object(dev_channel, "_github_immutable", side_effect=immutable),
            patch.object(dev_channel, "_github_upload"),
            patch.object(registry, "document", side_effect=document),
            patch.object(registry, "pages", side_effect=pages),
            patch.object(registry, "request", side_effect=request),
        ):
            dev_channel._github(ROOT, {"dev_release_retention": 1}, [], "a" * 40, "7", "notes", tag=tag)
        immutable_index = next(index for index, item in enumerate(operations) if item[0] == "immutable")
        draft_index = operations.index(("releases/17", "PATCH", {"draft": True}))
        publish_index = next(index for index, item in enumerate(operations) if item[1] == "PATCH" and item[2].get("draft") is False)
        self.assertLess(immutable_index, draft_index)
        deletions = [(endpoint, method) for endpoint, method, _ in operations if method == "DELETE"]
        self.assertEqual(deletions, [("releases/8", "DELETE"), ("git/refs/tags/" + older["tag_name"], "DELETE")])
        self.assertLess(publish_index, operations.index(("releases/8", "DELETE", None)))

        with (
            patch.object(dev_channel, "_github_release", return_value={"id": 17, "body": ""}),
            patch.object(dev_channel, "_github_immutable", side_effect=policy.ToolError("interrupted")),
            patch.object(registry, "document", return_value={"workflow_runs": []}) as document,
            patch.object(registry, "request") as request,
        ):
            with self.assertRaises(policy.ToolError):
                dev_channel._github(ROOT, {}, [], "a" * 40, "7", "notes", tag=tag)
            self.assertEqual(document.call_count, 1)
            request.assert_not_called()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
