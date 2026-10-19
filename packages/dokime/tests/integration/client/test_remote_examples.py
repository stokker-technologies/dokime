# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Example Cancellation And Recovery Boundaries"""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
import runpy
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock

# Third Party Imports
import pytest

# Local Imports
from dokime.client import RpcError

# Type Checking Imports


# === Public Exports === #

# === Constants === #

EXAMPLES = Path(__file__).resolve().parents[3] / "examples/remote/python"
CYCLE = "00000000-0000-0000-0000-000000000001"
EXECUTION = "00000000-0000-0000-0000-000000000002"

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("script", ["control-test", "control-prompt"])
@pytest.mark.parametrize("disconnected", [False, True])
def test_interrupted_submission_stops_own_execution_without_retry(
    script: str, disconnected: bool, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    run = runpy.run_path(str(EXAMPLES / f"{script}.py"))["run"]
    saved = SimpleNamespace(path=tmp_path / "cycle.zip", sha256="fixture-checksum")
    cycle = SimpleNamespace(seal=AsyncMock(), export=AsyncMock(return_value=saved))
    execution = SimpleNamespace(
        id=EXECUTION,
        start=AsyncMock(side_effect=asyncio.CancelledError),
        checkpoint=AsyncMock(return_value=None),
        stop=AsyncMock(),
        wait=AsyncMock(return_value=SimpleNamespace(status="cancelled")),
        export=AsyncMock(return_value=saved),
    )
    client = SimpleNamespace(
        create_cycle=AsyncMock(return_value=CYCLE),
        append_cases=AsyncMock(return_value=(execution,)),
        cycle=AsyncMock(return_value=cycle),
    )
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", MagicMock(return_value=connection))
    arguments = argparse.Namespace(
        endpoint="https://fixture.invalid:8765",
        ca_certificate=None,
        key_file=None,
        env_file=None,
        directory=tmp_path,
        overwrite=False,
        sample_count=8,
        answer="yes",
    )

    if disconnected:
        execution.stop.side_effect = RpcError("UNAVAILABLE", "connection lost during cancellation")
        with pytest.raises(RpcError, match="connection lost during cancellation"):
            asyncio.run(run(arguments))
        execution.wait.assert_not_awaited()
        execution.export.assert_not_awaited()
        cycle.seal.assert_not_awaited()
        cycle.export.assert_not_awaited()
    else:
        assert asyncio.run(run(arguments)) == 130
        execution.wait.assert_awaited_once()
        filename = "execution-1.zip" if script == "control-test" else "execution.zip"
        execution.export.assert_awaited_once_with(tmp_path / filename, overwrite=False)
        cycle.seal.assert_awaited_once()
        cycle.export.assert_awaited_once_with(tmp_path / "cycle.zip", overwrite=False)

    execution.stop.assert_awaited_once()
    execution.start.assert_awaited_once()
    client.create_cycle.assert_awaited_once()
    client.append_cases.assert_awaited_once()
    output = capsys.readouterr()
    assert f"Cycle: {CYCLE}" in output.out
    assert f"Execution: {EXECUTION} Request Key:" in output.out
    assert "Mode: automatic" in output.out


# === Private Functions === #

# === Entry Point === #
