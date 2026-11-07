# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, Awaitable, Sequence

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from .._transport import Transport
from ..models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

FILTER_BYTES: int
FILTER_ITEMS: int
MAXIMUM_SAMPLE_PAGE_ITEMS: int

# === Types === #

# === Classes === #

# === Public Functions === #

def read_records(
    transport: Transport,
    backend: Backend,
    scope: Message,
    record_types: Sequence[str],
    completion: Awaitable[object] | None,
) -> AsyncGenerator[Message]: ...
def read_samples(
    transport: Transport,
    backend: Backend,
    scope: Message,
    streams: Sequence[str],
    page_size: int,
    follow: bool,
) -> AsyncGenerator[Message]: ...
def validate_patterns(patterns: Sequence[str]) -> None: ...
def _records(
    epoch: str,
    page: Message,
    after: Message | None,
) -> None: ...
