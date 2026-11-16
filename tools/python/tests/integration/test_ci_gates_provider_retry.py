# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Provider Retry Tests."""

# === Imports === #

# Standard Library Imports
import hashlib
import io
import json
import os
import tempfile
import unittest
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from unittest.mock import Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.release import (
    nightly,
    notices,
    registry,
)

from .ci_gates_fixtures import FakeOpener
from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ProviderRetryTests(unittest.TestCase):
    def test_transient_responses(self) -> None:
        cases = (
            ("DELETE", [502, 404], None, 2),
            ("GET", [503, 503, 200], b"{}", 3),
            ("POST", [429], policy.ToolError, 1),
            ("POST", [502, 200], b"{}", 2),
            ("PATCH", [502] * notices.DOWNLOAD_ATTEMPTS, policy.ToolError, notices.DOWNLOAD_ATTEMPTS),
            ("PUT", [404], policy.ToolError, 1),
        )
        for method, codes, expected, calls in cases:
            with self.subTest(method=method, codes=codes):
                opener = FakeOpener(codes)
                pauses: list[float] = []
                with (
                    patch.dict(os.environ, {"GH_TOKEN": "token"}, clear=True),
                    patch.object(urllib.request, "build_opener", return_value=opener),
                    patch.object(notices.time, "sleep", side_effect=pauses.append),
                ):
                    settings = {"provider": "github", "repository": "owner/repo"}
                    if expected is policy.ToolError:
                        with self.assertRaises(policy.ToolError):
                            registry.provider.request(settings, "releases/1", method)
                    else:
                        self.assertEqual(registry.provider.request(settings, "releases/1", method), expected)
                self.assertEqual(opener.methods, [method] * calls)
                self.assertEqual(len(pauses), calls - 1)

    def test_upload_retry_deletes_partial_and_verifies_response(self) -> None:
        data = b"asset bytes"
        asset = {"id": 9, "name": "file.zip", "size": len(data), "digest": "sha256:" + hashlib.sha256(data).hexdigest()}
        opener = Mock()
        opener.open.side_effect = [
            TimeoutError("lost upload response"),
            io.BytesIO(json.dumps([asset]).encode()),
            io.BytesIO(b""),
            io.BytesIO(json.dumps(asset).encode()),
        ]
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                patch.dict(os.environ, {"GH_TOKEN": "token"}, clear=True),
                patch.object(urllib.request, "build_opener", return_value=opener),
                patch.object(notices.time, "sleep"),
            ):
                directory = Path(temporary)
                (directory / "file.zip").write_bytes(data)
                nightly._upload(
                    directory, {"provider": "github", "repository": "owner/repo"}, "releases/1", {"name": "file.zip", "path": "file.zip"}
                )
        finally:
            temporary_directory.cleanup()
        calls = opener.open.call_args_list
        self.assertEqual([call.args[0].method for call in calls], ["POST", "GET", "DELETE", "POST"])
        self.assertTrue(calls[2].args[0].full_url.endswith("releases/assets/9"))
        self.assertGreater(calls[0].kwargs["timeout"], calls[1].kwargs["timeout"])
        self.assertEqual(calls[0].kwargs["timeout"], calls[3].kwargs["timeout"])

    def test_connection_failure_and_final_diagnostic(self) -> None:
        opener = Mock()
        settings = {"provider": "github", "repository": "owner/repo"}
        with (
            patch.dict(os.environ, {"GH_TOKEN": "token"}, clear=True),
            patch.object(urllib.request, "build_opener", return_value=opener),
            patch.object(notices.time, "sleep"),
        ):
            opener.open.side_effect = [ConnectionResetError("reset"), io.BytesIO(b"{}")]
            self.assertEqual(registry.provider.request(settings, "releases/1"), b"{}")
            opener.open.side_effect = TimeoutError("read timed out")
            with self.assertRaisesRegex(
                policy.ToolError,
                r"release provider GET failed: https://api.github.com/repos/owner/repo/releases/1 after 6 attempts: read timed out",
            ):
                registry.provider.request(settings, "releases/1")

    def test_upload_rejects_incorrect_size_or_digest(self) -> None:
        for asset in ({"size": 0}, {"size": 4, "digest": "sha256:wrong"}):
            with (
                self.subTest(asset=asset),
                patch.dict(os.environ, {"GH_TOKEN": "token"}, clear=True),
                patch.object(urllib.request, "build_opener") as opener,
            ):
                opener.return_value.open.return_value = io.BytesIO(json.dumps(asset).encode())
                with self.assertRaisesRegex(policy.ToolError, "size or sha256 differs"):
                    registry.provider.request(
                        {"provider": "github", "repository": "owner/repo"},
                        "releases/1/assets?name=file",
                        "POST",
                        b"data",
                        binary=True,
                        upload=True,
                    )


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
