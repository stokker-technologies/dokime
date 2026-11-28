from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.host.v1 import executions_pb2 as _executions_pb2
from dokime_sdk._generated.proto.model.v1 import execution_pb2 as _execution_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from harmos._generated.proto.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class ProgressGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class ProgressGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: _execution_pb2.ExecutionProgressValue
    revision: bytes
    resolved_at: _position_pb2.Position
    def __init__(self, value: _Optional[_Union[_execution_pb2.ExecutionProgressValue, _Mapping]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class ProgressWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class ProgressWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    value: _execution_pb2.ExecutionProgressValue
    revision: bytes
    resolved_at: _position_pb2.Position
    phase: str
    sealed: bool
    outcome: _executions_pb2.ExecutionOutcome
    def __init__(self, value: _Optional[_Union[_execution_pb2.ExecutionProgressValue, _Mapping]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., phase: _Optional[str] = ..., sealed: _Optional[bool] = ..., outcome: _Optional[_Union[_executions_pb2.ExecutionOutcome, _Mapping]] = ...) -> None: ...

@final
class ProgressSetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: _execution_pb2.ExecutionProgressValue
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[_execution_pb2.ExecutionProgressValue, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class ProgressSetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: _execution_pb2.ExecutionProgressValue
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[_execution_pb2.ExecutionProgressValue, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...
