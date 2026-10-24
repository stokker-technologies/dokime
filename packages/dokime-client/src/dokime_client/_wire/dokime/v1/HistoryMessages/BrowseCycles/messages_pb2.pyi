from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("limit", "at", "after_sequence", "filter")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    AFTER_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    limit: int
    at: Row3
    after_sequence: int
    filter: Row4
    def __init__(self, limit: _Optional[int] = ..., at: _Optional[_Union[Row3, _Mapping]] = ..., after_sequence: _Optional[int] = ..., filter: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("entries", "at", "next_after_sequence")
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[Row5]
    at: Row3
    next_after_sequence: int
    def __init__(self, entries: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., at: _Optional[_Union[Row3, _Mapping]] = ..., next_after_sequence: _Optional[int] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row6
    sequence: int
    def __init__(self, history: _Optional[_Union[Row6, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("cycle_id", "metadata", "phase", "created_at", "updated_at", "execution_count", "archived")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: Row10
    phase: Row11
    created_at: Row8
    updated_at: Row8
    execution_count: int
    archived: bool
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[Row10, _Mapping]] = ..., phase: _Optional[_Union[Row11, _Mapping]] = ..., created_at: _Optional[_Union[Row8, _Mapping]] = ..., updated_at: _Optional[_Union[Row8, _Mapping]] = ..., execution_count: _Optional[int] = ..., archived: _Optional[bool] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

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
    Unspecified: Row12
    Ready: Row13
    Running: Row14
    Paused: Row15
    Stopping: Row16
    Completed: Row17
    Failed: Row18
    Blocked: Row19
    Stopped: Row20
    def __init__(self, Unspecified: _Optional[_Union[Row12, _Mapping]] = ..., Ready: _Optional[_Union[Row13, _Mapping]] = ..., Running: _Optional[_Union[Row14, _Mapping]] = ..., Paused: _Optional[_Union[Row15, _Mapping]] = ..., Stopping: _Optional[_Union[Row16, _Mapping]] = ..., Completed: _Optional[_Union[Row17, _Mapping]] = ..., Failed: _Optional[_Union[Row18, _Mapping]] = ..., Blocked: _Optional[_Union[Row19, _Mapping]] = ..., Stopped: _Optional[_Union[Row20, _Mapping]] = ...) -> None: ...

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
    Unarchived: Row21
    Archived: Row22
    All: Row23
    def __init__(self, Unarchived: _Optional[_Union[Row21, _Mapping]] = ..., Archived: _Optional[_Union[Row22, _Mapping]] = ..., All: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row24]
    created_at: Row8
    updated_at: Row8
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row24, _Mapping]]] = ..., created_at: _Optional[_Union[Row8, _Mapping]] = ..., updated_at: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Active", "Sealing", "Sealed")
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    SEALING_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    Active: Row25
    Sealing: Row26
    Sealed: Row27
    def __init__(self, Active: _Optional[_Union[Row25, _Mapping]] = ..., Sealing: _Optional[_Union[Row26, _Mapping]] = ..., Sealed: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row28
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row29]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row29, _Mapping]]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row30
    def __init__(self, kind: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row30(_message.Message):
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
    Null: Row31
    Bool: Row32
    Integer: Row33
    Unsigned: Row34
    Number: Row35
    Text: Row36
    Datetime: Row37
    Array: Row38
    Object: Row39
    def __init__(self, Null: _Optional[_Union[Row31, _Mapping]] = ..., Bool: _Optional[_Union[Row32, _Mapping]] = ..., Integer: _Optional[_Union[Row33, _Mapping]] = ..., Unsigned: _Optional[_Union[Row34, _Mapping]] = ..., Number: _Optional[_Union[Row35, _Mapping]] = ..., Text: _Optional[_Union[Row36, _Mapping]] = ..., Datetime: _Optional[_Union[Row37, _Mapping]] = ..., Array: _Optional[_Union[Row38, _Mapping]] = ..., Object: _Optional[_Union[Row39, _Mapping]] = ...) -> None: ...

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
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row40
    def __init__(self, field_0: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row41
    def __init__(self, field_0: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row42]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row42, _Mapping]]] = ...) -> None: ...

class Row42(_message.Message):
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
