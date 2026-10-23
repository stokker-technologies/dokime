from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("test_execution_id",)
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    test_execution_id: str
    def __init__(self, test_execution_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("revision", "resynchronized", "progress", "cleared")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESYNCHRONIZED_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    CLEARED_FIELD_NUMBER: _ClassVar[int]
    revision: int
    resynchronized: bool
    progress: Row4
    cleared: bool
    def __init__(self, revision: _Optional[int] = ..., resynchronized: _Optional[bool] = ..., progress: _Optional[_Union[Row4, _Mapping]] = ..., cleared: _Optional[bool] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("test_execution_id", "step_id", "phase", "completed", "total", "elapsed_ms", "remaining_ms")
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_MS_FIELD_NUMBER: _ClassVar[int]
    REMAINING_MS_FIELD_NUMBER: _ClassVar[int]
    test_execution_id: str
    step_id: str
    phase: str
    completed: int
    total: int
    elapsed_ms: int
    remaining_ms: int
    def __init__(self, test_execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., phase: _Optional[str] = ..., completed: _Optional[int] = ..., total: _Optional[int] = ..., elapsed_ms: _Optional[int] = ..., remaining_ms: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
