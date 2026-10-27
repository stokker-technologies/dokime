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
from contextlib import nullcontext
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import ResultSummary, RpcError

# Type Checking Imports


# === Public Exports === #

# === Constants === #

EXAMPLES = Path(__file__).resolve().parents[3] / "examples/remote/python"
CYCLE = "00000000-0000-0000-0000-000000000001"
EXECUTION = "00000000-0000-0000-0000-000000000002"

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("existing", [False, True])
def test_response_files(existing: bool, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    script_path = EXAMPLES / "control-device.py"
    namespace = runpy.run_path(str(script_path))
    run = namespace["run"]
    monkeypatch.chdir(tmp_path)
    if existing:
        (tmp_path / "before.json").write_text("previous reply", encoding="utf-8")
    off = {"enabled": False, "voltage_rms_v": 0.0}
    on = {"enabled": True, "voltage_rms_v": 230.0}
    call = AsyncMock(side_effect=[off, on, on, off, off])
    client = SimpleNamespace(operations=AsyncMock(return_value=[]), call=call)
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", MagicMock(return_value=connection))
    monkeypatch.setitem(run.__globals__, "MessageToDict", lambda message, **_options: message)

    assert asyncio.run(run()) == 0
    for name, expected in [("before", off), ("power-on", on), ("running", on), ("power-off", off), ("after", off)]:
        response = (tmp_path / f"{name}.json").read_text(encoding="utf-8")
        assert json.loads(response) == expected
    assert [item.args for item in call.await_args_list] == [
        ("obc.device.read", {}),
        ("obc.device.set_power", {"enabled": True}),
        ("obc.device.read", {}),
        ("obc.device.set_power", {"enabled": False}),
        ("obc.device.read", {}),
    ]
    connection.__aexit__.assert_awaited_once()


@pytest.mark.parametrize("script", ["control-test", "control-prompt"])
@pytest.mark.parametrize("disconnected", [False, True])
def test_interrupted_submission(
    script: str, disconnected: bool, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    script_path = EXAMPLES / f"{script}.py"
    namespace = runpy.run_path(str(script_path))
    run = namespace["run"]
    monkeypatch.chdir(tmp_path)
    saved_path = tmp_path / "cycle.zip"
    saved = SimpleNamespace(path=saved_path, sha256="fixture-checksum")
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
    test_case = object()
    materialize = AsyncMock(return_value=(test_case,))
    template = SimpleNamespace(materialize=materialize)
    client = SimpleNamespace(
        template=AsyncMock(return_value=template),
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
    expected = pytest.raises(RpcError, match="connection lost during cancellation") if disconnected else nullcontext()
    result = None
    with expected:
        result = asyncio.run(run())

    if disconnected:
        execution.wait.assert_not_awaited()
        execution.export.assert_not_awaited()
        cycle.seal.assert_not_awaited()
        cycle.export.assert_not_awaited()
        assert [event[0] for event in cleanup.mock_calls] == ["stop"]
    else:
        assert result == 130
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
    response = (tmp_path / "cycle.json").read_text(encoding="utf-8")
    assert json.loads(response) == {"cycle_id": CYCLE}
    client.create_cycle.assert_awaited_once()
    cycle.append.assert_awaited_once_with([execution])
    output = capsys.readouterr()
    assert f"Cycle: {CYCLE}" in output.out
    assert f"Execution: {EXECUTION}" in output.out
    assert "Request Key" not in output.out


def test_append_cancellation(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    script_path = EXAMPLES / "control-prompt.py"
    namespace = runpy.run_path(str(script_path))
    run = namespace["run"]
    monkeypatch.chdir(tmp_path)
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


def test_existing_execution(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    script_path = EXAMPLES.parents[1] / "remote_control.py"
    namespace = runpy.run_path(str(script_path))
    run = namespace["run"]
    monkeypatch.chdir(tmp_path)
    checkpoint = SimpleNamespace(execution_id=EXECUTION)
    outcome = SimpleNamespace(status="failed", checkpoint=checkpoint)
    execution = SimpleNamespace(id=EXECUTION, run=AsyncMock(return_value=outcome))
    backend = SimpleNamespace(id="backend", epoch="epoch")
    client = SimpleNamespace(
        connect=AsyncMock(return_value=backend),
        execution=AsyncMock(return_value=execution),
    )
    connection = MagicMock()
    connection.__aenter__ = AsyncMock(return_value=client)
    connection.__aexit__ = AsyncMock(return_value=False)
    monkeypatch.setitem(run.__globals__, "DokimeClient", MagicMock(return_value=connection))
    monkeypatch.setitem(run.__globals__, "EXECUTION_ID", EXECUTION)

    assert asyncio.run(run()) == 0
    client.execution.assert_awaited_once_with(EXECUTION)
    arguments = execution.run.await_args.kwargs
    assert set(arguments) == {"on_record", "on_sample", "timeout"}
    response = (tmp_path / "execution.json").read_text(encoding="utf-8")
    assert json.loads(response) == {
        "execution_id": EXECUTION,
        "status": "failed",
    }
    connection.__aexit__.assert_awaited_once()


def test_polling_example(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    script_path = EXAMPLES.parents[1] / "client/poll_run.py"
    namespace = runpy.run_path(str(script_path))
    main = namespace["main"]
    monkeypatch.chdir(tmp_path)

    async def samples(**_options):
        yield SimpleNamespace(stream="pulse")

    async def records(**_options):
        yield SimpleNamespace(type="run")

    result = ResultSummary(status="passed", passed=1, failed=0, blocked=0, skipped=0)
    outcome = SimpleNamespace(status="passed", result=result)
    execution = SimpleNamespace(
        id=EXECUTION,
        start=AsyncMock(),
        samples=samples,
        records=records,
        wait=AsyncMock(return_value=outcome),
    )
    cycle = SimpleNamespace(id=CYCLE, append=AsyncMock(), seal=AsyncMock())
    client = SimpleNamespace(
        template=AsyncMock(return_value=SimpleNamespace(materialize=AsyncMock(return_value=(object(),)))),
        create_cycle=AsyncMock(return_value=cycle),
        close=AsyncMock(),
    )
    monkeypatch.setitem(main.__globals__, "DokimeClient", MagicMock(return_value=client))
    monkeypatch.setitem(main.__globals__, "Execution", MagicMock(return_value=execution))

    asyncio.run(main())
    cycle.append.assert_awaited_once_with([execution])
    execution.start.assert_awaited_once_with()
    cycle.seal.assert_awaited_once()
    client.close.assert_awaited_once()
    response = (tmp_path / "summary.json").read_text(encoding="utf-8")
    summary = json.loads(response)
    assert summary["run"]["execution_id"] == EXECUTION
    assert summary["samples_by_stream"] == {"pulse": 1}
    assert summary["events_by_type"] == {"run": 1}


# === Private Functions === #

# === Entry Point === #
