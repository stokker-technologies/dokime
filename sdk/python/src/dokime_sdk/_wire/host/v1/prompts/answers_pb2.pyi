from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._wire.host.v1 import prompts_pb2 as _prompts_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class AnswersCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    PARENT_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    parent: str
    action: str
    values: _scalar_pb2.MapValue
    accepted: bool
    request_id: str
    def __init__(self, parent: _Optional[str] = ..., action: _Optional[str] = ..., values: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ..., accepted: _Optional[bool] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class AnswersCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ANSWER_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    answer: _prompts_pb2.PromptResponse
    receipt: _receipt_pb2.Receipt
    def __init__(self, answer: _Optional[_Union[_prompts_pb2.PromptResponse, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...
