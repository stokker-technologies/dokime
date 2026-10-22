from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("request_id", "command_id", "execution_id", "action", "step_id")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    command_id: str
    execution_id: str
    action: Row3
    step_id: str
    def __init__(self, request_id: _Optional[str] = ..., command_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., action: _Optional[_Union[Row3, _Mapping]] = ..., step_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("request_id", "command_id", "execution_id", "action", "step_id", "phase", "attempt_id", "accepted_at", "updated_at", "detail")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_ID_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    command_id: str
    execution_id: str
    action: Row3
    step_id: str
    phase: Row4
    attempt_id: str
    accepted_at: Row5
    updated_at: Row5
    detail: str
    def __init__(self, request_id: _Optional[str] = ..., command_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., action: _Optional[_Union[Row3, _Mapping]] = ..., step_id: _Optional[str] = ..., phase: _Optional[_Union[Row4, _Mapping]] = ..., attempt_id: _Optional[str] = ..., accepted_at: _Optional[_Union[Row5, _Mapping]] = ..., updated_at: _Optional[_Union[Row5, _Mapping]] = ..., detail: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Next", "RunStep", "Finish")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    RUNSTEP_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row6
    Next: Row7
    RunStep: Row8
    Finish: Row9
    def __init__(self, Unspecified: _Optional[_Union[Row6, _Mapping]] = ..., Next: _Optional[_Union[Row7, _Mapping]] = ..., RunStep: _Optional[_Union[Row8, _Mapping]] = ..., Finish: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Accepted", "Dispatching", "Running", "Completed", "Refused", "Failed", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    DISPATCHING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    REFUSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Accepted: Row11
    Dispatching: Row12
    Running: Row13
    Completed: Row14
    Refused: Row15
    Failed: Row16
    Interrupted: Row17
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Accepted: _Optional[_Union[Row11, _Mapping]] = ..., Dispatching: _Optional[_Union[Row12, _Mapping]] = ..., Running: _Optional[_Union[Row13, _Mapping]] = ..., Completed: _Optional[_Union[Row14, _Mapping]] = ..., Refused: _Optional[_Union[Row15, _Mapping]] = ..., Failed: _Optional[_Union[Row16, _Mapping]] = ..., Interrupted: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
