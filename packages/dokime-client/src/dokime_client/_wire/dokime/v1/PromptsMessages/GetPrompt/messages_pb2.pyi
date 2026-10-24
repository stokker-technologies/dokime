from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("prompt_id",)
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    def __init__(self, prompt_id: _Optional[str] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("prompt", "response", "observed_cursor")
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    prompt: Row3
    response: Row4
    observed_cursor: Row5
    def __init__(self, prompt: _Optional[_Union[Row3, _Mapping]] = ..., response: _Optional[_Union[Row4, _Mapping]] = ..., observed_cursor: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("prompt_id", "definition", "kind", "values", "scope", "created_at")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    definition: Row6
    kind: Row7
    values: Row8
    scope: Row9
    created_at: Row10
    def __init__(self, prompt_id: _Optional[str] = ..., definition: _Optional[_Union[Row6, _Mapping]] = ..., kind: _Optional[_Union[Row7, _Mapping]] = ..., values: _Optional[_Union[Row8, _Mapping]] = ..., scope: _Optional[_Union[Row9, _Mapping]] = ..., created_at: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("prompt_id", "action", "responder", "submitted_at", "values", "accepted")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    RESPONDER_FIELD_NUMBER: _ClassVar[int]
    SUBMITTED_AT_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    action: str
    responder: str
    submitted_at: Row10
    values: Row8
    accepted: bool
    def __init__(self, prompt_id: _Optional[str] = ..., action: _Optional[str] = ..., responder: _Optional[str] = ..., submitted_at: _Optional[_Union[Row10, _Mapping]] = ..., values: _Optional[_Union[Row8, _Mapping]] = ..., accepted: _Optional[bool] = ...) -> None: ...

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
    __slots__ = ("prompt_type", "schema_version", "archetype", "title", "message", "fields", "actions", "rules", "condition", "owner_suite_id", "template_identifier", "examples", "level")
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
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    schema_version: int
    archetype: Row11
    title: str
    message: str
    fields: _containers.RepeatedCompositeFieldContainer[Row12]
    actions: _containers.RepeatedCompositeFieldContainer[Row13]
    rules: _containers.RepeatedCompositeFieldContainer[Row14]
    condition: Row15
    owner_suite_id: str
    template_identifier: str
    examples: _containers.RepeatedCompositeFieldContainer[Row8]
    level: Row16
    def __init__(self, prompt_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., archetype: _Optional[_Union[Row11, _Mapping]] = ..., title: _Optional[str] = ..., message: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[Row12, _Mapping]]] = ..., actions: _Optional[_Iterable[_Union[Row13, _Mapping]]] = ..., rules: _Optional[_Iterable[_Union[Row14, _Mapping]]] = ..., condition: _Optional[_Union[Row15, _Mapping]] = ..., owner_suite_id: _Optional[str] = ..., template_identifier: _Optional[str] = ..., examples: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., level: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("Unspecified", "Confirmation", "Form", "Choice", "Status", "Verification")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    CONFIRMATION_FIELD_NUMBER: _ClassVar[int]
    FORM_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    VERIFICATION_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row17
    Confirmation: Row18
    Form: Row19
    Choice: Row20
    Status: Row21
    Verification: Row22
    def __init__(self, Unspecified: _Optional[_Union[Row17, _Mapping]] = ..., Confirmation: _Optional[_Union[Row18, _Mapping]] = ..., Form: _Optional[_Union[Row19, _Mapping]] = ..., Choice: _Optional[_Union[Row20, _Mapping]] = ..., Status: _Optional[_Union[Row21, _Mapping]] = ..., Verification: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row23]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row23, _Mapping]]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: Row24
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Unspecified", "Confirmation", "Form", "Choice", "Status")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    CONFIRMATION_FIELD_NUMBER: _ClassVar[int]
    FORM_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row25
    Confirmation: Row26
    Form: Row27
    Choice: Row28
    Status: Row29
    def __init__(self, Unspecified: _Optional[_Union[Row25, _Mapping]] = ..., Confirmation: _Optional[_Union[Row26, _Mapping]] = ..., Form: _Optional[_Union[Row27, _Mapping]] = ..., Choice: _Optional[_Union[Row28, _Mapping]] = ..., Status: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("metadata", "value_type", "kind", "multiple", "required", "default_value", "choices", "validation")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    MULTIPLE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_FIELD_NUMBER: _ClassVar[int]
    metadata: Row30
    value_type: Row31
    kind: Row32
    multiple: bool
    required: bool
    default_value: Row8
    choices: _containers.RepeatedScalarFieldContainer[str]
    validation: Row33
    def __init__(self, metadata: _Optional[_Union[Row30, _Mapping]] = ..., value_type: _Optional[_Union[Row31, _Mapping]] = ..., kind: _Optional[_Union[Row32, _Mapping]] = ..., multiple: _Optional[bool] = ..., required: _Optional[bool] = ..., default_value: _Optional[_Union[Row8, _Mapping]] = ..., choices: _Optional[_Iterable[str]] = ..., validation: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("id", "label", "emphasis", "requires_condition")
    ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    EMPHASIS_FIELD_NUMBER: _ClassVar[int]
    REQUIRES_CONDITION_FIELD_NUMBER: _ClassVar[int]
    id: str
    label: str
    emphasis: Row34
    requires_condition: bool
    def __init__(self, id: _Optional[str] = ..., label: _Optional[str] = ..., emphasis: _Optional[_Union[Row34, _Mapping]] = ..., requires_condition: _Optional[bool] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("predicate", "message")
    PREDICATE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    predicate: Row35
    message: str
    def __init__(self, predicate: _Optional[_Union[Row35, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row36]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row36, _Mapping]]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Unspecified", "Trace", "Debug", "Info", "Warn", "Error")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRACE_FIELD_NUMBER: _ClassVar[int]
    DEBUG_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    WARN_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row37
    Trace: Row38
    Debug: Row39
    Info: Row40
    Warn: Row41
    Error: Row42
    def __init__(self, Unspecified: _Optional[_Union[Row37, _Mapping]] = ..., Trace: _Optional[_Union[Row38, _Mapping]] = ..., Debug: _Optional[_Union[Row39, _Mapping]] = ..., Info: _Optional[_Union[Row40, _Mapping]] = ..., Warn: _Optional[_Union[Row41, _Mapping]] = ..., Error: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row43
    def __init__(self, kind: _Optional[_Union[Row43, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row44
    def __init__(self, kind: _Optional[_Union[Row44, _Mapping]] = ...) -> None: ...

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
    attributes: _containers.RepeatedCompositeFieldContainer[Row45]
    created_at: Row10
    updated_at: Row10
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Iterable[_Union[Row45, _Mapping]]] = ..., created_at: _Optional[_Union[Row10, _Mapping]] = ..., updated_at: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
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
    Unspecified: Row46
    Boolean: Row47
    Integer: Row48
    Number: Row49
    Text: Row50
    File: Row51
    Object: Row52
    Array: Row53
    Datetime: Row54
    def __init__(self, Unspecified: _Optional[_Union[Row46, _Mapping]] = ..., Boolean: _Optional[_Union[Row47, _Mapping]] = ..., Integer: _Optional[_Union[Row48, _Mapping]] = ..., Number: _Optional[_Union[Row49, _Mapping]] = ..., Text: _Optional[_Union[Row50, _Mapping]] = ..., File: _Optional[_Union[Row51, _Mapping]] = ..., Object: _Optional[_Union[Row52, _Mapping]] = ..., Array: _Optional[_Union[Row53, _Mapping]] = ..., Datetime: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("Unspecified", "Text", "Float", "Int", "Choice", "Boolean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    FLOAT_FIELD_NUMBER: _ClassVar[int]
    INT_FIELD_NUMBER: _ClassVar[int]
    CHOICE_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row55
    Text: Row56
    Float: Row57
    Int: Row58
    Choice: Row59
    Boolean: Row60
    def __init__(self, Unspecified: _Optional[_Union[Row55, _Mapping]] = ..., Text: _Optional[_Union[Row56, _Mapping]] = ..., Float: _Optional[_Union[Row57, _Mapping]] = ..., Int: _Optional[_Union[Row58, _Mapping]] = ..., Choice: _Optional[_Union[Row59, _Mapping]] = ..., Boolean: _Optional[_Union[Row60, _Mapping]] = ...) -> None: ...

class Row33(_message.Message):
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

class Row34(_message.Message):
    __slots__ = ("Unspecified", "Primary", "Secondary", "Danger")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PRIMARY_FIELD_NUMBER: _ClassVar[int]
    SECONDARY_FIELD_NUMBER: _ClassVar[int]
    DANGER_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row61
    Primary: Row62
    Secondary: Row63
    Danger: Row64
    def __init__(self, Unspecified: _Optional[_Union[Row61, _Mapping]] = ..., Primary: _Optional[_Union[Row62, _Mapping]] = ..., Secondary: _Optional[_Union[Row63, _Mapping]] = ..., Danger: _Optional[_Union[Row64, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[Row65]
    def __init__(self, nodes: _Optional[_Iterable[_Union[Row65, _Mapping]]] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row66
    def __init__(self, kind: _Optional[_Union[Row66, _Mapping]] = ...) -> None: ...

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
    Null: Row67
    Bool: Row68
    Integer: Row69
    Unsigned: Row70
    Number: Row71
    Text: Row72
    Datetime: Row73
    Array: Row74
    Object: Row75
    def __init__(self, Null: _Optional[_Union[Row67, _Mapping]] = ..., Bool: _Optional[_Union[Row68, _Mapping]] = ..., Integer: _Optional[_Union[Row69, _Mapping]] = ..., Unsigned: _Optional[_Union[Row70, _Mapping]] = ..., Number: _Optional[_Union[Row71, _Mapping]] = ..., Text: _Optional[_Union[Row72, _Mapping]] = ..., Datetime: _Optional[_Union[Row73, _Mapping]] = ..., Array: _Optional[_Union[Row74, _Mapping]] = ..., Object: _Optional[_Union[Row75, _Mapping]] = ...) -> None: ...

class Row44(_message.Message):
    __slots__ = ("Default", "Initialization", "StepAttemptId", "Cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEPATTEMPTID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    Default: Row76
    Initialization: Row77
    StepAttemptId: Row78
    Cleanup: Row79
    def __init__(self, Default: _Optional[_Union[Row76, _Mapping]] = ..., Initialization: _Optional[_Union[Row77, _Mapping]] = ..., StepAttemptId: _Optional[_Union[Row78, _Mapping]] = ..., Cleanup: _Optional[_Union[Row79, _Mapping]] = ...) -> None: ...

class Row45(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: Row8
    def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row49(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row50(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row53(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row54(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row55(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row56(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row57(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row58(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row59(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row60(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row61(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row62(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row63(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row64(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row65(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: Row80
    def __init__(self, kind: _Optional[_Union[Row80, _Mapping]] = ...) -> None: ...

class Row66(_message.Message):
    __slots__ = ("Record", "Telemetry", "All", "Any")
    RECORD_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    ANY_FIELD_NUMBER: _ClassVar[int]
    Record: Row81
    Telemetry: Row82
    All: Row83
    Any: Row84
    def __init__(self, Record: _Optional[_Union[Row81, _Mapping]] = ..., Telemetry: _Optional[_Union[Row82, _Mapping]] = ..., All: _Optional[_Union[Row83, _Mapping]] = ..., Any: _Optional[_Union[Row84, _Mapping]] = ...) -> None: ...

class Row67(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row68(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row69(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row70(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row71(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: float
    def __init__(self, field_0: _Optional[float] = ...) -> None: ...

class Row72(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row73(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row74(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row85
    def __init__(self, field_0: _Optional[_Union[Row85, _Mapping]] = ...) -> None: ...

class Row75(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row86
    def __init__(self, field_0: _Optional[_Union[Row86, _Mapping]] = ...) -> None: ...

class Row76(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row77(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row78(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row79(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row80(_message.Message):
    __slots__ = ("Literal", "Field", "Not", "Binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    NOT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    Literal: Row87
    Field: Row88
    Not: Row89
    Binary: Row90
    def __init__(self, Literal: _Optional[_Union[Row87, _Mapping]] = ..., Field: _Optional[_Union[Row88, _Mapping]] = ..., Not: _Optional[_Union[Row89, _Mapping]] = ..., Binary: _Optional[_Union[Row90, _Mapping]] = ...) -> None: ...

class Row81(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row91
    def __init__(self, field_0: _Optional[_Union[Row91, _Mapping]] = ...) -> None: ...

class Row82(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row92
    def __init__(self, field_0: _Optional[_Union[Row92, _Mapping]] = ...) -> None: ...

class Row83(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row93
    def __init__(self, field_0: _Optional[_Union[Row93, _Mapping]] = ...) -> None: ...

class Row84(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row93
    def __init__(self, field_0: _Optional[_Union[Row93, _Mapping]] = ...) -> None: ...

class Row85(_message.Message):
    __slots__ = ("items",)
    ITEMS_FIELD_NUMBER: _ClassVar[int]
    items: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, items: _Optional[_Iterable[int]] = ...) -> None: ...

class Row86(_message.Message):
    __slots__ = ("fields",)
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    fields: _containers.RepeatedCompositeFieldContainer[Row94]
    def __init__(self, fields: _Optional[_Iterable[_Union[Row94, _Mapping]]] = ...) -> None: ...

class Row87(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row8
    def __init__(self, field_0: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row88(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: str
    def __init__(self, field_0: _Optional[str] = ...) -> None: ...

class Row89(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: int
    def __init__(self, field_0: _Optional[int] = ...) -> None: ...

class Row90(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row95
    def __init__(self, field_0: _Optional[_Union[Row95, _Mapping]] = ...) -> None: ...

class Row91(_message.Message):
    __slots__ = ("record_type", "match_fields")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    MATCH_FIELDS_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    match_fields: Row8
    def __init__(self, record_type: _Optional[str] = ..., match_fields: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row92(_message.Message):
    __slots__ = ("stream", "field", "op", "target", "tolerance")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    op: Row96
    target: Row8
    tolerance: float
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ..., op: _Optional[_Union[Row96, _Mapping]] = ..., target: _Optional[_Union[Row8, _Mapping]] = ..., tolerance: _Optional[float] = ...) -> None: ...

class Row93(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, conditions: _Optional[_Iterable[int]] = ...) -> None: ...

class Row94(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: int
    def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...

class Row95(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: Row97
    left: int
    right: int
    def __init__(self, op: _Optional[_Union[Row97, _Mapping]] = ..., left: _Optional[int] = ..., right: _Optional[int] = ...) -> None: ...

class Row96(_message.Message):
    __slots__ = ("Unspecified", "Exists", "Eq", "Gt", "Gte", "Lt", "Lte", "Within")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    EQ_FIELD_NUMBER: _ClassVar[int]
    GT_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LT_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    WITHIN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row98
    Exists: Row99
    Eq: Row100
    Gt: Row101
    Gte: Row102
    Lt: Row103
    Lte: Row104
    Within: Row105
    def __init__(self, Unspecified: _Optional[_Union[Row98, _Mapping]] = ..., Exists: _Optional[_Union[Row99, _Mapping]] = ..., Eq: _Optional[_Union[Row100, _Mapping]] = ..., Gt: _Optional[_Union[Row101, _Mapping]] = ..., Gte: _Optional[_Union[Row102, _Mapping]] = ..., Lt: _Optional[_Union[Row103, _Mapping]] = ..., Lte: _Optional[_Union[Row104, _Mapping]] = ..., Within: _Optional[_Union[Row105, _Mapping]] = ...) -> None: ...

class Row97(_message.Message):
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
    Unspecified: Row106
    And: Row107
    Or: Row108
    Eq: Row109
    Ne: Row110
    Gt: Row111
    Gte: Row112
    Lt: Row113
    Lte: Row114
    Contains: Row115
    StartsWith: Row116
    EndsWith: Row117
    def __init__(self, Unspecified: _Optional[_Union[Row106, _Mapping]] = ..., And: _Optional[_Union[Row107, _Mapping]] = ..., Or: _Optional[_Union[Row108, _Mapping]] = ..., Eq: _Optional[_Union[Row109, _Mapping]] = ..., Ne: _Optional[_Union[Row110, _Mapping]] = ..., Gt: _Optional[_Union[Row111, _Mapping]] = ..., Gte: _Optional[_Union[Row112, _Mapping]] = ..., Lt: _Optional[_Union[Row113, _Mapping]] = ..., Lte: _Optional[_Union[Row114, _Mapping]] = ..., Contains: _Optional[_Union[Row115, _Mapping]] = ..., StartsWith: _Optional[_Union[Row116, _Mapping]] = ..., EndsWith: _Optional[_Union[Row117, _Mapping]] = ...) -> None: ...

class Row98(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row99(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row100(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row101(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row102(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row103(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row104(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row105(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row106(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row107(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row108(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row109(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row110(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row111(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row112(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row113(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row114(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row115(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row116(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row117(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
