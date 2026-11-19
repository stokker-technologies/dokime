# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run the Created C# Project, Bundle and Client Against Prepared Product Bytes."""

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
from asyncio.subprocess import Process
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

# Third Party Imports
from dokime_sdk import Client, ResultStatus

# Local Imports
from dokime_tools.process import spawn_flags
from dokime_tools.test import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Host(Protocol):
    endpoint: str
    client: Client


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase) -> Host: ...


class CreatedProjectStory(unittest.IsolatedAsyncioTestCase):
    async def test_create_bundle_develop_run_seal_and_save(self) -> None:
        # Create the Standalone Project Outside the Repository.
        repository = Path(__file__).resolve().parents[3]
        timeout = policy.load(repository).deadline("system")
        suffix = ".exe" if os.name == "nt" else ""
        binary = repository / "target/debug" / f"dokime{suffix}"
        self.assertTrue(binary.is_file(), "Create C# Project: Build dokime-cli Through the Project Task")
        scratch = Path(tempfile.mkdtemp(prefix="dokime-csharp-created-"))
        self.addCleanup(shutil.rmtree, scratch)
        project = scratch / "created-starter"
        environment = dict(os.environ)
        for variable in ("UV_PROJECT_ENVIRONMENT", "VIRTUAL_ENV", "PYTHONPATH", "DOKIME_REMOTE"):
            environment.pop(variable, None)
        environment["DOKIME_CSHARP_CONFIGURATION"] = "Debug"
        environment["NUGET_PACKAGES"] = str(repository / "target/cs/packages")
        process = await asyncio.create_subprocess_exec(
            str(binary),
            "new",
            str(project),
            "--lang",
            "csharp",
            "--from",
            str(repository),
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())

        process = await asyncio.create_subprocess_exec(
            "mise",
            "-C",
            str(repository / "sdk/csharp"),
            "run",
            "build:starter",
            "--",
            "--project",
            str(project / "CreatedStarter.slnx"),
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())

        # Trust the Authored Project and Run Its Host-Free Test.
        process = await asyncio.create_subprocess_exec(
            "mise",
            "trust",
            str(project / "mise.toml"),
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())
        process = await asyncio.create_subprocess_exec(
            "mise",
            "-C",
            str(project),
            "run",
            "test",
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())

        # Build and Inspect the Bundle Through the Product Command.
        bundle = scratch / "created.dokime"
        process = await asyncio.create_subprocess_exec(
            str(binary),
            "plugin",
            "bundle",
            str(project),
            "--out",
            str(bundle),
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())
        process = await asyncio.create_subprocess_exec(
            str(binary),
            "plugin",
            "inspect",
            str(bundle),
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())

        # Start Development Without a Desktop and Close Its Owned Host.
        process = await asyncio.create_subprocess_exec(
            str(binary),
            "dev",
            "--plugin",
            str(project),
            "--no-desktop",
            env=environment,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        try:
            assert process.stdout is not None
            while True:
                line = await asyncio.wait_for(process.stdout.readline(), timeout)
                self.assertTrue(line, "Develop C# Project: Keep the Host Running Until Its Endpoint Is Announced")
                if line.decode().strip().startswith("http://127.0.0.1:"):
                    break
            assert process.stdin is not None
            process.stdin.close()
            output, _ = await asyncio.wait_for(process.communicate(), timeout)
            self.assertEqual(process.returncode, 0, output.decode())
        finally:
            if process.returncode is None:
                process.kill()
            await process.wait()

        # Start the Prepared Product Host and Run the Unmodified Client.
        support = repository / "sdk/python/tests/support/host.py"
        host = await cast(HostFactory, runpy.run_path(str(support))["Host"]).start(self)
        environment["DOKIME_REMOTE"] = host.endpoint
        locating = await asyncio.create_subprocess_exec(
            "mise",
            "-C",
            str(repository / "sdk/csharp"),
            "which",
            "dotnet",
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, locating)
        output, _ = await asyncio.wait_for(locating.communicate(), timeout)
        self.assertEqual(locating.returncode, 0, output.decode())
        dotnet = output.decode().strip()
        process = await asyncio.create_subprocess_exec(
            dotnet,
            str(project / "src/CreatedStarter.Client/bin/Debug/net10.0/CreatedStarter.Client.dll"),
            cwd=project,
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=spawn_flags(),
        )
        self.addAsyncCleanup(_close, process)
        completed = asyncio.create_task(process.communicate())
        try:
            # Observe the Operator Question Without Polling or Scenario Delays.
            watch = host.client._transport.prompts.watch("", True)
            next_frame = None
            try:
                while True:
                    next_frame = asyncio.create_task(anext(watch))
                    done, _ = await asyncio.wait((next_frame, completed), timeout=timeout, return_when=asyncio.FIRST_COMPLETED)
                    self.assertTrue(done, "Answer C# Prompt: Inspect the Host for a Missing Prompt")
                    if completed in done:
                        output, _ = completed.result()
                        self.fail(f"Run C# Client: It Exited Before the Operator Question: {output.decode()}")
                    frame = next_frame.result()
                    self.assertFalse(frame.HasField("resync_required"))
                    pending = [view.prompt for view in frame.prompts if not view.HasField("response")]
                    if pending:
                        execution_id = pending[0].scope.execution_id
                        break
            finally:
                if next_frame is not None:
                    next_frame.cancel()
                    await asyncio.gather(next_frame, return_exceptions=True)
                await watch.aclose()

            # Read the Execution Identity From the CLI and Answer Through the CLI.
            listed = await asyncio.create_subprocess_exec(
                str(binary),
                "--endpoint",
                host.endpoint,
                "execution",
                "list",
                "--json",
                env=environment,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
                creationflags=spawn_flags(),
            )
            self.addAsyncCleanup(_close, listed)
            output, _ = await asyncio.wait_for(listed.communicate(), timeout)
            self.assertEqual(listed.returncode, 0, output.decode())
            rows = json.loads(output)["executions"]
            self.assertIn(execution_id, [row["summary"]["execution_id"] for row in rows])
            answer = await asyncio.create_subprocess_exec(
                str(binary),
                "--endpoint",
                host.endpoint,
                "execution",
                "prompt",
                "answer",
                execution_id,
                "--action",
                "0",
                "--accepted",
                env=environment,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
                creationflags=spawn_flags(),
            )
            self.addAsyncCleanup(_close, answer)
            output, _ = await asyncio.wait_for(answer.communicate(), timeout)
            self.assertEqual(answer.returncode, 0, output.decode())
            output, _ = await asyncio.wait_for(completed, timeout)
            self.assertEqual(process.returncode, 0, output.decode())
        finally:
            if process.returncode is None:
                process.kill()
            await process.wait()
            await asyncio.gather(completed, return_exceptions=True)

        # Verify the Stable Outcome, Cycle Seal and Retained Archive.
        execution = await host.client.executions.get(execution_id)
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
        detail = await host.client._transport.executions.get(execution_id)
        self.assertTrue(detail.execution.seal)
        cycle = await host.client._transport.cycles.get(execution.cycle.identifier)
        self.assertTrue(cycle.cycle.sealed)
        saved = project / "_output" / f"{execution.cycle.identifier}.zip"
        self.assertTrue(saved.is_file())
        archive = ZipFile(saved)
        try:
            self.assertIsNone(archive.testzip())
            self.assertIn("manifest.json", archive.namelist())
        finally:
            archive.close()


# === Public Functions === #

# === Private Functions === #


async def _close(process: Process) -> None:
    if process.returncode is None:
        process.kill()
    await process.wait()


# === Entry Point === #

if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0]])
