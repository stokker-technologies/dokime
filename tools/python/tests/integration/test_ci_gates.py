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
import contextlib
import copy
import hashlib
import io
import json
import os
import runpy
import tempfile
import unittest
import urllib.error
import urllib.parse
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.release import nightly, pins, policy, registry

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[4]

# === Types === #

# === Classes === #


class ImpactTests(unittest.TestCase):
    def setUp(self):
        self.impact = runpy.run_path(str(ROOT / ".github/actions/plan-impact/run.py"))
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        authored = next(line.removeprefix("        plan: ") for line in workflow.splitlines() if line.startswith("        plan: "))
        self.plan = json.loads(json.loads(authored))

    def test_dev_and_review(self):
        full = self.impact["select"](self.plan, {"Cargo.toml"})
        development = self.impact["scope"](self.plan, full, "build")
        self.assertTrue({"build", "build:rust:tests", "check:rust"} <= development)
        self.assertFalse(any(job.startswith(("package:", "test:", "release:", "bench:", "prepare:")) for job in development))
        self.assertTrue(any(job.startswith("test:") for job in self.impact["scope"](self.plan, full)))
        for job in development:
            self.assertTrue(set(self.plan["dependencies"][job]) <= development)
        print("Dev Impact:", ", ".join(sorted(development)))
        print("PR Impact:", ", ".join(sorted(full)))

    def test_guidance_scope(self):
        selected = self.impact["select"](self.plan, {"CONTRIBUTING.md"})
        self.assertEqual(self.impact["scope"](self.plan, selected, "build"), set())

    def test_conservative_scope(self):
        plan = {"dependencies": {"build": ["producer"], "package:rust": ["build"], "producer": []}, "rules": []}
        self.assertEqual(self.impact["close"](plan["dependencies"], {"unknown"}), set(plan["dependencies"]))
        self.assertEqual(self.impact["scope"](plan, {"build", "package:rust"}, "build"), {"build", "producer"})
        with self.assertRaises(ValueError):
            self.impact["scope"](plan, set(), "typo")


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


class ChannelTests(unittest.TestCase):
    def test_nightly_source(self):
        base = {"GITHUB_REPOSITORY": "owner/repo", "GITHUB_RUN_ID": "42", "GITHUB_SHA": "a" * 40, "GITHUB_SERVER_URL": "https://github.com"}
        expected = ("a" * 40, "42", "https://github.com/owner/repo/actions/runs/42")
        for event in ("schedule", "workflow_dispatch"):
            environment = {**base, "GITHUB_EVENT_NAME": event, "GITHUB_REF": "refs/heads/dev"}
            with self.subTest(event=event), patch.dict(os.environ, environment, clear=True):
                self.assertEqual(nightly._source(True), expected)

        for event, ref in (("workflow_dispatch", "refs/heads/feature"), ("push", "refs/heads/dev")):
            environment = {**base, "GITHUB_EVENT_NAME": event, "GITHUB_REF": ref}
            with self.subTest(event=event, ref=ref), patch.dict(os.environ, environment, clear=True), self.assertRaises(policy.ToolError):
                nightly._source(True)

    def test_publication_replaces_previous(self):
        for previous in (True, False):
            with self.subTest(previous=previous), tempfile.TemporaryDirectory() as temporary:
                directory, files = _staged_files(Path(temporary))
                github = FakeGitHub(previous=previous)
                github.releases[3] = {"id": 3, "name": "v0.1.2", "tag_name": "v0.1.2", "draft": False, "prerelease": False}
                github.releases[4] = {"id": 4, "name": nightly.TITLE, "tag_name": "untagged-4", "draft": True, "prerelease": True}
                github.assets.update({3: [], 4: []})
                with github.patched():
                    nightly._github(directory, {}, files, NEW, "7", "notes")
                published = github.published()
                self.assertEqual(published["target_commitish"], NEW)
                self.assertTrue(published["prerelease"])
                self.assertEqual(github.tag, NEW)
                self.assertEqual({asset["name"] for asset in github.assets[published["id"]]}, {item["name"] for item in files})
                self.assertEqual(set(github.releases), {3, published["id"]})

    def test_failure_before_swap_keeps_previous(self):
        for fault in ("create", "upload:linux.zip", "upload:SHA256SUMS", "size", "pending", "digest"):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                directory, files = _staged_files(Path(temporary))
                github = FakeGitHub(previous=True, faults={fault})
                before = copy.deepcopy((github.releases, github.assets, github.tag))
                with github.patched(), self.assertRaises(policy.ToolError):
                    nightly._github(directory, {}, files, NEW, "7", "notes")
                self.assertEqual((github.releases, github.assets, github.tag), before)

    def test_transient_upload_failure_retries(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory, files = _staged_files(Path(temporary))
            github = FakeGitHub(previous=True, faults={"partial:windows.zip"})
            with github.patched():
                nightly._github(directory, {}, files, NEW, "7", "notes")
            names = [asset["name"] for asset in github.assets[github.published()["id"]]]
            self.assertEqual(sorted(names), sorted(item["name"] for item in files))

    def test_failure_during_swap_restores_previous(self):
        for fault in ("tag-create", "draft-previous", "publish", "lookup"):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                directory, files = _staged_files(Path(temporary))
                github = FakeGitHub(previous=True, faults={fault})
                before = copy.deepcopy((github.releases, github.assets, github.tag))
                with github.patched(), self.assertRaises(policy.ToolError):
                    nightly._github(directory, {}, files, NEW, "7", "notes")
                self.assertEqual((github.releases, github.assets, github.tag), before)

    def test_superseded_and_immutable_previous(self):
        for state in ("superseded", "immutable"):
            with self.subTest(state=state), tempfile.TemporaryDirectory() as temporary:
                directory, files = _staged_files(Path(temporary))
                github = FakeGitHub(previous=True)
                if state == "superseded":
                    github.releases[1]["body"] = "| CI Run | [9](https://example.invalid/9) |"
                else:
                    github.releases[1]["immutable"] = True
                before = copy.deepcopy((github.releases, github.assets, github.tag))
                with github.patched():
                    if state == "superseded":
                        nightly._github(directory, {}, files, NEW, "7", "notes")
                    else:
                        with self.assertRaisesRegex(policy.ToolError, "Enable release immutability"):
                            nightly._github(directory, {}, files, NEW, "7", "notes")
                self.assertEqual((github.releases, github.assets, github.tag), before)

    def test_dev_workflow_change_supersedes(self):
        for workflows, published in (("changed", False), ("same", True)):
            with self.subTest(workflows=workflows), tempfile.TemporaryDirectory() as temporary:
                directory, files = _staged_files(Path(temporary))
                github = FakeGitHub(previous=True)
                github.dev = LATER
                github.workflows[LATER] = "changed-workflows" if workflows == "changed" else github.workflows[NEW]
                before = copy.deepcopy((github.releases, github.assets, github.tag))
                with github.patched():
                    nightly._github(directory, {}, files, NEW, "7", "notes")
                if published:
                    self.assertEqual(github.published()["target_commitish"], NEW)
                else:
                    self.assertEqual((github.releases, github.assets, github.tag), before)

    def test_repository_immutability_is_reported(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory, files = _staged_files(Path(temporary))
            github = FakeGitHub(previous=True, immutable=True)
            with github.patched(), self.assertRaisesRegex(policy.ToolError, "Enable release immutability"):
                nightly._github(directory, {}, files, NEW, "7", "notes")
            self.assertEqual(github.published()["target_commitish"], NEW)


class ProviderRetryTests(unittest.TestCase):
    def test_transient_responses(self):
        cases = (
            ("DELETE", [502, 404], None, 2),
            ("GET", [503, 503, 200], b"{}", 3),
            ("POST", [429, 200], b"{}", 2),
            ("POST", [502], policy.ToolError, 1),
            ("PATCH", [502, 502, 502, 502], policy.ToolError, registry.RETRY_ATTEMPTS),
            ("PUT", [404], policy.ToolError, 1),
        )
        for method, codes, expected, calls in cases:
            with self.subTest(method=method, codes=codes):
                opener = FakeOpener(codes)
                pauses: list[float] = []
                with (
                    patch.dict(os.environ, {"GH_TOKEN": "token"}, clear=True),
                    patch.object(registry.urllib.request, "build_opener", return_value=opener),
                    patch.object(registry, "_pause", side_effect=pauses.append),
                ):
                    settings = {"provider": "github", "repository": "owner/repo"}
                    if expected is policy.ToolError:
                        with self.assertRaises(policy.ToolError):
                            registry.request(settings, "releases/1", method)
                    else:
                        self.assertEqual(registry.request(settings, "releases/1", method), expected)
                self.assertEqual(opener.methods, [method] * calls)
                self.assertEqual(pauses, [registry.RETRY_PAUSE_SECONDS * 2**index for index in range(calls - 1)])


class FakeOpener:
    """Answer Each Provider Request With The Next Scripted Status."""

    def __init__(self, codes: list[int]):
        self.codes = list(codes)
        self.methods: list[str] = []

    def open(self, request, timeout):
        self.methods.append(request.get_method())
        code = self.codes.pop(0)
        if code != 200:
            raise urllib.error.HTTPError(request.full_url, code, "failure", {}, io.BytesIO(b"{}"))
        return io.BytesIO(b"{}")


class FakeGitHub:
    """An In-Memory Release Provider With GitHub's Draft, Tag And Asset Semantics And Injectable Faults."""

    def __init__(self, *, previous: bool, faults: set[str] | None = None, immutable: bool = False):
        self.faults = set(faults or ())
        self.immutable = immutable
        self.tag = OLD if previous else None
        self.dev = NEW
        self.workflows = {OLD: "previous-workflows", NEW: "build-workflows"}
        self.releases: dict[int, dict] = {}
        self.assets: dict[int, list[dict]] = {}
        self.next_id = 10
        if previous:
            self.releases[1] = {
                "id": 1,
                "name": nightly.TITLE,
                "tag_name": nightly.TAG,
                "target_commitish": OLD,
                "draft": False,
                "prerelease": True,
                "body": "",
            }
            self.assets[1] = [{"id": 1, "name": "old.zip", "size": 3, "state": "uploaded"}]

    def patched(self) -> contextlib.ExitStack:
        stack = contextlib.ExitStack()
        stack.enter_context(patch.object(registry, "document", side_effect=self.document))
        stack.enter_context(patch.object(registry, "pages", side_effect=self.pages))
        stack.enter_context(patch.object(registry, "request", side_effect=self.request))
        return stack

    def published(self) -> dict:
        (release,) = [item for item in self.releases.values() if item["tag_name"] == nightly.TAG and item["draft"] is False]
        return release

    def _fault(self, name: str, applies: bool = True, *, once: bool = True) -> None:
        if applies and name in self.faults:
            if once:
                self.faults.discard(name)
            raise policy.ToolError(f"injected {name}")

    def document(self, settings, endpoint, method="GET", body=None, **kwargs):
        body = copy.deepcopy(body or {})
        if endpoint == f"releases/tags/{nightly.TAG}":
            matches = [item for item in self.releases.values() if item["tag_name"] == nightly.TAG and item["draft"] is False]
            self._fault("lookup", any(item["id"] >= 10 for item in matches))
            return copy.deepcopy(matches[0]) if matches else None
        if endpoint == "releases" and method == "POST":
            self._fault("create")
            identity, self.next_id = self.next_id, self.next_id + 1
            self.releases[identity] = {"id": identity, **body}
            self.assets[identity] = []
            return copy.deepcopy(self.releases[identity])
        if endpoint.startswith("releases/") and method == "PATCH":
            release = self.releases[int(endpoint.split("/")[1])]
            if release.get("immutable"):
                raise policy.ToolError("immutable release")
            if body.get("draft") is True:
                self._fault("draft-previous", release["id"] < 10)
                # GitHub Lists A Release Patched Back To Draft Under An Untagged Placeholder.
                release.update(body, tag_name=f"untagged-{release['id']}")
            else:
                self._fault("publish", release["id"] >= 10)
                others = (item for item in self.releases.values() if item["id"] != release["id"])
                if any(item["tag_name"] == body.get("tag_name") and not item["draft"] for item in others):
                    raise policy.ToolError("tag_name already_exists")
                release.update(body)
                if self.immutable:
                    release["immutable"] = True
            return copy.deepcopy(release)
        if endpoint == f"git/ref/tags/{nightly.TAG}":
            return {"object": {"sha": self.tag}} if self.tag else None
        if endpoint == "git/ref/heads/dev":
            return {"object": {"sha": self.dev}}
        if endpoint.startswith("contents/.github?ref="):
            revision = endpoint.split("=", 1)[1]
            return [
                {"name": "actions", "type": "dir", "sha": "actions"},
                {"name": "workflows", "type": "dir", "sha": self.workflows[revision]},
            ]
        if endpoint == "git/refs" and method == "POST":
            self._fault("tag-create", body["sha"] == NEW)
            if self.tag is not None:
                raise policy.ToolError("Reference already exists")
            self.tag = body["sha"]
            return {}
        raise AssertionError((endpoint, method))

    def pages(self, settings, endpoint, **kwargs):
        if endpoint == "releases":
            return copy.deepcopy(list(self.releases.values()))
        assets = copy.deepcopy(self.assets[int(endpoint.split("/")[1])])
        for fault, change in (("size", {"size": 0}), ("pending", {"state": "new"}), ("digest", {"digest": "sha256:" + "0" * 64})):
            if fault in self.faults and assets:
                assets[0].update(change)
        return assets

    def request(self, settings, endpoint, method="GET", data=None, **kwargs):
        if endpoint == f"git/refs/tags/{nightly.TAG}" and method == "DELETE":
            self.tag = None
            return
        if endpoint.startswith("releases/assets/") and method == "DELETE":
            identity = int(endpoint.rsplit("/", 1)[1])
            for assets in self.assets.values():
                assets[:] = [asset for asset in assets if asset["id"] != identity]
            return
        if method == "DELETE":
            identity = int(endpoint.split("/")[1])
            del self.releases[identity]
            del self.assets[identity]
            return
        release, name = int(endpoint.split("/")[1]), urllib.parse.unquote(endpoint.split("name=")[1])
        # A Persistent Upload Fault Outlasts Every Retry.
        self._fault(f"upload:{name}", once=False)
        assert isinstance(data, bytes)
        self.assets[release].append(
            {
                "id": self.next_id,
                "name": name,
                "size": len(data),
                "state": "uploaded",
                "digest": "sha256:" + hashlib.sha256(data).hexdigest(),
            }
        )
        self.next_id += 1
        # The Upload Lands But The Response Is Lost, Leaving A Same-Name Asset For The Retry To Replace.
        self._fault(f"partial:{name}")
        return


# === Public Functions === #

# === Private Functions === #

OLD = "b" * 40
NEW = "a" * 40
LATER = "c" * 40


def _staged_files(directory: Path) -> tuple[Path, list[dict]]:
    files = []
    for name in ("linux.zip", "windows.zip", "SHA256SUMS"):
        data = name.encode()
        (directory / name).write_bytes(data)
        files.append({"name": name, "path": name, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    return directory, files


# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
