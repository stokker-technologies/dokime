from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("selection", "after")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    selection: Row4
    after: Row5
    def __init__(self, selection: _Optional[_Union[Row4, _Mapping]] = ..., after: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("records", "scanned", "through", "replay_complete", "retained_after")
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    SCANNED_FIELD_NUMBER: _ClassVar[int]
    THROUGH_FIELD_NUMBER: _ClassVar[int]
    REPLAY_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    RETAINED_AFTER_FIELD_NUMBER: _ClassVar[int]
    records: _containers.RepeatedCompositeFieldContainer[Row6]
    scanned: Row5
    through: Row5
    replay_complete: bool
    retained_after: int
    def __init__(self, records: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., scanned: _Optional[_Union[Row5, _Mapping]] = ..., through: _Optional[_Union[Row5, _Mapping]] = ..., replay_complete: _Optional[bool] = ..., retained_after: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("scope", "record_types", "sources", "severities")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPES_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SEVERITIES_FIELD_NUMBER: _ClassVar[int]
    scope: Row7
    record_types: _containers.RepeatedScalarFieldContainer[str]
    sources: _containers.RepeatedScalarFieldContainer[str]
    severities: _containers.RepeatedCompositeFieldContainer[Row8]
    def __init__(self, scope: _Optional[_Union[Row7, _Mapping]] = ..., record_types: _Optional[_Iterable[str]] = ..., sources: _Optional[_Iterable[str]] = ..., severities: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("record_id", "sequence", "timestamp", "record_type", "schema_version", "source", "severity", "message", "error", "details", "scope", "related_sample_id", "related_stream_id", "related_artifact_id", "received_at")
    RECORD_ID_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    DETAILS_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RELATED_SAMPLE_ID_FIELD_NUMBER: _ClassVar[int]
    RELATED_STREAM_ID_FIELD_NUMBER: _ClassVar[int]
    RELATED_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    RECEIVED_AT_FIELD_NUMBER: _ClassVar[int]
    record_id: str
    sequence: int
    timestamp: Row9
    record_type: str
    schema_version: int
    source: str
    severity: Row8
    message: str
    error: Row10
    details: Row11
    scope: Row7
    related_sample_id: str
    related_stream_id: str
    related_artifact_id: str
    received_at: Row9
    def __init__(self, record_id: _Optional[str] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[Row9, _Mapping]] = ..., record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., source: _Optional[str] = ..., severity: _Optional[_Union[Row8, _Mapping]] = ..., message: _Optional[str] = ..., error: _Optional[_Union[Row10, _Mapping]] = ..., details: _Optional[_Union[Row11, _Mapping]] = ..., scope: _Optional[_Union[Row7, _Mapping]] = ..., related_sample_id: _Optional[str] = ..., related_stream_id: _Optional[str] = ..., related_artifact_id: _Optional[str] = ..., received_at: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
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
    origin: Row12
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row13
    Trace: Row14
    Debug: Row15
    Info: Row16
    Warn: Row17
    Error: Row18
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., Trace: _Optional[_Union[Row14, _Mapping]] = ..., Debug: _Optional[_Union[Row15, _Mapping]] = ..., Info: _Optional[_Union[Row16, _Mapping]] = ..., Warn: _Optional[_Union[Row17, _Mapping]] = ..., Error: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("message", "code", "source", "cause_chain", "stacktrace_attachment", "trace_id", "correlation_id", "diagnostics")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    CAUSE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    STACKTRACE_ATTACHMENT_FIELD_NUMBER: _ClassVar[int]
    TRACE_ID_FIELD_NUMBER: _ClassVar[int]
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    DIAGNOSTICS_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    cause_chain: _containers.RepeatedCompositeFieldContainer[Row19]
    stacktrace_attachment: str
    trace_id: str
    correlation_id: str
    diagnostics: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ..., cause_chain: _Optional[_Iterable[_Union[Row19, _Mapping]]] = ..., stacktrace_attachment: _Optional[str] = ..., trace_id: _Optional[str] = ..., correlation_id: _Optional[str] = ..., diagnostics: _Optional[str] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row20]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row20, _Mapping]]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row21
    def __init__(self, kind: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

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

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row19(_message.Message):
    __slots__ = ("message", "code", "source")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row22
    def __init__(self, kind: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row23
    Initialization: Row24
    StepAttemptId: Row25
    Cleanup: Row26
    def __init__(self, Default: _Optional[_Union[Row23, _Mapping]] = ..., Initialization: _Optional[_Union[Row24, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row25, _Mapping]] = ..., Cleanup: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
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
    Null: Row27
    Bool: Row28
    Integer: Row29
    Unsigned: Row30
    Number: Row31
    Text: Row32
    Datetime: Row33
    Array: Row34
    Object: Row35
    def __init__(self, Null: _Optional[_Union[Row27, _Mapping]] = ..., Bool: _Optional[_Union[Row28, _Mapping]] = ..., Integer: _Optional[_Union[Row29, _Mapping]] = ..., Unsigned: _Optional[_Union[Row30, _Mapping]] = ..., Number: _Optional[_Union[Row31, _Mapping]] = ..., Text: _Optional[_Union[Row32, _Mapping]] = ..., Datetime: _Optional[_Union[Row33, _Mapping]] = ..., Array: _Optional[_Union[Row34, _Mapping]] = ..., Object: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row36
    def __init__(self, field_0: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row37
    def __init__(self, field_0: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row38]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row38, _Mapping]]] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
