from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cursor",)
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cursor: Row4
    def __init__(self, cursor: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("cursor", "resynchronized", "shutting_down", "cycle_count", "operations")
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    RESYNCHRONIZED_FIELD_NUMBER: _ClassVar[int]
    SHUTTING_DOWN_FIELD_NUMBER: _ClassVar[int]
    CYCLE_COUNT_FIELD_NUMBER: _ClassVar[int]
    OPERATIONS_FIELD_NUMBER: _ClassVar[int]
    cursor: Row4
    resynchronized: bool
    shutting_down: bool
    cycle_count: int
    operations: _containers.RepeatedCompositeFieldContainer[Row5]
    def __init__(self, cursor: _Optional[_Union[Row4, _Mapping]] = ..., resynchronized: _Optional[bool] = ..., shutting_down: _Optional[bool] = ..., cycle_count: _Optional[int] = ..., operations: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("operation_id", "label", "phase", "phases", "fraction", "started_at", "status", "finished_at", "failure")
    OPERATION_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    PHASES_FIELD_NUMBER: _ClassVar[int]
    FRACTION_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    operation_id: str
    label: str
    phase: str
    phases: _containers.RepeatedScalarFieldContainer[str]
    fraction: float
    started_at: Row6
    status: Row7
    finished_at: Row6
    failure: str
    def __init__(self, operation_id: _Optional[str] = ..., label: _Optional[str] = ..., phase: _Optional[str] = ..., phases: _Optional[_Iterable[str]] = ..., fraction: _Optional[float] = ..., started_at: _Optional[_Union[Row6, _Mapping]] = ..., status: _Optional[_Union[Row7, _Mapping]] = ..., finished_at: _Optional[_Union[Row6, _Mapping]] = ..., failure: _Optional[str] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Unspecified", "Running", "Finished", "Failed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    FINISHED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row8
    Running: Row9
    Finished: Row10
    Failed: Row11
    def __init__(self, Unspecified: _Optional[_Union[Row8, _Mapping]] = ..., Running: _Optional[_Union[Row9, _Mapping]] = ..., Finished: _Optional[_Union[Row10, _Mapping]] = ..., Failed: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
