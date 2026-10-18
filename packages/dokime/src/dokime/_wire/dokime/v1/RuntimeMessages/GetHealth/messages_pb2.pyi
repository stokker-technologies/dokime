from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row2(_message.Message):
    __slots__ = ("backend_id", "epoch", "readiness", "observed_at_seconds", "observed_at_nanoseconds", "message")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    READINESS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_SECONDS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_NANOSECONDS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    readiness: Row3
    observed_at_seconds: int
    observed_at_nanoseconds: int
    message: str
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., readiness: _Optional[_Union[Row3, _Mapping]] = ..., observed_at_seconds: _Optional[int] = ..., observed_at_nanoseconds: _Optional[int] = ..., message: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Starting", "Ready", "Degraded", "Stopping")
    STARTING_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    DEGRADED_FIELD_NUMBER: _ClassVar[int]
    STOPPING_FIELD_NUMBER: _ClassVar[int]
    Starting: Row4
    Ready: Row5
    Degraded: Row6
    Stopping: Row7
    def __init__(self, Starting: _Optional[_Union[Row4, _Mapping]] = ..., Ready: _Optional[_Union[Row5, _Mapping]] = ..., Degraded: _Optional[_Union[Row6, _Mapping]] = ..., Stopping: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row5(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row6(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
