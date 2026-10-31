from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("scope", "record_type", "versions", "where", "between", "page", "at")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    WHERE_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    scope: Row3
    record_type: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    where: _containers.RepeatedCompositeFieldContainer[Row4]
    between: Row5
    page: Row6
    at: Row7
    def __init__(self, scope: _Optional[_Union[Row3, _Mapping]] = ..., record_type: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., where: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., between: _Optional[_Union[Row5, _Mapping]] = ..., page: _Optional[_Union[Row6, _Mapping]] = ..., at: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("rows", "schema", "snapshot", "limits", "next", "complete")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    LIMITS_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    rows: _containers.RepeatedCompositeFieldContainer[Row8]
    schema: Row9
    snapshot: Row10
    limits: Row11
    next: str
    complete: bool
    def __init__(self, rows: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., schema: _Optional[_Union[Row9, _Mapping]] = ..., snapshot: _Optional[_Union[Row10, _Mapping]] = ..., limits: _Optional[_Union[Row11, _Mapping]] = ..., next: _Optional[str] = ..., complete: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Execution", "Cycle")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    Execution: Row12
    Cycle: Row13
    def __init__(self, Execution: _Optional[_Union[Row12, _Mapping]] = ..., Cycle: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("field", "operator", "values")
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    field: str
    operator: Row14
    values: _containers.RepeatedCompositeFieldContainer[Row15]
    def __init__(self, field: _Optional[str] = ..., operator: _Optional[_Union[Row14, _Mapping]] = ..., values: _Optional[_Iterable[_Union[Row15, _Mapping]]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("start", "end")
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    start: Row16
    end: Row16
    def __init__(self, start: _Optional[_Union[Row16, _Mapping]] = ..., end: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("rows", "bytes", "after")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    BYTES_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    rows: int
    bytes: int
    after: str
    def __init__(self, rows: _Optional[int] = ..., bytes: _Optional[int] = ..., after: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Token", "Checkpoint")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    Token: Row17
    Checkpoint: Row18
    def __init__(self, Token: _Optional[_Union[Row17, _Mapping]] = ..., Checkpoint: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
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
    timestamp: Row16
    record_type: str
    schema_version: int
    source: str
    severity: Row19
    message: str
    error: Row20
    details: Row21
    scope: Row22
    related_sample_id: str
    related_stream_id: str
    related_artifact_id: str
    received_at: Row16
    def __init__(self, record_id: _Optional[str] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[Row16, _Mapping]] = ..., record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., source: _Optional[str] = ..., severity: _Optional[_Union[Row19, _Mapping]] = ..., message: _Optional[str] = ..., error: _Optional[_Union[Row20, _Mapping]] = ..., details: _Optional[_Union[Row21, _Mapping]] = ..., scope: _Optional[_Union[Row22, _Mapping]] = ..., related_sample_id: _Optional[str] = ..., related_stream_id: _Optional[str] = ..., related_artifact_id: _Optional[str] = ..., received_at: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("identity", "versions", "fields")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    identity: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row23]
    def __init__(self, identity: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[_Union[Row23, _Mapping]]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("token", "identity", "expires_at")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    EXPIRES_AT_FIELD_NUMBER: _ClassVar[int]
    token: str
    identity: str
    expires_at: Row16
    def __init__(self, token: _Optional[str] = ..., identity: _Optional[str] = ..., expires_at: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("rows", "bytes")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    BYTES_FIELD_NUMBER: _ClassVar[int]
    rows: int
    bytes: int
    def __init__(self, rows: _Optional[int] = ..., bytes: _Optional[int] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row24
    def __init__(self, field_0: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("Eq", "Lt", "Lte", "Gt", "Gte", "In", "Exists")
    EQ_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    IN_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    Eq: Row25
    Lt: Row26
    Lte: Row27
    Gt: Row28
    Gte: Row29
    In: Row30
    Exists: Row31
    def __init__(self, Eq: _Optional[_Union[Row25, _Mapping]] = ..., Lt: _Optional[_Union[Row26, _Mapping]] = ..., Lte: _Optional[_Union[Row27, _Mapping]] = ..., Gt: _Optional[_Union[Row28, _Mapping]] = ..., Gte: _Optional[_Union[Row29, _Mapping]] = ..., In: _Optional[_Union[Row30, _Mapping]] = ..., Exists: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("Boolean", "Integer", "Unsigned", "Number", "Text", "Datetime", "Severity")
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    Boolean: Row32
    Integer: Row33
    Unsigned: Row34
    Number: Row35
    Text: Row36
    Datetime: Row37
    Severity: Row38
    def __init__(self, Boolean: _Optional[_Union[Row32, _Mapping]] = ..., Integer: _Optional[_Union[Row33, _Mapping]] = ..., Unsigned: _Optional[_Union[Row34, _Mapping]] = ..., Number: _Optional[_Union[Row35, _Mapping]] = ..., Text: _Optional[_Union[Row36, _Mapping]] = ..., Datetime: _Optional[_Union[Row37, _Mapping]] = ..., Severity: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row39
    def __init__(self, field_0: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row40
    Trace: Row41
    Debug: Row42
    Info: Row43
    Warn: Row44
    Error: Row45
    def __init__(self, Unspecified: _Optional[_Union[Row40, _Mapping]] = ..., Trace: _Optional[_Union[Row41, _Mapping]] = ..., Debug: _Optional[_Union[Row42, _Mapping]] = ..., Info: _Optional[_Union[Row43, _Mapping]] = ..., Warn: _Optional[_Union[Row44, _Mapping]] = ..., Error: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
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
    cause_chain: _containers.RepeatedCompositeFieldContainer[Row46]
    stacktrace_attachment: str
    trace_id: str
    correlation_id: str
    diagnostics: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ..., cause_chain: _Optional[_Iterable[_Union[Row46, _Mapping]]] = ..., stacktrace_attachment: _Optional[str] = ..., trace_id: _Optional[str] = ..., correlation_id: _Optional[str] = ..., diagnostics: _Optional[str] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row47]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row47, _Mapping]]] = ...) -> None: ...

class Row22(_message.Message):
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
    origin: Row48
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row48, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("id", "value_type", "nullable", "unit")
    ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    id: str
    value_type: Row49
    nullable: bool
    unit: str
    def __init__(self, id: _Optional[str] = ..., value_type: _Optional[_Union[Row49, _Mapping]] = ..., nullable: _Optional[bool] = ..., unit: _Optional[str] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("cycle_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row26(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row29(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row16
    def __init__(self, field_0: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row19
    def __init__(self, field_0: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row50
    version: bytes
    evidence_through: Row50
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row50, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row43(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ("message", "code", "source")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row51
    def __init__(self, kind: _Optional[_Union[Row51, _Mapping]] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row52
    def __init__(self, kind: _Optional[_Union[Row52, _Mapping]] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("Boolean", "Integer", "Int64", "Uint64", "Float64", "Text", "Datetime", "Severity", "Object", "Array")
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    INT64_FIELD_NUMBER: _ClassVar[int]
    UINT64_FIELD_NUMBER: _ClassVar[int]
    FLOAT64_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    Boolean: Row53
    Integer: Row54
    Int64: Row55
    Uint64: Row56
    Float64: Row57
    Text: Row58
    Datetime: Row59
    Severity: Row60
    Object: Row61
    Array: Row62
    def __init__(self, Boolean: _Optional[_Union[Row53, _Mapping]] = ..., Integer: _Optional[_Union[Row54, _Mapping]] = ..., Int64: _Optional[_Union[Row55, _Mapping]] = ..., Uint64: _Optional[_Union[Row56, _Mapping]] = ..., Float64: _Optional[_Union[Row57, _Mapping]] = ..., Text: _Optional[_Union[Row58, _Mapping]] = ..., Datetime: _Optional[_Union[Row59, _Mapping]] = ..., Severity: _Optional[_Union[Row60, _Mapping]] = ..., Object: _Optional[_Union[Row61, _Mapping]] = ..., Array: _Optional[_Union[Row62, _Mapping]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row63
    sequence: int
    def __init__(self, history: _Optional[_Union[Row63, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row51(_message.Message):
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
    Null: Row64
    Bool: Row65
    Integer: Row66
    Unsigned: Row67
    Number: Row68
    Text: Row69
    Datetime: Row70
    Array: Row71
    Object: Row72
    def __init__(self, Null: _Optional[_Union[Row64, _Mapping]] = ..., Bool: _Optional[_Union[Row65, _Mapping]] = ..., Integer: _Optional[_Union[Row66, _Mapping]] = ..., Unsigned: _Optional[_Union[Row67, _Mapping]] = ..., Number: _Optional[_Union[Row68, _Mapping]] = ..., Text: _Optional[_Union[Row69, _Mapping]] = ..., Datetime: _Optional[_Union[Row70, _Mapping]] = ..., Array: _Optional[_Union[Row71, _Mapping]] = ..., Object: _Optional[_Union[Row72, _Mapping]] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row73
    Initialization: Row74
    StepAttemptId: Row75
    Cleanup: Row76
    def __init__(self, Default: _Optional[_Union[Row73, _Mapping]] = ..., Initialization: _Optional[_Union[Row74, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row75, _Mapping]] = ..., Cleanup: _Optional[_Union[Row76, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row54(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row55(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row56(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row57(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row58(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row59(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row60(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row61(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row62(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row63(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row64(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row65(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row16
    def __init__(self, field_0: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row77
    def __init__(self, field_0: _Optional[_Union[Row77, _Mapping]] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row78
    def __init__(self, field_0: _Optional[_Union[Row78, _Mapping]] = ...) -> None: ...

class Row73(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row74(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row75(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row76(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row77(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row78(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row79]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row79, _Mapping]]] = ...) -> None: ...

class Row79(_message.Message):
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
