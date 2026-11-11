# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, Mapping

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ._transport import Transport
from .models import Backend, Prompt, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PAGE_ITEMS: int
VALUE_DEPTH: int

# === Types === #

# === Classes === #

class Scope:
    transport: Transport
    backend: Backend
    cycle_id: str
    execution_id: str | None
    def __init__(self, transport: Transport, backend: Backend, cycle_id: str, execution_id: str | None = None) -> None: ...
    async def read(self, prompt_id: str) -> Message: ...
    def pending(self) -> AsyncGenerator[Prompt]: ...
    async def respond(self, prompt_id: str, *, action: str, values: Mapping[str, Value] | None, accepted: bool) -> None: ...
    def validate_scope(self, prompt: Message) -> None: ...
    def validate_page(self, request: Message, page: Message, stream: str) -> None: ...

def _validate_listed(identity: Message, item: Message) -> None: ...
def _model(item: Message) -> Prompt: ...
