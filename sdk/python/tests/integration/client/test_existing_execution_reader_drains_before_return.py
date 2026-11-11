# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Existing Execution Readers Drain Before Returning"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import runpy
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock

# Third Party Imports
import pytest

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

EXAMPLES = Path(__file__).resolve().parents[3] / "examples"
CYCLE = "00000000-0000-0000-0000-000000000001"
EXECUTION = "00000000-0000-0000-0000-000000000002"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_existing_execution_reader_drains_before_return(
    integration_loop: asyncio.Runner, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    namespace = runpy.run_path(str(EXAMPLES / "remote_control.py"))
    run = namespace["run"]
    monkeypatch.chdir(tmp_path)
    drained: list[str] = []
    record_ready = asyncio.Event()

    class ExistingExecution:
        id = EXECUTION
        start = AsyncMock()

        async def records(self):
            drained.append("record")
            record_ready.set()
            yield SimpleNamespace(to_json=lambda: json.dumps({"sequence": 1}))

        async def samples(self):
            await record_ready.wait()
            drained.append("sample")
            yield SimpleNamespace(to_json=lambda: json.dumps({"sequence": 2}))

        def __await__(self):
            async def saved():
                return SimpleNamespace(status="passed", checkpoint=SimpleNamespace(execution_id=EXECUTION), exit_code=0)

            return saved().__await__()

    execution = ExistingExecution()
    client = SimpleNamespace(
        connect=AsyncMock(return_value=SimpleNamespace(id=CYCLE, epoch="epoch")),
        execution=AsyncMock(return_value=execution),
    )
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", SimpleNamespace(from_pairing=lambda _path: connection))

    assert integration_loop.run(run(EXECUTION)) == 0
    client.execution.assert_awaited_once_with(EXECUTION)
    execution.start.assert_awaited_once()
    assert set(drained) == {"record", "sample"}
    assert json.loads((tmp_path / "execution.json").read_text(encoding="utf-8")) == {"execution_id": EXECUTION, "status": "passed"}


# === Private Functions === #

# === Entry Point === #
