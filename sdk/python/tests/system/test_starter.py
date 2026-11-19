# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run the Installed Starter Against a Prepared Product Host."""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import os
import runpy
import shutil
import tempfile
import unittest
from datetime import datetime
from hashlib import sha256
from io import BytesIO
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

from dokime_tools.test import policy

# Third Party Imports
from starter_project.plugin import ExamplePlugin
from starter_project.plugin.suites.capture.confirmation import ConfirmationTestTemplate

# Local Imports
from dokime_sdk import Client, PromptAnswer, PromptQuestion, ResultStatus

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Host(Protocol):
    endpoint: str

    @property
    def client(self) -> Client: ...


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase) -> Host: ...

    def fixture(self, name: str) -> Path: ...


class StarterStory(unittest.IsolatedAsyncioTestCase):
    async def test_register_run_seal_and_save_retained_evidence(self) -> None:
        # Run the Created Project's Classes Against a Prepared Host.
        support = Path(__file__).resolve().parents[1] / "support/host.py"
        host = await cast(HostFactory, runpy.run_path(str(support))["Host"]).start(self)
        client = host.client
        await client.plugins.register(ExamplePlugin())

        async def answer(question: PromptQuestion) -> PromptAnswer:
            return PromptAnswer(question.choices[0])

        client.on_prompt(answer)
        cycle = await client.cycle(label="Starter Capture")
        cases = await client.cases.materialize(ConfirmationTestTemplate)
        self.assertEqual(len(cases), 1)
        execution = await cycle.execution(cases[0])
        sealed = False
        async for progress in execution.run():
            sealed = progress.sealed
        self.assertTrue(sealed)
        outcome = await execution.seal()
        self.assertEqual(outcome.status, ResultStatus.PASSED)
        self.assertTrue(await execution.records(ConfirmationTestTemplate.confirmation).last())
        await cycle.seal()

        # Validate the Retained Evidence Archive Without Authored-Prose Assertions.
        evidence = await cycle.evidence()
        archive = ZipFile(BytesIO(evidence))
        try:
            self.assertIn("report.html", archive.namelist())
            self.assertIn("executions/01 Confirmation/report.html", archive.namelist())
            self.assertIn("executions/01 Confirmation/data/records.json", archive.namelist())
            self.assertIn("executions/01 Confirmation/telemetry/capture-confirmation-temperature.arrow", archive.namelist())
            self.assertEqual(
                json.loads(archive.read("executions/01 Confirmation/data/records.json"))[0]["kind"],
                "record:capture.confirmation.confirmation",
            )
            self.assertEqual(await cycle.report(), archive.read("report.html"))
            self.assertEqual(sha256(evidence).digest(), sha256(await cycle.evidence()).digest())
            self.assertIsNone(archive.testzip())
        finally:
            archive.close()

    async def test_created_project_runs_unmodified_client_and_saves_host_archive(self) -> None:
        # Create the Project Outside the Repository With the Built Command Line.
        repository = Path(__file__).resolve().parents[4]
        timeout = policy.load(repository).deadline("system")
        support = Path(__file__).resolve().parents[1] / "support/host.py"
        factory = cast(HostFactory, runpy.run_path(str(support))["Host"])
        binary = factory.fixture("dokime")
        directory = Path(tempfile.mkdtemp(prefix="dokime-created-starter-"))
        self.addCleanup(shutil.rmtree, directory)
        project = directory / "created-starter"
        environment = dict(os.environ)
        environment.pop("UV_PROJECT_ENVIRONMENT", None)
        environment.pop("VIRTUAL_ENV", None)
        flags = 0x08000000 if os.name == "nt" else 0
        process = await asyncio.create_subprocess_exec(
            str(binary),
            "new",
            str(project),
            "--lang",
            "python",
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            env=environment,
            creationflags=flags,
        )
        output, _ = await asyncio.wait_for(process.communicate(), timeout)
        self.assertEqual(process.returncode, 0, output.decode())

        # Resolve This Worktree's SDK Only in the Disposable Consumer.
        manifest = project / "pyproject.toml"
        source = json.dumps(str(repository / "sdk/python"))
        manifest.write_text(manifest.read_text() + f"\n[tool.uv.sources]\ndokime-sdk = {{ path = {source} }}\n")
        for arguments in [
            ("sync", "--refresh-package", "dokime-sdk"),
            ("run", "ruff", "check"),
            ("run", "ruff", "format", "--check"),
            ("run", "ty", "check"),
        ]:
            process = await asyncio.create_subprocess_exec(
                "uv",
                *arguments,
                cwd=project,
                env=environment,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
                creationflags=flags,
            )
            output, _ = await asyncio.wait_for(process.communicate(), timeout)
            self.assertEqual(process.returncode, 0, output.decode())

        # Start the Product Host and Run the Shipped Client Without Modifying It.
        host = await factory.start(self)
        environment["DOKIME_REMOTE"] = host.endpoint
        process = await asyncio.create_subprocess_exec(
            "uv",
            "run",
            "python",
            "-m",
            "created_starter",
            cwd=project,
            env=environment,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            creationflags=flags,
        )
        completed = asyncio.create_task(process.communicate())
        try:
            # Observe the Pending Prompt, Then Answer Through the Operator Command.
            watch = host.client._transport.prompts.watch("", True)
            next_frame = None
            try:
                while True:
                    next_frame = asyncio.create_task(anext(watch))
                    done, _ = await asyncio.wait((next_frame, completed), timeout=timeout, return_when=asyncio.FIRST_COMPLETED)
                    self.assertTrue(done, "Answer Starter Prompt: Inspect the Host for a Missing Prompt")
                    if completed in done:
                        output, _ = completed.result()
                        self.fail(f"Run Starter: Client Exited Before Its Prompt: {output.decode()}")
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

            # List Client-Created Executions Through the Operator Command.
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
                creationflags=flags,
            )
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
                creationflags=flags,
            )
            output, _ = await asyncio.wait_for(answer.communicate(), timeout)
            self.assertEqual(answer.returncode, 0, output.decode())
            output, _ = await asyncio.wait_for(completed, timeout)
            self.assertEqual(process.returncode, 0, output.decode())
        finally:
            if process.returncode is None:
                process.kill()
            await process.wait()
            await asyncio.gather(completed, return_exceptions=True)

        # Verify Seals and Compare Every Saved Archive Member With the Host Archive.
        execution = await host.client.executions.get(execution_id)
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
        detail = await host.client._transport.executions.get(execution_id)
        self.assertTrue(detail.execution.seal)
        cycle = await host.client.cycles.get(execution.cycle.identifier)
        detail = await host.client._transport.cycles.get(cycle.identifier)
        self.assertTrue(detail.cycle.sealed)
        saved_archives = list((project / "_output").glob("*.zip"))
        self.assertEqual(len(saved_archives), 1)
        saved = saved_archives[0]
        started = datetime.strptime(detail.cycle.metadata.label.split(" ", 1)[1], "%Y-%m-%d %H:%M:%S")
        self.assertEqual(saved.name, f"{started:%Y-%m-%d_%H-%M-%S}.zip")
        self.assertTrue(saved.is_file())
        archive = ZipFile(saved)
        retained = ZipFile(BytesIO(await cycle.evidence()))
        try:
            self.assertIsNone(archive.testzip())
            self.assertEqual(archive.namelist(), retained.namelist())
            self.assertIn("manifest.json", archive.namelist())
            self.assertIn("report.html", archive.namelist())
            self.assertIn("executions/01 Confirmation/report.html", archive.namelist())
            self.assertIn("executions/01 Confirmation/data/records.json", archive.namelist())
            self.assertIn("executions/01 Confirmation/telemetry/capture-confirmation-temperature.arrow", archive.namelist())
            self.assertEqual(
                json.loads(archive.read("executions/01 Confirmation/data/records.json"))[0]["kind"],
                "record:capture.confirmation.confirmation",
            )
            self.assertEqual(archive.read("report.html"), await cycle.report())
            for name in archive.namelist():
                self.assertEqual(archive.read(name), retained.read(name), name)
            print(f"Created Starter Archive: {archive.namelist()}")
        finally:
            retained.close()
            archive.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
