# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run the Installed Starter Against a Prepared Product Host."""

# === Imports === #

# Standard Library Imports
import runpy
import unittest
from io import BytesIO
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

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
    @property
    def client(self) -> Client: ...


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase) -> Host: ...


class StarterStory(unittest.IsolatedAsyncioTestCase):
    async def test_register_run_seal_and_save_retained_evidence(self) -> None:
        # Run the Created Project's Classes Against a Prepared Host.
        support = Path(__file__).resolve().parents[1] / "support/host.py"
        host = await cast(HostFactory, runpy.run_path(str(support))["Host"]).start(self)
        client = host.client
        await client.plugins.register(ExamplePlugin())

        async def answer(question: PromptQuestion) -> PromptAnswer:
            self.assertEqual(question.choices, ("Yes", "No"))
            return PromptAnswer("Yes")

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
            self.assertTrue(archive.namelist())
            self.assertIsNone(archive.testzip())
        finally:
            archive.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
