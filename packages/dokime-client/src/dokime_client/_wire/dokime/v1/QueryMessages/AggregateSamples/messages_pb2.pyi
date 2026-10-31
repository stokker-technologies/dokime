from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "stream", "versions", "where", "between", "outputs", "at")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    WHERE_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    stream: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    where: _containers.RepeatedCompositeFieldContainer[Row3]
    between: Row4
    outputs: _containers.RepeatedCompositeFieldContainer[Row5]
    at: Row6
    def __init__(self, execution_id: _Optional[str] = ..., stream: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., where: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ..., between: _Optional[_Union[Row4, _Mapping]] = ..., outputs: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., at: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("schema", "values", "selected_rows", "quality_counts", "snapshot", "complete")
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SELECTED_ROWS_FIELD_NUMBER: _ClassVar[int]
    QUALITY_COUNTS_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    schema: Row7
    values: _containers.RepeatedCompositeFieldContainer[Row8]
    selected_rows: int
    quality_counts: _containers.RepeatedCompositeFieldContainer[Row9]
    snapshot: Row10
    complete: bool
    def __init__(self, schema: _Optional[_Union[Row7, _Mapping]] = ..., values: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., selected_rows: _Optional[int] = ..., quality_counts: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., snapshot: _Optional[_Union[Row10, _Mapping]] = ..., complete: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("field", "operator", "values")
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    field: str
    operator: Row11
    values: _containers.RepeatedCompositeFieldContainer[Row12]
    def __init__(self, field: _Optional[str] = ..., operator: _Optional[_Union[Row11, _Mapping]] = ..., values: _Optional[_Iterable[_Union[Row12, _Mapping]]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("start", "end")
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    start: Row13
    end: Row13
    def __init__(self, start: _Optional[_Union[Row13, _Mapping]] = ..., end: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("alias", "field", "aggregate")
    ALIAS_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    alias: str
    field: str
    aggregate: Row14
    def __init__(self, alias: _Optional[str] = ..., field: _Optional[str] = ..., aggregate: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Token", "Checkpoint")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    Token: Row15
    Checkpoint: Row16
    def __init__(self, Token: _Optional[_Union[Row15, _Mapping]] = ..., Checkpoint: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("identity", "versions", "fields")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    identity: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row17]
    def __init__(self, identity: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[_Union[Row17, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("alias", "field", "aggregate", "value")
    ALIAS_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    alias: str
    field: str
    aggregate: Row14
    value: Row18
    def __init__(self, alias: _Optional[str] = ..., field: _Optional[str] = ..., aggregate: _Optional[_Union[Row14, _Mapping]] = ..., value: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("field", "finite", "missing", "null", "non_finite")
    FIELD_FIELD_NUMBER: _ClassVar[int]
    FINITE_FIELD_NUMBER: _ClassVar[int]
    MISSING_FIELD_NUMBER: _ClassVar[int]
    NULL_FIELD_NUMBER: _ClassVar[int]
    NON_FINITE_FIELD_NUMBER: _ClassVar[int]
    field: str
    finite: int
    missing: int
    null: int
    non_finite: int
    def __init__(self, field: _Optional[str] = ..., finite: _Optional[int] = ..., missing: _Optional[int] = ..., null: _Optional[int] = ..., non_finite: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("token", "identity", "expires_at")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    EXPIRES_AT_FIELD_NUMBER: _ClassVar[int]
    token: str
    identity: str
    expires_at: Row13
    def __init__(self, token: _Optional[str] = ..., identity: _Optional[str] = ..., expires_at: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Eq", "Lt", "Lte", "Gt", "Gte", "In", "Exists")
    EQ_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    IN_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    Eq: Row19
    Lt: Row20
    Lte: Row21
    Gt: Row22
    Gte: Row23
    In: Row24
    Exists: Row25
    def __init__(self, Eq: _Optional[_Union[Row19, _Mapping]] = ..., Lt: _Optional[_Union[Row20, _Mapping]] = ..., Lte: _Optional[_Union[Row21, _Mapping]] = ..., Gt: _Optional[_Union[Row22, _Mapping]] = ..., Gte: _Optional[_Union[Row23, _Mapping]] = ..., In: _Optional[_Union[Row24, _Mapping]] = ..., Exists: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Boolean", "Integer", "Unsigned", "Number", "Text", "Datetime", "Severity")
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    UNSIGNED_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    Boolean: Row26
    Integer: Row27
    Unsigned: Row28
    Number: Row29
    Text: Row30
    Datetime: Row31
    Severity: Row32
    def __init__(self, Boolean: _Optional[_Union[Row26, _Mapping]] = ..., Integer: _Optional[_Union[Row27, _Mapping]] = ..., Unsigned: _Optional[_Union[Row28, _Mapping]] = ..., Number: _Optional[_Union[Row29, _Mapping]] = ..., Text: _Optional[_Union[Row30, _Mapping]] = ..., Datetime: _Optional[_Union[Row31, _Mapping]] = ..., Severity: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row14(_message.Message):
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
    Count: Row33
    Sum: Row33
    Mean: Row33
    Median: Row33
    Min: Row33
    Max: Row33
    Rms: Row33
    Std: Row33
    Percentile: Row34
    def __init__(self, Count: _Optional[_Union[Row33, _Mapping]] = ..., Sum: _Optional[_Union[Row33, _Mapping]] = ..., Mean: _Optional[_Union[Row33, _Mapping]] = ..., Median: _Optional[_Union[Row33, _Mapping]] = ..., Min: _Optional[_Union[Row33, _Mapping]] = ..., Max: _Optional[_Union[Row33, _Mapping]] = ..., Rms: _Optional[_Union[Row33, _Mapping]] = ..., Std: _Optional[_Union[Row33, _Mapping]] = ..., Percentile: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row35
    def __init__(self, field_0: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("id", "value_type", "nullable", "unit")
    ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    id: str
    value_type: Row36
    nullable: bool
    unit: str
    def __init__(self, id: _Optional[str] = ..., value_type: _Optional[_Union[Row36, _Mapping]] = ..., nullable: _Optional[bool] = ..., unit: _Optional[str] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("Count", "Number")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    Count: Row37
    Number: Row38
    def __init__(self, Count: _Optional[_Union[Row37, _Mapping]] = ..., Number: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row13
    def __init__(self, field_0: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row39
    def __init__(self, field_0: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ("rank",)
    RANK_FIELD_NUMBER: _ClassVar[int]
    rank: int
    def __init__(self, rank: _Optional[int] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row40
    version: bytes
    evidence_through: Row40
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row40, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row36(_message.Message):
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
    Boolean: Row41
    Integer: Row42
    Int64: Row43
    Uint64: Row44
    Float64: Row45
    Text: Row46
    Datetime: Row47
    Severity: Row48
    Object: Row49
    Array: Row50
    def __init__(self, Boolean: _Optional[_Union[Row41, _Mapping]] = ..., Integer: _Optional[_Union[Row42, _Mapping]] = ..., Int64: _Optional[_Union[Row43, _Mapping]] = ..., Uint64: _Optional[_Union[Row44, _Mapping]] = ..., Float64: _Optional[_Union[Row45, _Mapping]] = ..., Text: _Optional[_Union[Row46, _Mapping]] = ..., Datetime: _Optional[_Union[Row47, _Mapping]] = ..., Severity: _Optional[_Union[Row48, _Mapping]] = ..., Object: _Optional[_Union[Row49, _Mapping]] = ..., Array: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row51
    Trace: Row52
    Debug: Row53
    Info: Row54
    Warn: Row55
    Error: Row56
    def __init__(self, Unspecified: _Optional[_Union[Row51, _Mapping]] = ..., Trace: _Optional[_Union[Row52, _Mapping]] = ..., Debug: _Optional[_Union[Row53, _Mapping]] = ..., Info: _Optional[_Union[Row54, _Mapping]] = ..., Warn: _Optional[_Union[Row55, _Mapping]] = ..., Error: _Optional[_Union[Row56, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row57
    sequence: int
    def __init__(self, history: _Optional[_Union[Row57, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
