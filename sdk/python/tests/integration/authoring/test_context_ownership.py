# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Keep Native Resources Shared and Refuse Closed Callback Handles"""

# === Imports === #

# Standard Library Imports
import asyncio
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# Third Party Imports
import dokime
import pytest

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_context_ownership(integration_loop: asyncio.Runner, tmp_path: Path) -> None:
    async def story() -> None:
        completed = asyncio.Event()
        retained: dict[str, Any] = {}

        @dataclass
        class Resource:
            values: list[int] = field(default_factory=list)

        @dataclass
        class Reply:
            values: list[int]
            payload: bytes | None

        @dokime.operation(id="ownership.copy", response=Reply, resources=[Resource], policy=dokime.OperationPolicy(5.0, 5.0))
        class Copy:
            value: int
            payload: bytes | None

            def run(self, ctx: dokime.Context) -> Reply:
                resource = ctx.resource(Resource)
                resource.values.append(self.value)
                retained["operation"] = ctx
                return Reply(resource.values, self.payload)

        @dokime.observer(id="copy", interval_ms=100)
        async def copy(ctx: dokime.Context) -> None:
            if completed.is_set():
                return
            retained["observer"] = ctx
            try:
                operation = Copy(2**53 + 1, b"\x00\xff")
                first = await ctx.operation(operation)
                first.values.clear()
                operation = Copy(-7, None)
                retained["reply"] = await ctx.operation(operation)
            finally:
                completed.set()

        resource = Resource()
        plugin = dokime.Plugin("ownership", "0.1.0", catalog=[Copy, copy], resources={Resource: resource})
        async with dokime.Backend(port=0, application_root=tmp_path, plugins=[plugin]):
            await completed.wait()
            assert retained["reply"] == Reply([2**53 + 1, -7], None)
            assert resource.values == [2**53 + 1, -7]

        for ctx in (retained["operation"], retained["observer"]):
            assert ctx.cancelled
            with pytest.raises(RuntimeError):
                operation = Copy(0, None)
                await ctx.operation(operation)
        assert resource.values == [2**53 + 1, -7]

    integration_loop.run(story())


# === Private Functions === #

# === Entry Point === #
