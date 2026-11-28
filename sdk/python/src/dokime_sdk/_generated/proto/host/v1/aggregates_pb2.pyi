from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import expressions_pb2 as _expressions_pb2
from dokime_sdk._generated.proto.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

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

class QuerySource(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    QUERY_SOURCE_UNSPECIFIED: _ClassVar[QuerySource]
    QUERY_SOURCE_CYCLES: _ClassVar[QuerySource]
    QUERY_SOURCE_CASES: _ClassVar[QuerySource]
    QUERY_SOURCE_EXECUTIONS: _ClassVar[QuerySource]
    QUERY_SOURCE_STEPS: _ClassVar[QuerySource]
    QUERY_SOURCE_RECORDS: _ClassVar[QuerySource]
QUERY_FILTER_OPERATOR_UNSPECIFIED: QueryFilterOperator
QUERY_FILTER_OPERATOR_EQ: QueryFilterOperator
QUERY_FILTER_OPERATOR_NEQ: QueryFilterOperator
QUERY_FILTER_OPERATOR_IN: QueryFilterOperator
QUERY_FILTER_OPERATOR_GT: QueryFilterOperator
QUERY_FILTER_OPERATOR_GTE: QueryFilterOperator
QUERY_FILTER_OPERATOR_LT: QueryFilterOperator
QUERY_FILTER_OPERATOR_LTE: QueryFilterOperator
QUERY_FILTER_OPERATOR_IS_NULL: QueryFilterOperator
QUERY_SOURCE_UNSPECIFIED: QuerySource
QUERY_SOURCE_CYCLES: QuerySource
QUERY_SOURCE_CASES: QuerySource
QUERY_SOURCE_EXECUTIONS: QuerySource
QUERY_SOURCE_STEPS: QuerySource
QUERY_SOURCE_RECORDS: QuerySource

@final
class AggregatesGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    GROUP_BY_FIELD_NUMBER: _ClassVar[int]
    AGGREGATES_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    source: QuerySource
    filters: _containers.RepeatedCompositeFieldContainer[QueryFilter]
    group_by: _containers.RepeatedScalarFieldContainer[str]
    aggregates: _containers.RepeatedCompositeFieldContainer[QueryAggregate]
    limit: int
    at: _position_pb2.ReadCut
    def __init__(self, source: _Optional[_Union[QuerySource, str]] = ..., filters: _Optional[_Iterable[_Union[QueryFilter, _Mapping]]] = ..., group_by: _Optional[_Iterable[str]] = ..., aggregates: _Optional[_Iterable[_Union[QueryAggregate, _Mapping]]] = ..., limit: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class AggregatesGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: AggregateReply
    resolved_at: _position_pb2.ReadCut
    def __init__(self, value: _Optional[_Union[AggregateReply, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class AggregateReply(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    columns: _containers.RepeatedScalarFieldContainer[str]
    rows: _containers.RepeatedCompositeFieldContainer[QueryRow]
    complete: bool
    def __init__(self, columns: _Optional[_Iterable[str]] = ..., rows: _Optional[_Iterable[_Union[QueryRow, _Mapping]]] = ..., complete: _Optional[bool] = ...) -> None: ...

@final
class QueryAggregate(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    ALIAS_FIELD_NUMBER: _ClassVar[int]
    PERCENTILE_FIELD_NUMBER: _ClassVar[int]
    aggregate: _expressions_pb2.Aggregate
    column: str
    alias: str
    percentile: float
    def __init__(self, aggregate: _Optional[_Union[_expressions_pb2.Aggregate, str]] = ..., column: _Optional[str] = ..., alias: _Optional[str] = ..., percentile: _Optional[float] = ...) -> None: ...

@final
class QueryFilter(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    column: str
    op: QueryFilterOperator
    values: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    def __init__(self, column: _Optional[str] = ..., op: _Optional[_Union[QueryFilterOperator, str]] = ..., values: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ...) -> None: ...

@final
class QueryRow(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    def __init__(self, values: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ...) -> None: ...
