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
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, call, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.dev import desktop, runtime

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DevelopmentDesktopTests(unittest.TestCase):
    def test_report_source_changes_regenerate_before_runtime_rebuild(self) -> None:
        # Register Report Inputs With the Existing Tauri Watcher
        command = desktop.command({"DOKIME_DEV_PORT": "5180"})
        build = json.loads(command[command.index("--config") + 1])["build"]
        folders = [Path(folder) for folder in build["additionalWatchFolders"]]
        self.assertEqual(build["devUrl"], "http://127.0.0.1:5180")
        self.assertEqual(build["runner"]["cmd"], "uv")
        self.assertEqual(build["runner"]["args"][-2:], ["-m", runtime.__name__])

        # Each Source Event Reaches the Runner; Generation Completes Before Cargo Runs
        actions = Mock()
        actions.cargo.return_value = subprocess.CompletedProcess([], 0)
        generation_patch = patch.object(runtime.frontend, "build_report", actions.generate)
        cargo_patch = patch.object(runtime.subprocess, "run", actions.cargo)
        generation_patch.start()
        cargo_patch.start()
        try:
            arguments = ["--features", "console", "--no-default-features", "--", "--example"]
            for source in ("report", "plots", "components/document"):
                changed = desktop.ROOT / "crates/dokime-gui/src" / source / "renderer.tsx"
                self.assertTrue(any(changed.is_relative_to(folder) for folder in folders))
                actions.reset_mock()
                self.assertEqual(runtime.main(arguments), 0)
                self.assertEqual([action[0] for action in actions.mock_calls], ["generate", "cargo"])
                actions.generate.assert_called_once_with([])
                self.assertEqual(actions.cargo.call_args.args[0], ["cargo", "run", *arguments])

            # Failed Generation Must Never Launch a Runtime With a Stale Template
            actions.reset_mock()
            actions.generate.side_effect = subprocess.CalledProcessError(1, ["vite", "build"])
            self.assertEqual(runtime.main(arguments), 101)
            self.assertEqual(actions.mock_calls, [call.generate([])])
        finally:
            cargo_patch.stop()
            generation_patch.stop()

    def test_repeatable_plugin_arguments_accept_bundles_and_manifest_folders(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary_directory.name)
            bundle = root / "peer.dokime"
            bundle.write_bytes(b"bundle")
            project = root / "caller"
            project.mkdir()
            (project / "dokime.toml").write_text("[plugin]\n", encoding="utf-8")
            self.assertEqual(desktop.plugin_paths([]), [])
            self.assertEqual(desktop.plugin_paths(["--plugin", str(bundle), "--plugin", str(project)]), [bundle, project])
            self.assertRaises(SystemExit, desktop.plugin_paths, ["--plugin", str(root)])
            self.assertRaises(SystemExit, desktop.plugin_paths, ["--plugin"])
        finally:
            temporary_directory.cleanup()

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
            refused_patch = patch.object(desktop.subprocess, "run", return_value=refused)
            refused_patch.start()
            try:
                self.assertRaises(policy.ToolError, desktop.discard, folder, "dokime-dev-1", host, {})
            finally:
                refused_patch.stop()
            self.assertTrue(folder.is_dir())

            # Successful Removal Covers Every Root in the Namespace, Then Deletes the Folder.
            removed = subprocess.CompletedProcess(["dokime-host"], 0)
            removed_patch = patch.object(desktop.subprocess, "run", return_value=removed)
            run = removed_patch.start()
            try:
                desktop.discard(folder, "dokime-dev-1", host, {})
            finally:
                removed_patch.stop()
            storages = [call.args[0][4] for call in run.call_args_list]
            self.assertEqual(storages, [str(root) for root in roots])
            self.assertTrue(all(call.args[0][1:3] == ["--credential-namespace", "dokime-dev-1"] for call in run.call_args_list))
            self.assertFalse(folder.exists())
        finally:
            temporary_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
