from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._generated.proto.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._generated.proto.model.v1 import scalar_pb2 as _scalar_pb2
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

class PromptFieldKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PROMPT_FIELD_KIND_UNSPECIFIED: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_TEXT: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_FLOAT: _ClassVar[PromptFieldKind]
    PROMPT_FIELD_KIND_INTEGER: _ClassVar[PromptFieldKind]
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
    ANSWER_OPERATOR_GE: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_LT: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_LE: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_CONTAINS: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_STARTS_WITH: _ClassVar[AnswerOperator]
    ANSWER_OPERATOR_ENDS_WITH: _ClassVar[AnswerOperator]
PROMPT_ARCHETYPE_UNSPECIFIED: PromptArchetype
PROMPT_ARCHETYPE_CONFIRMATION: PromptArchetype
PROMPT_ARCHETYPE_FORM: PromptArchetype
PROMPT_ARCHETYPE_CHOICE: PromptArchetype
PROMPT_ARCHETYPE_STATUS: PromptArchetype
PROMPT_FIELD_KIND_UNSPECIFIED: PromptFieldKind
PROMPT_FIELD_KIND_TEXT: PromptFieldKind
PROMPT_FIELD_KIND_FLOAT: PromptFieldKind
PROMPT_FIELD_KIND_INTEGER: PromptFieldKind
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
ANSWER_OPERATOR_GE: AnswerOperator
ANSWER_OPERATOR_LT: AnswerOperator
ANSWER_OPERATOR_LE: AnswerOperator
ANSWER_OPERATOR_CONTAINS: AnswerOperator
ANSWER_OPERATOR_STARTS_WITH: AnswerOperator
ANSWER_OPERATOR_ENDS_WITH: AnswerOperator

@final
class PromptDefinitions(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    prompts: _containers.RepeatedCompositeFieldContainer[Prompt]
    def __init__(self, prompts: _Optional[_Iterable[_Union[Prompt, _Mapping]]] = ...) -> None: ...

@final
class Prompt(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    ACTIONS_FIELD_NUMBER: _ClassVar[int]
    ARCHETYPE_FIELD_NUMBER: _ClassVar[int]
    RULES_FIELD_NUMBER: _ClassVar[int]
    CONDITION_FIELD_NUMBER: _ClassVar[int]
    OWNER_SUITE_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    EXAMPLES_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    message: str
    fields: _containers.RepeatedCompositeFieldContainer[PromptField]
    actions: _containers.RepeatedCompositeFieldContainer[PromptAction]
    archetype: PromptArchetype
    rules: _containers.RepeatedCompositeFieldContainer[AnswerRule]
    condition: _evidence_pb2.EvidenceCondition
    owner_suite: str
    template: str
    examples: _containers.RepeatedCompositeFieldContainer[PromptValues]
    level: _evidence_pb2.Severity
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., message: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[PromptField, _Mapping]]] = ..., actions: _Optional[_Iterable[_Union[PromptAction, _Mapping]]] = ..., archetype: _Optional[_Union[PromptArchetype, str]] = ..., rules: _Optional[_Iterable[_Union[AnswerRule, _Mapping]]] = ..., condition: _Optional[_Union[_evidence_pb2.EvidenceCondition, _Mapping]] = ..., owner_suite: _Optional[str] = ..., template: _Optional[str] = ..., examples: _Optional[_Iterable[_Union[PromptValues, _Mapping]]] = ..., level: _Optional[_Union[_evidence_pb2.Severity, str]] = ...) -> None: ...

@final
class PromptValues(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    @final
    class ValuesEntry(_message.Message, metaclass=GeneratedProtocolMessageType):
        __slots__: tuple[str, ...]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.MessageMap[str, _scalar_pb2.Value]
    def __init__(self, values: _Optional[_Mapping[str, _scalar_pb2.Value]] = ...) -> None: ...

@final
class PromptField(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    REQUIRED_FIELD_NUMBER: _ClassVar[int]
    MULTIPLE_FIELD_NUMBER: _ClassVar[int]
    DEFAULT_VALUE_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    VALIDATION_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    type: _scalar_pb2.ValueType
    required: bool
    multiple: bool
    default_value: _scalar_pb2.Value
    choices: _containers.RepeatedCompositeFieldContainer[_scalar_pb2.Value]
    validation: PromptFieldValidation
    kind: PromptFieldKind
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., type: _Optional[_Union[_scalar_pb2.ValueType, str]] = ..., required: _Optional[bool] = ..., multiple: _Optional[bool] = ..., default_value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., choices: _Optional[_Iterable[_Union[_scalar_pb2.Value, _Mapping]]] = ..., validation: _Optional[_Union[PromptFieldValidation, _Mapping]] = ..., kind: _Optional[_Union[PromptFieldKind, str]] = ...) -> None: ...

@final
class PromptFieldValidation(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
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
    minimum: _scalar_pb2.Value
    maximum: _scalar_pb2.Value
    min_items: int
    max_items: int
    def __init__(self, min_length: _Optional[int] = ..., max_length: _Optional[int] = ..., pattern: _Optional[str] = ..., minimum: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., maximum: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., min_items: _Optional[int] = ..., max_items: _Optional[int] = ...) -> None: ...

@final
class PromptAction(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    EMPHASIS_FIELD_NUMBER: _ClassVar[int]
    REQUIRES_CONDITION_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    emphasis: PromptActionEmphasis
    requires_condition: bool
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., emphasis: _Optional[_Union[PromptActionEmphasis, str]] = ..., requires_condition: _Optional[bool] = ...) -> None: ...

@final
class AnswerRule(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    PREDICATE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    predicate: AnswerPredicate
    message: str
    def __init__(self, predicate: _Optional[_Union[AnswerPredicate, _Mapping]] = ..., message: _Optional[str] = ...) -> None: ...

@final
class AnswerPredicate(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    LITERAL_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    NOT_FIELD_NUMBER: _ClassVar[int]
    BINARY_FIELD_NUMBER: _ClassVar[int]
    literal: _scalar_pb2.Value
    field: str
    binary: AnswerBinaryPredicate
    def __init__(self, literal: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., field: _Optional[str] = ..., binary: _Optional[_Union[AnswerBinaryPredicate, _Mapping]] = ..., **kwargs) -> None: ...

@final
class AnswerBinaryPredicate(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    LEFT_FIELD_NUMBER: _ClassVar[int]
    RIGHT_FIELD_NUMBER: _ClassVar[int]
    operator: AnswerOperator
    left: AnswerPredicate
    right: AnswerPredicate
    def __init__(self, operator: _Optional[_Union[AnswerOperator, str]] = ..., left: _Optional[_Union[AnswerPredicate, _Mapping]] = ..., right: _Optional[_Union[AnswerPredicate, _Mapping]] = ...) -> None: ...
