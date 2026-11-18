# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Compare the Four Installed SDKs' Shared Capture Description."""

# === Imports === #

# Standard Library Imports
import asyncio
import os
import subprocess
import sys
import unittest
from pathlib import Path

# isort: split

# Third Party Imports
from grpc.aio import insecure_channel
from harmos._wire.model.v1.description_pb2 import Declarations
from harmos._wire.plugin.v1.instance_pb2 import InstanceGetRequest, InstanceGetResponse

# isort: split

# Local Imports
from dokime import RecordContracts, Suites
from dokime_tools.process import spawn_flags
from dokime_tools.test import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class AuthoringEquivalence(unittest.IsolatedAsyncioTestCase):
    async def test_shared_capture_identifiers_and_record_contracts_match(self) -> None:
        root = Path(__file__).resolve().parents[2]
        extension = ".exe" if os.name == "nt" else ""
        configuration = os.environ.get("DOKIME_CSHARP_CONFIGURATION", "Release").lower()
        dotnet = subprocess.check_output(
            ["mise", "-C", str(root / "sdk/csharp"), "which", "dotnet"],
            text=True,
            creationflags=spawn_flags(),
        ).strip()
        commands = {
            "python": [sys.executable, str(root / "sdk/python/examples/test/plugin.py"), "--address", "127.0.0.1:0"],
            "rust": [str(root / f"target/debug/dokime-rust-test{extension}"), "--remote"],
            "csharp": [dotnet, str(root / f"target/cs/bin/Test/{configuration}/dokime-example-test.dll"), "--remote"],
            "cpp": [str(root / f"target/cp/tests/tests/dokime-story-plugin{extension}"), "--address", "127.0.0.1:0"],
        }
        descriptions = {}
        timeout = policy.load(root).deadline("integration")
        environment = dict(os.environ)
        environment.pop("HARMOS_ADDRESS", None)
        for language, command in commands.items():
            process = await asyncio.create_subprocess_exec(
                *command,
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                env=environment,
                creationflags=spawn_flags(),
            )
            try:
                async with asyncio.timeout(timeout):
                    assert process.stdout is not None
                    while True:
                        line = (await process.stdout.readline()).decode().strip()
                        if not line:
                            raise RuntimeError(f"Describe {language} Capture: Build and Correct the Example Plugin")
                        if line.startswith("harmos listening "):
                            address = line.removeprefix("harmos listening ")
                            break
                        if line.startswith("http://"):
                            address = line
                            break
                    channel = insecure_channel(address.removeprefix("http://"))
                    try:
                        get = channel.unary_unary(
                            "/harmos.plugin.v1.Instance/Get",
                            request_serializer=InstanceGetRequest.SerializeToString,
                            response_deserializer=InstanceGetResponse.FromString,
                        )
                        response = await get(InstanceGetRequest(revision=1, host="authoring_equivalence", host_version="1.0.0"))
                        descriptions[language] = contracts(response.declarations)
                    finally:
                        await channel.close()
            finally:
                if process.returncode is None:
                    process.terminate()
                await process.wait()
        expected = descriptions["python"]
        self.assertEqual(expected["operations"], ["capture.cleanup", "capture.confirm", "capture.confirmed", "capture.materialize"])
        self.assertEqual(expected["records"], [("capture.confirmation", "google.protobuf.BoolValue")])
        self.assertEqual(
            expected["suites"],
            [("capture", [("capture", "capture.materialize", "", "capture.cleanup", [("capture.confirm", "capture.confirm")])])],
        )
        for language, description in descriptions.items():
            self.assertEqual(description, expected, language)


# === Public Functions === #


def contracts(declarations: Declarations) -> dict[str, object]:
    suites = Suites()
    records = RecordContracts()
    for section in declarations.sections:
        if section.type == suites.DESCRIPTOR.full_name:
            suites.ParseFromString(section.payload)
        if section.type == records.DESCRIPTOR.full_name:
            records.ParseFromString(section.payload)
    return {
        "operations": sorted(operation.id for operation in declarations.operations),
        "streams": sorted(stream.id for stream in declarations.streams),
        "records": sorted((record.metadata.identifier, record.message) for record in records.records),
        "suites": [
            (
                suite.metadata.identifier,
                [
                    (
                        template.metadata.identifier,
                        template.materialize,
                        template.initialize,
                        template.cleanup,
                        [(step.metadata.identifier, step.operation) for step in template.steps],
                    )
                    for template in suite.templates
                ],
            )
            for suite in suites.suites
        ],
    }


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
