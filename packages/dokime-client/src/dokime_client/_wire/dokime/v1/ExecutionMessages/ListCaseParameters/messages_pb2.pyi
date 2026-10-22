from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("cycle_id", "case_id", "limit", "after_name", "cursor")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    limit: int
    after_name: str
    cursor: Row3
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_name: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("parameters", "next_after_name", "cursor")
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.RepeatedCompositeFieldContainer[Row4]
    next_after_name: str
    cursor: Row3
    def __init__(self, parameters: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_after_name: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("observed", "selection")
    OBSERVED_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    observed: Row5
    selection: Row6
    def __init__(self, observed: _Optional[_Union[Row5, _Mapping]] = ..., selection: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("name", "value")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: Row7
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("ExecutionInputsId", "ExecutionOutputsId", "PlannedCase")
    EXECUTIONINPUTSID_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONOUTPUTSID_FIELD_NUMBER: _ClassVar[int]
    PLANNEDCASE_FIELD_NUMBER: _ClassVar[int]
    ExecutionInputsId: Row8
    ExecutionOutputsId: Row9
    PlannedCase: Row10
    def __init__(self, ExecutionInputsId: _Optional[_Union[Row8, _Mapping]] = ..., ExecutionOutputsId: _Optional[_Union[Row9, _Mapping]] = ..., PlannedCase: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row11]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row11, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row12
    def __init__(self, field_0: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row13
    def __init__(self, kind: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("cycle_id", "case_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ...) -> None: ...

class Row13(_message.Message):
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
    Null: Row14
    Bool: Row15
    Integer: Row16
    Unsigned: Row17
    Number: Row18
    Text: Row19
    Datetime: Row20
    Array: Row21
    Object: Row22
    def __init__(self, Null: _Optional[_Union[Row14, _Mapping]] = ..., Bool: _Optional[_Union[Row15, _Mapping]] = ..., Integer: _Optional[_Union[Row16, _Mapping]] = ..., Unsigned: _Optional[_Union[Row17, _Mapping]] = ..., Number: _Optional[_Union[Row18, _Mapping]] = ..., Text: _Optional[_Union[Row19, _Mapping]] = ..., Datetime: _Optional[_Union[Row20, _Mapping]] = ..., Array: _Optional[_Union[Row21, _Mapping]] = ..., Object: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row23
    def __init__(self, field_0: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row24
    def __init__(self, field_0: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row25
    def __init__(self, field_0: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row26]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row26, _Mapping]]] = ...) -> None: ...

class Row26(_message.Message):
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
