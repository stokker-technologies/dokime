# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Pin Tests."""

# === Imports === #

# Standard Library Imports
import contextlib
import io
import json
import os
import unittest
from unittest.mock import Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.ci import tasks
from dokime_tools.release import (
    dry,
    pins,
    registry,
    run,
)

from .ci_gates_fixtures import ROOT
from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PinTests(unittest.TestCase):
    def test_release_and_nightly(self) -> None:
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

    def test_nightly_skips(self) -> None:
        with (
            patch.object(pins, "check", return_value=False) as check,
            patch.dict(os.environ, {"GITHUB_WORKFLOW_REF": "owner/repo/.github/workflows/nightly.yml@refs/heads/dev"}, clear=True),
        ):
            registry.crates.rehearse(ROOT, channel="nightly")
            self.assertTrue(check.call_args.kwargs["warn"])
            registry.crates.rehearse(ROOT, channel="stable")
            self.assertFalse(check.call_args.kwargs["warn"])

    def test_dry_run_channel_reaches_the_crate_dry_run(self) -> None:
        # Nightly Reports Git Pins As Pending; Every Other Caller Keeps The Stable Refusal.
        cases = (
            ({"RELEASE_CHANNEL": "nightly"}, [], "nightly"),
            ({}, [], "stable"),
            ({"RELEASE_CHANNEL": "nightly"}, ["--channel", "stable"], "stable"),
        )
        for environment, arguments, expected in cases:
            with (
                self.subTest(environment=environment, arguments=arguments),
                patch.dict(os.environ, environment, clear=True),
                patch.object(policy, "policy", return_value={}),
                patch.object(registry.crates, "rehearse") as rehearse,
            ):
                self.assertEqual(run.main(["crates-dry", *arguments]), 0)
                self.assertEqual(rehearse.call_args.kwargs["channel"], expected)
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(policy, "policy", return_value={}),
            patch.object(dry, "dry") as stage_dry_run,
            contextlib.redirect_stderr(io.StringIO()),
        ):
            self.assertEqual(run.main(["dry", "--channel", "nightly"]), 1)
            stage_dry_run.assert_not_called()
        for channel in ("stable", "nightly"):
            with (
                self.subTest(channel=channel),
                patch.dict(os.environ, {}, clear=True),
                patch.object(tasks.subprocess, "run", return_value=Mock(returncode=0)) as execute,
            ):
                self.assertEqual(tasks.main(["publish", "--dry", "--channel", channel]), 0)
                self.assertEqual(execute.call_args.args[0], ["mise", "run", "release:publish:dry", "--"])
                self.assertEqual(os.environ["RELEASE_CHANNEL"], channel)


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
