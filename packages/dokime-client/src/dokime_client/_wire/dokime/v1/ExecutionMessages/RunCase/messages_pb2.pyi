from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id", "case_id", "context", "idempotency_key")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    IDEMPOTENCY_KEY_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    context: Row3
    idempotency_key: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., context: _Optional[_Union[Row3, _Mapping]] = ..., idempotency_key: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("outcome",)
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    outcome: Row4
    def __init__(self, outcome: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("correlation_id",)
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    correlation_id: str
    def __init__(self, correlation_id: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Accepted", "Rejected")
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    Accepted: Row5
    Rejected: Row6
    def __init__(self, Accepted: _Optional[_Union[Row5, _Mapping]] = ..., Rejected: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row7
    def __init__(self, field_0: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("command_id", "job_id", "cursor", "accepted_at")
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    command_id: str
    job_id: str
    cursor: Row9
    accepted_at: Row10
    def __init__(self, command_id: _Optional[str] = ..., job_id: _Optional[str] = ..., cursor: _Optional[_Union[Row9, _Mapping]] = ..., accepted_at: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("code", "message", "issues")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    issues: _containers.RepeatedCompositeFieldContainer[Row11]
    def __init__(self, code: _Optional[str] = ..., message: _Optional[str] = ..., issues: _Optional[_Iterable[_Union[Row11, _Mapping]]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("path", "message", "expected")
    PATH_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_FIELD_NUMBER: _ClassVar[int]
    path: str
    message: str
    expected: str
    def __init__(self, path: _Optional[str] = ..., message: _Optional[str] = ..., expected: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
