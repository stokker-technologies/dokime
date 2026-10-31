from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("selection", "page", "at")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    selection: Row3
    page: Row4
    at: Row5
    def __init__(self, selection: _Optional[_Union[Row3, _Mapping]] = ..., page: _Optional[_Union[Row4, _Mapping]] = ..., at: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("rows", "schema", "snapshot", "limits", "next", "complete")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    LIMITS_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    rows: _containers.RepeatedCompositeFieldContainer[Row6]
    schema: Row7
    snapshot: Row8
    limits: Row9
    next: str
    complete: bool
    def __init__(self, rows: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., schema: _Optional[_Union[Row7, _Mapping]] = ..., snapshot: _Optional[_Union[Row8, _Mapping]] = ..., limits: _Optional[_Union[Row9, _Mapping]] = ..., next: _Optional[str] = ..., complete: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("cycle_ids", "phases", "created_at", "archive")
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    PHASES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    phases: _containers.RepeatedCompositeFieldContainer[Row10]
    created_at: Row11
    archive: Row12
    def __init__(self, cycle_ids: _Optional[_Iterable[str]] = ..., phases: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., created_at: _Optional[_Union[Row11, _Mapping]] = ..., archive: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("rows", "bytes", "after")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    BYTES_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    rows: int
    bytes: int
    after: str
    def __init__(self, rows: _Optional[int] = ..., bytes: _Optional[int] = ..., after: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Token", "Checkpoint")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    Token: Row13
    Checkpoint: Row14
    def __init__(self, Token: _Optional[_Union[Row13, _Mapping]] = ..., Checkpoint: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("cycle_id", "label", "phase", "created_at", "updated_at", "execution_count", "archived")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    label: str
    phase: Row10
    created_at: Row15
    updated_at: Row15
    execution_count: int
    archived: bool
    def __init__(self, cycle_id: _Optional[str] = ..., label: _Optional[str] = ..., phase: _Optional[_Union[Row10, _Mapping]] = ..., created_at: _Optional[_Union[Row15, _Mapping]] = ..., updated_at: _Optional[_Union[Row15, _Mapping]] = ..., execution_count: _Optional[int] = ..., archived: _Optional[bool] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("identity", "versions", "fields")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    identity: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row16]
    def __init__(self, identity: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[_Union[Row16, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("token", "identity", "expires_at")
    TOKEN_FIELD_NUMBER: _ClassVar[int]
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    EXPIRES_AT_FIELD_NUMBER: _ClassVar[int]
    token: str
    identity: str
    expires_at: Row15
    def __init__(self, token: _Optional[str] = ..., identity: _Optional[str] = ..., expires_at: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("rows", "bytes")
    ROWS_FIELD_NUMBER: _ClassVar[int]
    BYTES_FIELD_NUMBER: _ClassVar[int]
    rows: int
    bytes: int
    def __init__(self, rows: _Optional[int] = ..., bytes: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Active", "Sealing", "Sealed")
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    SEALING_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    Active: Row17
    Sealing: Row18
    Sealed: Row19
    def __init__(self, Active: _Optional[_Union[Row17, _Mapping]] = ..., Sealing: _Optional[_Union[Row18, _Mapping]] = ..., Sealed: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("start", "end")
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    start: Row15
    end: Row15
    def __init__(self, start: _Optional[_Union[Row15, _Mapping]] = ..., end: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Unarchived", "Archived", "All")
    UNARCHIVED_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    Unarchived: Row20
    Archived: Row21
    All: Row22
    def __init__(self, Unarchived: _Optional[_Union[Row20, _Mapping]] = ..., Archived: _Optional[_Union[Row21, _Mapping]] = ..., All: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row23
    def __init__(self, field_0: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("id", "value_type", "nullable", "unit")
    ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    id: str
    value_type: Row24
    nullable: bool
    unit: str
    def __init__(self, id: _Optional[str] = ..., value_type: _Optional[_Union[Row24, _Mapping]] = ..., nullable: _Optional[bool] = ..., unit: _Optional[str] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row25
    version: bytes
    evidence_through: Row25
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row25, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
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
    Boolean: Row26
    Integer: Row27
    Int64: Row28
    Uint64: Row29
    Float64: Row30
    Text: Row31
    Datetime: Row32
    Severity: Row33
    Object: Row34
    Array: Row35
    def __init__(self, Boolean: _Optional[_Union[Row26, _Mapping]] = ..., Integer: _Optional[_Union[Row27, _Mapping]] = ..., Int64: _Optional[_Union[Row28, _Mapping]] = ..., Uint64: _Optional[_Union[Row29, _Mapping]] = ..., Float64: _Optional[_Union[Row30, _Mapping]] = ..., Text: _Optional[_Union[Row31, _Mapping]] = ..., Datetime: _Optional[_Union[Row32, _Mapping]] = ..., Severity: _Optional[_Union[Row33, _Mapping]] = ..., Object: _Optional[_Union[Row34, _Mapping]] = ..., Array: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row36
    sequence: int
    def __init__(self, history: _Optional[_Union[Row36, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
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
