from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("streams", "limit", "schema_versions", "sources", "signals", "scope", "after_stream_cursors", "start", "end", "latest")
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSIONS_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SIGNALS_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AFTER_STREAM_CURSORS_FIELD_NUMBER: _ClassVar[int]
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    streams: _containers.RepeatedScalarFieldContainer[str]
    limit: int
    schema_versions: _containers.RepeatedScalarFieldContainer[int]
    sources: _containers.RepeatedScalarFieldContainer[str]
    signals: _containers.RepeatedScalarFieldContainer[str]
    scope: Row3
    after_stream_cursors: _containers.RepeatedCompositeFieldContainer[Row4]
    start: Row5
    end: Row5
    latest: bool
    def __init__(self, streams: _Optional[_Iterable[str]] = ..., limit: _Optional[int] = ..., schema_versions: _Optional[_Iterable[int]] = ..., sources: _Optional[_Iterable[str]] = ..., signals: _Optional[_Iterable[str]] = ..., scope: _Optional[_Union[Row3, _Mapping]] = ..., after_stream_cursors: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., start: _Optional[_Union[Row5, _Mapping]] = ..., end: _Optional[_Union[Row5, _Mapping]] = ..., latest: _Optional[bool] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("samples", "through_stream_cursors", "complete")
    SAMPLES_FIELD_NUMBER: _ClassVar[int]
    THROUGH_STREAM_CURSORS_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    samples: _containers.RepeatedCompositeFieldContainer[Row6]
    through_stream_cursors: _containers.RepeatedCompositeFieldContainer[Row7]
    complete: bool
    def __init__(self, samples: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., through_stream_cursors: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., complete: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: Row8
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("sample_id", "stream", "schema_version", "sequence", "timestamp", "received_at", "source", "scope", "values", "summary", "metadata")
    SAMPLE_ID_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    RECEIVED_AT_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    sample_id: str
    stream: str
    schema_version: int
    sequence: int
    timestamp: Row5
    received_at: Row5
    source: str
    scope: Row3
    values: Row9
    summary: Row9
    metadata: Row9
    def __init__(self, sample_id: _Optional[str] = ..., stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[Row5, _Mapping]] = ..., received_at: _Optional[_Union[Row5, _Mapping]] = ..., source: _Optional[str] = ..., scope: _Optional[_Union[Row3, _Mapping]] = ..., values: _Optional[_Union[Row9, _Mapping]] = ..., summary: _Optional[_Union[Row9, _Mapping]] = ..., metadata: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row10
    def __init__(self, kind: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row11]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row11, _Mapping]]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row12
    Initialization: Row13
    StepAttemptId: Row14
    Cleanup: Row15
    def __init__(self, Default: _Optional[_Union[Row12, _Mapping]] = ..., Initialization: _Optional[_Union[Row13, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row14, _Mapping]] = ..., Cleanup: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row16
    def __init__(self, kind: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Null", "Bool", "Integer", "Unsigned", "Number", "Text", "Datetime", "Array", "Object")
    NULL_FIELD_NUMBER: _ClassVar[int]
    BOOL_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    Null: Row17
    Bool: Row18
    Integer: Row19
    Unsigned: Row20
    Number: Row21
    Text: Row22
    Datetime: Row23
    Array: Row24
    Object: Row25
    def __init__(self, Null: _Optional[_Union[Row17, _Mapping]] = ..., Bool: _Optional[_Union[Row18, _Mapping]] = ..., Integer: _Optional[_Union[Row19, _Mapping]] = ..., Unsigned: _Optional[_Union[Row20, _Mapping]] = ..., Number: _Optional[_Union[Row21, _Mapping]] = ..., Text: _Optional[_Union[Row22, _Mapping]] = ..., Datetime: _Optional[_Union[Row23, _Mapping]] = ..., Array: _Optional[_Union[Row24, _Mapping]] = ..., Object: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row5
    def __init__(self, field_0: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row26
    def __init__(self, field_0: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row27
    def __init__(self, field_0: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row28]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row28, _Mapping]]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
