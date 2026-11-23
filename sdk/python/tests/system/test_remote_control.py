# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Control a Host From Another Process Through Only Its Exported .pair File."""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import os
import runpy
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

# Third Party Imports
from dokime_tools.test import policy
from starter_project.plugin import ExamplePlugin

# Local Imports
from dokime_sdk import Client, connect
from dokime_sdk.operations import OperationError

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPOSITORY = Path(__file__).resolve().parents[4]

# === Types === #

# === Classes === #


class Host(Protocol):
    endpoint: str
    credentials: str

    @property
    def client(self) -> Client: ...


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase) -> Host: ...

    def fixture(self, name: str) -> Path: ...


class RemoteControlStory(unittest.IsolatedAsyncioTestCase):
    timeout: float
    cli: Path
    host: Host

    async def asyncSetUp(self) -> None:
        # Start a Host Serving a Plugin Through Its Local Management Endpoint.
        self.timeout = policy.load(REPOSITORY).deadline("system")
        factory = cast(HostFactory, runpy.run_path(str(REPOSITORY / "sdk/python/tests/support/host.py"))["Host"])
        self.cli = factory.fixture("dokime")
        self.host = await factory.start(self)
        await self.host.client.plugins.register(ExamplePlugin())

    async def test_remote_script_runs_a_suite_through_the_exported_pairing(self) -> None:
        template = (await self.host.client.cases.templates())[0]
        directory = Path(tempfile.mkdtemp(prefix="dokime-remote-control-"))
        self.addCleanup(shutil.rmtree, directory)

        # The Operator Binds Remote Access on Every Interface and Exports the Pairing.
        listener = json.loads(await self.operator("operation", "call", "dokime.host.v1.access.Listener/Get", "--json", "{}"))
        request = {"value": {**listener["value"], "enabled": True, "endpoint": "0.0.0.0:0"}, "expected_revision": listener["revision"]}
        await self.operator("operation", "call", "dokime.host.v1.access.Listener/Set", "--json", json.dumps(request))
        pairing = directory / "remote.pair"
        exported = json.loads(await self.operator("access", "pairing", "export", "--label", "Remote", "--out", str(pairing), "--json"))
        endpoints = json.loads(pairing.read_bytes())["endpoints"]
        self.assertTrue(
            endpoints and all(endpoint.startswith("https://") and "0.0.0.0" not in endpoint for endpoint in endpoints), endpoints
        )

        # A Separate Process Holding Only the Pairing Runs the Owner's Rehearsal Script.
        environment = {key: value for key, value in os.environ.items() if not key.startswith(("DOKIME_REMOTE", "HARMOS_"))}
        output = directory / "output"
        process = await asyncio.create_subprocess_exec(
            sys.executable,
            str(REPOSITORY / "sdk/python/examples/remote/rehearse.py"),
            str(pairing),
            template.identifier,
            "--answer",
            "Yes",
            "--output",
            str(output),
            cwd=directory,
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )
        printed = (await asyncio.wait_for(process.communicate(), self.timeout))[0].decode()
        self.assertEqual(process.returncode, 0, printed)
        self.assertIn(f"Template {template.identifier}", printed)
        self.assertIn("Prompt Continue?", printed)
        self.assertIn("Outcome PASSED", printed)
        archive = ZipFile(output / "evidence.zip")
        try:
            self.assertIsNone(archive.testzip())
            self.assertEqual(archive.read("report.html"), (output / "report.html").read_bytes())
        finally:
            archive.close()

        # An Altered Key and a Revoked Pairing Are Refused With the Export Action.
        tampered = directory / "tampered.pair"
        tampered.write_text(json.dumps({**json.loads(pairing.read_bytes()), "access_key": "x" * 43}), encoding="utf-8")
        await self.refused(tampered)
        await self.operator(
            "operation", "call", "dokime.host.v1.access.Pairings/Delete", "--json", json.dumps({"name": exported["pairing_id"]})
        )
        await self.refused(pairing)

    async def operator(self, *arguments: str) -> str:
        process = await asyncio.create_subprocess_exec(
            str(self.cli),
            "--credential-namespace",
            self.host.credentials,
            "--endpoint",
            self.host.endpoint,
            *arguments,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            creationflags=0x08000000 if os.name == "nt" else 0,
        )
        output, errors = await asyncio.wait_for(process.communicate(), self.timeout)
        self.assertEqual(process.returncode, 0, errors.decode())
        return output.decode()

    async def refused(self, pairing: Path) -> None:
        with self.assertRaises(OperationError) as refusal:
            await connect(pairing, connect_timeout=self.timeout)
        failure = refusal.exception.value
        self.assertIn("Refused the Access Key", failure.message)
        self.assertIn("Export a New Pairing", failure.cause.action)
        self.assertNotIn(json.loads(pairing.read_bytes())["access_key"], str(failure))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
