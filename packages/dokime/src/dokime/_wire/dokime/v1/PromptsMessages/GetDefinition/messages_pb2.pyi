from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("prompt_type",)
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    def __init__(self, prompt_type: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("prompt_type", "schema_version", "archetype", "title", "message", "fields", "actions", "rules", "condition", "owner_suite_id", "template_identifier", "examples")
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    ARCHETYPE_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    RULES_FIELD_NUMBER: _ClassVar[int]
    CONDITION_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    EXAMPLES_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    schema_version: int
    archetype: Row3
    title: str
    message: str
    fields: _containers.RepeatedCompositeFieldContainer[Row4]
    actions: _containers.RepeatedCompositeFieldContainer[Row5]
    rules: _containers.RepeatedCompositeFieldContainer[Row6]
    condition: Row7
    owner_suite_id: str
    template_identifier: str
    examples: _containers.RepeatedCompositeFieldContainer[Row8]
    def __init__(self, prompt_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., archetype: _Optional[_Union[Row3, _Mapping]] = ..., title: _Optional[str] = ..., message: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., actions: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., rules: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., condition: _Optional[_Union[Row7, _Mapping]] = ..., owner_suite_id: _Optional[str] = ..., template_identifier: _Optional[str] = ..., examples: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("Unspecified", "Confirmation", "Form", "Choice", "Status")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    CONFIRMATION_FIELD_NUMBER: _ClassVar[int]
    FORM_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row9
    Confirmation: Row10
    Form: Row11
    Choice: Row12
    Status: Row13
    def __init__(self, Unspecified: _Optional[_Union[Row9, _Mapping]] = ..., Confirmation: _Optional[_Union[Row10, _Mapping]] = ..., Form: _Optional[_Union[Row11, _Mapping]] = ..., Choice: _Optional[_Union[Row12, _Mapping]] = ..., Status: _Optional[_Union[Row13, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("metadata", "value_type", "kind", "multiple", "required", "default_value", "choices", "validation")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    MULTIPLE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_FIELD_NUMBER: _ClassVar[int]
    metadata: Row14
    value_type: Row15
    kind: Row16
    multiple: bool
    required: bool
    default_value: Row8
    choices: _containers.RepeatedScalarFieldContainer[str]
    validation: Row17
    def __init__(self, metadata: _Optional[_Union[Row14, _Mapping]] = ..., value_type: _Optional[_Union[Row15, _Mapping]] = ..., kind: _Optional[_Union[Row16, _Mapping]] = ..., multiple: _Optional[bool] = ..., required: _Optional[bool] = ..., default_value: _Optional[_Union[Row8, _Mapping]] = ..., choices: _Optional[_Iterable[str]] = ..., validation: _Optional[_Union[Row17, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("id", "label", "emphasis", "requires_condition")
    ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    EMPHASIS_FIELD_NUMBER: _ClassVar[int]
    REQUIRES_CONDITION_FIELD_NUMBER: _ClassVar[int]
    id: str
    label: str
    emphasis: Row18
    requires_condition: bool
    def __init__(self, id: _Optional[str] = ..., label: _Optional[str] = ..., emphasis: _Optional[_Union[Row18, _Mapping]] = ..., requires_condition: _Optional[bool] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("predicate", "message")
    PREDICATE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    predicate: Row19
    message: str
    def __init__(self, predicate: _Optional[_Union[Row19, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row20]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row20, _Mapping]]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row21]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row21, _Mapping]]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
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
    attributes: _containers.RepeatedCompositeFieldContainer[Row22]
    created_at: Row23
    updated_at: Row23
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row22, _Mapping]]] = ..., created_at: _Optional[_Union[Row23, _Mapping]] = ..., updated_at: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
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
    Unspecified: Row24
    Boolean: Row25
    Integer: Row26
    Number: Row27
    Text: Row28
    File: Row29
    Object: Row30
    Array: Row31
    Datetime: Row32
    def __init__(self, Unspecified: _Optional[_Union[Row24, _Mapping]] = ..., Boolean: _Optional[_Union[Row25, _Mapping]] = ..., Integer: _Optional[_Union[Row26, _Mapping]] = ..., Number: _Optional[_Union[Row27, _Mapping]] = ..., Text: _Optional[_Union[Row28, _Mapping]] = ..., File: _Optional[_Union[Row29, _Mapping]] = ..., Object: _Optional[_Union[Row30, _Mapping]] = ..., Array: _Optional[_Union[Row31, _Mapping]] = ..., Datetime: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Unspecified", "Text", "Float", "Int", "Choice", "Boolean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FLOAT_FIELD_NUMBER: _ClassVar[int]
    INT_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row33
    Text: Row34
    Float: Row35
    Int: Row36
    Choice: Row37
    Boolean: Row38
    def __init__(self, Unspecified: _Optional[_Union[Row33, _Mapping]] = ..., Text: _Optional[_Union[Row34, _Mapping]] = ..., Float: _Optional[_Union[Row35, _Mapping]] = ..., Int: _Optional[_Union[Row36, _Mapping]] = ..., Choice: _Optional[_Union[Row37, _Mapping]] = ..., Boolean: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("min_length", "max_length", "pattern", "minimum", "maximum", "min_items", "max_items")
    MIN_LENGTH_FIELD_NUMBER: _ClassVar[int]
    MAX_LENGTH_FIELD_NUMBER: _ClassVar[int]
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    MINIMUM_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_FIELD_NUMBER: _ClassVar[int]
    MIN_ITEMS_FIELD_NUMBER: _ClassVar[int]
    MAX_ITEMS_FIELD_NUMBER: _ClassVar[int]
    min_length: int
    max_length: int
    pattern: str
    minimum: Row8
    maximum: Row8
    min_items: int
    max_items: int
    def __init__(self, min_length: _Optional[int] = ..., max_length: _Optional[int] = ..., pattern: _Optional[str] = ..., minimum: _Optional[_Union[Row8, _Mapping]] = ..., maximum: _Optional[_Union[Row8, _Mapping]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("Unspecified", "Primary", "Secondary", "Danger")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PRIMARY_FIELD_NUMBER: _ClassVar[int]
    SECONDARY_FIELD_NUMBER: _ClassVar[int]
    DANGER_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row39
    Primary: Row40
    Secondary: Row41
    Danger: Row42
    def __init__(self, Unspecified: _Optional[_Union[Row39, _Mapping]] = ..., Primary: _Optional[_Union[Row40, _Mapping]] = ..., Secondary: _Optional[_Union[Row41, _Mapping]] = ..., Danger: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row43]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row43, _Mapping]]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row44
    def __init__(self, kind: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row45
    def __init__(self, kind: _Optional[_Union[Row45, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row8
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row46
    def __init__(self, kind: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("Record", "Telemetry", "All", "Any")
    RECORD_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    ANY_FIELD_NUMBER: _ClassVar[int]
    Record: Row47
    Telemetry: Row48
    All: Row49
    Any: Row50
    def __init__(self, Record: _Optional[_Union[Row47, _Mapping]] = ..., Telemetry: _Optional[_Union[Row48, _Mapping]] = ..., All: _Optional[_Union[Row49, _Mapping]] = ..., Any: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row45(_message.Message):
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
    Null: Row51
    Bool: Row52
    Integer: Row53
    Unsigned: Row54
    Number: Row55
    Text: Row56
    Datetime: Row57
    Array: Row58
    Object: Row59
    def __init__(self, Null: _Optional[_Union[Row51, _Mapping]] = ..., Bool: _Optional[_Union[Row52, _Mapping]] = ..., Integer: _Optional[_Union[Row53, _Mapping]] = ..., Unsigned: _Optional[_Union[Row54, _Mapping]] = ..., Number: _Optional[_Union[Row55, _Mapping]] = ..., Text: _Optional[_Union[Row56, _Mapping]] = ..., Datetime: _Optional[_Union[Row57, _Mapping]] = ..., Array: _Optional[_Union[Row58, _Mapping]] = ..., Object: _Optional[_Union[Row59, _Mapping]] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ("Literal", "Field", "Not", "Binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    NOT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    Literal: Row60
    Field: Row61
    Not: Row62
    Binary: Row63
    def __init__(self, Literal: _Optional[_Union[Row60, _Mapping]] = ..., Field: _Optional[_Union[Row61, _Mapping]] = ..., Not: _Optional[_Union[Row62, _Mapping]] = ..., Binary: _Optional[_Union[Row63, _Mapping]] = ...) -> None: ...

class Row47(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row64
    def __init__(self, field_0: _Optional[_Union[Row64, _Mapping]] = ...) -> None: ...

class Row48(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row65
    def __init__(self, field_0: _Optional[_Union[Row65, _Mapping]] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row66
    def __init__(self, field_0: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row66
    def __init__(self, field_0: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row53(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row54(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row55(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row56(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row57(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row23
    def __init__(self, field_0: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row58(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row67
    def __init__(self, field_0: _Optional[_Union[Row67, _Mapping]] = ...) -> None: ...

class Row59(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row68
    def __init__(self, field_0: _Optional[_Union[Row68, _Mapping]] = ...) -> None: ...

class Row60(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row61(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row62(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row63(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row69
    def __init__(self, field_0: _Optional[_Union[Row69, _Mapping]] = ...) -> None: ...

class Row64(_message.Message):
    __slots__ = ("record_type", "match_fields")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    MATCH_FIELDS_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    match_fields: Row8
    def __init__(self, record_type: _Optional[str] = ..., match_fields: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row65(_message.Message):
    __slots__ = ("stream", "field", "op", "target", "tolerance")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    op: Row70
    target: Row8
    tolerance: float
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ..., op: _Optional[_Union[Row70, _Mapping]] = ..., target: _Optional[_Union[Row8, _Mapping]] = ..., tolerance: _Optional[float] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, conditions: _Optional[_Iterable[int]] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row68(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row71]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row71, _Mapping]]] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: Row72
    left: int
    right: int
    def __init__(self, op: _Optional[_Union[Row72, _Mapping]] = ..., left: _Optional[int] = ..., right: _Optional[int] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("Unspecified", "Exists", "Eq", "Gt", "Gte", "Lt", "Lte", "Within")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    WITHIN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row73
    Exists: Row74
    Eq: Row75
    Gt: Row76
    Gte: Row77
    Lt: Row78
    Lte: Row79
    Within: Row80
    def __init__(self, Unspecified: _Optional[_Union[Row73, _Mapping]] = ..., Exists: _Optional[_Union[Row74, _Mapping]] = ..., Eq: _Optional[_Union[Row75, _Mapping]] = ..., Gt: _Optional[_Union[Row76, _Mapping]] = ..., Gte: _Optional[_Union[Row77, _Mapping]] = ..., Lt: _Optional[_Union[Row78, _Mapping]] = ..., Lte: _Optional[_Union[Row79, _Mapping]] = ..., Within: _Optional[_Union[Row80, _Mapping]] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ("Unspecified", "And", "Or", "Eq", "Ne", "Gt", "Gte", "Lt", "Lte", "Contains", "StartsWith", "EndsWith")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    AND_FIELD_NUMBER: _ClassVar[int]
    OR_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    NE_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    CONTAINS_FIELD_NUMBER: _ClassVar[int]
    STARTSWITH_FIELD_NUMBER: _ClassVar[int]
    ENDSWITH_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row81
    And: Row82
    Or: Row83
    Eq: Row84
    Ne: Row85
    Gt: Row86
    Gte: Row87
    Lt: Row88
    Lte: Row89
    Contains: Row90
    StartsWith: Row91
    EndsWith: Row92
    def __init__(self, Unspecified: _Optional[_Union[Row81, _Mapping]] = ..., And: _Optional[_Union[Row82, _Mapping]] = ..., Or: _Optional[_Union[Row83, _Mapping]] = ..., Eq: _Optional[_Union[Row84, _Mapping]] = ..., Ne: _Optional[_Union[Row85, _Mapping]] = ..., Gt: _Optional[_Union[Row86, _Mapping]] = ..., Gte: _Optional[_Union[Row87, _Mapping]] = ..., Lt: _Optional[_Union[Row88, _Mapping]] = ..., Lte: _Optional[_Union[Row89, _Mapping]] = ..., Contains: _Optional[_Union[Row90, _Mapping]] = ..., StartsWith: _Optional[_Union[Row91, _Mapping]] = ..., EndsWith: _Optional[_Union[Row92, _Mapping]] = ...) -> None: ...

class Row73(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row74(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row75(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row76(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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

class Row89(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row90(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row91(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row92(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
