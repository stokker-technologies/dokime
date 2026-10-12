from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
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

class UnaryOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    UNARY_OPERATOR_UNSPECIFIED: _ClassVar[UnaryOperator]
    UNARY_OPERATOR_NEGATE: _ClassVar[UnaryOperator]

class BinaryOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    BINARY_OPERATOR_UNSPECIFIED: _ClassVar[BinaryOperator]
    BINARY_OPERATOR_ADD: _ClassVar[BinaryOperator]
    BINARY_OPERATOR_SUBTRACT: _ClassVar[BinaryOperator]
    BINARY_OPERATOR_MULTIPLY: _ClassVar[BinaryOperator]
    BINARY_OPERATOR_DIVIDE: _ClassVar[BinaryOperator]
DYNAMIC_DEFAULT_KEY_UNSPECIFIED: DynamicDefaultKey
DYNAMIC_DEFAULT_KEY_OPERATOR_NAME: DynamicDefaultKey
DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL: DynamicDefaultKey
UNARY_OPERATOR_UNSPECIFIED: UnaryOperator
UNARY_OPERATOR_NEGATE: UnaryOperator
BINARY_OPERATOR_UNSPECIFIED: BinaryOperator
BINARY_OPERATOR_ADD: BinaryOperator
BINARY_OPERATOR_SUBTRACT: BinaryOperator
BINARY_OPERATOR_MULTIPLY: BinaryOperator
BINARY_OPERATOR_DIVIDE: BinaryOperator

class ListSuitesRequest(_message.Message):
    __slots__ = ("limit", "after_suite_id", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_suite_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, limit: _Optional[int] = ..., after_suite_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListSuitesReply(_message.Message):
    __slots__ = ("suites", "next_after_suite_id", "cursor")
    SUITES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    suites: _containers.RepeatedCompositeFieldContainer[SuiteIdentity]
    next_after_suite_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, suites: _Optional[_Iterable[_Union[SuiteIdentity, _Mapping]]] = ..., next_after_suite_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class GetSuiteRequest(_message.Message):
    __slots__ = ("suite_id",)
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    def __init__(self, suite_id: _Optional[str] = ...) -> None: ...

class SuiteIdentity(_message.Message):
    __slots__ = ("suite_id", "source", "metadata", "product")
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PRODUCT_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    source: str
    metadata: _common_pb2.Metadata
    product: str
    def __init__(self, suite_id: _Optional[str] = ..., source: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., product: _Optional[str] = ...) -> None: ...

class SuiteDefinition(_message.Message):
    __slots__ = ("identity", "templates", "tags", "parameter_groups")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    TEMPLATES_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_GROUPS_FIELD_NUMBER: _ClassVar[int]
    identity: SuiteIdentity
    templates: _containers.RepeatedCompositeFieldContainer[TemplateDefinition]
    tags: _containers.RepeatedScalarFieldContainer[str]
    parameter_groups: _containers.RepeatedCompositeFieldContainer[ParameterGroupDefinition]
    def __init__(self, identity: _Optional[_Union[SuiteIdentity, _Mapping]] = ..., templates: _Optional[_Iterable[_Union[TemplateDefinition, _Mapping]]] = ..., tags: _Optional[_Iterable[str]] = ..., parameter_groups: _Optional[_Iterable[_Union[ParameterGroupDefinition, _Mapping]]] = ...) -> None: ...

class TemplateDefinition(_message.Message):
    __slots__ = ("template_id", "metadata", "parameters", "tags", "files", "steps", "parameter_groups", "continue_on_failure")
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_GROUPS_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_ON_FAILURE_FIELD_NUMBER: _ClassVar[int]
    template_id: str
    metadata: _common_pb2.Metadata
    parameters: _containers.RepeatedCompositeFieldContainer[ParameterSchema]
    tags: _containers.RepeatedScalarFieldContainer[str]
    files: _containers.RepeatedCompositeFieldContainer[FileParameterDefinition]
    steps: _containers.RepeatedCompositeFieldContainer[StepDefinition]
    parameter_groups: _containers.RepeatedCompositeFieldContainer[ParameterGroupDefinition]
    continue_on_failure: bool
    def __init__(self, template_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., parameters: _Optional[_Iterable[_Union[ParameterSchema, _Mapping]]] = ..., tags: _Optional[_Iterable[str]] = ..., files: _Optional[_Iterable[_Union[FileParameterDefinition, _Mapping]]] = ..., steps: _Optional[_Iterable[_Union[StepDefinition, _Mapping]]] = ..., parameter_groups: _Optional[_Iterable[_Union[ParameterGroupDefinition, _Mapping]]] = ..., continue_on_failure: _Optional[bool] = ...) -> None: ...

class FileParameterDefinition(_message.Message):
    __slots__ = ("metadata", "default_value", "required", "media_types", "attributes")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    MEDIA_TYPES_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    default_value: str
    required: bool
    media_types: _containers.RepeatedScalarFieldContainer[str]
    attributes: _value_pb2.ValueObject
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., default_value: _Optional[str] = ..., required: _Optional[bool] = ..., media_types: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...

class StepDefinition(_message.Message):
    __slots__ = ("step_id", "metadata", "repeatable")
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEATABLE_FIELD_NUMBER: _ClassVar[int]
    step_id: str
    metadata: _common_pb2.Metadata
    repeatable: bool
    def __init__(self, step_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., repeatable: _Optional[bool] = ...) -> None: ...

class ParameterGroupDefinition(_message.Message):
    __slots__ = ("group_id", "metadata")
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    group_id: str
    metadata: _common_pb2.Metadata
    def __init__(self, group_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ...) -> None: ...

class ParameterSchema(_message.Message):
    __slots__ = ("parameter_id", "metadata", "group_id", "value_type", "unit", "required", "runtime_writable", "expression", "selection", "extensions", "list", "min", "max", "expansion")
    class ExtensionsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _value_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ...) -> None: ...
    PARAMETER_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    RUNTIME_WRITABLE_FIELD_NUMBER: _ClassVar[int]
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    EXTENSIONS_FIELD_NUMBER: _ClassVar[int]
    LIST_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    EXPANSION_FIELD_NUMBER: _ClassVar[int]
    parameter_id: str
    metadata: _common_pb2.Metadata
    group_id: str
    value_type: _schema_pb2.ValueType
    unit: _schema_pb2.MeasurementUnit
    required: bool
    runtime_writable: bool
    expression: ParameterExpression
    selection: ParameterSelection
    extensions: _containers.MessageMap[str, _value_pb2.Value]
    list: ParameterList
    min: float
    max: float
    expansion: ParameterExpansion
    def __init__(self, parameter_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., group_id: _Optional[str] = ..., value_type: _Optional[_Union[_schema_pb2.ValueType, str]] = ..., unit: _Optional[_Union[_schema_pb2.MeasurementUnit, _Mapping]] = ..., required: _Optional[bool] = ..., runtime_writable: _Optional[bool] = ..., expression: _Optional[_Union[ParameterExpression, _Mapping]] = ..., selection: _Optional[_Union[ParameterSelection, _Mapping]] = ..., extensions: _Optional[_Mapping[str, _value_pb2.Value]] = ..., list: _Optional[_Union[ParameterList, _Mapping]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., expansion: _Optional[_Union[ParameterExpansion, _Mapping]] = ...) -> None: ...

class ParameterList(_message.Message):
    __slots__ = ("element_type", "min_items", "max_items")
    ELEMENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    MIN_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_ITEMS_FIELD_NUMBER: _ClassVar[int]
    element_type: _schema_pb2.ValueType
    min_items: int
    max_items: int
    def __init__(self, element_type: _Optional[_Union[_schema_pb2.ValueType, str]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

class ParameterSelection(_message.Message):
    __slots__ = ("choices",)
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    choices: _containers.RepeatedCompositeFieldContainer[_value_pb2.Value]
    def __init__(self, choices: _Optional[_Iterable[_Union[_value_pb2.Value, _Mapping]]] = ...) -> None: ...

class ParameterExpansion(_message.Message):
    __slots__ = ("single", "cartesian", "zip")
    SINGLE_FIELD_NUMBER: _ClassVar[int]
    CARTESIAN_FIELD_NUMBER: _ClassVar[int]
    ZIP_FIELD_NUMBER: _ClassVar[int]
    single: bool
    cartesian: bool
    zip: str
    def __init__(self, single: _Optional[bool] = ..., cartesian: _Optional[bool] = ..., zip: _Optional[str] = ...) -> None: ...

class ParameterExpression(_message.Message):
    __slots__ = ("literal", "array", "range", "arange", "linspace", "reference", "dynamic_default", "unary", "binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    ARANGE_FIELD_NUMBER: _ClassVar[int]
    LINSPACE_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    DYNAMIC_DEFAULT_FIELD_NUMBER: _ClassVar[int]
    UNARY_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    literal: _value_pb2.Value
    array: ParameterExpressions
    range: IntegerRange
    arange: NumberRange
    linspace: LinearSpace
    reference: str
    dynamic_default: DynamicDefault
    unary: UnaryExpression
    binary: BinaryExpression
    def __init__(self, literal: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., array: _Optional[_Union[ParameterExpressions, _Mapping]] = ..., range: _Optional[_Union[IntegerRange, _Mapping]] = ..., arange: _Optional[_Union[NumberRange, _Mapping]] = ..., linspace: _Optional[_Union[LinearSpace, _Mapping]] = ..., reference: _Optional[str] = ..., dynamic_default: _Optional[_Union[DynamicDefault, _Mapping]] = ..., unary: _Optional[_Union[UnaryExpression, _Mapping]] = ..., binary: _Optional[_Union[BinaryExpression, _Mapping]] = ...) -> None: ...

class ParameterExpressions(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[ParameterExpression]
    def __init__(self, values: _Optional[_Iterable[_Union[ParameterExpression, _Mapping]]] = ...) -> None: ...

class IntegerRange(_message.Message):
    __slots__ = ("start", "stop", "step")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: int
    stop: int
    step: int
    def __init__(self, start: _Optional[int] = ..., stop: _Optional[int] = ..., step: _Optional[int] = ...) -> None: ...

class NumberRange(_message.Message):
    __slots__ = ("start", "stop", "step")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    step: float
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., step: _Optional[float] = ...) -> None: ...

class LinearSpace(_message.Message):
    __slots__ = ("start", "stop", "count")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    count: int
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

class DynamicDefault(_message.Message):
    __slots__ = ("key", "fallback")
    KEY_FIELD_NUMBER: _ClassVar[int]
    FALLBACK_FIELD_NUMBER: _ClassVar[int]
    key: DynamicDefaultKey
    fallback: ParameterExpression
    def __init__(self, key: _Optional[_Union[DynamicDefaultKey, str]] = ..., fallback: _Optional[_Union[ParameterExpression, _Mapping]] = ...) -> None: ...

class UnaryExpression(_message.Message):
    __slots__ = ("op", "value")
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    op: UnaryOperator
    value: ParameterExpression
    def __init__(self, op: _Optional[_Union[UnaryOperator, str]] = ..., value: _Optional[_Union[ParameterExpression, _Mapping]] = ...) -> None: ...

class BinaryExpression(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: BinaryOperator
    left: ParameterExpression
    right: ParameterExpression
    def __init__(self, op: _Optional[_Union[BinaryOperator, str]] = ..., left: _Optional[_Union[ParameterExpression, _Mapping]] = ..., right: _Optional[_Union[ParameterExpression, _Mapping]] = ...) -> None: ...

class ListRecordSchemasRequest(_message.Message):
    __slots__ = ("limit", "after_record_type", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_record_type: str
    cursor: _common_pb2.Cursor
    def __init__(self, limit: _Optional[int] = ..., after_record_type: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListRecordSchemasReply(_message.Message):
    __slots__ = ("schemas", "next_after_record_type", "cursor")
    SCHEMAS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    schemas: _containers.RepeatedCompositeFieldContainer[RecordSchemaIdentity]
    next_after_record_type: str
    cursor: _common_pb2.Cursor
    def __init__(self, schemas: _Optional[_Iterable[_Union[RecordSchemaIdentity, _Mapping]]] = ..., next_after_record_type: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class RecordSchemaIdentity(_message.Message):
    __slots__ = ("record_type", "schema_version", "metadata")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    schema_version: int
    metadata: _common_pb2.Metadata
    def __init__(self, record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ...) -> None: ...

class GetRecordSchemaRequest(_message.Message):
    __slots__ = ("record_type",)
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    def __init__(self, record_type: _Optional[str] = ...) -> None: ...

class ListTelemetrySchemasRequest(_message.Message):
    __slots__ = ("limit", "after_stream", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_STREAM_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_stream: str
    cursor: _common_pb2.Cursor
    def __init__(self, limit: _Optional[int] = ..., after_stream: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListTelemetrySchemasReply(_message.Message):
    __slots__ = ("schemas", "next_after_stream", "cursor")
    SCHEMAS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_STREAM_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    schemas: _containers.RepeatedCompositeFieldContainer[TelemetrySchemaIdentity]
    next_after_stream: str
    cursor: _common_pb2.Cursor
    def __init__(self, schemas: _Optional[_Iterable[_Union[TelemetrySchemaIdentity, _Mapping]]] = ..., next_after_stream: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class TelemetrySchemaIdentity(_message.Message):
    __slots__ = ("stream", "schema_version", "metadata")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    stream: str
    schema_version: int
    metadata: _common_pb2.Metadata
    def __init__(self, stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ...) -> None: ...

class GetTelemetrySchemaRequest(_message.Message):
    __slots__ = ("stream",)
    STREAM_FIELD_NUMBER: _ClassVar[int]
    stream: str
    def __init__(self, stream: _Optional[str] = ...) -> None: ...
