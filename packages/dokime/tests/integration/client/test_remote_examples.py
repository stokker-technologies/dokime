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
import asyncio
import json
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


def test_output_preflight(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    run = runpy.run_path(str(EXAMPLES / "control-device.py"))["run"]
    existing = tmp_path / "before.json"
    existing.write_text("original", encoding="utf-8")
    connect = MagicMock()
    monkeypatch.setitem(run.__globals__, "DokimeClient", connect)
    monkeypatch.setitem(run.__globals__, "RESPONSE_DIRECTORY", tmp_path)

    with pytest.raises(FileExistsError):
        asyncio.run(run())
    connect.assert_not_called()
    assert existing.read_text(encoding="utf-8") == "original"


@pytest.mark.parametrize("conflict", [False, True])
def test_response_files(conflict: bool, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    run = runpy.run_path(str(EXAMPLES / "control-device.py"))["run"]
    off = {"enabled": False, "voltage_rms_v": 0.0}
    on = {"enabled": True, "voltage_rms_v": 230.0}
    directory = tmp_path / "new responses"
    replies = iter([off, on, on, off, off])

    async def reply(operation: str, request: dict) -> dict:
        if conflict and operation == "obc.device.set_power" and request["enabled"]:
            (directory / "power-on.json").write_text("concurrent output", encoding="utf-8")
        return next(replies)

    call = AsyncMock(side_effect=reply)
    client = SimpleNamespace(operations=AsyncMock(return_value=[]), call=call)
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", MagicMock(return_value=connection))
    monkeypatch.setitem(run.__globals__, "MessageToDict", lambda message, **_options: message)
    monkeypatch.setitem(run.__globals__, "RESPONSE_DIRECTORY", directory)

    if conflict:
        with pytest.raises(FileExistsError):
            asyncio.run(run())
        assert (directory / "power-on.json").read_text(encoding="utf-8") == "concurrent output"
        assert [item.args for item in call.await_args_list] == [
            ("obc.device.read", {}),
            ("obc.device.set_power", {"enabled": True}),
        ]
        return

    assert asyncio.run(run()) == 0
    for name, expected in [("before", off), ("power-on", on), ("running", on), ("power-off", off), ("after", off)]:
        saved = json.loads((directory / f"{name}.json").read_text(encoding="utf-8"))
        assert saved == expected
    assert [item.args for item in call.await_args_list] == [
        ("obc.device.read", {}),
        ("obc.device.set_power", {"enabled": True}),
        ("obc.device.read", {}),
        ("obc.device.set_power", {"enabled": False}),
        ("obc.device.read", {}),
    ]


@pytest.mark.parametrize("script", ["control-test", "control-prompt"])
@pytest.mark.parametrize("disconnected", [False, True])
def test_interrupted_submission(
    script: str, disconnected: bool, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    run = runpy.run_path(str(EXAMPLES / f"{script}.py"))["run"]
    saved = SimpleNamespace(path=tmp_path / "cycle.zip", sha256="fixture-checksum")
    cycle = SimpleNamespace(id=CYCLE, append=AsyncMock(), seal=AsyncMock(), export=AsyncMock(return_value=saved))
    execution = SimpleNamespace(
        id=EXECUTION,
        start=AsyncMock(side_effect=asyncio.CancelledError),
        run=AsyncMock(side_effect=asyncio.CancelledError),
        checkpoint=AsyncMock(return_value=None),
        stop=AsyncMock(),
        wait=AsyncMock(return_value=SimpleNamespace(status="cancelled")),
        export=AsyncMock(return_value=saved),
    )
    cleanup = MagicMock()
    cleanup.attach_mock(execution.stop, "stop")
    cleanup.attach_mock(cycle.seal, "seal")
    cleanup.attach_mock(execution.wait, "wait")
    cleanup.attach_mock(execution.export, "execution_export")
    cleanup.attach_mock(cycle.export, "cycle_export")
    client = SimpleNamespace(
        template=AsyncMock(return_value=SimpleNamespace(materialize=AsyncMock(return_value=(object(),)))),
        create_cycle=AsyncMock(return_value=cycle),
    )
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", MagicMock(return_value=connection))
    monkeypatch.setitem(run.__globals__, "Execution", MagicMock(return_value=execution))
    monkeypatch.setitem(run.__globals__, "OUTPUT_DIRECTORY", tmp_path)
    monkeypatch.setitem(run.__globals__, "EXECUTION_COUNT", 1)

    if disconnected:
        execution.stop.side_effect = RpcError("UNAVAILABLE", "connection lost during cancellation")
        with pytest.raises(RpcError, match="connection lost during cancellation"):
            asyncio.run(run())
        execution.wait.assert_not_awaited()
        execution.export.assert_not_awaited()
        cycle.seal.assert_not_awaited()
        cycle.export.assert_not_awaited()
        assert [event[0] for event in cleanup.mock_calls] == ["stop"]
    else:
        assert asyncio.run(run()) == 130
        execution.wait.assert_awaited_once()
        filename = "execution-1.zip" if script == "control-test" else "execution.zip"
        execution.export.assert_awaited_once_with(tmp_path / filename, overwrite=False)
        cycle.seal.assert_awaited_once()
        cycle.export.assert_awaited_once_with(tmp_path / "cycle.zip", overwrite=False)
        assert [event[0] for event in cleanup.mock_calls] == ["stop", "seal", "wait", "execution_export", "cycle_export"]

    execution.stop.assert_awaited_once()
    if script == "control-test":
        execution.run.assert_awaited_once()
        execution.start.assert_not_awaited()
    else:
        execution.start.assert_awaited_once()
        execution.run.assert_not_awaited()
    client.create_cycle.assert_awaited_once()
    cycle.append.assert_awaited_once_with([execution])
    output = capsys.readouterr()
    assert f"Cycle: {CYCLE}" in output.out
    assert f"Execution: {EXECUTION}" in output.out
    assert "Request Key" not in output.out


def test_append_cancellation(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    run = runpy.run_path(str(EXAMPLES / "control-prompt.py"))["run"]
    saved = SimpleNamespace(path=tmp_path / "cycle.zip", sha256="fixture-checksum")
    cycle = SimpleNamespace(
        id=CYCLE,
        append=AsyncMock(side_effect=asyncio.CancelledError),
        seal=AsyncMock(),
        export=AsyncMock(return_value=saved),
    )
    execution = SimpleNamespace(id=EXECUTION, checkpoint=AsyncMock(), stop=AsyncMock(), export=AsyncMock())
    client = SimpleNamespace(
        template=AsyncMock(return_value=SimpleNamespace(materialize=AsyncMock(return_value=(object(),)))),
        create_cycle=AsyncMock(return_value=cycle),
    )
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", MagicMock(return_value=connection))
    monkeypatch.setitem(run.__globals__, "Execution", MagicMock(return_value=execution))
    monkeypatch.setitem(run.__globals__, "OUTPUT_DIRECTORY", tmp_path)

    assert asyncio.run(run()) == 130
    execution.checkpoint.assert_not_awaited()
    execution.stop.assert_not_awaited()
    execution.export.assert_not_awaited()
    cycle.seal.assert_awaited_once()
    cycle.export.assert_awaited_once_with(tmp_path / "cycle.zip", overwrite=False)


# === Private Functions === #

# === Entry Point === #
