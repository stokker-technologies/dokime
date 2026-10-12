import datetime

from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
from dokime.client._generated.dokime.v1 import execution_pb2 as _execution_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PromptArchetype(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PROMPT_ARCHETYPE_UNSPECIFIED: _ClassVar[PromptArchetype]
    PROMPT_ARCHETYPE_CONFIRMATION: _ClassVar[PromptArchetype]
    PROMPT_ARCHETYPE_FORM: _ClassVar[PromptArchetype]
    PROMPT_ARCHETYPE_CHOICE: _ClassVar[PromptArchetype]
    PROMPT_ARCHETYPE_STATUS: _ClassVar[PromptArchetype]

class PromptKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PROMPT_KIND_UNSPECIFIED: _ClassVar[PromptKind]
    PROMPT_KIND_CONFIRMATION: _ClassVar[PromptKind]
    PROMPT_KIND_FORM: _ClassVar[PromptKind]
    PROMPT_KIND_CHOICE: _ClassVar[PromptKind]
    PROMPT_KIND_STATUS: _ClassVar[PromptKind]
    PROMPT_KIND_VERIFICATION: _ClassVar[PromptKind]

class PromptFieldKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PROMPT_FIELD_KIND_UNSPECIFIED: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_TEXT: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_FLOAT: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_INT: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_CHOICE: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_BOOLEAN: _ClassVar[PromptFieldKind]

class PromptActionEmphasis(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PROMPT_ACTION_EMPHASIS_UNSPECIFIED: _ClassVar[PromptActionEmphasis]
    PROMPT_ACTION_EMPHASIS_PRIMARY: _ClassVar[PromptActionEmphasis]
    PROMPT_ACTION_EMPHASIS_SECONDARY: _ClassVar[PromptActionEmphasis]
    PROMPT_ACTION_EMPHASIS_DANGER: _ClassVar[PromptActionEmphasis]

class AnswerOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ANSWER_OPERATOR_UNSPECIFIED: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_AND: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_OR: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_EQ: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_NE: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_GT: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_GTE: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_LT: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_LTE: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_CONTAINS: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_STARTS_WITH: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_ENDS_WITH: _ClassVar[AnswerOperator]

class TelemetryMatchOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TELEMETRY_MATCH_OPERATOR_UNSPECIFIED: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_EXISTS: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_EQ: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_GT: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_GTE: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_LT: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_LTE: _ClassVar[TelemetryMatchOperator]
    TELEMETRY_MATCH_OPERATOR_WITHIN: _ClassVar[TelemetryMatchOperator]
PROMPT_ARCHETYPE_UNSPECIFIED: PromptArchetype
PROMPT_ARCHETYPE_CONFIRMATION: PromptArchetype
PROMPT_ARCHETYPE_FORM: PromptArchetype
PROMPT_ARCHETYPE_CHOICE: PromptArchetype
PROMPT_ARCHETYPE_STATUS: PromptArchetype
PROMPT_KIND_UNSPECIFIED: PromptKind
PROMPT_KIND_CONFIRMATION: PromptKind
PROMPT_KIND_FORM: PromptKind
PROMPT_KIND_CHOICE: PromptKind
PROMPT_KIND_STATUS: PromptKind
PROMPT_KIND_VERIFICATION: PromptKind
PROMPT_FIELD_KIND_UNSPECIFIED: PromptFieldKind
PROMPT_FIELD_KIND_TEXT: PromptFieldKind
PROMPT_FIELD_KIND_FLOAT: PromptFieldKind
PROMPT_FIELD_KIND_INT: PromptFieldKind
PROMPT_FIELD_KIND_CHOICE: PromptFieldKind
PROMPT_FIELD_KIND_BOOLEAN: PromptFieldKind
PROMPT_ACTION_EMPHASIS_UNSPECIFIED: PromptActionEmphasis
PROMPT_ACTION_EMPHASIS_PRIMARY: PromptActionEmphasis
PROMPT_ACTION_EMPHASIS_SECONDARY: PromptActionEmphasis
PROMPT_ACTION_EMPHASIS_DANGER: PromptActionEmphasis
ANSWER_OPERATOR_UNSPECIFIED: AnswerOperator
ANSWER_OPERATOR_AND: AnswerOperator
ANSWER_OPERATOR_OR: AnswerOperator
ANSWER_OPERATOR_EQ: AnswerOperator
ANSWER_OPERATOR_NE: AnswerOperator
ANSWER_OPERATOR_GT: AnswerOperator
ANSWER_OPERATOR_GTE: AnswerOperator
ANSWER_OPERATOR_LT: AnswerOperator
ANSWER_OPERATOR_LTE: AnswerOperator
ANSWER_OPERATOR_CONTAINS: AnswerOperator
ANSWER_OPERATOR_STARTS_WITH: AnswerOperator
ANSWER_OPERATOR_ENDS_WITH: AnswerOperator
TELEMETRY_MATCH_OPERATOR_UNSPECIFIED: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_EXISTS: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_EQ: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_GT: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_GTE: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_LT: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_LTE: TelemetryMatchOperator
TELEMETRY_MATCH_OPERATOR_WITHIN: TelemetryMatchOperator

class ListPromptDefinitionsRequest(_message.Message):
    __slots__ = ("limit", "after_prompt_type", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_prompt_type: str
    cursor: _common_pb2.Cursor
    def __init__(self, limit: _Optional[int] = ..., after_prompt_type: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class ListPromptDefinitionsReply(_message.Message):
    __slots__ = ("definitions", "next_after_prompt_type", "cursor")
    DEFINITIONS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    definitions: _containers.RepeatedCompositeFieldContainer[PromptDefinitionIdentity]
    next_after_prompt_type: str
    cursor: _common_pb2.Cursor
    def __init__(self, definitions: _Optional[_Iterable[_Union[PromptDefinitionIdentity, _Mapping]]] = ..., next_after_prompt_type: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class PromptDefinitionIdentity(_message.Message):
    __slots__ = ("prompt_type", "schema_version", "title")
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    schema_version: int
    title: str
    def __init__(self, prompt_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., title: _Optional[str] = ...) -> None: ...

class GetPromptDefinitionRequest(_message.Message):
    __slots__ = ("prompt_type",)
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    prompt_type: str
    def __init__(self, prompt_type: _Optional[str] = ...) -> None: ...

class ListPendingPromptsRequest(_message.Message):
    __slots__ = ("limit", "after_prompt_id", "cursor", "cycle_id", "execution_id")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_prompt_id: str
    cursor: _common_pb2.Cursor
    cycle_id: str
    execution_id: str
    def __init__(self, limit: _Optional[int] = ..., after_prompt_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class ListPendingPromptsReply(_message.Message):
    __slots__ = ("prompts", "next_after_prompt_id", "cursor")
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    prompts: _containers.RepeatedCompositeFieldContainer[PendingPromptIdentity]
    next_after_prompt_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, prompts: _Optional[_Iterable[_Union[PendingPromptIdentity, _Mapping]]] = ..., next_after_prompt_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class PendingPromptIdentity(_message.Message):
    __slots__ = ("prompt_id", "definition", "scope", "created_at")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    definition: PromptDefinitionIdentity
    scope: _common_pb2.EvidenceScope
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, prompt_id: _Optional[str] = ..., definition: _Optional[_Union[PromptDefinitionIdentity, _Mapping]] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class GetPromptRequest(_message.Message):
    __slots__ = ("prompt_id",)
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    def __init__(self, prompt_id: _Optional[str] = ...) -> None: ...

class PromptView(_message.Message):
    __slots__ = ("prompt", "response", "observed_cursor")
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    prompt: PromptInstance
    response: PromptResponse
    observed_cursor: _common_pb2.Cursor
    def __init__(self, prompt: _Optional[_Union[PromptInstance, _Mapping]] = ..., response: _Optional[_Union[PromptResponse, _Mapping]] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class PromptInstance(_message.Message):
    __slots__ = ("prompt_id", "definition", "kind", "values", "scope", "created_at")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    definition: PromptDefinition
    kind: PromptKind
    values: _value_pb2.ValueObject
    scope: _common_pb2.EvidenceScope
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, prompt_id: _Optional[str] = ..., definition: _Optional[_Union[PromptDefinition, _Mapping]] = ..., kind: _Optional[_Union[PromptKind, str]] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class PromptDefinition(_message.Message):
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
    archetype: PromptArchetype
    title: str
    message: str
    fields: _containers.RepeatedCompositeFieldContainer[PromptField]
    actions: _containers.RepeatedCompositeFieldContainer[PromptAction]
    rules: _containers.RepeatedCompositeFieldContainer[AnswerRule]
    condition: EvidenceCondition
    owner_suite_id: str
    template_identifier: str
    examples: _containers.RepeatedCompositeFieldContainer[_value_pb2.ValueObject]
    def __init__(self, prompt_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., archetype: _Optional[_Union[PromptArchetype, str]] = ..., title: _Optional[str] = ..., message: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[PromptField, _Mapping]]] = ..., actions: _Optional[_Iterable[_Union[PromptAction, _Mapping]]] = ..., rules: _Optional[_Iterable[_Union[AnswerRule, _Mapping]]] = ..., condition: _Optional[_Union[EvidenceCondition, _Mapping]] = ..., owner_suite_id: _Optional[str] = ..., template_identifier: _Optional[str] = ..., examples: _Optional[_Iterable[_Union[_value_pb2.ValueObject, _Mapping]]] = ...) -> None: ...

class PromptField(_message.Message):
    __slots__ = ("metadata", "value_type", "kind", "multiple", "required", "default_value", "choices", "validation")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    MULTIPLE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    value_type: _schema_pb2.ValueType
    kind: PromptFieldKind
    multiple: bool
    required: bool
    default_value: _value_pb2.Value
    choices: _containers.RepeatedScalarFieldContainer[str]
    validation: PromptFieldValidation
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value_type: _Optional[_Union[_schema_pb2.ValueType, str]] = ..., kind: _Optional[_Union[PromptFieldKind, str]] = ..., multiple: _Optional[bool] = ..., required: _Optional[bool] = ..., default_value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., choices: _Optional[_Iterable[str]] = ..., validation: _Optional[_Union[PromptFieldValidation, _Mapping]] = ...) -> None: ...

class PromptFieldValidation(_message.Message):
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
    minimum: _value_pb2.Value
    maximum: _value_pb2.Value
    min_items: int
    max_items: int
    def __init__(self, min_length: _Optional[int] = ..., max_length: _Optional[int] = ..., pattern: _Optional[str] = ..., minimum: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., maximum: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

class PromptAction(_message.Message):
    __slots__ = ("id", "label", "emphasis", "requires_condition")
    ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    EMPHASIS_FIELD_NUMBER: _ClassVar[int]
    REQUIRES_CONDITION_FIELD_NUMBER: _ClassVar[int]
    id: str
    label: str
    emphasis: PromptActionEmphasis
    requires_condition: bool
    def __init__(self, id: _Optional[str] = ..., label: _Optional[str] = ..., emphasis: _Optional[_Union[PromptActionEmphasis, str]] = ..., requires_condition: _Optional[bool] = ...) -> None: ...

class AnswerRule(_message.Message):
    __slots__ = ("predicate", "message")
    PREDICATE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    predicate: AnswerPredicate
    message: str
    def __init__(self, predicate: _Optional[_Union[AnswerPredicate, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

class AnswerPredicate(_message.Message):
    __slots__ = ("literal", "field", "binary")
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    NOT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    literal: _value_pb2.Value
    field: str
    binary: AnswerBinaryPredicate
    def __init__(self, literal: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., field: _Optional[str] = ..., binary: _Optional[_Union[AnswerBinaryPredicate, _Mapping]] = ..., **kwargs) -> None: ...

class AnswerBinaryPredicate(_message.Message):
    __slots__ = ("op", "left", "right")
    OP_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    op: AnswerOperator
    left: AnswerPredicate
    right: AnswerPredicate
    def __init__(self, op: _Optional[_Union[AnswerOperator, str]] = ..., left: _Optional[_Union[AnswerPredicate, _Mapping]] = ..., right: _Optional[_Union[AnswerPredicate, _Mapping]] = ...) -> None: ...

class EvidenceCondition(_message.Message):
    __slots__ = ("record", "telemetry", "all", "any")
    RECORD_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    ANY_FIELD_NUMBER: _ClassVar[int]
    record: RecordMatchCondition
    telemetry: TelemetryMatchCondition
    all: EvidenceConditions
    any: EvidenceConditions
    def __init__(self, record: _Optional[_Union[RecordMatchCondition, _Mapping]] = ..., telemetry: _Optional[_Union[TelemetryMatchCondition, _Mapping]] = ..., all: _Optional[_Union[EvidenceConditions, _Mapping]] = ..., any: _Optional[_Union[EvidenceConditions, _Mapping]] = ...) -> None: ...

class EvidenceConditions(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedCompositeFieldContainer[EvidenceCondition]
    def __init__(self, conditions: _Optional[_Iterable[_Union[EvidenceCondition, _Mapping]]] = ...) -> None: ...

class RecordMatchCondition(_message.Message):
    __slots__ = ("record_type", "match_fields")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    MATCH_FIELDS_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    match_fields: _value_pb2.ValueObject
    def __init__(self, record_type: _Optional[str] = ..., match_fields: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...

class TelemetryMatchCondition(_message.Message):
    __slots__ = ("stream", "field", "op", "target", "tolerance")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    op: TelemetryMatchOperator
    target: _value_pb2.Value
    tolerance: float
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ..., op: _Optional[_Union[TelemetryMatchOperator, str]] = ..., target: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., tolerance: _Optional[float] = ...) -> None: ...

class RespondToPromptRequest(_message.Message):
    __slots__ = ("prompt_id", "action", "values", "accepted", "context")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    action: str
    values: _value_pb2.ValueObject
    accepted: bool
    context: _execution_pb2.MutationContext
    def __init__(self, prompt_id: _Optional[str] = ..., action: _Optional[str] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., accepted: _Optional[bool] = ..., context: _Optional[_Union[_execution_pb2.MutationContext, _Mapping]] = ...) -> None: ...

class PromptResponseReceipt(_message.Message):
    __slots__ = ("command_id", "prompt_id", "committed_revision", "accepted_at", "observed_cursor")
    COMMAND_ID_FIELD_NUMBER: _ClassVar[int]
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    COMMITTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_AT_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    command_id: str
    prompt_id: str
    committed_revision: int
    accepted_at: _timestamp_pb2.Timestamp
    observed_cursor: _common_pb2.Cursor
    def __init__(self, command_id: _Optional[str] = ..., prompt_id: _Optional[str] = ..., committed_revision: _Optional[int] = ..., accepted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class PromptResponse(_message.Message):
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
    submitted_at: _timestamp_pb2.Timestamp
    values: _value_pb2.ValueObject
    accepted: bool
    def __init__(self, prompt_id: _Optional[str] = ..., action: _Optional[str] = ..., responder: _Optional[str] = ..., submitted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., accepted: _Optional[bool] = ...) -> None: ...
