from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row2(_message.Message):
    __slots__ = ("backend_id", "epoch", "readiness", "observed_at", "message")
    BACKEND_ID_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    READINESS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_AT_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    backend_id: str
    epoch: str
    readiness: Row3
    observed_at: Row4
    message: str
    def __init__(self, backend_id: _Optional[str] = ..., epoch: _Optional[str] = ..., readiness: _Optional[_Union[Row3, _Mapping]] = ..., observed_at: _Optional[_Union[Row4, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Starting", "Ready", "Degraded", "Stopping")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STARTING_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    DEGRADED_FIELD_NUMBER: _ClassVar[int]
    STOPPING_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row5
    Starting: Row6
    Ready: Row7
    Degraded: Row8
    Stopping: Row9
    def __init__(self, Unspecified: _Optional[_Union[Row5, _Mapping]] = ..., Starting: _Optional[_Union[Row6, _Mapping]] = ..., Ready: _Optional[_Union[Row7, _Mapping]] = ..., Degraded: _Optional[_Union[Row8, _Mapping]] = ..., Stopping: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
