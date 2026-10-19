from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("prompt_id", "action", "values", "accepted", "context")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    action: str
    values: Row3
    accepted: bool
    context: Row4
    def __init__(self, prompt_id: _Optional[str] = ..., action: _Optional[str] = ..., values: _Optional[_Union[Row3, _Mapping]] = ..., accepted: _Optional[bool] = ..., context: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("command_id", "prompt_id", "committed_revision", "accepted_at", "observed_cursor")
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    COMMITTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    command_id: str
    prompt_id: str
    committed_revision: int
    accepted_at: Row5
    observed_cursor: Row6
    def __init__(self, command_id: _Optional[str] = ..., prompt_id: _Optional[str] = ..., committed_revision: _Optional[int] = ..., accepted_at: _Optional[_Union[Row5, _Mapping]] = ..., observed_cursor: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row7]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("correlation_id",)
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    correlation_id: str
    def __init__(self, correlation_id: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row8
    def __init__(self, kind: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
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
    Null: Row9
    Bool: Row10
    Integer: Row11
    Unsigned: Row12
    Number: Row13
    Text: Row14
    Datetime: Row15
    Array: Row16
    Object: Row17
    def __init__(self, Null: _Optional[_Union[Row9, _Mapping]] = ..., Bool: _Optional[_Union[Row10, _Mapping]] = ..., Integer: _Optional[_Union[Row11, _Mapping]] = ..., Unsigned: _Optional[_Union[Row12, _Mapping]] = ..., Number: _Optional[_Union[Row13, _Mapping]] = ..., Text: _Optional[_Union[Row14, _Mapping]] = ..., Datetime: _Optional[_Union[Row15, _Mapping]] = ..., Array: _Optional[_Union[Row16, _Mapping]] = ..., Object: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row10(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row5
    def __init__(self, field_0: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row18
    def __init__(self, field_0: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row19
    def __init__(self, field_0: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row20]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row20, _Mapping]]] = ...) -> None: ...

class Row20(_message.Message):
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
