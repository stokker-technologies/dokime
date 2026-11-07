# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Reject Invalid Batches Atomically and Preserve Portable Column Values"""

# === Imports === #

# Standard Library Imports
import asyncio
from pathlib import Path

# Third Party Imports
import dokime
import numpy as np
import pyarrow as pa
import pytest

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_column_conversions(tmp_path: Path) -> None:
    async def story() -> None:
        retained = []

        @dokime.stream(id="conversion.samples", label="Samples", window=1000)
        class Row:
            index: int
            voltage_v: float | None
            label: str
            payload: bytes

        @dokime.template(id="conversion.capture", label="Capture", streams=[Row])
        class Template:
            @dokime.step()
            async def capture(self, ctx: dokime.Context) -> None:
                writer = ctx.stream(Row).open()
                retained.append(writer)
                valid = {
                    "index": np.array([2**53 + 1, -7], dtype=np.int64),
                    "voltage_v": pa.array([1.25, None], type=pa.float64()),
                    "label": pa.array(["µV", "空"]),
                    "payload": pa.array([b"\x00\xff", b""], type=pa.binary()),
                }
                for invalid in (
                    valid | {"index": np.array([1], dtype=np.int64)},
                    valid | {"index": np.array([[1, 2]], dtype=np.int64)},
                    valid | {"index": pa.array([None, 1], type=pa.int64())},
                    valid | {"voltage_v": [float("nan"), 1.0]},
                    {"unknown": [1, 2]},
                ):
                    with pytest.raises((ValueError, TypeError)):
                        await writer.push_columns(invalid)
                with pytest.raises(TypeError):
                    await writer.push(object())
                await writer.push_columns(valid)
                await writer.push_columns({name: values[:0] for name, values in valid.items()})
                await writer.push(Row(0, -0.5, "tail", b"\x10"))

        @dokime.suite(id="conversion", label="Conversion", templates=[Template])
        class Suite:
            pass

        plugin = dokime.Plugin("conversion", "0.1.0", catalog=[Row, Suite])
        async with dokime.Backend(port=0, application_root=tmp_path, plugins=[plugin]) as backend, backend.client() as client:
            template = await client.template("conversion.capture")
            (case,) = await template.materialize()
            execution = dokime.Execution(case)
            cycle = await client.create_cycle(label="Conversions", continue_automatically=False)
            await cycle.append([execution])
            await execution.start()
            outcome = await execution.wait()
            assert outcome.status == "passed"
            rows = [
                {name: row.values[name] for name in ("index", "label", "payload")} | {"voltage_v": row.values.get("voltage_v")}
                async for row in execution.samples("conversion.samples")
            ]
            assert rows == [
                {"index": 2**53 + 1, "voltage_v": 1.25, "label": "µV", "payload": [0, 255]},
                {"index": -7, "voltage_v": None, "label": "空", "payload": []},
                {"index": 0, "voltage_v": -0.5, "label": "tail", "payload": [16]},
            ]
        with pytest.raises(RuntimeError):
            await retained[0].push(Row(1, 1.0, "late", b""))

    asyncio.run(story())


# === Private Functions === #

# === Entry Point === #
