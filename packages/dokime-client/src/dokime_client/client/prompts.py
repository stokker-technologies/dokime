# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Read And Answer Identity-Bound Operator Prompts"""

import hashlib
import json
from collections.abc import AsyncGenerator, Mapping

from google.protobuf.message import Message

from . import _wire
from ._transport import Transport, _cursor
from .errors import ProtocolError
from .execution.request import _identity
from .models import Backend, Prompt, Value

PAGE_ITEMS = 100
VALUE_DEPTH = 64


class Scope:
    """One Prompt Selection Over The Existing Native Prompt Protocol."""

    def __init__(self, transport: Transport, backend: Backend, cycle_id: str, execution_id: str | None = None) -> None:
        _identity(cycle_id)
        if execution_id is not None:
            _identity(execution_id)
        self.transport = transport
        self.backend = backend
        self.cycle_id = cycle_id
        self.execution_id = execution_id

    async def read(self, prompt_id: str) -> Message:
        _identity(prompt_id)
        await self.transport.verify(self.backend, live=True)
        view = await self.transport.unary("Prompts", "GetPrompt", _wire.request("Prompts", "GetPrompt", prompt_id=prompt_id))
        _cursor(self.backend, view)
        if (
            not view.HasField("prompt")
            or view.prompt.prompt_id != prompt_id
            or not view.prompt.HasField("definition")
            or not view.prompt.definition.prompt_type
            or view.HasField("response")
            and view.response.prompt_id != prompt_id
        ):
            raise ProtocolError("prompt view differs from admitted identity")
        self.validate_scope(view.prompt)
        return view

    async def pending(self) -> AsyncGenerator[Prompt]:
        request = _wire.request("Prompts", "ListPending", cycle_id=self.cycle_id, limit=PAGE_ITEMS)
        if self.execution_id is not None:
            request.execution_id = self.execution_id
        selection = json.dumps([self.cycle_id, self.execution_id], separators=(",", ":")).encode()
        stream = "prompts.pending:" + hashlib.sha256(selection).hexdigest()
        while True:
            await self.transport.verify(self.backend, live=True)
            page = await self.transport.unary("Prompts", "ListPending", request)
            self.validate_page(request, page, stream)
            for identity in page.prompts:
                view = await self.read(identity.prompt_id)
                _validate_listed(identity, view.prompt)
                if not view.HasField("response"):
                    yield _model(view.prompt)
            if not page.HasField("next_after_prompt_id"):
                return
            request.after_prompt_id = page.next_after_prompt_id
            _wire.copy(request.cursor, page.cursor)

    async def respond(self, prompt_id: str, *, action: str, values: Mapping[str, Value] | None, accepted: bool) -> None:
        encoded = _wire.encode(dict(values or {}), VALUE_DEPTH)
        await self.read(prompt_id)
        await self.transport.verify(self.backend, live=True)
        request = _wire.request("Prompts", "Respond", prompt_id=prompt_id, action=action, values=encoded, accepted=accepted)
        receipt = await self.transport.unary("Prompts", "Respond", request)
        _cursor(self.backend, receipt)
        try:
            _identity(receipt.command_id)
        except ValueError as error:
            raise ProtocolError("prompt receipt command identity is invalid") from error
        if (
            receipt.prompt_id != prompt_id
            or not receipt.committed_revision
            or receipt.observed_cursor.position < receipt.committed_revision
        ):
            raise ProtocolError("prompt receipt differs from submitted response")

    def validate_scope(self, prompt: Message) -> None:
        if not prompt.HasField("scope") or prompt.scope.cycle_id != self.cycle_id:
            raise ValueError("prompt belongs to another cycle")
        if self.execution_id is not None and prompt.scope.execution_id != self.execution_id:
            raise ValueError("prompt belongs to another execution")

    def validate_page(self, request: Message, page: Message, stream: str) -> None:
        _cursor(self.backend, page, "cursor")
        changed = request.HasField("cursor") and not _wire.equal(request.cursor, page.cursor)
        if len(page.prompts) > request.limit or page.cursor.stream != stream or changed:
            raise ProtocolError("pending prompt page changed cursor or exceeded limit")
        previous = request.after_prompt_id
        for identity in page.prompts:
            try:
                _identity(identity.prompt_id)
            except ValueError as error:
                raise ProtocolError("pending prompt identity is invalid") from error
            self.validate_scope(identity)
            if identity.prompt_id <= previous:
                raise ProtocolError("pending prompt identity does not advance within selection")
            previous = identity.prompt_id
        if page.HasField("next_after_prompt_id") and (not page.prompts or page.next_after_prompt_id != previous):
            raise ProtocolError("pending prompt continuation is invalid")


def _validate_listed(identity: Message, item: Message) -> None:
    same_definition = identity.HasField("definition") and (
        item.definition.prompt_type,
        item.definition.schema_version,
        item.definition.title,
    ) == (identity.definition.prompt_type, identity.definition.schema_version, identity.definition.title)
    if not _wire.equal(item.scope, identity.scope) or not _wire.equal(item.created_at, identity.created_at) or not same_definition:
        raise ProtocolError("prompt view differs from listed request")


def _model(item: Message) -> Prompt:
    level = _wire.enum_name(item.definition.level)
    return Prompt(
        item.prompt_id,
        item.definition.prompt_type,
        item.definition.title,
        item.definition.message if item.definition.HasField("message") else None,
        tuple(action.id for action in item.definition.actions),
        tuple(field.metadata.identifier for field in item.definition.fields),
        _wire.fields(item.values, VALUE_DEPTH),
        level="info" if level == "unspecified" else level,
    )
