from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id",)
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    def __init__(self, cycle_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
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
    metadata: Row3
    phase: Row4
    status: Row5
    case_count: int
    execution_count: int
    continue_automatically: bool
    continue_on_failure: bool
    observed_cursor: Row6
    created_at: Row7
    started_at: Row7
    updated_at: Row7
    result: Row8
    archived: bool
    sealed: bool
    running_count: int
    queued_count: int
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[Row3, _Mapping]] = ..., phase: _Optional[_Union[Row4, _Mapping]] = ..., status: _Optional[_Union[Row5, _Mapping]] = ..., case_count: _Optional[int] = ..., execution_count: _Optional[int] = ..., continue_automatically: _Optional[bool] = ..., continue_on_failure: _Optional[bool] = ..., observed_cursor: _Optional[_Union[Row6, _Mapping]] = ..., created_at: _Optional[_Union[Row7, _Mapping]] = ..., started_at: _Optional[_Union[Row7, _Mapping]] = ..., updated_at: _Optional[_Union[Row7, _Mapping]] = ..., result: _Optional[_Union[Row8, _Mapping]] = ..., archived: _Optional[bool] = ..., sealed: _Optional[bool] = ..., running_count: _Optional[int] = ..., queued_count: _Optional[int] = ...) -> None: ...

class Row3(_message.Message):
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
    created_at: Row7
    updated_at: Row7
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., created_at: _Optional[_Union[Row7, _Mapping]] = ..., updated_at: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Active", "Sealing", "Sealed")
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    SEALING_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    Active: Row10
    Sealing: Row11
    Sealed: Row12
    def __init__(self, Active: _Optional[_Union[Row10, _Mapping]] = ..., Sealing: _Optional[_Union[Row11, _Mapping]] = ..., Sealed: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
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
    Unspecified: Row13
    Ready: Row14
    Running: Row15
    Paused: Row16
    Stopping: Row17
    Completed: Row18
    Failed: Row19
    Blocked: Row20
    Stopped: Row21
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., Ready: _Optional[_Union[Row14, _Mapping]] = ..., Running: _Optional[_Union[Row15, _Mapping]] = ..., Paused: _Optional[_Union[Row16, _Mapping]] = ..., Stopping: _Optional[_Union[Row17, _Mapping]] = ..., Completed: _Optional[_Union[Row18, _Mapping]] = ..., Failed: _Optional[_Union[Row19, _Mapping]] = ..., Blocked: _Optional[_Union[Row20, _Mapping]] = ..., Stopped: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row22
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row22, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row23
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row24
    Pending: Row25
    Passed: Row26
    Failed: Row27
    Blocked: Row28
    Cancelled: Row29
    def __init__(self, Unspecified: _Optional[_Union[Row24, _Mapping]] = ..., Pending: _Optional[_Union[Row25, _Mapping]] = ..., Passed: _Optional[_Union[Row26, _Mapping]] = ..., Failed: _Optional[_Union[Row27, _Mapping]] = ..., Blocked: _Optional[_Union[Row28, _Mapping]] = ..., Cancelled: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row30]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row30, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row31
    def __init__(self, kind: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
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
    Null: Row32
    Bool: Row33
    Integer: Row34
    Unsigned: Row35
    Number: Row36
    Text: Row37
    Datetime: Row38
    Array: Row39
    Object: Row40
    def __init__(self, Null: _Optional[_Union[Row32, _Mapping]] = ..., Bool: _Optional[_Union[Row33, _Mapping]] = ..., Integer: _Optional[_Union[Row34, _Mapping]] = ..., Unsigned: _Optional[_Union[Row35, _Mapping]] = ..., Number: _Optional[_Union[Row36, _Mapping]] = ..., Text: _Optional[_Union[Row37, _Mapping]] = ..., Datetime: _Optional[_Union[Row38, _Mapping]] = ..., Array: _Optional[_Union[Row39, _Mapping]] = ..., Object: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row7
    def __init__(self, field_0: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row41
    def __init__(self, field_0: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row42
    def __init__(self, field_0: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row43]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row43, _Mapping]]] = ...) -> None: ...

class Row43(_message.Message):
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
