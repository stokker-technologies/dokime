from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("checkpoint", "table", "limit", "after")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    table: str
    limit: int
    after: str
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., table: _Optional[str] = ..., limit: _Optional[int] = ..., after: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("checkpoint", "table", "rows", "next_after", "issues", "issues_truncated")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    ISSUES_TRUNCATED_FIELD_NUMBER: _ClassVar[int]
    checkpoint: Row3
    table: str
    rows: _containers.RepeatedCompositeFieldContainer[Row4]
    next_after: str
    issues: _containers.RepeatedCompositeFieldContainer[Row5]
    issues_truncated: bool
    def __init__(self, checkpoint: _Optional[_Union[Row3, _Mapping]] = ..., table: _Optional[str] = ..., rows: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_after: _Optional[str] = ..., issues: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., issues_truncated: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row6
    version: bytes
    evidence_through: Row6
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row6, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("epoch", "ordinal", "sample")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    ordinal: int
    sample: Row7
    def __init__(self, epoch: _Optional[str] = ..., ordinal: _Optional[int] = ..., sample: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("source", "dataset", "message")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    DATASET_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    source: str
    dataset: str
    message: str
    def __init__(self, source: _Optional[str] = ..., dataset: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row8
    sequence: int
    def __init__(self, history: _Optional[_Union[Row8, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
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
    timestamp: Row9
    received_at: Row9
    source: str
    scope: Row10
    values: Row11
    summary: Row11
    metadata: Row11
    def __init__(self, sample_id: _Optional[str] = ..., stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[Row9, _Mapping]] = ..., received_at: _Optional[_Union[Row9, _Mapping]] = ..., source: _Optional[str] = ..., scope: _Optional[_Union[Row10, _Mapping]] = ..., values: _Optional[_Union[Row11, _Mapping]] = ..., summary: _Optional[_Union[Row11, _Mapping]] = ..., metadata: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
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

class Row11(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row13]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row13, _Mapping]]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row14
    def __init__(self, kind: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row15
    def __init__(self, kind: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row16
    Initialization: Row17
    StepAttemptId: Row18
    Cleanup: Row19
    def __init__(self, Default: _Optional[_Union[Row16, _Mapping]] = ..., Initialization: _Optional[_Union[Row17, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row18, _Mapping]] = ..., Cleanup: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
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
    Null: Row20
    Bool: Row21
    Integer: Row22
    Unsigned: Row23
    Number: Row24
    Text: Row25
    Datetime: Row26
    Array: Row27
    Object: Row28
    def __init__(self, Null: _Optional[_Union[Row20, _Mapping]] = ..., Bool: _Optional[_Union[Row21, _Mapping]] = ..., Integer: _Optional[_Union[Row22, _Mapping]] = ..., Unsigned: _Optional[_Union[Row23, _Mapping]] = ..., Number: _Optional[_Union[Row24, _Mapping]] = ..., Text: _Optional[_Union[Row25, _Mapping]] = ..., Datetime: _Optional[_Union[Row26, _Mapping]] = ..., Array: _Optional[_Union[Row27, _Mapping]] = ..., Object: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row31]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row31, _Mapping]]] = ...) -> None: ...

class Row31(_message.Message):
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
