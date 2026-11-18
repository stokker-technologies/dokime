import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class EffectsCreateRequest(_message.Message):
    __slots__ = ("resource", "inverse_operation", "inverse", "description", "request_id")
    RESOURCE_FIELD_NUMBER: _ClassVar[int]
    INVERSE_OPERATION_FIELD_NUMBER: _ClassVar[int]
    INVERSE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    resource: str
    inverse_operation: str
    inverse: bytes
    description: str
    request_id: str
    def __init__(self, resource: _Optional[str] = ..., inverse_operation: _Optional[str] = ..., inverse: _Optional[bytes] = ..., description: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

class EffectsCreateResponse(_message.Message):
    __slots__ = ("name", "value", "revision", "receipt")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: EffectWritable
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[EffectWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class EffectsGetRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class EffectsGetResponse(_message.Message):
    __slots__ = ("value", "revision", "resolved_at")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: EffectWritable
    revision: bytes
    resolved_at: _position_pb2.Position
    def __init__(self, value: _Optional[_Union[EffectWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

class EffectsSetRequest(_message.Message):
    __slots__ = ("name", "value", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: EffectWritable
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[EffectWritable, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class EffectsSetResponse(_message.Message):
    __slots__ = ("value", "revision", "receipt")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: EffectWritable
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[EffectWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class EffectWritable(_message.Message):
    __slots__ = ("applied_at", "restored")
    APPLIED_AT_FIELD_NUMBER: _ClassVar[int]
    RESTORED_FIELD_NUMBER: _ClassVar[int]
    applied_at: _timestamp_pb2.Timestamp
    restored: bool
    def __init__(self, applied_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., restored: _Optional[bool] = ...) -> None: ...
