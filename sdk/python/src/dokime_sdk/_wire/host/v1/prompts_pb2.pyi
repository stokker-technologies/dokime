import datetime

from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._wire.model.v1 import prompts_pb2 as _prompts_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PromptKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PROMPT_KIND_UNSPECIFIED: _ClassVar[PromptKind]
    PROMPT_KIND_CONFIRMATION: _ClassVar[PromptKind]
    PROMPT_KIND_FORM: _ClassVar[PromptKind]
    PROMPT_KIND_CHOICE: _ClassVar[PromptKind]
    PROMPT_KIND_STATUS: _ClassVar[PromptKind]
    PROMPT_KIND_VERIFICATION: _ClassVar[PromptKind]
PROMPT_KIND_UNSPECIFIED: PromptKind
PROMPT_KIND_CONFIRMATION: PromptKind
PROMPT_KIND_FORM: PromptKind
PROMPT_KIND_CHOICE: PromptKind
PROMPT_KIND_STATUS: PromptKind
PROMPT_KIND_VERIFICATION: PromptKind

class PromptsGetRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class PromptsGetResponse(_message.Message):
    __slots__ = ("prompt", "resolved_at")
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    prompt: PromptView
    resolved_at: _position_pb2.ReadCut
    def __init__(self, prompt: _Optional[_Union[PromptView, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class PromptsListRequest(_message.Message):
    __slots__ = ("parent", "cycle_id", "execution_id", "pending_only", "at", "page")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    PENDING_ONLY_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    parent: str
    cycle_id: str
    execution_id: str
    pending_only: bool
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, parent: _Optional[str] = ..., cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., pending_only: _Optional[bool] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class PromptsListResponse(_message.Message):
    __slots__ = ("prompts", "resolved_at", "next_page_token")
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    prompts: _containers.RepeatedCompositeFieldContainer[PromptView]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, prompts: _Optional[_Iterable[_Union[PromptView, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

class PromptsWatchRequest(_message.Message):
    __slots__ = ("parent", "cycle_id", "execution_id", "pending_only", "at", "page")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    PENDING_ONLY_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    parent: str
    cycle_id: str
    execution_id: str
    pending_only: bool
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, parent: _Optional[str] = ..., cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., pending_only: _Optional[bool] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class PromptsWatchResponse(_message.Message):
    __slots__ = ("prompts", "resolved_at", "opening", "resync_required")
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    prompts: _containers.RepeatedCompositeFieldContainer[PromptView]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, prompts: _Optional[_Iterable[_Union[PromptView, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class PromptsCreateRequest(_message.Message):
    __slots__ = ("prompt", "prompt_type", "values", "definition")
    class ValuesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    PROMPT_TYPE_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    prompt: str
    prompt_type: str
    values: _containers.MessageMap[str, _scalar_pb2.Value]
    definition: _prompts_pb2.Prompt
    def __init__(self, prompt: _Optional[str] = ..., prompt_type: _Optional[str] = ..., values: _Optional[_Mapping[str, _scalar_pb2.Value]] = ..., definition: _Optional[_Union[_prompts_pb2.Prompt, _Mapping]] = ...) -> None: ...

class PromptsCreateResponse(_message.Message):
    __slots__ = ("prompt", "action", "values", "responder", "submitted_at", "accepted")
    class ValuesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    RESPONDER_FIELD_NUMBER: _ClassVar[int]
    SUBMITTED_AT_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    prompt: str
    action: str
    values: _containers.MessageMap[str, _scalar_pb2.Value]
    responder: str
    submitted_at: _timestamp_pb2.Timestamp
    accepted: bool
    def __init__(self, prompt: _Optional[str] = ..., action: _Optional[str] = ..., values: _Optional[_Mapping[str, _scalar_pb2.Value]] = ..., responder: _Optional[str] = ..., submitted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., accepted: _Optional[bool] = ...) -> None: ...

class PromptInstance(_message.Message):
    __slots__ = ("prompt_id", "definition", "kind", "values", "scope", "created_at")
    PROMPT_ID_FIELD_NUMBER: _ClassVar[int]
    DEFINITION_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    prompt_id: str
    definition: _prompts_pb2.Prompt
    kind: PromptKind
    values: _scalar_pb2.MapValue
    scope: _evidence_pb2.EvidenceScope
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, prompt_id: _Optional[str] = ..., definition: _Optional[_Union[_prompts_pb2.Prompt, _Mapping]] = ..., kind: _Optional[_Union[PromptKind, str]] = ..., values: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class PromptView(_message.Message):
    __slots__ = ("prompt", "response", "observed_cursor")
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    prompt: PromptInstance
    response: PromptResponse
    observed_cursor: _position_pb2.ReadCut
    def __init__(self, prompt: _Optional[_Union[PromptInstance, _Mapping]] = ..., response: _Optional[_Union[PromptResponse, _Mapping]] = ..., observed_cursor: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

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
    values: _scalar_pb2.MapValue
    accepted: bool
    def __init__(self, prompt_id: _Optional[str] = ..., action: _Optional[str] = ..., responder: _Optional[str] = ..., submitted_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., values: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ..., accepted: _Optional[bool] = ...) -> None: ...
