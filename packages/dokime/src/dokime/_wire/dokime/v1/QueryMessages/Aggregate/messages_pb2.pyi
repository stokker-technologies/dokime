from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("source", "filters", "group_by", "aggregates", "limit")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    GROUP_BY_FIELD_NUMBER: _ClassVar[int]
    AGGREGATES_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    source: Row3
    filters: _containers.RepeatedCompositeFieldContainer[Row4]
    group_by: _containers.RepeatedScalarFieldContainer[str]
    aggregates: _containers.RepeatedCompositeFieldContainer[Row5]
    limit: int
    def __init__(self, source: _Optional[_Union[Row3, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., group_by: _Optional[_Iterable[str]] = ..., aggregates: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., limit: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("columns", "rows", "complete")
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    columns: _containers.RepeatedScalarFieldContainer[str]
    rows: _containers.RepeatedCompositeFieldContainer[Row6]
    complete: bool
    def __init__(self, columns: _Optional[_Iterable[str]] = ..., rows: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., complete: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Cycles", "Cases", "Executions", "Steps", "Records")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    CASES_FIELD_NUMBER: _ClassVar[int]
    EXECUTIONS_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row7
    Cycles: Row8
    Cases: Row9
    Executions: Row10
    Steps: Row11
    Records: Row12
    def __init__(self, Unspecified: _Optional[_Union[Row7, _Mapping]] = ..., Cycles: _Optional[_Union[Row8, _Mapping]] = ..., Cases: _Optional[_Union[Row9, _Mapping]] = ..., Executions: _Optional[_Union[Row10, _Mapping]] = ..., Steps: _Optional[_Union[Row11, _Mapping]] = ..., Records: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("column", "op", "values")
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    column: str
    op: Row13
    values: _containers.RepeatedCompositeFieldContainer[Row14]
    def __init__(self, column: _Optional[str] = ..., op: _Optional[_Union[Row13, _Mapping]] = ..., values: _Optional[_Iterable[_Union[Row14, _Mapping]]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("function", "column", "alias")
    FUNCTION_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    ALIAS_FIELD_NUMBER: _ClassVar[int]
    function: Row15
    column: str
    alias: str
    def __init__(self, function: _Optional[_Union[Row15, _Mapping]] = ..., column: _Optional[str] = ..., alias: _Optional[str] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[Row14]
    def __init__(self, values: _Optional[_Iterable[_Union[Row14, _Mapping]]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("Unspecified", "Eq", "Neq", "In", "Gt", "Gte", "Lt", "Lte", "IsNull")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    NEQ_FIELD_NUMBER: _ClassVar[int]
    IN_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    ISNULL_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row16
    Eq: Row17
    Neq: Row18
    In: Row19
    Gt: Row20
    Gte: Row21
    Lt: Row22
    Lte: Row23
    IsNull: Row24
    def __init__(self, Unspecified: _Optional[_Union[Row16, _Mapping]] = ..., Eq: _Optional[_Union[Row17, _Mapping]] = ..., Neq: _Optional[_Union[Row18, _Mapping]] = ..., In: _Optional[_Union[Row19, _Mapping]] = ..., Gt: _Optional[_Union[Row20, _Mapping]] = ..., Gte: _Optional[_Union[Row21, _Mapping]] = ..., Lt: _Optional[_Union[Row22, _Mapping]] = ..., Lte: _Optional[_Union[Row23, _Mapping]] = ..., IsNull: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row25]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row25, _Mapping]]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("Unspecified", "Count", "Sum", "Mean", "Min", "Max")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    SUM_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row26
    Count: Row27
    Sum: Row28
    Mean: Row29
    Min: Row30
    Max: Row31
    def __init__(self, Unspecified: _Optional[_Union[Row26, _Mapping]] = ..., Count: _Optional[_Union[Row27, _Mapping]] = ..., Sum: _Optional[_Union[Row28, _Mapping]] = ..., Mean: _Optional[_Union[Row29, _Mapping]] = ..., Min: _Optional[_Union[Row30, _Mapping]] = ..., Max: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row32
    def __init__(self, kind: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

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
    Null: Row33
    Bool: Row34
    Integer: Row35
    Unsigned: Row36
    Number: Row37
    Text: Row38
    Datetime: Row39
    Array: Row40
    Object: Row41
    def __init__(self, Null: _Optional[_Union[Row33, _Mapping]] = ..., Bool: _Optional[_Union[Row34, _Mapping]] = ..., Integer: _Optional[_Union[Row35, _Mapping]] = ..., Unsigned: _Optional[_Union[Row36, _Mapping]] = ..., Number: _Optional[_Union[Row37, _Mapping]] = ..., Text: _Optional[_Union[Row38, _Mapping]] = ..., Datetime: _Optional[_Union[Row39, _Mapping]] = ..., Array: _Optional[_Union[Row40, _Mapping]] = ..., Object: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row38(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row42
    def __init__(self, field_0: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row40(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row43
    def __init__(self, field_0: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row41(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row44
    def __init__(self, field_0: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row42(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row43(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row45]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row45, _Mapping]]] = ...) -> None: ...

class Row45(_message.Message):
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
