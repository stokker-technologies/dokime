# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove Immediate Release Merges and Green Review Evidence."""

# === Imports === #

# Standard Library Imports
import os
import unittest
from pathlib import Path
from unittest.mock import Mock, call, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.ci.review
from dokime_tools import policy
from dokime_tools.release import registry, review

from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ReleaseReviewsTests(unittest.TestCase):
    def test_immediate_squash(self) -> None:
        settings = {"provider": "github", "repository": "stokker-technologies/dokime"}
        head, merged = "a" * 40, "b" * 40
        title = "chore(release): prepare v0.1.3"
        with (
            patch.object(registry.provider, "pages", return_value=[{"number": 7, "head": {"sha": head}}]) as pages,
            patch.object(
                registry.provider,
                "document",
                side_effect=[{"merged": True, "sha": merged}, {"sha": merged, "commit": {"message": title}}],
            ) as document,
        ):
            registry.reviews.merge(settings, "release/v0.1.3", title)
        pages.assert_called_once_with(settings, "pulls?state=open&base=dev&head=stokker-technologies%3Arelease%2Fv0.1.3")
        self.assertEqual(
            document.call_args_list,
            [
                call(settings, "pulls/7/merge", "PUT", {"merge_method": "squash", "commit_title": title, "sha": head}),
                call(settings, f"commits/{merged}"),
            ],
        )

    def test_merge_verification(self) -> None:
        settings = {"provider": "github", "repository": "stokker-technologies/dokime"}
        merged = "b" * 40
        title = "chore(release): prepare v0.1.3"
        responses = [
            [{"merged": False, "sha": merged}],
            [{"merged": True, "sha": "invalid"}],
            [{"merged": True, "sha": merged}, {"sha": "c" * 40, "commit": {"message": title}}],
            [{"merged": True, "sha": merged}, {"sha": merged, "commit": {"message": "Wrong Headline"}}],
        ]
        for response in responses:
            with (
                self.subTest(response=response),
                patch.object(registry.provider, "pages", return_value=[{"number": 7, "head": {"sha": "a" * 40}}]),
                patch.object(registry.provider, "document", side_effect=response),
                self.assertRaises(policy.ToolError),
            ):
                registry.reviews.merge(settings, "release/v0.1.3", title)

    def test_gitlab_squash(self) -> None:
        settings = {"provider": "gitlab"}
        head, merged = "a" * 40, "b" * 40
        title = "chore(release): prepare v0.1.3"
        with (
            patch.object(registry.provider, "pages", return_value=[{"iid": 7, "sha": head}]),
            patch.object(
                registry.provider,
                "document",
                side_effect=[{"state": "merged", "squash_commit_sha": merged}, {"id": merged, "message": title}],
            ) as document,
        ):
            registry.reviews.merge(settings, "release/v0.1.3", title)
        self.assertEqual(
            document.call_args_list,
            [
                call(
                    settings,
                    "merge_requests/7/merge",
                    "PUT",
                    {"sha": head, "squash": True, "squash_commit_message": title, "should_remove_source_branch": True},
                ),
                call(settings, f"repository/commits/{merged}"),
            ],
        )

    def test_preparation_merges(self) -> None:
        directory = Path.cwd()
        settings = {"provider": "github"}
        with (
            patch.dict(os.environ, {"RELEASE_TOKEN": "release"}),
            patch.object(review.context, "continuous_integration", return_value=True),
            patch.object(review.context, "release_tag", return_value="v0.1.3"),
            patch.object(review.version, "run"),
            patch.object(registry.provider, "document", return_value=None),
            patch.object(review.policy, "execute"),
            patch.object(review, "commit_review"),
            patch.object(review.policy, "inside", return_value=Mock()),
            patch.object(review, "notes", return_value="Release Notes"),
            patch.object(review.subprocess, "run", return_value=Mock(returncode=0)) as opened,
            patch.object(registry.reviews, "merge") as merge,
        ):
            merge.side_effect = lambda *args: opened.assert_called_once()
            review.review(directory, settings)
        merge.assert_called_once_with(settings, "release/v0.1.3", "chore(release): prepare v0.1.3")

    def test_pending_success(self) -> None:
        pending = {"id": 9, "head_sha": "a" * 40, "display_title": "CI / Validation", "status": "in_progress", "conclusion": None}
        completed = {**pending, "status": "completed", "conclusion": "success"}
        jobs = [{"name": "Check / CI / Required", "conclusion": "success"}]
        jobs_for = Mock(return_value=jobs)
        run_for = Mock(return_value=completed)
        sleep = Mock()
        result = dokime_tools.ci.review.github_validation_run([pending], jobs_for, run_for, sleep=sleep, clock=lambda: 0)
        self.assertEqual(result, (completed, jobs))
        run_for.assert_called_once_with(9)
        jobs_for.assert_called_once_with(9)
        sleep.assert_called_once()

    def test_pending_failure(self) -> None:
        pending = {"id": 9, "head_sha": "a" * 40, "display_title": "CI / Validation", "status": "in_progress", "conclusion": None}
        older = {**pending, "id": 8, "status": "completed", "conclusion": "success"}
        completed = {**pending, "status": "completed", "conclusion": "failure"}
        jobs_for = Mock()
        run_for = Mock(return_value=completed)
        with self.assertRaisesRegex(ValueError, "review run is not green validation"):
            dokime_tools.ci.review.github_validation_run([older, pending], jobs_for, run_for, sleep=Mock(), clock=lambda: 0)
        run_for.assert_called_once_with(9)
        jobs_for.assert_not_called()

    def test_pending_timeout(self) -> None:
        pending = {"id": 9, "head_sha": "a" * 40, "display_title": "CI / Validation", "status": "queued"}
        jobs_for, run_for, sleep = Mock(), Mock(), Mock()
        times = iter([0, 1800])
        with self.assertRaisesRegex(ValueError, "rerun Release after CI succeeds"):
            dokime_tools.ci.review.github_validation_run([pending], jobs_for, run_for, sleep=sleep, clock=lambda: next(times))
        sleep.assert_not_called()
        run_for.assert_not_called()
        jobs_for.assert_not_called()

    def test_evidence_identity(self) -> None:
        pending = {"id": 9, "head_sha": "a" * 40, "display_title": "CI / Validation", "status": "in_progress"}
        jobs_for = Mock()
        for changed in ({"id": 10}, {"head_sha": "b" * 40}):
            with self.subTest(changed=changed), self.assertRaisesRegex(ValueError, "review run identity changed"):
                dokime_tools.ci.review.github_validation_run(
                    [pending], jobs_for, Mock(return_value={**pending, **changed}), sleep=Mock(), clock=lambda: 0
                )
        jobs_for.assert_not_called()


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
