from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("suite_id",)
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    def __init__(self, suite_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("identity", "templates", "tags", "parameter_groups")
    IDENTITY_FIELD_NUMBER: _ClassVar[int]
    TEMPLATES_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_GROUPS_FIELD_NUMBER: _ClassVar[int]
    identity: Row3
    templates: _containers.RepeatedCompositeFieldContainer[Row4]
    tags: _containers.RepeatedScalarFieldContainer[str]
    parameter_groups: _containers.RepeatedCompositeFieldContainer[Row5]
    def __init__(self, identity: _Optional[_Union[Row3, _Mapping]] = ..., templates: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., tags: _Optional[_Iterable[str]] = ..., parameter_groups: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("suite_id", "source", "metadata", "product")
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PRODUCT_FIELD_NUMBER: _ClassVar[int]
    suite_id: str
    source: str
    metadata: Row6
    product: str
    def __init__(self, suite_id: _Optional[str] = ..., source: _Optional[str] = ..., metadata: _Optional[_Union[Row6, _Mapping]] = ..., product: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
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
    metadata: Row6
    parameters: _containers.RepeatedCompositeFieldContainer[Row7]
    tags: _containers.RepeatedScalarFieldContainer[str]
    files: _containers.RepeatedCompositeFieldContainer[Row8]
    steps: _containers.RepeatedCompositeFieldContainer[Row9]
    parameter_groups: _containers.RepeatedCompositeFieldContainer[Row5]
    continue_on_failure: bool
    def __init__(self, template_id: _Optional[str] = ..., metadata: _Optional[_Union[Row6, _Mapping]] = ..., parameters: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., tags: _Optional[_Iterable[str]] = ..., files: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., steps: _Optional[_Iterable[_Union[Row9, _Mapping]]] = ..., parameter_groups: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., continue_on_failure: _Optional[bool] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("group_id", "metadata")
    GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    group_id: str
    metadata: Row6
    def __init__(self, group_id: _Optional[str] = ..., metadata: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row10]
    created_at: Row11
    updated_at: Row11
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row10, _Mapping]]] = ..., created_at: _Optional[_Union[Row11, _Mapping]] = ..., updated_at: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("parameter_id", "metadata", "group_id", "value_type", "unit", "required", "runtime_writable", "expression", "selection", "extensions", "list", "min", "max", "expansion")
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
    metadata: Row6
    group_id: str
    value_type: Row12
    unit: Row13
    required: bool
    runtime_writable: bool
    expression: Row14
    selection: Row15
    extensions: _containers.RepeatedCompositeFieldContainer[Row16]
    list: Row17
    min: float
    max: float
    expansion: Row18
    def __init__(self, parameter_id: _Optional[str] = ..., metadata: _Optional[_Union[Row6, _Mapping]] = ..., group_id: _Optional[str] = ..., value_type: _Optional[_Union[Row12, _Mapping]] = ..., unit: _Optional[_Union[Row13, _Mapping]] = ..., required: _Optional[bool] = ..., runtime_writable: _Optional[bool] = ..., expression: _Optional[_Union[Row14, _Mapping]] = ..., selection: _Optional[_Union[Row15, _Mapping]] = ..., extensions: _Optional[_Iterable[_Union[Row16, _Mapping]]] = ..., list: _Optional[_Union[Row17, _Mapping]] = ..., min: _Optional[float] = ..., max: _Optional[float] = ..., expansion: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("metadata", "default_value", "required", "media_types", "attributes")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    MEDIA_TYPES_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    metadata: Row6
    default_value: str
    required: bool
    media_types: _containers.RepeatedScalarFieldContainer[str]
    attributes: Row19
    def __init__(self, metadata: _Optional[_Union[Row6, _Mapping]] = ..., default_value: _Optional[str] = ..., required: _Optional[bool] = ..., media_types: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("step_id", "metadata", "repeatable")
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REPEATABLE_FIELD_NUMBER: _ClassVar[int]
    step_id: str
    metadata: Row6
    repeatable: bool
    def __init__(self, step_id: _Optional[str] = ..., metadata: _Optional[_Union[Row6, _Mapping]] = ..., repeatable: _Optional[bool] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row19
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Unspecified", "Boolean", "Integer", "Number", "Text", "File", "Object", "Array", "Datetime")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    OBJECT_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    DATETIME_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row20
    Boolean: Row21
    Integer: Row22
    Number: Row23
    Text: Row24
    File: Row25
    Object: Row26
    Array: Row27
    Datetime: Row28
    def __init__(self, Unspecified: _Optional[_Union[Row20, _Mapping]] = ..., Boolean: _Optional[_Union[Row21, _Mapping]] = ..., Integer: _Optional[_Union[Row22, _Mapping]] = ..., Number: _Optional[_Union[Row23, _Mapping]] = ..., Text: _Optional[_Union[Row24, _Mapping]] = ..., File: _Optional[_Union[Row25, _Mapping]] = ..., Object: _Optional[_Union[Row26, _Mapping]] = ..., Array: _Optional[_Union[Row27, _Mapping]] = ..., Datetime: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: Row6
    family: Row29
    symbol: str
    def __init__(self, metadata: _Optional[_Union[Row6, _Mapping]] = ..., family: _Optional[_Union[Row29, _Mapping]] = ..., symbol: _Optional[str] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row30]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row30, _Mapping]]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("choices",)
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    choices: _containers.RepeatedCompositeFieldContainer[Row19]
    def __init__(self, choices: _Optional[_Iterable[_Union[Row19, _Mapping]]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row19
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("element_type", "min_items", "max_items")
    ELEMENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    MIN_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_ITEMS_FIELD_NUMBER: _ClassVar[int]
    element_type: Row12
    min_items: int
    max_items: int
    def __init__(self, element_type: _Optional[_Union[Row12, _Mapping]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row31
    def __init__(self, kind: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row32]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row32, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("Unspecified", "Count", "Distance", "Duration", "Frequency", "Power", "Temperature", "Voltage", "Ratio", "Custom")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    FREQUENCY_FIELD_NUMBER: _ClassVar[int]
    POWER_FIELD_NUMBER: _ClassVar[int]
    TEMPERATURE_FIELD_NUMBER: _ClassVar[int]
    VOLTAGE_FIELD_NUMBER: _ClassVar[int]
    RATIO_FIELD_NUMBER: _ClassVar[int]
    CUSTOM_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row33
    Count: Row34
    Distance: Row35
    Duration: Row36
    Frequency: Row37
    Power: Row38
    Temperature: Row39
    Voltage: Row40
    Ratio: Row41
    Custom: Row42
    def __init__(self, Unspecified: _Optional[_Union[Row33, _Mapping]] = ..., Count: _Optional[_Union[Row34, _Mapping]] = ..., Distance: _Optional[_Union[Row35, _Mapping]] = ..., Duration: _Optional[_Union[Row36, _Mapping]] = ..., Frequency: _Optional[_Union[Row37, _Mapping]] = ..., Power: _Optional[_Union[Row38, _Mapping]] = ..., Temperature: _Optional[_Union[Row39, _Mapping]] = ..., Voltage: _Optional[_Union[Row40, _Mapping]] = ..., Ratio: _Optional[_Union[Row41, _Mapping]] = ..., Custom: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row43
    def __init__(self, kind: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("Single", "Cartesian", "Zip")
    SINGLE_FIELD_NUMBER: _ClassVar[int]
    CARTESIAN_FIELD_NUMBER: _ClassVar[int]
    ZIP_FIELD_NUMBER: _ClassVar[int]
    Single: Row44
    Cartesian: Row45
    Zip: Row46
    def __init__(self, Single: _Optional[_Union[Row44, _Mapping]] = ..., Cartesian: _Optional[_Union[Row45, _Mapping]] = ..., Zip: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row47
    def __init__(self, kind: _Optional[_Union[Row47, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row35(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row36(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row37(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row38(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row39(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row43(_message.Message):
    __slots__ = ("Literal", "Array", "Range", "Arange", "Linspace", "Reference", "DynamicDefault", "Unary", "Binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    ARRAY_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    ARANGE_FIELD_NUMBER: _ClassVar[int]
    LINSPACE_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    DYNAMICDEFAULT_FIELD_NUMBER: _ClassVar[int]
    UNARY_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    Literal: Row48
    Array: Row49
    Range: Row50
    Arange: Row51
    Linspace: Row52
    Reference: Row53
    DynamicDefault: Row54
    Unary: Row55
    Binary: Row56
    def __init__(self, Literal: _Optional[_Union[Row48, _Mapping]] = ..., Array: _Optional[_Union[Row49, _Mapping]] = ..., Range: _Optional[_Union[Row50, _Mapping]] = ..., Arange: _Optional[_Union[Row51, _Mapping]] = ..., Linspace: _Optional[_Union[Row52, _Mapping]] = ..., Reference: _Optional[_Union[Row53, _Mapping]] = ..., DynamicDefault: _Optional[_Union[Row54, _Mapping]] = ..., Unary: _Optional[_Union[Row55, _Mapping]] = ..., Binary: _Optional[_Union[Row56, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row47(_message.Message):
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
    Null: Row57
    Bool: Row58
    Integer: Row59
    Unsigned: Row60
    Number: Row61
    Text: Row62
    Datetime: Row63
    Array: Row64
    Object: Row65
    def __init__(self, Null: _Optional[_Union[Row57, _Mapping]] = ..., Bool: _Optional[_Union[Row58, _Mapping]] = ..., Integer: _Optional[_Union[Row59, _Mapping]] = ..., Unsigned: _Optional[_Union[Row60, _Mapping]] = ..., Number: _Optional[_Union[Row61, _Mapping]] = ..., Text: _Optional[_Union[Row62, _Mapping]] = ..., Datetime: _Optional[_Union[Row63, _Mapping]] = ..., Array: _Optional[_Union[Row64, _Mapping]] = ..., Object: _Optional[_Union[Row65, _Mapping]] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row19
    def __init__(self, field_0: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row66
    def __init__(self, field_0: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row67
    def __init__(self, field_0: _Optional[_Union[Row67, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row68
    def __init__(self, field_0: _Optional[_Union[Row68, _Mapping]] = ...) -> None: ...

class Row52(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row69
    def __init__(self, field_0: _Optional[_Union[Row69, _Mapping]] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row70
    def __init__(self, field_0: _Optional[_Union[Row70, _Mapping]] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row71
    def __init__(self, field_0: _Optional[_Union[Row71, _Mapping]] = ...) -> None: ...

class Row56(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row72
    def __init__(self, field_0: _Optional[_Union[Row72, _Mapping]] = ...) -> None: ...

class Row57(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row58(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row59(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row61(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row62(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row63(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row11
    def __init__(self, field_0: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row64(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row73
    def __init__(self, field_0: _Optional[_Union[Row73, _Mapping]] = ...) -> None: ...

class Row65(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row74
    def __init__(self, field_0: _Optional[_Union[Row74, _Mapping]] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, values: _Optional[_Iterable[int]] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("start", "stop", "step")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: int
    stop: int
    step: int
    def __init__(self, start: _Optional[int] = ..., stop: _Optional[int] = ..., step: _Optional[int] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("start", "stop", "step")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    step: float
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., step: _Optional[float] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("start", "stop", "count")
    START_FIELD_NUMBER: _ClassVar[int]
    STOP_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    start: float
    stop: float
    count: int
    def __init__(self, start: _Optional[float] = ..., stop: _Optional[float] = ..., count: _Optional[int] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("key", "fallback")
    KEY_FIELD_NUMBER: _ClassVar[int]
    FALLBACK_FIELD_NUMBER: _ClassVar[int]
    key: Row75
    fallback: int
    def __init__(self, key: _Optional[_Union[Row75, _Mapping]] = ..., fallback: _Optional[int] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("op", "value")
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    op: Row76
    value: int
    def __init__(self, op: _Optional[_Union[Row76, _Mapping]] = ..., value: _Optional[int] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: Row77
    left: int
    right: int
    def __init__(self, op: _Optional[_Union[Row77, _Mapping]] = ..., left: _Optional[int] = ..., right: _Optional[int] = ...) -> None: ...

class Row73(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row74(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row78]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row78, _Mapping]]] = ...) -> None: ...

class Row75(_message.Message):
    __slots__ = ("Unspecified", "OperatorName", "OperatorEmail")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    OPERATORNAME_FIELD_NUMBER: _ClassVar[int]
    OPERATOREMAIL_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row79
    OperatorName: Row80
    OperatorEmail: Row81
    def __init__(self, Unspecified: _Optional[_Union[Row79, _Mapping]] = ..., OperatorName: _Optional[_Union[Row80, _Mapping]] = ..., OperatorEmail: _Optional[_Union[Row81, _Mapping]] = ...) -> None: ...

class Row76(_message.Message):
    __slots__ = ("Unspecified", "Negate")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    NEGATE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row82
    Negate: Row83
    def __init__(self, Unspecified: _Optional[_Union[Row82, _Mapping]] = ..., Negate: _Optional[_Union[Row83, _Mapping]] = ...) -> None: ...

class Row77(_message.Message):
    __slots__ = ("Unspecified", "Add", "Subtract", "Multiply", "Divide")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    ADD_FIELD_NUMBER: _ClassVar[int]
    SUBTRACT_FIELD_NUMBER: _ClassVar[int]
    MULTIPLY_FIELD_NUMBER: _ClassVar[int]
    DIVIDE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row84
    Add: Row85
    Subtract: Row86
    Multiply: Row87
    Divide: Row88
    def __init__(self, Unspecified: _Optional[_Union[Row84, _Mapping]] = ..., Add: _Optional[_Union[Row85, _Mapping]] = ..., Subtract: _Optional[_Union[Row86, _Mapping]] = ..., Multiply: _Optional[_Union[Row87, _Mapping]] = ..., Divide: _Optional[_Union[Row88, _Mapping]] = ...) -> None: ...

class Row78(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row79(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row80(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row81(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row82(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row83(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row84(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row85(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row86(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row87(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row88(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
