from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("epoch", "scope", "min_level", "after_sequence", "max_events", "max_bytes", "follow")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    MIN_LEVEL_FIELD_NUMBER: _ClassVar[int]
    AFTER_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    MAX_EVENTS_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    FOLLOW_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    scope: Row4
    min_level: Row5
    after_sequence: int
    max_events: int
    max_bytes: int
    follow: bool
    def __init__(self, epoch: _Optional[str] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., min_level: _Optional[_Union[Row5, _Mapping]] = ..., after_sequence: _Optional[int] = ..., max_events: _Optional[int] = ..., max_bytes: _Optional[int] = ..., follow: _Optional[bool] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "events", "dropped", "next_sequence")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    EVENTS_FIELD_NUMBER: _ClassVar[int]
    DROPPED_FIELD_NUMBER: _ClassVar[int]
    NEXT_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    events: _containers.RepeatedCompositeFieldContainer[Row6]
    dropped: int
    next_sequence: int
    def __init__(self, epoch: _Optional[str] = ..., events: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., dropped: _Optional[int] = ..., next_sequence: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Instance", "Cycle", "Execution")
    INSTANCE_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    Instance: Row7
    Cycle: Row8
    Execution: Row9
    def __init__(self, Instance: _Optional[_Union[Row7, _Mapping]] = ..., Cycle: _Optional[_Union[Row8, _Mapping]] = ..., Execution: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Trace: Row11
    Debug: Row12
    Info: Row13
    Warn: Row14
    Error: Row15
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Trace: _Optional[_Union[Row11, _Mapping]] = ..., Debug: _Optional[_Union[Row12, _Mapping]] = ..., Info: _Optional[_Union[Row13, _Mapping]] = ..., Warn: _Optional[_Union[Row14, _Mapping]] = ..., Error: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("sequence", "time", "level", "target", "message", "plugin_id", "cycle_id", "execution_id", "fields")
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    TIME_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    sequence: int
    time: Row16
    level: Row5
    target: str
    message: str
    plugin_id: str
    cycle_id: str
    execution_id: str
    fields: _containers.RepeatedCompositeFieldContainer[Row17]
    def __init__(self, sequence: _Optional[int] = ..., time: _Optional[_Union[Row16, _Mapping]] = ..., level: _Optional[_Union[Row5, _Mapping]] = ..., target: _Optional[str] = ..., message: _Optional[str] = ..., plugin_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[Row17, _Mapping]]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

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
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
