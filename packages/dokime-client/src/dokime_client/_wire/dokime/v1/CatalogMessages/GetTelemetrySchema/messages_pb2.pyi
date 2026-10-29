from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("stream",)
    STREAM_FIELD_NUMBER: _ClassVar[int]
    stream: str
    def __init__(self, stream: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("stream", "schema_version", "storage_schema", "metadata", "template_identifier", "owner_suite_id", "columns", "default_x", "allow_extra_fields", "scalar", "validation_mode", "foreign")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    STORAGE_SCHEMA_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_X_FIELD_NUMBER: _ClassVar[int]
    ALLOW_EXTRA_FIELDS_FIELD_NUMBER: _ClassVar[int]
    SCALAR_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_MODE_FIELD_NUMBER: _ClassVar[int]
    FOREIGN_FIELD_NUMBER: _ClassVar[int]
    stream: str
    schema_version: int
    storage_schema: Row3
    metadata: Row4
    template_identifier: str
    owner_suite_id: str
    columns: _containers.RepeatedCompositeFieldContainer[Row5]
    default_x: str
    allow_extra_fields: bool
    scalar: bool
    validation_mode: Row6
    foreign: bool
    def __init__(self, stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., storage_schema: _Optional[_Union[Row3, _Mapping]] = ..., metadata: _Optional[_Union[Row4, _Mapping]] = ..., template_identifier: _Optional[str] = ..., owner_suite_id: _Optional[str] = ..., columns: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., default_x: _Optional[str] = ..., allow_extra_fields: _Optional[bool] = ..., scalar: _Optional[bool] = ..., validation_mode: _Optional[_Union[Row6, _Mapping]] = ..., foreign: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("root", "definitions")
    ROOT_FIELD_NUMBER: _ClassVar[int]
    DEFINITIONS_FIELD_NUMBER: _ClassVar[int]
    root: int
    definitions: _containers.RepeatedCompositeFieldContainer[Row7]
    def __init__(self, root: _Optional[int] = ..., definitions: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ("metadata", "value_type", "role", "unit", "display_unit", "required", "nullable", "default_value", "format")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    ROLE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_UNIT_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    NULLABLE_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    metadata: Row4
    value_type: Row10
    role: Row11
    unit: Row12
    display_unit: str
    required: bool
    nullable: bool
    default_value: Row13
    format: Row14
    def __init__(self, metadata: _Optional[_Union[Row4, _Mapping]] = ..., value_type: _Optional[_Union[Row10, _Mapping]] = ..., role: _Optional[_Union[Row11, _Mapping]] = ..., unit: _Optional[_Union[Row12, _Mapping]] = ..., display_unit: _Optional[str] = ..., required: _Optional[bool] = ..., nullable: _Optional[bool] = ..., default_value: _Optional[_Union[Row13, _Mapping]] = ..., format: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "Strict", "Prototype")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STRICT_FIELD_NUMBER: _ClassVar[int]
    PROTOTYPE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row15
    Strict: Row16
    Prototype: Row17
    def __init__(self, Unspecified: _Optional[_Union[Row15, _Mapping]] = ..., Strict: _Optional[_Union[Row16, _Mapping]] = ..., Prototype: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row18]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row18, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row13
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
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
    Unspecified: Row19
    Boolean: Row20
    Integer: Row21
    Number: Row22
    Text: Row23
    File: Row24
    Object: Row25
    Array: Row26
    Datetime: Row27
    def __init__(self, Unspecified: _Optional[_Union[Row19, _Mapping]] = ..., Boolean: _Optional[_Union[Row20, _Mapping]] = ..., Integer: _Optional[_Union[Row21, _Mapping]] = ..., Number: _Optional[_Union[Row22, _Mapping]] = ..., Text: _Optional[_Union[Row23, _Mapping]] = ..., File: _Optional[_Union[Row24, _Mapping]] = ..., Object: _Optional[_Union[Row25, _Mapping]] = ..., Array: _Optional[_Union[Row26, _Mapping]] = ..., Datetime: _Optional[_Union[Row27, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Unspecified", "Value", "Counter", "State", "Distance", "Latency", "Throughput")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    COUNTER_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    LATENCY_FIELD_NUMBER: _ClassVar[int]
    THROUGHPUT_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row28
    Value: Row29
    Counter: Row30
    State: Row31
    Distance: Row32
    Latency: Row33
    Throughput: Row34
    def __init__(self, Unspecified: _Optional[_Union[Row28, _Mapping]] = ..., Value: _Optional[_Union[Row29, _Mapping]] = ..., Counter: _Optional[_Union[Row30, _Mapping]] = ..., State: _Optional[_Union[Row31, _Mapping]] = ..., Distance: _Optional[_Union[Row32, _Mapping]] = ..., Latency: _Optional[_Union[Row33, _Mapping]] = ..., Throughput: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("metadata", "family", "symbol")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    SYMBOL_FIELD_NUMBER: _ClassVar[int]
    metadata: Row4
    family: Row35
    symbol: str
    def __init__(self, metadata: _Optional[_Union[Row4, _Mapping]] = ..., family: _Optional[_Union[Row35, _Mapping]] = ..., symbol: _Optional[str] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row36]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row36, _Mapping]]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row37
    def __init__(self, kind: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ("name", "tag", "kind", "cardinality", "definition")
    NAME_FIELD_NUMBER: _ClassVar[int]
    TAG_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    CARDINALITY_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    name: str
    tag: int
    kind: int
    cardinality: int
    definition: int
    def __init__(self, name: _Optional[str] = ..., tag: _Optional[int] = ..., kind: _Optional[int] = ..., cardinality: _Optional[int] = ..., definition: _Optional[int] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row35(_message.Message):
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
    Unspecified: Row38
    Count: Row39
    Distance: Row40
    Duration: Row41
    Frequency: Row42
    Power: Row43
    Temperature: Row44
    Voltage: Row45
    Ratio: Row46
    Custom: Row47
    def __init__(self, Unspecified: _Optional[_Union[Row38, _Mapping]] = ..., Count: _Optional[_Union[Row39, _Mapping]] = ..., Distance: _Optional[_Union[Row40, _Mapping]] = ..., Duration: _Optional[_Union[Row41, _Mapping]] = ..., Frequency: _Optional[_Union[Row42, _Mapping]] = ..., Power: _Optional[_Union[Row43, _Mapping]] = ..., Temperature: _Optional[_Union[Row44, _Mapping]] = ..., Voltage: _Optional[_Union[Row45, _Mapping]] = ..., Ratio: _Optional[_Union[Row46, _Mapping]] = ..., Custom: _Optional[_Union[Row47, _Mapping]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row48
    def __init__(self, kind: _Optional[_Union[Row48, _Mapping]] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("Normal", "Fixed", "Scientific", "Engineering", "SiPrefix", "AbsoluteTime", "ElapsedSinceRun", "ElapsedSinceCycle")
    NORMAL_FIELD_NUMBER: _ClassVar[int]
    FIXED_FIELD_NUMBER: _ClassVar[int]
    SCIENTIFIC_FIELD_NUMBER: _ClassVar[int]
    ENGINEERING_FIELD_NUMBER: _ClassVar[int]
    SIPREFIX_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTETIME_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCERUN_FIELD_NUMBER: _ClassVar[int]
    ELAPSEDSINCECYCLE_FIELD_NUMBER: _ClassVar[int]
    Normal: Row49
    Fixed: Row50
    Scientific: Row51
    Engineering: Row52
    SiPrefix: Row53
    AbsoluteTime: Row54
    ElapsedSinceRun: Row55
    ElapsedSinceCycle: Row56
    def __init__(self, Normal: _Optional[_Union[Row49, _Mapping]] = ..., Fixed: _Optional[_Union[Row50, _Mapping]] = ..., Scientific: _Optional[_Union[Row51, _Mapping]] = ..., Engineering: _Optional[_Union[Row52, _Mapping]] = ..., SiPrefix: _Optional[_Union[Row53, _Mapping]] = ..., AbsoluteTime: _Optional[_Union[Row54, _Mapping]] = ..., ElapsedSinceRun: _Optional[_Union[Row55, _Mapping]] = ..., ElapsedSinceCycle: _Optional[_Union[Row56, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
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
    field_0: Row70
    def __init__(self, field_0: _Optional[_Union[Row70, _Mapping]] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row71
    def __init__(self, field_0: _Optional[_Union[Row71, _Mapping]] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row72
    def __init__(self, field_0: _Optional[_Union[Row72, _Mapping]] = ...) -> None: ...

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
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("max_fraction_digits",)
    MAX_FRACTION_DIGITS_FIELD_NUMBER: _ClassVar[int]
    max_fraction_digits: int
    def __init__(self, max_fraction_digits: _Optional[int] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("decimals",)
    DECIMALS_FIELD_NUMBER: _ClassVar[int]
    decimals: int
    def __init__(self, decimals: _Optional[int] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("significant_digits",)
    SIGNIFICANT_DIGITS_FIELD_NUMBER: _ClassVar[int]
    significant_digits: int
    def __init__(self, significant_digits: _Optional[int] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("pattern",)
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    def __init__(self, pattern: _Optional[str] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ("unit",)
    UNIT_FIELD_NUMBER: _ClassVar[int]
    unit: Row75
    def __init__(self, unit: _Optional[_Union[Row75, _Mapping]] = ...) -> None: ...

class Row73(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row74(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row76]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row76, _Mapping]]] = ...) -> None: ...

class Row75(_message.Message):
    __slots__ = ("Unspecified", "Seconds", "Milliseconds", "Minutes")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    MILLISECONDS_FIELD_NUMBER: _ClassVar[int]
    MINUTES_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row77
    Seconds: Row78
    Milliseconds: Row79
    Minutes: Row80
    def __init__(self, Unspecified: _Optional[_Union[Row77, _Mapping]] = ..., Seconds: _Optional[_Union[Row78, _Mapping]] = ..., Milliseconds: _Optional[_Union[Row79, _Mapping]] = ..., Minutes: _Optional[_Union[Row80, _Mapping]] = ...) -> None: ...

class Row76(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row77(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row78(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row79(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row80(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
