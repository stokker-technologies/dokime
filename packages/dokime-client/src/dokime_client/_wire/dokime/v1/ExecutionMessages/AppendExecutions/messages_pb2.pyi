from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id", "executions")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    executions: _containers.RepeatedCompositeFieldContainer[Row3]
    def __init__(self, cycle_id: _Optional[str] = ..., executions: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("observed_cursor", "execution_ids")
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_IDS_FIELD_NUMBER: _ClassVar[int]
    observed_cursor: Row4
    execution_ids: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, observed_cursor: _Optional[_Union[Row4, _Mapping]] = ..., execution_ids: _Optional[_Iterable[str]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("execution_id", "case_id")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    case_id: str
    def __init__(self, execution_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
