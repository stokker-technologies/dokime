# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prove the Development Desktop Stays on Its Disposable Root and Removes Its Credentials First."""

# === Imports === #

# Standard Library Imports
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.dev import desktop

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DevelopmentDesktopTests(unittest.TestCase):
    def test_isolation_replaces_inherited_roots_and_namespace(self) -> None:
        inherited = {
            "PATH": "tools",
            "DOKIME_CREDENTIAL_NAMESPACE": "dokime",
            "DOKIME_APPLICATION_ROOT": "real",
            "DOKIME_OUTPUT_ROOT": "real/data",
            "DOKIME_REMOTE": "real.pair",
        }
        environment = desktop.isolated(inherited, Path("disposable"), "dokime-dev-1")
        self.assertEqual(
            environment,
            {"PATH": "tools", "DOKIME_CREDENTIAL_NAMESPACE": "dokime-dev-1", "DOKIME_APPLICATION_ROOT": "disposable"},
        )

    def test_discard_removes_each_roots_credentials_before_the_folder(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            folder = Path(temporary) / "dev"
            roots = [folder / "data", folder / "data/nested"]
            for root in roots:
                root.mkdir(parents=True, exist_ok=True)
                (root / "backend-id").write_text("host", encoding="utf-8")
            host = Path("dokime-host")

            # A Failed Removal Keeps the Folder So Its Credentials Are Not Orphaned.
            refused = subprocess.CompletedProcess(["dokime-host"], 1)
            with patch.object(desktop.subprocess, "run", return_value=refused), self.assertRaises(policy.ToolError):
                desktop.discard(folder, "dokime-dev-1", host, {})
            self.assertTrue(folder.is_dir())

            # Successful Removal Covers Every Root in the Namespace, Then Deletes the Folder.
            removed = subprocess.CompletedProcess(["dokime-host"], 0)
            with patch.object(desktop.subprocess, "run", return_value=removed) as run:
                desktop.discard(folder, "dokime-dev-1", host, {})
            storages = [call.args[0][4] for call in run.call_args_list]
            self.assertEqual(storages, [str(root) for root in roots])
            self.assertTrue(all(call.args[0][1:3] == ["--credential-namespace", "dokime-dev-1"] for call in run.call_args_list))
            self.assertFalse(folder.exists())
        finally:
            temporary_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
