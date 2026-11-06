# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from typing import Any

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ...operation import Operation
from .._transport import Transport
from ..errors import RpcError

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

class QueryDependencyError(ImportError):
    extra: str

    def __init__(self, extra: str): ...

class QueryConversionError(ValueError):
    field: str

    def __init__(self, field: str, message: str): ...

class QueryError(RpcError):
    details: dict[str, Any]
    reason: Any
    operation: Any
    field: Any
    operator: Any
    snapshot: Any
    limit: Any

    def __init__(self, error: RpcError, details: dict[str, Any]): ...

class QueryLimitError(QueryError): ...
class QuerySnapshotError(QueryError): ...

# === Public Functions === #

async def query_call(transport: Transport, operation: Operation, request: Message) -> Message: ...
def _fields(payload: bytes, number: int) -> list[bytes]: ...
def _details(error: RpcError) -> dict[str, Any]: ...
