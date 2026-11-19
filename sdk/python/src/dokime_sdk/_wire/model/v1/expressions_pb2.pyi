from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Aggregate(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    AGGREGATE_UNSPECIFIED: _ClassVar[Aggregate]
    AGGREGATE_COUNT: _ClassVar[Aggregate]
    AGGREGATE_SUM: _ClassVar[Aggregate]
    AGGREGATE_MIN: _ClassVar[Aggregate]
    AGGREGATE_MAX: _ClassVar[Aggregate]
    AGGREGATE_MEAN: _ClassVar[Aggregate]
    AGGREGATE_STDDEV: _ClassVar[Aggregate]
    AGGREGATE_MEDIAN: _ClassVar[Aggregate]
    AGGREGATE_PERCENTILE: _ClassVar[Aggregate]
    AGGREGATE_FIRST: _ClassVar[Aggregate]
    AGGREGATE_LAST: _ClassVar[Aggregate]
    AGGREGATE_RMS: _ClassVar[Aggregate]

class FilterOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FILTER_OPERATOR_UNSPECIFIED: _ClassVar[FilterOperator]
    FILTER_OPERATOR_EQ: _ClassVar[FilterOperator]
    FILTER_OPERATOR_NE: _ClassVar[FilterOperator]
    FILTER_OPERATOR_GT: _ClassVar[FilterOperator]
    FILTER_OPERATOR_GE: _ClassVar[FilterOperator]
    FILTER_OPERATOR_LT: _ClassVar[FilterOperator]
    FILTER_OPERATOR_LE: _ClassVar[FilterOperator]
    FILTER_OPERATOR_CONTAINS: _ClassVar[FilterOperator]
    FILTER_OPERATOR_EXISTS: _ClassVar[FilterOperator]
AGGREGATE_UNSPECIFIED: Aggregate
AGGREGATE_COUNT: Aggregate
AGGREGATE_SUM: Aggregate
AGGREGATE_MIN: Aggregate
AGGREGATE_MAX: Aggregate
AGGREGATE_MEAN: Aggregate
AGGREGATE_STDDEV: Aggregate
AGGREGATE_MEDIAN: Aggregate
AGGREGATE_PERCENTILE: Aggregate
AGGREGATE_FIRST: Aggregate
AGGREGATE_LAST: Aggregate
AGGREGATE_RMS: Aggregate
FILTER_OPERATOR_UNSPECIFIED: FilterOperator
FILTER_OPERATOR_EQ: FilterOperator
FILTER_OPERATOR_NE: FilterOperator
FILTER_OPERATOR_GT: FilterOperator
FILTER_OPERATOR_GE: FilterOperator
FILTER_OPERATOR_LT: FilterOperator
FILTER_OPERATOR_LE: FilterOperator
FILTER_OPERATOR_CONTAINS: FilterOperator
FILTER_OPERATOR_EXISTS: FilterOperator

@final
class DataExpression(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    LABEL_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    TRANSFORM_FIELD_NUMBER: _ClassVar[int]
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    label: str
    stream: FieldReference
    record: FieldReference
    parameter: str
    dimension: str
    artifact: str
    aggregate: AggregateExpression
    transform: TransformExpression
    literal: _scalar_pb2.Value
    def __init__(self, label: _Optional[str] = ..., stream: _Optional[_Union[FieldReference, _Mapping]] = ..., record: _Optional[_Union[FieldReference, _Mapping]] = ..., parameter: _Optional[str] = ..., dimension: _Optional[str] = ..., artifact: _Optional[str] = ..., aggregate: _Optional[_Union[AggregateExpression, _Mapping]] = ..., transform: _Optional[_Union[TransformExpression, _Mapping]] = ..., literal: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...

@final
class FieldReference(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    field: str
    def __init__(self, identifier: _Optional[str] = ..., field: _Optional[str] = ...) -> None: ...

@final
class AggregateExpression(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    INPUT_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PERCENTILE_FIELD_NUMBER: _ClassVar[int]
    input: DataExpression
    kind: Aggregate
    percentile: float
    def __init__(self, input: _Optional[_Union[DataExpression, _Mapping]] = ..., kind: _Optional[_Union[Aggregate, str]] = ..., percentile: _Optional[float] = ...) -> None: ...

@final
class TransformExpression(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    INPUT_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    TIME_BETWEEN_FIELD_NUMBER: _ClassVar[int]
    PER_CASE_FIELD_NUMBER: _ClassVar[int]
    SCALE_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    input: DataExpression
    detail: str
    time_between: str
    per_case: bool
    scale: float
    offset: float
    def __init__(self, input: _Optional[_Union[DataExpression, _Mapping]] = ..., detail: _Optional[str] = ..., time_between: _Optional[str] = ..., per_case: _Optional[bool] = ..., scale: _Optional[float] = ..., offset: _Optional[float] = ...) -> None: ...

@final
class DocumentFilter(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: DataExpression
    operator: FilterOperator
    value: _scalar_pb2.Value
    def __init__(self, expression: _Optional[_Union[DataExpression, _Mapping]] = ..., operator: _Optional[_Union[FilterOperator, str]] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
