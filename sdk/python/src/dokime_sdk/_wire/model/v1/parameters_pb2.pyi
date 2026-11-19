from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class DynamicDefaultKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DYNAMIC_DEFAULT_KEY_UNSPECIFIED: _ClassVar[DynamicDefaultKey]
    DYNAMIC_DEFAULT_KEY_OPERATOR_NAME: _ClassVar[DynamicDefaultKey]
    DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL: _ClassVar[DynamicDefaultKey]

class ParameterUnaryOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PARAMETER_UNARY_OPERATOR_UNSPECIFIED: _ClassVar[ParameterUnaryOperator]
    PARAMETER_UNARY_OPERATOR_NEGATE: _ClassVar[ParameterUnaryOperator]

class ParameterBinaryOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PARAMETER_BINARY_OPERATOR_UNSPECIFIED: _ClassVar[ParameterBinaryOperator]
    PARAMETER_BINARY_OPERATOR_ADD: _ClassVar[ParameterBinaryOperator]
    PARAMETER_BINARY_OPERATOR_SUBTRACT: _ClassVar[ParameterBinaryOperator]
    PARAMETER_BINARY_OPERATOR_MULTIPLY: _ClassVar[ParameterBinaryOperator]
    PARAMETER_BINARY_OPERATOR_DIVIDE: _ClassVar[ParameterBinaryOperator]
DYNAMIC_DEFAULT_KEY_UNSPECIFIED: DynamicDefaultKey
DYNAMIC_DEFAULT_KEY_OPERATOR_NAME: DynamicDefaultKey
DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL: DynamicDefaultKey
PARAMETER_UNARY_OPERATOR_UNSPECIFIED: ParameterUnaryOperator
PARAMETER_UNARY_OPERATOR_NEGATE: ParameterUnaryOperator
PARAMETER_BINARY_OPERATOR_UNSPECIFIED: ParameterBinaryOperator
PARAMETER_BINARY_OPERATOR_ADD: ParameterBinaryOperator
PARAMETER_BINARY_OPERATOR_SUBTRACT: ParameterBinaryOperator
PARAMETER_BINARY_OPERATOR_MULTIPLY: ParameterBinaryOperator
PARAMETER_BINARY_OPERATOR_DIVIDE: ParameterBinaryOperator

@final
class Parameter(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    GROUP_FIELD_NUMBER: _ClassVar[int]
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    EXPANSION_FIELD_NUMBER: _ClassVar[int]
    ELEMENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    MIN_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_ITEMS_FIELD_NUMBER: _ClassVar[int]
    RUNTIME_WRITABLE_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    type: _scalar_pb2.ValueType
    unit: str
    required: bool
    group: str
    expression: ParameterExpression
    choices: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    min: float
    max: float
    expansion: ParameterExpansion
    element_type: _scalar_pb2.ValueType
    min_items: int
    max_items: int
    runtime_writable: bool
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., type: _Optional[_Union[_scalar_pb2.ValueType, str]] = ..., unit: _Optional[str] = ..., required: _Optional[bool] = ..., group: _Optional[str] = ..., expression: _Optional[_Union[ParameterExpression, _Mapping]] = ..., choices: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., expansion: _Optional[_Union[ParameterExpansion, _Mapping]] = ..., element_type: _Optional[_Union[_scalar_pb2.ValueType, str]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ..., runtime_writable: _Optional[bool] = ...) -> None: ...

@final
class ParameterExpansion(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SINGLE_FIELD_NUMBER: _ClassVar[int]
    CARTESIAN_FIELD_NUMBER: _ClassVar[int]
    ZIP_FIELD_NUMBER: _ClassVar[int]
    single: bool
    cartesian: bool
    zip: str
    def __init__(self, single: _Optional[bool] = ..., cartesian: _Optional[bool] = ..., zip: _Optional[str] = ...) -> None: ...

@final
class ParameterExpression(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    ARANGE_FIELD_NUMBER: _ClassVar[int]
    LINSPACE_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    DYNAMIC_DEFAULT_FIELD_NUMBER: _ClassVar[int]
    UNARY_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    literal: _scalar_pb2.Value
    array: ParameterExpressions
    range: IntegerRange
    arange: NumberRange
    linspace: LinearSpace
    reference: str
    dynamic_default: DynamicDefault
    unary: ParameterUnary
    binary: ParameterBinary
    def __init__(self, literal: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., array: _Optional[_Union[ParameterExpressions, _Mapping]] = ..., range: _Optional[_Union[IntegerRange, _Mapping]] = ..., arange: _Optional[_Union[NumberRange, _Mapping]] = ..., linspace: _Optional[_Union[LinearSpace, _Mapping]] = ..., reference: _Optional[str] = ..., dynamic_default: _Optional[_Union[DynamicDefault, _Mapping]] = ..., unary: _Optional[_Union[ParameterUnary, _Mapping]] = ..., binary: _Optional[_Union[ParameterBinary, _Mapping]] = ...) -> None: ...

@final
class ParameterExpressions(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[ParameterExpression]
    def __init__(self, values: _Optional[_Iterable[_Union[ParameterExpression, _Mapping]]] = ...) -> None: ...

@final
class IntegerRange(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: int
    stop: int
    step: int
    def __init__(self, start: _Optional[int] = ..., stop: _Optional[int] = ..., step: _Optional[int] = ...) -> None: ...

@final
class NumberRange(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    step: float
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., step: _Optional[float] = ...) -> None: ...

@final
class LinearSpace(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    count: int
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

@final
class DynamicDefault(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    KEY_FIELD_NUMBER: _ClassVar[int]
    FALLBACK_FIELD_NUMBER: _ClassVar[int]
    key: DynamicDefaultKey
    fallback: ParameterExpression
    def __init__(self, key: _Optional[_Union[DynamicDefaultKey, str]] = ..., fallback: _Optional[_Union[ParameterExpression, _Mapping]] = ...) -> None: ...

@final
class ParameterUnary(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    operator: ParameterUnaryOperator
    value: ParameterExpression
    def __init__(self, operator: _Optional[_Union[ParameterUnaryOperator, str]] = ..., value: _Optional[_Union[ParameterExpression, _Mapping]] = ...) -> None: ...

@final
class ParameterBinary(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    operator: ParameterBinaryOperator
    left: ParameterExpression
    right: ParameterExpression
    def __init__(self, operator: _Optional[_Union[ParameterBinaryOperator, str]] = ..., left: _Optional[_Union[ParameterExpression, _Mapping]] = ..., right: _Optional[_Union[ParameterExpression, _Mapping]] = ...) -> None: ...

@final
class FileParameter(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    METADATA_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    MEDIA_TYPES_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    default_value: str
    required: bool
    media_types: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., default_value: _Optional[str] = ..., required: _Optional[bool] = ..., media_types: _Optional[_Iterable[str]] = ...) -> None: ...
