# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""List And Resolve Suites By Readable Identity"""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
import pytest

# Local Imports
from dokime.client import ProtocolError

from .backend import SUITE_ID, TEMPLATE, TEMPLATE_ID, serve
from .canonical import _message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_readable_catalog() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            # Suites And Templates List And Resolve By Their Readable IDs
            suites = [suite async for suite in client.suites()]
            assert [suite.id for suite in suites] == [SUITE_ID]
            assert [template.id for template in suites[0].templates] == [TEMPLATE]
            assert (await client.suite(SUITE_ID)).templates[0].id == TEMPLATE

            # The Registered UUID Is Wire Identity, Never A Template Reference
            with pytest.raises(ValueError):
                await client.new_cycle(suite=SUITE_ID, template=TEMPLATE_ID)
            assert server.append_calls == 0
            assert server.created == server.run_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize(
    ("page", "continuation"),
    [
        ("duplicate", None),
        ("oversized", None),
        ("single", "incorrect-continuation"),
        ("invalid", None),
    ],
)
def test_suite_page_refused(page: str, continuation: str | None) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            valid = _suite(SUITE_ID)
            identities = {
                "duplicate": [valid, valid],
                "oversized": [valid] * 101,
                "single": [valid],
                "invalid": [valid, _suite("bad suite")],
            }[page]
            server.suite_page = _message(
                "Catalog",
                "ListSuites",
                direction="response",
                path="",
                suites=identities,
                cursor=_message(
                    "Evidence",
                    "ListRecords",
                    direction="request",
                    path="after",
                    epoch=server.epoch,
                    stream="runtime.state",
                    position=1,
                ),
            )
            if continuation is not None:
                server.suite_page.next_after_suite_id = continuation

            reader = client._suite_identities()
            with pytest.raises(ProtocolError):
                await anext(reader)
            await reader.aclose()
            assert server.created == server.run_calls == 0

    asyncio.run(scenario())


# === Private Functions === #


def _suite(suite_id: str):
    return _message(
        "Catalog", "ListSuites", direction="response", path="suites", suite_id=suite_id
    )


# === Entry Point === #
