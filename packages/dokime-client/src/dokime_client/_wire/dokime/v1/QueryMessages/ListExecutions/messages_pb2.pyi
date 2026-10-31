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
    __slots__ = ("template_ids", "cycle_ids", "statuses", "started_at", "archive")
    TEMPLATE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    template_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    statuses: _containers.RepeatedCompositeFieldContainer[Row10]
    started_at: Row11
    archive: Row12
    def __init__(self, template_ids: _Optional[_Iterable[str]] = ..., cycle_ids: _Optional[_Iterable[str]] = ..., statuses: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., started_at: _Optional[_Union[Row11, _Mapping]] = ..., archive: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("execution_id", "cycle_id", "case_id", "template_id", "label", "status", "started_at", "finished_at", "result", "availability", "checkpoint_available", "archived")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    AVAILABILITY_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_AVAILABLE_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    cycle_id: str
    case_id: str
    template_id: str
    label: str
    status: Row10
    started_at: Row15
    finished_at: Row15
    result: Row16
    availability: Row17
    checkpoint_available: bool
    archived: bool
    def __init__(self, execution_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., template_id: _Optional[str] = ..., label: _Optional[str] = ..., status: _Optional[_Union[Row10, _Mapping]] = ..., started_at: _Optional[_Union[Row15, _Mapping]] = ..., finished_at: _Optional[_Union[Row15, _Mapping]] = ..., result: _Optional[_Union[Row16, _Mapping]] = ..., availability: _Optional[_Union[Row17, _Mapping]] = ..., checkpoint_available: _Optional[bool] = ..., archived: _Optional[bool] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("identity", "versions", "fields")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    identity: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row18]
    def __init__(self, identity: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[_Union[Row18, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Invalidated", "Pending", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted", "Skipped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row19
    Invalidated: Row20
    Pending: Row21
    Running: Row22
    Passed: Row23
    Failed: Row24
    Blocked: Row25
    Cancelled: Row26
    Interrupted: Row27
    Skipped: Row28
    def __init__(self, Unspecified: _Optional[_Union[Row19, _Mapping]] = ..., Invalidated: _Optional[_Union[Row20, _Mapping]] = ..., Pending: _Optional[_Union[Row21, _Mapping]] = ..., Running: _Optional[_Union[Row22, _Mapping]] = ..., Passed: _Optional[_Union[Row23, _Mapping]] = ..., Failed: _Optional[_Union[Row24, _Mapping]] = ..., Blocked: _Optional[_Union[Row25, _Mapping]] = ..., Cancelled: _Optional[_Union[Row26, _Mapping]] = ..., Interrupted: _Optional[_Union[Row27, _Mapping]] = ..., Skipped: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

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
    Unarchived: Row29
    Archived: Row30
    All: Row31
    def __init__(self, Unarchived: _Optional[_Union[Row29, _Mapping]] = ..., Archived: _Optional[_Union[Row30, _Mapping]] = ..., All: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row33
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row33, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("Unspecified", "Live", "Stored", "Interrupted", "Uncheckpointed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LIVE_FIELD_NUMBER: _ClassVar[int]
    STORED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    UNCHECKPOINTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row34
    Live: Row35
    Stored: Row36
    Interrupted: Row37
    Uncheckpointed: Row38
    def __init__(self, Unspecified: _Optional[_Union[Row34, _Mapping]] = ..., Live: _Optional[_Union[Row35, _Mapping]] = ..., Stored: _Optional[_Union[Row36, _Mapping]] = ..., Interrupted: _Optional[_Union[Row37, _Mapping]] = ..., Uncheckpointed: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("id", "value_type", "nullable", "unit")
    ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    id: str
    value_type: Row39
    nullable: bool
    unit: str
    def __init__(self, id: _Optional[str] = ..., value_type: _Optional[_Union[Row39, _Mapping]] = ..., nullable: _Optional[bool] = ..., unit: _Optional[str] = ...) -> None: ...

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

class Row33(_message.Message):
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row41
    Pending: Row42
    Passed: Row43
    Failed: Row44
    Blocked: Row45
    Cancelled: Row46
    def __init__(self, Unspecified: _Optional[_Union[Row41, _Mapping]] = ..., Pending: _Optional[_Union[Row42, _Mapping]] = ..., Passed: _Optional[_Union[Row43, _Mapping]] = ..., Failed: _Optional[_Union[Row44, _Mapping]] = ..., Blocked: _Optional[_Union[Row45, _Mapping]] = ..., Cancelled: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row37(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row38(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row39(_message.Message):
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
    Boolean: Row47
    Integer: Row48
    Int64: Row49
    Uint64: Row50
    Float64: Row51
    Text: Row52
    Datetime: Row53
    Severity: Row54
    Object: Row55
    Array: Row56
    def __init__(self, Boolean: _Optional[_Union[Row47, _Mapping]] = ..., Integer: _Optional[_Union[Row48, _Mapping]] = ..., Int64: _Optional[_Union[Row49, _Mapping]] = ..., Uint64: _Optional[_Union[Row50, _Mapping]] = ..., Float64: _Optional[_Union[Row51, _Mapping]] = ..., Text: _Optional[_Union[Row52, _Mapping]] = ..., Datetime: _Optional[_Union[Row53, _Mapping]] = ..., Severity: _Optional[_Union[Row54, _Mapping]] = ..., Object: _Optional[_Union[Row55, _Mapping]] = ..., Array: _Optional[_Union[Row56, _Mapping]] = ...) -> None: ...

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
