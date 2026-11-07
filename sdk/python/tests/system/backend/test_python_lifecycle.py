# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate, Pair, Capture and Reopen Python Evidence Without Its Plugin"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import runpy
import subprocess
from contextlib import aclosing
from pathlib import Path

# Third Party Imports
import dokime
import pytest

# Local Imports
from dokime.client import Execution

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_python_lifecycle(host: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Generate the Project with the Built CLI
    project = tmp_path / "bench"
    result = subprocess.run([str(host), "new", str(project), "--lang", "python"], capture_output=True, check=False)
    assert result.returncode == 0, result.stderr.decode()
    # Refuse to Overwrite an Existing Project
    before = {path.relative_to(project): path.read_bytes() for path in project.rglob("*") if path.is_file()}
    result = subprocess.run([str(host), "new", str(project), "--lang", "python"], capture_output=True, check=False)
    assert result.returncode != 0
    assert {path.relative_to(project): path.read_bytes() for path in project.rglob("*") if path.is_file()} == before

    # Load the Generated Plugin
    monkeypatch.syspath_prepend(str(project))
    namespace = runpy.run_path(str(project / "main.py"))
    plugin = namespace["create"]()

    async def story() -> None:
        application = tmp_path / "application"
        pairing = tmp_path / "bench.dokime-remote"

        # Start and Pair the Python Backend
        async with dokime.Backend(port=0, application_root=application, plugins=[plugin]) as backend:
            root = backend.root
            backend.export_pairing(pairing, label="Bench")
            async with dokime.DokimeClient.from_pairing(pairing) as client:
                suite = await client.suite("bench")
                template = await client.template(suite.templates[0].id)
                (case,) = await template.materialize(parameters={"samples": 4})
                execution = Execution(case)
                cycle = await client.create_cycle(label="Python Capture", continue_automatically=False)
                await cycle.append([execution])
                await execution.start()

                # Answer the Published Prompt
                async with aclosing(execution.records()) as records:
                    async for record in records:
                        if record.details.get("prompt_type") == "bench.confirm" and "prompt_id" in record.details:
                            await execution.respond(str(record.details["prompt_id"]), action="submit", values={"connected": True})
                            break

                # Read the Saved Measurements and Seal the Report
                outcome = await execution.wait()
                assert outcome.status == "passed"
                rows = [row async for row in execution.samples("bench.readings", page_size=2)]
                assert [(row.values["index"], row.values["voltage_v"]) for row in rows] == [(i, i * 0.25) for i in range(4)]
                await cycle.seal()
                report = await cycle.download_report(tmp_path / "original.html")
                original = report.path.read_bytes()
                assert original
                assert hashlib.sha256(original).hexdigest() == report.sha256
                cycle_id = cycle.id
                execution_id = execution.id

        # Reopen the Root Without Registering the Ephemeral Plugin
        async with dokime.Backend(port=0, root=root, application_root=application) as backend, backend.client() as client:
            suites = [suite async for suite in client.suites()]
            assert not any(suite.id == "bench" for suite in suites)
            cycle = await client.cycle(cycle_id)
            report = await cycle.download_report(tmp_path / "reopened.html")
            assert report.path.read_bytes() == original
            execution = await client.execution(execution_id)
            rows = [row async for row in execution.samples("bench.readings", page_size=1)]
            assert [(row.values["index"], row.values["voltage_v"]) for row in rows] == [(i, i * 0.25) for i in range(4)]

    asyncio.run(story())


# === Private Functions === #

# === Entry Point === #
