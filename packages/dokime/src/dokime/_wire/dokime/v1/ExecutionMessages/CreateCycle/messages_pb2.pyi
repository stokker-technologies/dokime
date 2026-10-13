from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("plan", "metadata", "continue_on_failure", "context")
    PLAN_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_ON_FAILURE_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    plan: Row3
    metadata: Row4
    continue_on_failure: bool
    context: Row5
    def __init__(self, plan: _Optional[_Union[Row3, _Mapping]] = ..., metadata: _Optional[_Union[Row4, _Mapping]] = ..., continue_on_failure: _Optional[bool] = ..., context: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("cycle_id", "observed_cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    observed_cursor: Row6
    def __init__(self, cycle_id: _Optional[str] = ..., observed_cursor: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("selections", "max_cases", "case_ids")
    SELECTIONS_FIELD_NUMBER: _ClassVar[int]
    MAX_CASES_FIELD_NUMBER: _ClassVar[int]
    CASE_IDS_FIELD_NUMBER: _ClassVar[int]
    selections: _containers.RepeatedCompositeFieldContainer[Row7]
    max_cases: int
    case_ids: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, selections: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., max_cases: _Optional[int] = ..., case_ids: _Optional[_Iterable[str]] = ...) -> None: ...

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
    attributes: _containers.RepeatedCompositeFieldContainer[Row8]
    created_at: Row9
    updated_at: Row9
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., created_at: _Optional[_Union[Row9, _Mapping]] = ..., updated_at: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("correlation_id",)
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    correlation_id: str
    def __init__(self, correlation_id: _Optional[str] = ...) -> None: ...

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
    __slots__ = ("suite_id", "template_id", "parameters", "files")
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    template_id: str
    parameters: _containers.RepeatedCompositeFieldContainer[Row10]
    files: _containers.RepeatedCompositeFieldContainer[Row11]
    def __init__(self, suite_id: _Optional[str] = ..., template_id: _Optional[str] = ..., parameters: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., files: _Optional[_Iterable[_Union[Row11, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row12
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row13
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row14
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row15]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row15, _Mapping]]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[Row12]
    def __init__(self, values: _Optional[_Iterable[_Union[Row12, _Mapping]]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row16
    def __init__(self, kind: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row17
    def __init__(self, kind: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Value", "Clear_")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    CLEAR__FIELD_NUMBER: _ClassVar[int]
    Value: Row18
    Clear_: Row19
    def __init__(self, Value: _Optional[_Union[Row18, _Mapping]] = ..., Clear_: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
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

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
