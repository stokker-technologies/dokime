from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("scope", "limit", "after_traceability_id", "cursor", "host_traceability_fingerprint")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    HOST_TRACEABILITY_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    scope: Row3
    limit: int
    after_traceability_id: str
    cursor: Row4
    host_traceability_fingerprint: str
    def __init__(self, scope: _Optional[_Union[Row3, _Mapping]] = ..., limit: _Optional[int] = ..., after_traceability_id: _Optional[str] = ..., cursor: _Optional[_Union[Row4, _Mapping]] = ..., host_traceability_fingerprint: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("scope", "snapshots", "cursor", "next_after_traceability_id", "complete", "host_traceability_fingerprint")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOTS_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    HOST_TRACEABILITY_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    scope: Row3
    snapshots: _containers.RepeatedCompositeFieldContainer[Row5]
    cursor: Row4
    next_after_traceability_id: str
    complete: bool
    host_traceability_fingerprint: str
    def __init__(self, scope: _Optional[_Union[Row3, _Mapping]] = ..., snapshots: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., cursor: _Optional[_Union[Row4, _Mapping]] = ..., next_after_traceability_id: _Optional[str] = ..., complete: _Optional[bool] = ..., host_traceability_fingerprint: _Optional[str] = ...) -> None: ...

class Row3(_message.Message):
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
    origin: Row6
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("traceability_id", "scope", "created_at", "entries", "metadata")
    TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    traceability_id: str
    scope: Row3
    created_at: Row7
    entries: _containers.RepeatedCompositeFieldContainer[Row8]
    metadata: Row9
    def __init__(self, traceability_id: _Optional[str] = ..., scope: _Optional[_Union[Row3, _Mapping]] = ..., created_at: _Optional[_Union[Row7, _Mapping]] = ..., entries: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., metadata: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row10
    def __init__(self, kind: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("metadata", "value", "source", "unit")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    metadata: Row9
    value: Row11
    source: str
    unit: str
    def __init__(self, metadata: _Optional[_Union[Row9, _Mapping]] = ..., value: _Optional[_Union[Row11, _Mapping]] = ..., source: _Optional[str] = ..., unit: _Optional[str] = ...) -> None: ...

class Row9(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row12]
    created_at: Row7
    updated_at: Row7
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row12, _Mapping]]] = ..., created_at: _Optional[_Union[Row7, _Mapping]] = ..., updated_at: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row13
    Initialization: Row14
    StepAttemptId: Row15
    Cleanup: Row16
    def __init__(self, Default: _Optional[_Union[Row13, _Mapping]] = ..., Initialization: _Optional[_Union[Row14, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row15, _Mapping]] = ..., Cleanup: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row17]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row17, _Mapping]]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row11
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row18
    def __init__(self, kind: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
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
    Null: Row19
    Bool: Row20
    Integer: Row21
    Unsigned: Row22
    Number: Row23
    Text: Row24
    Datetime: Row25
    Array: Row26
    Object: Row27
    def __init__(self, Null: _Optional[_Union[Row19, _Mapping]] = ..., Bool: _Optional[_Union[Row20, _Mapping]] = ..., Integer: _Optional[_Union[Row21, _Mapping]] = ..., Unsigned: _Optional[_Union[Row22, _Mapping]] = ..., Number: _Optional[_Union[Row23, _Mapping]] = ..., Text: _Optional[_Union[Row24, _Mapping]] = ..., Datetime: _Optional[_Union[Row25, _Mapping]] = ..., Array: _Optional[_Union[Row26, _Mapping]] = ..., Object: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row7
    def __init__(self, field_0: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row28
    def __init__(self, field_0: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row30]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row30, _Mapping]]] = ...) -> None: ...

class Row30(_message.Message):
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
