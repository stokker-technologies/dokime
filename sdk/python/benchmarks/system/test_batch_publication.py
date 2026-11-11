# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Measure Bridge Preparation Separately from Admitted Publication"""

# === Imports === #

# Standard Library Imports
import asyncio
import importlib.machinery
import importlib.util
import os
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

# Third Party Imports
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


@pytest.mark.parametrize("library", ["numpy", "arrow"])
def test_batch_publication(benchmark: Any, library: str) -> None:
    # Load The Normal Extension Built By Cargo In The Bench Profile.
    root = Path(__file__).resolve().parents[4]
    filename = "_native.dll" if os.name == "nt" else "lib_native.so"
    extension = root / "target/release" / filename
    native = sys.modules.get("dokime._native")
    if native is None:
        loader = importlib.machinery.ExtensionFileLoader("dokime._native", str(extension))
        spec = importlib.util.spec_from_loader(loader.name, loader)
        if spec is None:
            raise RuntimeError("could not load the bench extension; build it with the documented focused task")
        native = importlib.util.module_from_spec(spec)
        sys.modules[loader.name] = native
        loader.exec_module(native)
    if Path(native.__file__).resolve() != extension.resolve():
        raise RuntimeError("the Python benchmark requires the bench-profile native extension")
    import dokime

    # Prepare Equal Workloads Before Timing
    rows = 10000
    batch = 1000
    columns = (
        {"counter": np.arange(batch, dtype=np.int64), "value": np.arange(batch, dtype=np.float64)}
        if library == "numpy"
        else {"counter": pa.array(range(batch)), "value": pa.array([float(i) for i in range(batch)])}
    )
    measured: list[tuple[float, float]] = []

    async def story() -> None:
        ready = asyncio.Event()
        completed = asyncio.Event()
        failure: list[BaseException] = []

        @dokime.stream(id="batch.samples", label="Samples", window=10000)
        class Reading:
            counter: int
            value: float

        @dokime.observer(id="publish", interval_ms=100, streams=[Reading])
        async def publish(ctx: dokime.Context) -> None:
            await ready.wait()
            if completed.is_set():
                return
            bridge_s = 0.0
            publication_s = 0.0
            try:
                for _ in range(rows // batch):
                    started = time.perf_counter()
                    publication = ctx._inner.push_columns("batch.samples", columns)
                    prepared = time.perf_counter()
                    await publication
                    finished = time.perf_counter()
                    bridge_s += prepared - started
                    publication_s += finished - prepared
                measured.append((bridge_s, publication_s))
            except (RuntimeError, ValueError, TypeError) as error:
                failure.append(error)
            finally:
                completed.set()

        plugin = dokime.Plugin("batch", "0.1.0", catalog=[Reading, publish])
        with TemporaryDirectory(prefix="dokime-batch-") as root:
            async with dokime.Backend(port=0, application_root=root, plugins=[plugin]):
                ready.set()
                await completed.wait()
        if failure:
            raise failure[0]

    def measure() -> None:
        asyncio.run(story())

    benchmark.pedantic(measure, rounds=3, iterations=1)
    bridge_s = sum(item[0] for item in measured) / len(measured)
    publication_s = sum(item[1] for item in measured) / len(measured)
    benchmark.extra_info.update(
        native_profile="bench",
        rows=rows,
        batch=batch,
        rows_s=rows / (bridge_s + publication_s),
        bridge_s=bridge_s,
        publication_s=publication_s,
    )
    print(
        f"AUTHORING_BATCH {library} rows_s={rows / (bridge_s + publication_s):.0f} bridge_s={bridge_s:.6f} publication_s={publication_s:.6f}"
    )


# === Private Functions === #

# === Entry Point === #
