from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("limit", "after_cycle_id", "cursor", "filter", "order")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_cycle_id: str
    cursor: Row3
    filter: Row4
    order: Row5
    def __init__(self, limit: _Optional[int] = ..., after_cycle_id: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ..., filter: _Optional[_Union[Row4, _Mapping]] = ..., order: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("cycles", "next_after_cycle_id", "cursor")
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycles: _containers.RepeatedCompositeFieldContainer[Row6]
    next_after_cycle_id: str
    cursor: Row3
    def __init__(self, cycles: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., next_after_cycle_id: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("text", "statuses", "created_after", "created_before", "archive")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AFTER_FIELD_NUMBER: _ClassVar[int]
    CREATED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedCompositeFieldContainer[Row7]
    created_after: Row8
    created_before: Row8
    archive: Row9
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., created_after: _Optional[_Union[Row8, _Mapping]] = ..., created_before: _Optional[_Union[Row8, _Mapping]] = ..., archive: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("key", "descending")
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: Row10
    descending: bool
    def __init__(self, key: _Optional[_Union[Row10, _Mapping]] = ..., descending: _Optional[bool] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("cycle_id", "metadata", "phase", "status", "case_count", "execution_count", "continue_automatically", "continue_on_failure", "observed_cursor", "created_at", "started_at", "updated_at", "result", "archived", "sealed", "running_count", "queued_count")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    CASE_COUNT_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_AUTOMATICALLY_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_ON_FAILURE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    RUNNING_COUNT_FIELD_NUMBER: _ClassVar[int]
    QUEUED_COUNT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: Row11
    phase: Row12
    status: Row7
    case_count: int
    execution_count: int
    continue_automatically: bool
    continue_on_failure: bool
    observed_cursor: Row3
    created_at: Row8
    started_at: Row8
    updated_at: Row8
    result: Row13
    archived: bool
    sealed: bool
    running_count: int
    queued_count: int
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[Row11, _Mapping]] = ..., phase: _Optional[_Union[Row12, _Mapping]] = ..., status: _Optional[_Union[Row7, _Mapping]] = ..., case_count: _Optional[int] = ..., execution_count: _Optional[int] = ..., continue_automatically: _Optional[bool] = ..., continue_on_failure: _Optional[bool] = ..., observed_cursor: _Optional[_Union[Row3, _Mapping]] = ..., created_at: _Optional[_Union[Row8, _Mapping]] = ..., started_at: _Optional[_Union[Row8, _Mapping]] = ..., updated_at: _Optional[_Union[Row8, _Mapping]] = ..., result: _Optional[_Union[Row13, _Mapping]] = ..., archived: _Optional[bool] = ..., sealed: _Optional[bool] = ..., running_count: _Optional[int] = ..., queued_count: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Unspecified", "Ready", "Running", "Paused", "Stopping", "Completed", "Failed", "Blocked", "Stopped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    READY_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    STOPPING_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    STOPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row14
    Ready: Row15
    Running: Row16
    Paused: Row17
    Stopping: Row18
    Completed: Row19
    Failed: Row20
    Blocked: Row21
    Stopped: Row22
    def __init__(self, Unspecified: _Optional[_Union[Row14, _Mapping]] = ..., Ready: _Optional[_Union[Row15, _Mapping]] = ..., Running: _Optional[_Union[Row16, _Mapping]] = ..., Paused: _Optional[_Union[Row17, _Mapping]] = ..., Stopping: _Optional[_Union[Row18, _Mapping]] = ..., Completed: _Optional[_Union[Row19, _Mapping]] = ..., Failed: _Optional[_Union[Row20, _Mapping]] = ..., Blocked: _Optional[_Union[Row21, _Mapping]] = ..., Stopped: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("Unarchived", "Archived", "All")
    UNARCHIVED_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    Unarchived: Row23
    Archived: Row24
    All: Row25
    def __init__(self, Unarchived: _Optional[_Union[Row23, _Mapping]] = ..., Archived: _Optional[_Union[Row24, _Mapping]] = ..., All: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Unspecified", "UpdatedAt", "CreatedAt", "Label")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    UPDATEDAT_FIELD_NUMBER: _ClassVar[int]
    CREATEDAT_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row26
    UpdatedAt: Row27
    CreatedAt: Row28
    Label: Row29
    def __init__(self, Unspecified: _Optional[_Union[Row26, _Mapping]] = ..., UpdatedAt: _Optional[_Union[Row27, _Mapping]] = ..., CreatedAt: _Optional[_Union[Row28, _Mapping]] = ..., Label: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("identifier", "label", "description", "tags", "attributes", "created_at", "updated_at")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    description: str
    tags: _containers.RepeatedScalarFieldContainer[str]
    attributes: _containers.RepeatedCompositeFieldContainer[Row30]
    created_at: Row8
    updated_at: Row8
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row30, _Mapping]]] = ..., created_at: _Optional[_Union[Row8, _Mapping]] = ..., updated_at: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Active", "Sealing", "Sealed")
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    SEALING_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    Active: Row31
    Sealing: Row32
    Sealed: Row33
    def __init__(self, Active: _Optional[_Union[Row31, _Mapping]] = ..., Sealing: _Optional[_Union[Row32, _Mapping]] = ..., Sealed: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row34
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row34, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row35
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row36
    Pending: Row37
    Passed: Row38
    Failed: Row39
    Blocked: Row40
    Cancelled: Row41
    def __init__(self, Unspecified: _Optional[_Union[Row36, _Mapping]] = ..., Pending: _Optional[_Union[Row37, _Mapping]] = ..., Passed: _Optional[_Union[Row38, _Mapping]] = ..., Failed: _Optional[_Union[Row39, _Mapping]] = ..., Blocked: _Optional[_Union[Row40, _Mapping]] = ..., Cancelled: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row42]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row42, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row43
    def __init__(self, kind: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row43(_message.Message):
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
    Null: Row44
    Bool: Row45
    Integer: Row46
    Unsigned: Row47
    Number: Row48
    Text: Row49
    Datetime: Row50
    Array: Row51
    Object: Row52
    def __init__(self, Null: _Optional[_Union[Row44, _Mapping]] = ..., Bool: _Optional[_Union[Row45, _Mapping]] = ..., Integer: _Optional[_Union[Row46, _Mapping]] = ..., Unsigned: _Optional[_Union[Row47, _Mapping]] = ..., Number: _Optional[_Union[Row48, _Mapping]] = ..., Text: _Optional[_Union[Row49, _Mapping]] = ..., Datetime: _Optional[_Union[Row50, _Mapping]] = ..., Array: _Optional[_Union[Row51, _Mapping]] = ..., Object: _Optional[_Union[Row52, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row53
    def __init__(self, field_0: _Optional[_Union[Row53, _Mapping]] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row54
    def __init__(self, field_0: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row55]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row55, _Mapping]]] = ...) -> None: ...

class Row55(_message.Message):
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
