from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class QuerySource(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    QUERY_SOURCE_UNSPECIFIED: _ClassVar[QuerySource]
    QUERY_SOURCE_CYCLES: _ClassVar[QuerySource]
    QUERY_SOURCE_CASES: _ClassVar[QuerySource]
    QUERY_SOURCE_EXECUTIONS: _ClassVar[QuerySource]
    QUERY_SOURCE_STEPS: _ClassVar[QuerySource]
    QUERY_SOURCE_RECORDS: _ClassVar[QuerySource]

class QueryFilterOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    QUERY_FILTER_OPERATOR_UNSPECIFIED: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_EQ: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_NEQ: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_IN: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_GT: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_GTE: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_LT: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_LTE: _ClassVar[QueryFilterOperator]
    QUERY_FILTER_OPERATOR_IS_NULL: _ClassVar[QueryFilterOperator]

class AggregateFunction(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    AGGREGATE_FUNCTION_UNSPECIFIED: _ClassVar[AggregateFunction]
    AGGREGATE_FUNCTION_COUNT: _ClassVar[AggregateFunction]
    AGGREGATE_FUNCTION_SUM: _ClassVar[AggregateFunction]
    AGGREGATE_FUNCTION_MEAN: _ClassVar[AggregateFunction]
    AGGREGATE_FUNCTION_MIN: _ClassVar[AggregateFunction]
    AGGREGATE_FUNCTION_MAX: _ClassVar[AggregateFunction]
QUERY_SOURCE_UNSPECIFIED: QuerySource
QUERY_SOURCE_CYCLES: QuerySource
QUERY_SOURCE_CASES: QuerySource
QUERY_SOURCE_EXECUTIONS: QuerySource
QUERY_SOURCE_STEPS: QuerySource
QUERY_SOURCE_RECORDS: QuerySource
QUERY_FILTER_OPERATOR_UNSPECIFIED: QueryFilterOperator
QUERY_FILTER_OPERATOR_EQ: QueryFilterOperator
QUERY_FILTER_OPERATOR_NEQ: QueryFilterOperator
QUERY_FILTER_OPERATOR_IN: QueryFilterOperator
QUERY_FILTER_OPERATOR_GT: QueryFilterOperator
QUERY_FILTER_OPERATOR_GTE: QueryFilterOperator
QUERY_FILTER_OPERATOR_LT: QueryFilterOperator
QUERY_FILTER_OPERATOR_LTE: QueryFilterOperator
QUERY_FILTER_OPERATOR_IS_NULL: QueryFilterOperator
AGGREGATE_FUNCTION_UNSPECIFIED: AggregateFunction
AGGREGATE_FUNCTION_COUNT: AggregateFunction
AGGREGATE_FUNCTION_SUM: AggregateFunction
AGGREGATE_FUNCTION_MEAN: AggregateFunction
AGGREGATE_FUNCTION_MIN: AggregateFunction
AGGREGATE_FUNCTION_MAX: AggregateFunction

class AggregateRequest(_message.Message):
    __slots__ = ("source", "filters", "group_by", "aggregates", "limit")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    GROUP_BY_FIELD_NUMBER: _ClassVar[int]
    AGGREGATES_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    source: QuerySource
    filters: _containers.RepeatedCompositeFieldContainer[QueryFilter]
    group_by: _containers.RepeatedScalarFieldContainer[str]
    aggregates: _containers.RepeatedCompositeFieldContainer[QueryAggregate]
    limit: int
    def __init__(self, source: _Optional[_Union[QuerySource, str]] = ..., filters: _Optional[_Iterable[_Union[QueryFilter, _Mapping]]] = ..., group_by: _Optional[_Iterable[str]] = ..., aggregates: _Optional[_Iterable[_Union[QueryAggregate, _Mapping]]] = ..., limit: _Optional[int] = ...) -> None: ...

class QueryFilter(_message.Message):
    __slots__ = ("column", "op", "values")
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    column: str
    op: QueryFilterOperator
    values: _containers.RepeatedCompositeFieldContainer[_value_pb2.Value]
    def __init__(self, column: _Optional[str] = ..., op: _Optional[_Union[QueryFilterOperator, str]] = ..., values: _Optional[_Iterable[_Union[_value_pb2.Value, _Mapping]]] = ...) -> None: ...

class QueryAggregate(_message.Message):
    __slots__ = ("function", "column", "alias")
    FUNCTION_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    ALIAS_FIELD_NUMBER: _ClassVar[int]
    function: AggregateFunction
    column: str
    alias: str
    def __init__(self, function: _Optional[_Union[AggregateFunction, str]] = ..., column: _Optional[str] = ..., alias: _Optional[str] = ...) -> None: ...

class AggregateReply(_message.Message):
    __slots__ = ("columns", "rows", "complete")
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    columns: _containers.RepeatedScalarFieldContainer[str]
    rows: _containers.RepeatedCompositeFieldContainer[QueryRow]
    complete: bool
    def __init__(self, columns: _Optional[_Iterable[str]] = ..., rows: _Optional[_Iterable[_Union[QueryRow, _Mapping]]] = ..., complete: _Optional[bool] = ...) -> None: ...

class QueryRow(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[_value_pb2.Value]
    def __init__(self, values: _Optional[_Iterable[_Union[_value_pb2.Value, _Mapping]]] = ...) -> None: ...
