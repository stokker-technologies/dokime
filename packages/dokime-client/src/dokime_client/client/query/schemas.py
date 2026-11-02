# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Typed Record Schema Discovery"""

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator
from dataclasses import dataclass
from enum import StrEnum
from typing import Any, cast

# Third Party Imports
# Local Imports
from ..._wire.dokime.v1.CatalogMessages.GetRecordSchema import messages_pb2 as definitions
from ..._wire.dokime.v1.CatalogMessages.ListRecordSchemas import messages_pb2 as catalog
from .. import _wire
from .._transport import Transport, _cursor
from ..errors import ProtocolError
from .selection import positive

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RecordValueType(StrEnum):
    """Declared Record Field Types With Stable Lowercase String Values."""

    UNSPECIFIED = "unspecified"
    BOOLEAN = "boolean"
    INTEGER = "integer"
    NUMBER = "number"
    TEXT = "text"
    FILE = "file"
    OBJECT = "object"
    ARRAY = "array"
    DATETIME = "datetime"


@dataclass(frozen=True, slots=True)
class RecordField:
    identifier: str
    value_type: RecordValueType
    label: str | None
    unit: str | None
    required: bool
    nullable: bool


@dataclass(frozen=True, slots=True)
class RecordSchema:
    record_type: str
    schema_version: int
    label: str | None
    fields: tuple[RecordField, ...]


# === Public Functions === #


def record_schemas(transport: Transport, *, page_rows: int) -> AsyncGenerator[RecordSchema]:
    positive(page_rows, "page_rows", 2**32 - 1)
    return _schemas(transport, page_rows)


# === Private Functions === #


async def _schemas(transport: Transport, page_rows: int) -> AsyncGenerator[RecordSchema]:
    backend = await transport.connect()
    request = cast(catalog.Row1, _wire.request("Catalog", "ListRecordSchemas", limit=page_rows))
    while True:
        await transport.verify(backend, live=True)
        page = cast(catalog.Row2, await transport.unary("Catalog", "ListRecordSchemas", request))
        _cursor(backend, page, "cursor")
        if request.HasField("cursor") and not _wire.equal(request.cursor, page.cursor):
            raise ProtocolError("Record catalog page changed revision")
        if len(page.schemas) > request.limit:
            raise ProtocolError("Record catalog page exceeds its limit")
        previous = request.after_record_type
        for identity in page.schemas:
            if not identity.record_type.strip() or identity.record_type <= previous:
                raise ProtocolError("Record schema identities do not advance")
            previous = identity.record_type
        if page.HasField("next_after_record_type") and (not page.schemas or page.next_after_record_type != previous):
            raise ProtocolError("Invalid record catalog continuation")
        for identity in page.schemas:
            await transport.verify(backend, live=True)
            schema = cast(
                definitions.Row2,
                await transport.unary(
                    "Catalog", "GetRecordSchema", _wire.request("Catalog", "GetRecordSchema", record_type=identity.record_type)
                ),
            )
            if (
                schema.record_type != identity.record_type
                or schema.schema_version != identity.schema_version
                or schema.HasField("metadata") != identity.HasField("metadata")
                or not _wire.equal(schema.metadata, identity.metadata)
            ):
                raise ProtocolError("Record definition differs from listed identity")
            yield RecordSchema(schema.record_type, schema.schema_version, _label(schema), tuple(_field(item) for item in schema.fields))
        if not page.HasField("next_after_record_type"):
            return
        request.after_record_type = previous
        _wire.copy(request.cursor, page.cursor)


def _label(item: Any) -> str | None:
    return item.metadata.label or None if item.HasField("metadata") else None


def _field(item: Any) -> RecordField:
    if not item.HasField("metadata") or not item.metadata.identifier.strip():
        raise ProtocolError("Record field has no declared identifier")
    try:
        value_type = RecordValueType(_wire.enum_name(item.value_type))
    except (ValueError, ProtocolError):
        raise ProtocolError("Unknown record field value type") from None
    return RecordField(
        item.metadata.identifier,
        value_type,
        _label(item),
        item.unit.symbol if item.HasField("unit") else None,
        item.required,
        item.nullable,
    )


# === Entry Point === #
