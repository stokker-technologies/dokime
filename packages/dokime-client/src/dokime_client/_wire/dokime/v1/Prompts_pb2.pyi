from dokime_client._wire.dokime.v1.PromptsMessages.GetDefinition import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.PromptsMessages.GetPrompt import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.PromptsMessages.ListDefinitions import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.PromptsMessages.ListPending import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.PromptsMessages.Respond import messages_pb2 as _messages_pb2_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PromptsGetDefinitionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PromptsGetDefinitionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: PromptsGetDefinitionCancellation
    finish: PromptsGetDefinitionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PromptsGetDefinitionCancellation, _Mapping]] = ..., finish: _Optional[_Union[PromptsGetDefinitionCancellation, _Mapping]] = ...) -> None: ...

class PromptsGetDefinitionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class PromptsGetPromptCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PromptsGetPromptRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: PromptsGetPromptCancellation
    finish: PromptsGetPromptCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PromptsGetPromptCancellation, _Mapping]] = ..., finish: _Optional[_Union[PromptsGetPromptCancellation, _Mapping]] = ...) -> None: ...

class PromptsGetPromptRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class PromptsListDefinitionsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PromptsListDefinitionsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: PromptsListDefinitionsCancellation
    finish: PromptsListDefinitionsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PromptsListDefinitionsCancellation, _Mapping]] = ..., finish: _Optional[_Union[PromptsListDefinitionsCancellation, _Mapping]] = ...) -> None: ...

class PromptsListDefinitionsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class PromptsListPendingCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PromptsListPendingRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: PromptsListPendingCancellation
    finish: PromptsListPendingCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PromptsListPendingCancellation, _Mapping]] = ..., finish: _Optional[_Union[PromptsListPendingCancellation, _Mapping]] = ...) -> None: ...

class PromptsListPendingRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class PromptsRespondCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PromptsRespondRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: PromptsRespondCancellation
    finish: PromptsRespondCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PromptsRespondCancellation, _Mapping]] = ..., finish: _Optional[_Union[PromptsRespondCancellation, _Mapping]] = ...) -> None: ...

class PromptsRespondRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
