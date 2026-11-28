# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Release Tag Tests."""

# === Imports === #

# Standard Library Imports
import contextlib
import email.message
import hashlib
import io
import json
import os
import runpy
import tempfile
import tomllib
import unittest
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any
from unittest.mock import Mock, call, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.build import wheel
from dokime_tools.ci import tasks, transport, workflow
from dokime_tools.release import (
    context,
    guard,
    notices,
    prepare,
    registry,
    run,
    tag,
)

from .ci_gates_fixtures import LATER, NEW, OLD, ROOT
from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ReleaseTagTests(unittest.TestCase):
    def test_workflow_artifact_arguments(self) -> None:
        archives = [Path("target/ci/package-release-python-any.tar.zst")]
        for entry, arguments in ((tasks.main, ["publish"]), (workflow.main, ["release-verify"])):
            with (
                self.subTest(arguments=arguments),
                patch.dict(os.environ, {"CI_TRANSPORT": "true"}, clear=True),
                patch.object(transport.Path, "glob", return_value=archives),
                patch.object(transport.artifacts, "extract", autospec=True) as extract,
                patch.object(tasks.subprocess, "run", return_value=Mock(returncode=0)),
                patch.object(guard, "check"),
                patch.object(run, "main", return_value=0),
            ):
                self.assertEqual(entry(arguments), 0)
                self.assertEqual(extract.call_args_list, [call(ROOT, archive) for archive in archives])

    def test_shared_publication_never_selects_pypi(self) -> None:
        cases = (
            ([], ["release:publish:assets", "release:publish:finalize"], True),
            (["--stage", "assets"], ["release:publish:assets"], True),
            (["--registry", "pypi"], ["release:publish:python"], True),
            (["--registry", "pypi", "--dry"], ["release:publish:python:dry"], False),
        )
        for arguments, selected, guarded in cases:
            with (
                self.subTest(arguments=arguments),
                patch.dict(os.environ, {}, clear=True),
                patch.object(guard, "check") as guard_call,
                patch.object(tasks.subprocess, "run", return_value=Mock(returncode=0)) as execute,
            ):
                self.assertEqual(tasks.main(["publish", "--channel", "stable", *arguments]), 0)
                self.assertEqual([call.args[0] for call in execute.call_args_list], [["mise", "run", task, "--"] for task in selected])
                self.assertEqual(guard_call.called, guarded)
        for arguments in (["--channel", "nightly"], ["--stage", "assets"], ["--stage", "python"]):
            with (
                self.subTest(refused=arguments),
                patch.object(tasks.subprocess, "run") as execute,
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit),
            ):
                tasks.main(["publish", "--registry", "pypi", *arguments])
            execute.assert_not_called()

    def test_pypi_reruns_skip_matching_files_and_refuse_mismatches(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            (root / "sdk/python").mkdir(parents=True)
            (root / "sdk/python/pyproject.toml").write_text('[project]\nname = "dokime-sdk"\n', encoding="utf-8")
            wheel_file = root / "dokime_sdk-1.0.0-py3-none-any.whl"
            source = root / "dokime_sdk-1.0.0.tar.gz"
            wheel_file.write_bytes(b"wheel")
            source.write_bytes(b"source")
            digest = hashlib.sha256(b"wheel").hexdigest()
            settings = {"provider": "github"}
            with (
                patch.object(wheel, "distributions", return_value=[wheel_file, source]),
                patch.object(policy, "config", return_value={"version": {"value": "1.0.0"}}),
                patch.object(policy, "execute") as execute,
            ):
                with patch.object(registry.python, "digests", return_value={wheel_file.name: digest}):
                    registry.python.publish(root, settings, dry_run=False)
                self.assertEqual(
                    execute.call_args.args[1],
                    ["uv", "publish", "--trusted-publishing", "always", "--check-url", "https://pypi.org/simple/", source.name],
                )
                execute.reset_mock()
                with patch.object(
                    registry.python, "digests", return_value={wheel_file.name: digest, source.name: hashlib.sha256(b"source").hexdigest()}
                ):
                    registry.python.publish(root, settings, dry_run=False)
                execute.assert_not_called()
                with (
                    patch.object(registry.python, "digests", return_value={wheel_file.name: "0" * 64}),
                    self.assertRaises(policy.ToolError),
                ):
                    registry.python.publish(root, settings, dry_run=False)
                execute.assert_not_called()
                registry.python.publish(root, settings, dry_run=True)
                self.assertEqual(
                    execute.call_args.args[1], ["uv", "publish", "--dry-run", "--trusted-publishing", "never", wheel_file.name, source.name]
                )
        finally:
            temporary_directory.cleanup()

    def test_first_pypi_release_and_lookup_failure(self) -> None:
        def refused(code: int) -> policy.ToolError:
            error = policy.ToolError("PyPI project read failed")
            error.__cause__ = urllib.error.HTTPError(
                "https://pypi.org/pypi/dokime-sdk/json", code, "refused", email.message.Message(), None
            )
            self.addCleanup(error.__cause__.close)
            return error

        with patch.object(notices, "download", side_effect=refused(404)):
            self.assertEqual(registry.python.digests("dokime-sdk"), {})
        with patch.object(notices, "download", side_effect=refused(403)), self.assertRaises(policy.ToolError):
            registry.python.digests("dokime-sdk")
        document = {"releases": {"1.0.0": [{"filename": "dokime_sdk-1.0.0.tar.gz", "digests": {"sha256": "a" * 64}}]}}
        with patch.object(notices, "download", return_value=json.dumps(document).encode()) as download:
            self.assertEqual(registry.python.digests("dokime-sdk"), {"dokime_sdk-1.0.0.tar.gz": "a" * 64})
        self.assertEqual(download.call_args.args[0], "https://pypi.org/pypi/dokime-sdk/json")

    def setUp(self) -> None:
        self.action = runpy.run_path(str(ROOT / "tools/python/src/dokime_tools/ci/review.py"))
        self.project = (ROOT / "tools/project.toml").read_text(encoding="utf-8")
        self.tag = "v" + tomllib.loads(self.project)["version"]["value"]
        self.review: dict = {
            "number": 211,
            "merged": True,
            "merged_at": "2026-11-10T00:00:00Z",
            "merge_commit_sha": NEW,
            "head": {"sha": OLD, "ref": "release/" + self.tag, "repo": {"full_name": "owner/product"}},
            "base": {"ref": "dev", "repo": {"full_name": "owner/product"}},
        }
        self.environment: dict = {
            "GITHUB_EVENT_NAME": "workflow_run",
            "GITHUB_REF": "refs/heads/dev",
            "GITHUB_SHA": LATER,
            "GITHUB_REPOSITORY": "owner/product",
            "GITHUB_WORKFLOW_REF": "owner/product/.github/workflows/release.yml@refs/heads/dev",
        }

    def command(self, *args: Any) -> str:
        if args == ("git", "rev-parse", "HEAD"):
            return LATER
        if args == ("git", "ls-remote", "--tags", "origin", "refs/tags/" + self.tag):
            return ""
        if args == ("git", "merge-base", "--is-ancestor", NEW, LATER):
            return ""
        if args == ("git", "show", f"{NEW}:tools/project.toml"):
            return self.project
        raise AssertionError(args)

    def mode(self, event: str, completion: dict | None = None, runs: tuple[dict, ...] = (), reviews: tuple[dict, ...] = ()) -> dict:
        def pages(endpoint: str, key: str | None = None) -> list[dict]:
            return list(runs) if endpoint.startswith("actions/") else list(reviews or (self.review,))

        temporary_directory = tempfile.TemporaryDirectory()
        try:
            path = Path(temporary_directory.name) / "event.json"
            path.write_text(json.dumps({"workflow_run": completion or {}}), encoding="utf-8")
            with (
                patch.dict(os.environ, {**self.environment, "GITHUB_EVENT_NAME": event, "GITHUB_EVENT_PATH": str(path)}),
                patch.dict(self.action["github_release_mode"].__globals__, command=self.command, pages=pages),
            ):
                return self.action["github_release_mode"]()
        finally:
            temporary_directory.cleanup()

    def test_only_the_review_validation_run_tags_its_target(self) -> None:
        validation = {
            "head_sha": OLD,
            "event": "pull_request",
            "display_title": "CI / Validation",
            "status": "completed",
            "conclusion": "success",
            "pull_requests": [{"number": 211}],
        }
        older = {**self.review, "merged_at": "2026-11-09T00:00:00Z", "merge_commit_sha": OLD}
        tagged = {"mode": "tag", "sha": NEW}
        self.assertEqual(self.mode("workflow_run", validation, reviews=(self.review, older)), tagged)
        self.assertEqual(self.mode("workflow_dispatch", runs=(validation,)), tagged)
        unrelated = (
            {"display_title": "CI / Closed Review"},
            {"head_sha": LATER},
            {"event": "push"},
            {"conclusion": "failure"},
            {"pull_requests": [{"number": 212}]},
        )
        for mutation in unrelated:
            with self.subTest(mutation=mutation):
                self.assertEqual(self.mode("workflow_run", {**validation, **mutation}), {"mode": "skip", "sha": ""})
                # A Dispatch Re-Prepares the Untagged Version Instead of Tagging Unvalidated Evidence.
                self.assertEqual(self.mode("workflow_dispatch", runs=({**validation, **mutation},)), {"mode": "prepare", "sha": ""})
        self.assertEqual(self.mode("workflow_dispatch"), {"mode": "prepare", "sha": ""})
        failed = {**validation, "conclusion": "failure"}
        for pending in ({"status": "queued", "conclusion": None}, {"status": "in_progress", "conclusion": None}):
            running = {**validation, **pending}
            with self.subTest(pending=pending), contextlib.redirect_stdout(io.StringIO()) as printed:
                # A Running Validation Restarts Release on Completion; a Dispatch Neither Tags Nor Prepares Again.
                self.assertEqual(self.mode("workflow_dispatch", runs=(failed, running)), {"mode": "skip", "sha": ""})
                self.assertEqual(printed.getvalue(), f"{self.tag} review validation is still running; wait for it\n")
                self.assertEqual(self.mode("workflow_dispatch", runs=(validation, running)), tagged)

    def test_untagged_version_is_prepared_again_from_dev(self) -> None:
        with patch.object(prepare.policy, "execute", return_value="") as execute:
            self.assertEqual(prepare.planned_version(ROOT, "1.2.3"), "1.2.3")
        execute.assert_called_once_with(ROOT, ["git", "tag", "--list", "v1.2.3"], capture=True)

    def test_dispatch_without_prepared_release_prepares(self) -> None:
        with (
            patch.dict(os.environ, {**self.environment, "GITHUB_EVENT_NAME": "workflow_dispatch"}),
            patch.dict(self.action["github_release_mode"].__globals__, command=self.command, pages=lambda *args: []),
        ):
            self.assertEqual(self.action["github_release_mode"](), {"mode": "prepare", "sha": ""})
            with patch.dict(self.action["github_release_mode"].__globals__, command=lambda *args: LATER):
                self.assertEqual(self.action["github_release_mode"](), {"mode": "prepare", "sha": ""})

    def test_prepared_release_events_require_exact_merged_review(self) -> None:
        settings = {"provider": "github", "repository": "owner/product"}
        for event in ("workflow_run", "workflow_dispatch"):
            for event_sha in (NEW, LATER):
                with (
                    self.subTest(event=event, event_sha=event_sha),
                    patch.dict(os.environ, {**self.environment, "GITHUB_EVENT_NAME": event, "GITHUB_SHA": event_sha}),
                    patch.object(policy, "execute", return_value="") as execute,
                    patch.object(registry.provider, "pages", return_value=[self.review]),
                    patch.object(registry.provider, "document", return_value=self.review),
                    patch.object(context, "release_tag", return_value=self.tag),
                ):
                    self.assertEqual(tag.tag_event(ROOT, settings, NEW), self.review)
                    if event_sha == NEW:
                        execute.assert_not_called()
                    else:
                        execute.assert_called_once_with(ROOT, ["git", "merge-base", "--is-ancestor", NEW, LATER])
                        with (
                            patch.object(policy, "execute", side_effect=policy.ToolError("not an ancestor")),
                            self.assertRaises(policy.ToolError),
                        ):
                            tag.tag_event(ROOT, settings, NEW)
                    for mutation in (
                        {"merge_commit_sha": LATER},
                        {"merged_at": None},
                        {"merged": False},
                        {"head": {**self.review["head"], "ref": "feature/ordinary"}},
                        {"base": {**self.review["base"], "ref": "main"}},
                        {"head": {**self.review["head"], "repo": {"full_name": "fork/product"}}},
                    ):
                        with (
                            self.subTest(mutation=mutation),
                            patch.object(registry.provider, "pages", return_value=[{**self.review, **mutation}]),
                            patch.object(registry.provider, "document", return_value={**self.review, **mutation}),
                            self.assertRaises(policy.ToolError),
                        ):
                            tag.tag_event(ROOT, settings, NEW)
                    for invalid in (
                        {"GITHUB_EVENT_NAME": "push"},
                        {"GITHUB_REF": "refs/heads/feature"},
                        {"GITHUB_WORKFLOW_REF": "owner/product/.github/workflows/ci.yml@refs/heads/dev"},
                    ):
                        with patch.dict(os.environ, invalid), self.assertRaises(policy.ToolError):
                            tag.tag_event(ROOT, settings, NEW)


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
