from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "stream", "versions", "fields", "where", "between", "bucket", "page", "at")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    WHERE_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    BUCKET_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    stream: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedScalarFieldContainer[str]
    where: _containers.RepeatedCompositeFieldContainer[Row3]
    between: Row4
    bucket: Row5
    page: Row6
    at: Row7
    def __init__(self, execution_id: _Optional[str] = ..., stream: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[str]] = ..., where: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ..., between: _Optional[_Union[Row4, _Mapping]] = ..., bucket: _Optional[_Union[Row5, _Mapping]] = ..., page: _Optional[_Union[Row6, _Mapping]] = ..., at: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("field", "operator", "values")
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    field: str
    operator: Row12
    values: _containers.RepeatedCompositeFieldContainer[Row13]
    def __init__(self, field: _Optional[str] = ..., operator: _Optional[_Union[Row12, _Mapping]] = ..., values: _Optional[_Iterable[_Union[Row13, _Mapping]]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("start", "end")
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    start: Row14
    end: Row14
    def __init__(self, start: _Optional[_Union[Row14, _Mapping]] = ..., end: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("every_nanos", "aggregate")
    EVERY_NANOS_FIELD_NUMBER: _ClassVar[int]
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    every_nanos: int
    aggregate: Row15
    def __init__(self, every_nanos: _Optional[int] = ..., aggregate: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

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
    Token: Row16
    Checkpoint: Row17
    def __init__(self, Token: _Optional[_Union[Row16, _Mapping]] = ..., Checkpoint: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Raw", "Bucket")
    RAW_FIELD_NUMBER: _ClassVar[int]
    BUCKET_FIELD_NUMBER: _ClassVar[int]
    Raw: Row18
    Bucket: Row19
    def __init__(self, Raw: _Optional[_Union[Row18, _Mapping]] = ..., Bucket: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("identity", "versions", "fields")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    identity: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row20]
    def __init__(self, identity: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[_Union[Row20, _Mapping]]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("token", "identity", "expires_at")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    EXPIRES_AT_FIELD_NUMBER: _ClassVar[int]
    token: str
    identity: str
    expires_at: Row14
    def __init__(self, token: _Optional[str] = ..., identity: _Optional[str] = ..., expires_at: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("rows", "bytes")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    BYTES_FIELD_NUMBER: _ClassVar[int]
    rows: int
    bytes: int
    def __init__(self, rows: _Optional[int] = ..., bytes: _Optional[int] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Eq", "Lt", "Lte", "Gt", "Gte", "In", "Exists")
    EQ_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    IN_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    Eq: Row21
    Lt: Row22
    Lte: Row23
    Gt: Row24
    Gte: Row25
    In: Row26
    Exists: Row27
    def __init__(self, Eq: _Optional[_Union[Row21, _Mapping]] = ..., Lt: _Optional[_Union[Row22, _Mapping]] = ..., Lte: _Optional[_Union[Row23, _Mapping]] = ..., Gt: _Optional[_Union[Row24, _Mapping]] = ..., Gte: _Optional[_Union[Row25, _Mapping]] = ..., In: _Optional[_Union[Row26, _Mapping]] = ..., Exists: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("Boolean", "Integer", "Unsigned", "Number", "Text", "Datetime", "Severity")
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    Boolean: Row28
    Integer: Row29
    Unsigned: Row30
    Number: Row31
    Text: Row32
    Datetime: Row33
    Severity: Row34
    def __init__(self, Boolean: _Optional[_Union[Row28, _Mapping]] = ..., Integer: _Optional[_Union[Row29, _Mapping]] = ..., Unsigned: _Optional[_Union[Row30, _Mapping]] = ..., Number: _Optional[_Union[Row31, _Mapping]] = ..., Text: _Optional[_Union[Row32, _Mapping]] = ..., Datetime: _Optional[_Union[Row33, _Mapping]] = ..., Severity: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("Count", "Sum", "Mean", "Median", "Min", "Max", "Rms", "Std", "Percentile")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    SUM_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    RMS_FIELD_NUMBER: _ClassVar[int]
    STD_FIELD_NUMBER: _ClassVar[int]
    PERCENTILE_FIELD_NUMBER: _ClassVar[int]
    Count: Row35
    Sum: Row35
    Mean: Row35
    Median: Row35
    Min: Row35
    Max: Row35
    Rms: Row35
    Std: Row35
    Percentile: Row36
    def __init__(self, Count: _Optional[_Union[Row35, _Mapping]] = ..., Sum: _Optional[_Union[Row35, _Mapping]] = ..., Mean: _Optional[_Union[Row35, _Mapping]] = ..., Median: _Optional[_Union[Row35, _Mapping]] = ..., Min: _Optional[_Union[Row35, _Mapping]] = ..., Max: _Optional[_Union[Row35, _Mapping]] = ..., Rms: _Optional[_Union[Row35, _Mapping]] = ..., Std: _Optional[_Union[Row35, _Mapping]] = ..., Percentile: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row37
    def __init__(self, field_0: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row38
    def __init__(self, field_0: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row39
    def __init__(self, field_0: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("id", "value_type", "nullable", "unit")
    ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    id: str
    value_type: Row40
    nullable: bool
    unit: str
    def __init__(self, id: _Optional[str] = ..., value_type: _Optional[_Union[Row40, _Mapping]] = ..., nullable: _Optional[bool] = ..., unit: _Optional[str] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row23(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row24(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    field_0: Row14
    def __init__(self, field_0: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row41
    def __init__(self, field_0: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
    __slots__ = ("rank",)
    RANK_FIELD_NUMBER: _ClassVar[int]
    rank: int
    def __init__(self, rank: _Optional[int] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row42
    version: bytes
    evidence_through: Row42
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row42, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
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
    timestamp: Row14
    received_at: Row14
    source: str
    scope: Row43
    values: Row44
    summary: Row44
    metadata: Row44
    def __init__(self, sample_id: _Optional[str] = ..., stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[Row14, _Mapping]] = ..., received_at: _Optional[_Union[Row14, _Mapping]] = ..., source: _Optional[str] = ..., scope: _Optional[_Union[Row43, _Mapping]] = ..., values: _Optional[_Union[Row44, _Mapping]] = ..., summary: _Optional[_Union[Row44, _Mapping]] = ..., metadata: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("start", "end", "execution_id", "stream", "source", "versions", "sample_count", "fields")
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_COUNT_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    start: Row14
    end: Row14
    execution_id: str
    stream: str
    source: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    sample_count: int
    fields: _containers.RepeatedCompositeFieldContainer[Row45]
    def __init__(self, start: _Optional[_Union[Row14, _Mapping]] = ..., end: _Optional[_Union[Row14, _Mapping]] = ..., execution_id: _Optional[str] = ..., stream: _Optional[str] = ..., source: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., sample_count: _Optional[int] = ..., fields: _Optional[_Iterable[_Union[Row45, _Mapping]]] = ...) -> None: ...

class Row40(_message.Message):
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
    Boolean: Row46
    Integer: Row47
    Int64: Row48
    Uint64: Row49
    Float64: Row50
    Text: Row51
    Datetime: Row52
    Severity: Row53
    Object: Row54
    Array: Row55
    def __init__(self, Boolean: _Optional[_Union[Row46, _Mapping]] = ..., Integer: _Optional[_Union[Row47, _Mapping]] = ..., Int64: _Optional[_Union[Row48, _Mapping]] = ..., Uint64: _Optional[_Union[Row49, _Mapping]] = ..., Float64: _Optional[_Union[Row50, _Mapping]] = ..., Text: _Optional[_Union[Row51, _Mapping]] = ..., Datetime: _Optional[_Union[Row52, _Mapping]] = ..., Severity: _Optional[_Union[Row53, _Mapping]] = ..., Object: _Optional[_Union[Row54, _Mapping]] = ..., Array: _Optional[_Union[Row55, _Mapping]] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row56
    Trace: Row57
    Debug: Row58
    Info: Row59
    Warn: Row60
    Error: Row61
    def __init__(self, Unspecified: _Optional[_Union[Row56, _Mapping]] = ..., Trace: _Optional[_Union[Row57, _Mapping]] = ..., Debug: _Optional[_Union[Row58, _Mapping]] = ..., Info: _Optional[_Union[Row59, _Mapping]] = ..., Warn: _Optional[_Union[Row60, _Mapping]] = ..., Error: _Optional[_Union[Row61, _Mapping]] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row62
    sequence: int
    def __init__(self, history: _Optional[_Union[Row62, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row43(_message.Message):
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
    origin: Row63
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row63, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row64]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row64, _Mapping]]] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("field", "count", "value", "unit", "reducer")
    FIELD_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    REDUCER_FIELD_NUMBER: _ClassVar[int]
    field: str
    count: int
    value: Row65
    unit: str
    reducer: Row15
    def __init__(self, field: _Optional[str] = ..., count: _Optional[int] = ..., value: _Optional[_Union[Row65, _Mapping]] = ..., unit: _Optional[str] = ..., reducer: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row49(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row50(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row63(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row66
    def __init__(self, kind: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row64(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row67
    def __init__(self, kind: _Optional[_Union[Row67, _Mapping]] = ...) -> None: ...

class Row65(_message.Message):
    __slots__ = ("Count", "Number")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    Count: Row68
    Number: Row69
    def __init__(self, Count: _Optional[_Union[Row68, _Mapping]] = ..., Number: _Optional[_Union[Row69, _Mapping]] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row70
    Initialization: Row71
    StepAttemptId: Row72
    Cleanup: Row73
    def __init__(self, Default: _Optional[_Union[Row70, _Mapping]] = ..., Initialization: _Optional[_Union[Row71, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row72, _Mapping]] = ..., Cleanup: _Optional[_Union[Row73, _Mapping]] = ...) -> None: ...

class Row67(_message.Message):
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
    Null: Row74
    Bool: Row75
    Integer: Row76
    Unsigned: Row77
    Number: Row78
    Text: Row79
    Datetime: Row80
    Array: Row81
    Object: Row82
    def __init__(self, Null: _Optional[_Union[Row74, _Mapping]] = ..., Bool: _Optional[_Union[Row75, _Mapping]] = ..., Integer: _Optional[_Union[Row76, _Mapping]] = ..., Unsigned: _Optional[_Union[Row77, _Mapping]] = ..., Number: _Optional[_Union[Row78, _Mapping]] = ..., Text: _Optional[_Union[Row79, _Mapping]] = ..., Datetime: _Optional[_Union[Row80, _Mapping]] = ..., Array: _Optional[_Union[Row81, _Mapping]] = ..., Object: _Optional[_Union[Row82, _Mapping]] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row73(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row74(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row75(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row76(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row77(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row78(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row79(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row80(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row14
    def __init__(self, field_0: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row81(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row83
    def __init__(self, field_0: _Optional[_Union[Row83, _Mapping]] = ...) -> None: ...

class Row82(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row84
    def __init__(self, field_0: _Optional[_Union[Row84, _Mapping]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row84(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row85]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row85, _Mapping]]] = ...) -> None: ...

class Row85(_message.Message):
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
