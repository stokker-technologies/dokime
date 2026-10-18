from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "step_id", "limit", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    step_id: str
    limit: int
    cursor: Row3
    def __init__(self, execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., limit: _Optional[int] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("criteria", "next_cursor", "observed_cursor")
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    NEXT_CURSOR_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    criteria: _containers.RepeatedCompositeFieldContainer[Row4]
    next_cursor: Row3
    observed_cursor: Row5
    def __init__(self, criteria: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_cursor: _Optional[_Union[Row3, _Mapping]] = ..., observed_cursor: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("revision", "execution_id", "step_id", "kind", "next_ordinal")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    NEXT_ORDINAL_FIELD_NUMBER: _ClassVar[int]
    revision: Row5
    execution_id: str
    step_id: str
    kind: Row6
    next_ordinal: int
    def __init__(self, revision: _Optional[_Union[Row5, _Mapping]] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., kind: _Optional[_Union[Row6, _Mapping]] = ..., next_ordinal: _Optional[int] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("criterion_id", "metadata", "status", "observed", "message", "ordinal")
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    criterion_id: str
    metadata: Row7
    status: Row8
    observed: Row9
    message: str
    ordinal: int
    def __init__(self, criterion_id: _Optional[str] = ..., metadata: _Optional[_Union[Row7, _Mapping]] = ..., status: _Optional[_Union[Row8, _Mapping]] = ..., observed: _Optional[_Union[Row9, _Mapping]] = ..., message: _Optional[str] = ..., ordinal: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Steps", "Measurements", "Criteria")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    MEASUREMENTS_FIELD_NUMBER: _ClassVar[int]
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Steps: Row11
    Measurements: Row12
    Criteria: Row13
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Steps: _Optional[_Union[Row11, _Mapping]] = ..., Measurements: _Optional[_Union[Row12, _Mapping]] = ..., Criteria: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row14]
    created_at: Row15
    updated_at: Row15
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row14, _Mapping]]] = ..., created_at: _Optional[_Union[Row15, _Mapping]] = ..., updated_at: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Indeterminate", "Skipped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    INDETERMINATE_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row16
    Pending: Row17
    Passed: Row18
    Failed: Row19
    Indeterminate: Row20
    Skipped: Row21
    def __init__(self, Unspecified: _Optional[_Union[Row16, _Mapping]] = ..., Pending: _Optional[_Union[Row17, _Mapping]] = ..., Passed: _Optional[_Union[Row18, _Mapping]] = ..., Failed: _Optional[_Union[Row19, _Mapping]] = ..., Indeterminate: _Optional[_Union[Row20, _Mapping]] = ..., Skipped: _Optional[_Union[Row21, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row22]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row22, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row9
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row23
    def __init__(self, kind: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
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
    Null: Row24
    Bool: Row25
    Integer: Row26
    Unsigned: Row27
    Number: Row28
    Text: Row29
    Datetime: Row30
    Array: Row31
    Object: Row32
    def __init__(self, Null: _Optional[_Union[Row24, _Mapping]] = ..., Bool: _Optional[_Union[Row25, _Mapping]] = ..., Integer: _Optional[_Union[Row26, _Mapping]] = ..., Unsigned: _Optional[_Union[Row27, _Mapping]] = ..., Number: _Optional[_Union[Row28, _Mapping]] = ..., Text: _Optional[_Union[Row29, _Mapping]] = ..., Datetime: _Optional[_Union[Row30, _Mapping]] = ..., Array: _Optional[_Union[Row31, _Mapping]] = ..., Object: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row29(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row15
    def __init__(self, field_0: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row33
    def __init__(self, field_0: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row34
    def __init__(self, field_0: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row35]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row35, _Mapping]]] = ...) -> None: ...

class Row35(_message.Message):
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
