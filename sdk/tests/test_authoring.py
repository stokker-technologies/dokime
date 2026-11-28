# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run Each SDK Capture Through the Product Host With Its Current Authoring Shape."""

# === Imports === #

# Standard Library Imports
import asyncio
import os
import runpy
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import ZipFile

# isort: split

# Third Party Imports
from grpc.aio import insecure_channel, server
from harmos._generated.proto.plugin.v1.instance_pb2 import InstanceGetRequest, InstanceGetResponse
from harmos._generated.proto.plugin.v1.launches_pb2 import (
    LaunchesAnnounceRequest,
    LaunchesAnnounceResponse,
)
from harmos._generated.proto.plugin.v1.launches_pb2_grpc import (
    LaunchesServicer,
    add_LaunchesServicer_to_server,
)

# isort: split

# Local Imports
from dokime_tools.process import spawn_flags
from dokime_tools.test import policy

from dokime_sdk import PromptAnswer, PromptQuestion, ResultStatus
from dokime_sdk._generated.proto.host.v1.plugins_pb2 import PLUGIN_STATE_READY, PluginPlacement, PluginRemote, PluginValue
from dokime_sdk._generated.proto.model.v1.record_contracts_pb2 import RecordContracts
from dokime_sdk._generated.proto.model.v1.suites_pb2 import Suites

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class AuthoringEquivalence(unittest.IsolatedAsyncioTestCase):
    async def test_four_languages_capture_prompt_record_and_sealed_outcome(
        self,
    ) -> None:
        # Start the Product Host for All Four Current Authoring Surfaces.
        root = Path(__file__).resolve().parents[2]
        factory = runpy.run_path(str(root / "sdk/python/tests/support/host.py"))["Host"]
        host = await factory.start(self)
        client = host.client

        async def answer(question: PromptQuestion) -> PromptAnswer:
            return PromptAnswer(question.choices[0])

        client.on_prompt(answer)
        locating = await asyncio.create_subprocess_exec(
            "mise",
            "-C",
            str(root / "sdk/csharp"),
            "which",
            "dotnet",
            stdout=asyncio.subprocess.PIPE,
            creationflags=spawn_flags(),
        )
        output, _ = await locating.communicate()
        self.assertEqual(
            locating.returncode,
            0,
            "Locate C# Fixture: Install Its Declared Tool With Mise",
        )
        dotnet = output.decode().strip()
        commands = {
            "python": [
                sys.executable,
                str(root / "sdk/tests/fixtures/python/capture/plugin.py"),
            ],
            "rust": [str(factory.fixture("rust", home="fixtures/cli"))],
            "csharp": [
                dotnet,
                str(
                    factory.fixture(
                        "csharp/source/assets/plugin/Csharp.Plugin.dll",
                        home="fixtures/cli",
                    )
                ),
            ],
            "cpp": [str(factory.fixture("cpp/dokime-plugin", home="fixtures/cli"))],
        }
        timeout = policy.load(root).deadline("integration")
        environment = dict(os.environ)
        ready: asyncio.Queue[str] = asyncio.Queue()

        class Launches(LaunchesServicer):
            async def Announce(self, request: LaunchesAnnounceRequest, context: object) -> LaunchesAnnounceResponse:
                await ready.put(request.address)
                return LaunchesAnnounceResponse()

        launches = server()
        add_LaunchesServicer_to_server(Launches(), launches)
        port = launches.add_insecure_port("127.0.0.1:0")
        await launches.start()
        self.addAsyncCleanup(launches.stop, None)
        environment.update(
            HARMOS_HOST_ENDPOINT=f"127.0.0.1:{port}",
            HARMOS_LAUNCH_TOKEN="fixture-launch",
        )
        for language, command in commands.items():
            # Launch the Prepared Fixture and Observe Its Bound Listener.
            process = await asyncio.create_subprocess_exec(
                *command,
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.DEVNULL,
                env=environment,
                creationflags=spawn_flags(),
            )
            try:
                address = await asyncio.wait_for(ready.get(), timeout)
                channel = insecure_channel(address)
                try:
                    get = channel.unary_unary(
                        "/harmos.plugin.v1.Instance/Get",
                        request_serializer=InstanceGetRequest.SerializeToString,
                        response_deserializer=InstanceGetResponse.FromString,
                    )
                    response = await get(
                        InstanceGetRequest(
                            revision=1,
                            host="authoring_equivalence",
                            host_version="1.0.0",
                        )
                    )
                finally:
                    await channel.close()

                # Preserve Each Native Identity and the Shared Boolean Record Contract.
                suites = Suites()
                records = RecordContracts()
                for section in response.declarations.sections:
                    if section.type == suites.DESCRIPTOR.full_name:
                        suites.ParseFromString(section.payload)
                    if section.type == records.DESCRIPTOR.full_name:
                        records.ParseFromString(section.payload)
                expected = "capture.confirmation"
                templates = {template.metadata.identifier: suite for suite in suites.suites for template in suite.templates}
                self.assertIn(expected, templates, language)
                suite = templates[expected]
                contracts = {record.metadata.identifier: record for record in records.records}
                self.assertIn(f"{expected}.confirmation", contracts, language)
                record = contracts[f"{expected}.confirmation"]
                self.assertEqual(record.message, "google.protobuf.BoolValue", language)

                # Register the Fixture and Wait on Host Readiness.
                created = await client._transport.plugins.create(
                    response.id,
                    PluginPlacement(remote=PluginRemote(url=f"http://{address}")),
                    PluginValue(enabled=True),
                )
                try:
                    watching = client._transport.plugins.watch(name=created.entry.name)
                    try:
                        async for frame in watching:
                            observed = await client._transport.plugins.get(created.entry.name)
                            self.assertFalse(
                                observed.entry.HasField("failure"),
                                str(observed.entry.failure),
                            )
                            if observed.entry.state == PLUGIN_STATE_READY:
                                break
                    finally:
                        await watching.aclose()

                    # Run the Shared Operator Flow and Read Its Frozen Boolean Evidence.
                    cases = await client.cases.materialize(
                        expected,
                        plugin=created.entry.name,
                        suite=suite.metadata.identifier,
                    )
                    self.assertEqual(len(cases), 1, language)
                    cycle = await client.cycle(label=f"{language.title()} Capture")
                    execution = await cycle.execution(cases[0])
                    async for progress in execution.run():
                        pass
                    self.assertEqual((await execution.seal()).status, ResultStatus.PASSED, language)
                    self.assertTrue(
                        await execution.records(record.metadata.identifier, row_type=bool).last(),
                        language,
                    )
                    self.assertEqual(
                        await execution.records(record.metadata.identifier, row_type=bool).count(),
                        1,
                        language,
                    )
                    await cycle.seal()
                    if language == "rust":
                        # Read Frozen History Through the Rust Bound Client.
                        temporary = TemporaryDirectory(prefix="dokime-rust-history-")
                        reader = None
                        try:
                            reader_environment = dict(environment)
                            reader_environment["DOKIME_REMOTE"] = host.endpoint
                            reader_environment["DOKIME_TEST_EXECUTION"] = execution.identifier
                            reader = await asyncio.create_subprocess_exec(
                                str(factory.fixture("dokime-rust-history")),
                                cwd=temporary.name,
                                env=reader_environment,
                                creationflags=spawn_flags(),
                            )
                            self.assertEqual(await asyncio.wait_for(reader.wait(), timeout), 0)
                            for archive in ("execution.zip", "cycle.zip"):
                                saved = ZipFile(Path(temporary.name) / "_output" / archive)
                                try:
                                    self.assertIsNone(saved.testzip())
                                    self.assertTrue(saved.namelist())
                                finally:
                                    saved.close()
                        finally:
                            if reader is not None and reader.returncode is None:
                                reader.kill()
                                await reader.wait()
                            temporary.cleanup()
                    print(f"Capture Host Story: {language}: Passed", flush=True)
                finally:
                    current = await client._transport.plugins.get(created.entry.name)
                    await client._transport.plugins.delete(created.entry.name, current.entry.revision)
            finally:
                if process.returncode is None:
                    process.terminate()
                await process.wait()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
