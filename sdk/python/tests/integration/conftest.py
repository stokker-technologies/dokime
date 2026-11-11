# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Keep The Integration Event Loop Alive Until Worker Teardown"""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import Iterator

# Third Party Imports
import pytest

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.fixture(scope="session")
def integration_loop() -> Iterator[asyncio.Runner]:
    # Late gRPC Completions Must Retain A Live Loop Until Worker Teardown.
    with asyncio.Runner() as runner:
        yield runner


# === Private Functions === #

# === Entry Point === #
