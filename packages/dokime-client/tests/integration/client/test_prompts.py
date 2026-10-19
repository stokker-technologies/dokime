# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Read Pending Operator Prompts Of A Cycle"""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import ProtocolError, RpcError, _wire

from .backend import CYCLE_ID, EXECUTION_ID
from .capture_backend import serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PROMPT_ID = "0199d6a4-6f1e-7c3a-9a51-2b7c4e8f1d20"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_prompt_selection() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            cycle = await client.cycle(CYCLE_ID)
            backend.prompt_stream = "unrelated-selection"
            with pytest.raises(ProtocolError):
                await anext(cycle.pending_prompts())
            backend.prompt_stream = "prompts.pending:c4e98c8f616061bebf0d5119b0b549f764f8207d9a4660c207dcff2523da467b"
            assert [item async for item in cycle.pending_prompts()] == []
            assert backend.created == backend.run_calls == 0

    asyncio.run(scenario())


def test_prompt_level() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            # Declare A Warning Prompt
            backend.pending.append(
                {
                    "prompt_id": PROMPT_ID,
                    "kind": {"Form": {}},
                    "definition": {
                        "prompt_type": "obc.charger.steady.check",
                        "schema_version": 1,
                        "archetype": {"Form": {}},
                        "title": "Inspect Charging State",
                        "message": "Check that the charger status indicator is steady.",
                        "level": {"Warn": {}},
                    },
                    "values": _wire.encode({}),
                    "scope": {"cycle_id": CYCLE_ID},
                    "created_at": {"seconds": 1_760_000_000, "nanos": 0},
                }
            )

            # Read It Back
            cycle = await client.cycle(CYCLE_ID)
            for declared, expected in (
                ("Warn", "warn"),
                ("Info", "info"),
                ("Unspecified", "info"),
            ):
                backend.pending[0]["definition"]["level"] = {declared: {}}
                (prompt,) = [item async for item in cycle.pending_prompts()]
                assert prompt.id == PROMPT_ID
                assert prompt.title == "Inspect Charging State"
                assert prompt.level == expected

    asyncio.run(scenario())


def test_execution_prompt_scope() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            # Read Only The Selected Execution's Question.
            own = {
                "prompt_id": PROMPT_ID,
                "kind": {"Confirmation": {}},
                "definition": {
                    "prompt_type": "obc.checks.pulse.confirm",
                    "schema_version": 1,
                    "archetype": {"Confirmation": {}},
                    "title": "Start Simulated Pulse",
                    "level": {"Info": {}},
                    "actions": [{"id": "yes", "label": "Yes"}, {"id": "no", "label": "No"}],
                },
                "values": _wire.encode({}),
                "scope": {"cycle_id": CYCLE_ID, "execution_id": EXECUTION_ID},
                "created_at": {"seconds": 1_760_000_000, "nanos": 0},
            }
            foreign_id = "0199d6a4-6f1e-7c3a-9a51-2b7c4e8f1d21"
            foreign = {**own, "prompt_id": foreign_id, "scope": {"cycle_id": CYCLE_ID, "execution_id": foreign_id}}
            backend.pending.extend([own, foreign])
            execution = await client.execution(EXECUTION_ID)
            (prompt,) = [item async for item in execution.pending_prompts()]
            assert prompt.id == PROMPT_ID

            # A Foreign Identity Cannot Reach Respond.
            with pytest.raises(ValueError, match="another execution"):
                await execution.prompt(foreign_id)
            with pytest.raises(ValueError, match="another execution"):
                await execution.respond(foreign_id, action="yes")
            assert backend.response_calls == []

            # Inspect The Committed Response After A Duplicate Is Refused.
            await execution.respond(PROMPT_ID, action="yes")
            assert (await execution.prompt(PROMPT_ID))["response"]["action"] == "yes"
            assert [item async for item in execution.pending_prompts()] == []
            with pytest.raises(RpcError):
                await execution.respond(PROMPT_ID, action="no")
            assert backend.prompt_responses[PROMPT_ID]["action"] == "yes"
            assert backend.created == backend.run_calls == 0

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
