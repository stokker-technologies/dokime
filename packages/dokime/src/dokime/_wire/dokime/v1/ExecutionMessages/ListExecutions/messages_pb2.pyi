from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id", "limit", "after_execution_id", "cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    limit: int
    after_execution_id: str
    cursor: Row3
    def __init__(self, cycle_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_execution_id: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("executions", "next_after_execution_id", "cursor")
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    executions: _containers.RepeatedCompositeFieldContainer[Row4]
    next_after_execution_id: str
    cursor: Row3
    def __init__(self, executions: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_after_execution_id: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("execution_id", "case_id", "status", "ordinal")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    case_id: str
    status: Row5
    ordinal: int
    def __init__(self, execution_id: _Optional[str] = ..., case_id: _Optional[str] = ..., status: _Optional[_Union[Row5, _Mapping]] = ..., ordinal: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Unspecified", "Invalidated", "Pending", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted", "Skipped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row6
    Invalidated: Row7
    Pending: Row8
    Running: Row9
    Passed: Row10
    Failed: Row11
    Blocked: Row12
    Cancelled: Row13
    Interrupted: Row14
    Skipped: Row15
    def __init__(self, Unspecified: _Optional[_Union[Row6, _Mapping]] = ..., Invalidated: _Optional[_Union[Row7, _Mapping]] = ..., Pending: _Optional[_Union[Row8, _Mapping]] = ..., Running: _Optional[_Union[Row9, _Mapping]] = ..., Passed: _Optional[_Union[Row10, _Mapping]] = ..., Failed: _Optional[_Union[Row11, _Mapping]] = ..., Blocked: _Optional[_Union[Row12, _Mapping]] = ..., Cancelled: _Optional[_Union[Row13, _Mapping]] = ..., Interrupted: _Optional[_Union[Row14, _Mapping]] = ..., Skipped: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row12(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
