from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("summary", "suite", "parameter_count", "generation_index", "observed_cursor")
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    SUITE_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_COUNT_FIELD_NUMBER: _ClassVar[int]
    GENERATION_INDEX_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    summary: Row3
    suite: Row4
    parameter_count: int
    generation_index: int
    observed_cursor: Row5
    def __init__(self, summary: _Optional[_Union[Row3, _Mapping]] = ..., suite: _Optional[_Union[Row4, _Mapping]] = ..., parameter_count: _Optional[int] = ..., generation_index: _Optional[int] = ..., observed_cursor: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("case_id", "template_id", "metadata", "ordinal", "status", "latest_execution_id", "latest_execution_status", "attempt_count", "rerun_scheduled", "actions", "suite_id")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LATEST_EXECUTION_STATUS_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_COUNT_FIELD_NUMBER: _ClassVar[int]
    RERUN_SCHEDULED_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    template_id: str
    metadata: Row4
    ordinal: int
    status: Row6
    latest_execution_id: str
    latest_execution_status: Row7
    attempt_count: int
    rerun_scheduled: bool
    actions: Row8
    suite_id: str
    def __init__(self, case_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[Row4, _Mapping]] = ..., ordinal: _Optional[int] = ..., status: _Optional[_Union[Row6, _Mapping]] = ..., latest_execution_id: _Optional[str] = ..., latest_execution_status: _Optional[_Union[Row7, _Mapping]] = ..., attempt_count: _Optional[int] = ..., rerun_scheduled: _Optional[bool] = ..., actions: _Optional[_Union[Row8, _Mapping]] = ..., suite_id: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row9]
    created_at: Row10
    updated_at: Row10
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., created_at: _Optional[_Union[Row10, _Mapping]] = ..., updated_at: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Skipped", "Invalidated", "Planned", "Queued", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PLANNED_FIELD_NUMBER: _ClassVar[int]
    QUEUED_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row11
    Skipped: Row12
    Invalidated: Row13
    Planned: Row14
    Queued: Row15
    Running: Row16
    Passed: Row17
    Failed: Row18
    Blocked: Row19
    Cancelled: Row20
    Interrupted: Row21
    def __init__(self, Unspecified: _Optional[_Union[Row11, _Mapping]] = ..., Skipped: _Optional[_Union[Row12, _Mapping]] = ..., Invalidated: _Optional[_Union[Row13, _Mapping]] = ..., Planned: _Optional[_Union[Row14, _Mapping]] = ..., Queued: _Optional[_Union[Row15, _Mapping]] = ..., Running: _Optional[_Union[Row16, _Mapping]] = ..., Passed: _Optional[_Union[Row17, _Mapping]] = ..., Failed: _Optional[_Union[Row18, _Mapping]] = ..., Blocked: _Optional[_Union[Row19, _Mapping]] = ..., Cancelled: _Optional[_Union[Row20, _Mapping]] = ..., Interrupted: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
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
    Unspecified: Row22
    Invalidated: Row23
    Pending: Row24
    Running: Row25
    Passed: Row26
    Failed: Row27
    Blocked: Row28
    Cancelled: Row29
    Interrupted: Row30
    Skipped: Row31
    def __init__(self, Unspecified: _Optional[_Union[Row22, _Mapping]] = ..., Invalidated: _Optional[_Union[Row23, _Mapping]] = ..., Pending: _Optional[_Union[Row24, _Mapping]] = ..., Running: _Optional[_Union[Row25, _Mapping]] = ..., Passed: _Optional[_Union[Row26, _Mapping]] = ..., Failed: _Optional[_Union[Row27, _Mapping]] = ..., Blocked: _Optional[_Union[Row28, _Mapping]] = ..., Cancelled: _Optional[_Union[Row29, _Mapping]] = ..., Interrupted: _Optional[_Union[Row30, _Mapping]] = ..., Skipped: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("run", "schedule_rerun", "delete", "unavailable_reasons")
    RUN_FIELD_NUMBER: _ClassVar[int]
    SCHEDULE_RERUN_FIELD_NUMBER: _ClassVar[int]
    DELETE_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_REASONS_FIELD_NUMBER: _ClassVar[int]
    run: bool
    schedule_rerun: bool
    delete: bool
    unavailable_reasons: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, run: _Optional[bool] = ..., schedule_rerun: _Optional[bool] = ..., delete: _Optional[bool] = ..., unavailable_reasons: _Optional[_Iterable[str]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row32
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row33]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row33, _Mapping]]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row34
    def __init__(self, kind: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row34(_message.Message):
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
    Null: Row35
    Bool: Row36
    Integer: Row37
    Unsigned: Row38
    Number: Row39
    Text: Row40
    Datetime: Row41
    Array: Row42
    Object: Row43
    def __init__(self, Null: _Optional[_Union[Row35, _Mapping]] = ..., Bool: _Optional[_Union[Row36, _Mapping]] = ..., Integer: _Optional[_Union[Row37, _Mapping]] = ..., Unsigned: _Optional[_Union[Row38, _Mapping]] = ..., Number: _Optional[_Union[Row39, _Mapping]] = ..., Text: _Optional[_Union[Row40, _Mapping]] = ..., Datetime: _Optional[_Union[Row41, _Mapping]] = ..., Array: _Optional[_Union[Row42, _Mapping]] = ..., Object: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row44
    def __init__(self, field_0: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row43(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row45
    def __init__(self, field_0: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row46]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row46, _Mapping]]] = ...) -> None: ...

class Row46(_message.Message):
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
