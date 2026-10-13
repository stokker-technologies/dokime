# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Serve Canonical Generated Operations In Socket Fixtures"""

# === Imports === #

# Standard Library Imports
import json
from collections.abc import Mapping
from typing import Any

# Third Party Imports
import grpc
from google.protobuf.message import Message
from google.protobuf.message_factory import GetMessageClass

# Local Imports
import dokime
from dokime.client import DatetimeValue, UnsignedValue, _contracts, _wire
from dokime.operation import Operation

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def _message(
    service: str,
    method: str,
    *,
    direction: str = "response",
    path: str = "",
    **fields: Any,
) -> Message:
    operation = _contracts.operation(service, method)
    descriptor = getattr(operation, direction).DESCRIPTOR
    for name in filter(None, path.split(".")):
        descriptor = descriptor.fields_by_name[name].message_type
    return _construct(descriptor, fields)


def _construct(descriptor, fields: Mapping[str, Any]) -> Message:
    result = GetMessageClass(descriptor)()
    for name, value in fields.items():
        if value is None:
            continue
        field = descriptor.fields_by_name[name]
        if field.message_type is None:
            if field.is_repeated:
                getattr(result, name).extend(value)
            else:
                setattr(result, name, value)
        elif field.is_repeated:
            if isinstance(value, Mapping):
                value = [{"key": key, "value": item} for key, item in value.items()]
            for item in value:
                getattr(result, name).add().CopyFrom(
                    _construct(field.message_type, _contracts.values(item))
                )
        else:
            getattr(result, name).CopyFrom(
                _construct(field.message_type, _contracts.values(value))
            )
    return result


def _value(**fields: Any) -> Message:
    if len(fields) != 1:
        raise ValueError("fixture values require exactly one branch")
    kind, item = next(iter(fields.items()))
    if kind == "null_value":
        item = None
    elif kind == "number_value":
        item = float(item)
    elif kind == "unsigned_value":
        item = UnsignedValue(item)
    elif kind == "datetime_value":
        item = DatetimeValue.from_rfc3339(item)
    elif kind == "object_value":
        item = _wire.decode(item)
    elif kind == "array_value":
        item = [_wire.decode(value) for value in item]
    return _message(
        "Evidence", "ListRecords", path="records.details", **_wire.encode(item)
    )


def _value_object(*, fields: Mapping[str, Message]) -> Message:
    return _message(
        "Evidence",
        "ListRecords",
        path="records.details",
        **_wire.encode({key: _wire.decode(value) for key, value in fields.items()}),
    )


def _value_list(*, values: list[Message]) -> list[Message]:
    return values


def register(server, implementation) -> None:
    operations = [
        value for value in vars(dokime).values() if isinstance(value, Operation)
    ]
    handlers = {}
    providers = []
    for operation in operations:
        service, method = operation.grpc_path[1:].split("/")
        callback = getattr(implementation, method, None)
        if callback is None or not service.startswith("dokime.v1."):
            continue
        providers.append(
            {
                "owner": "fixture",
                "generation": 1,
                "contract": json.loads(operation.contract),
                "grants": [],
                "conditional_access": False,
                "grpc_available": True,
            }
        )
        if operation.mode == "unary":

            async def unary(request, context, callback=callback, operation=operation):
                result = await callback(request, context)
                return _construct(
                    operation.response.DESCRIPTOR, _contracts.values(result)
                )

            handler = grpc.unary_unary_rpc_method_handler(
                unary,
                request_deserializer=operation.request.FromString,
                response_serializer=lambda result: result.SerializeToString(),
            )
        elif operation.mode == "server_stream":

            async def stream(request, context, callback=callback, operation=operation):
                async for item in callback(request, context):
                    yield _construct(operation.rpc_response.DESCRIPTOR, {"item": item})
                yield _construct(operation.rpc_response.DESCRIPTOR, {"response": {}})

            handler = grpc.unary_stream_rpc_method_handler(
                stream,
                request_deserializer=operation.request.FromString,
                response_serializer=lambda result: result.SerializeToString(),
            )
        else:
            continue
        handlers.setdefault(service, {})[method] = handler
    discovery = dokime.HOST_OPERATIONS

    async def catalog(request, context):
        # One Fixture Page; Production Paging Is Exercised By Discovery Tests.
        return discovery.response(catalog=json.dumps(providers).encode())

    service, method = discovery.grpc_path[1:].split("/")
    handlers.setdefault(service, {})[method] = grpc.unary_unary_rpc_method_handler(
        catalog,
        request_deserializer=discovery.request.FromString,
        response_serializer=lambda result: result.SerializeToString(),
    )
    server.add_generic_rpc_handlers(
        [
            grpc.method_handlers_generic_handler(service, methods)
            for service, methods in handlers.items()
        ]
    )


# === Private Functions === #

# === Entry Point === #
