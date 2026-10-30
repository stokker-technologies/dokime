from dokime_client._wire.dokime.v1.BackendMessages.CollectDiagnostics import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.BackendMessages.GetAvailability import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.BackendMessages.GetHealth import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.BackendMessages.GetIdentity import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.BackendMessages.WatchLogs import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.BackendMessages.WatchTrace import messages_pb2 as _messages_pb2_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class BackendCollectDiagnosticsOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2.Row3
    response: _messages_pb2.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class BackendCollectDiagnosticsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BackendCollectDiagnosticsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: BackendCollectDiagnosticsCancellation
    finish: BackendCollectDiagnosticsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[BackendCollectDiagnosticsCancellation, _Mapping]] = ..., finish: _Optional[_Union[BackendCollectDiagnosticsCancellation, _Mapping]] = ...) -> None: ...

class BackendCollectDiagnosticsRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    item: _messages_pb2.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ...) -> None: ...

class BackendGetAvailabilityCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BackendGetAvailabilityRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: BackendGetAvailabilityCancellation
    finish: BackendGetAvailabilityCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[BackendGetAvailabilityCancellation, _Mapping]] = ..., finish: _Optional[_Union[BackendGetAvailabilityCancellation, _Mapping]] = ...) -> None: ...

class BackendGetAvailabilityRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class BackendGetHealthCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BackendGetHealthRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: BackendGetHealthCancellation
    finish: BackendGetHealthCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[BackendGetHealthCancellation, _Mapping]] = ..., finish: _Optional[_Union[BackendGetHealthCancellation, _Mapping]] = ...) -> None: ...

class BackendGetHealthRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class BackendGetIdentityCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BackendGetIdentityRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: BackendGetIdentityCancellation
    finish: BackendGetIdentityCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[BackendGetIdentityCancellation, _Mapping]] = ..., finish: _Optional[_Union[BackendGetIdentityCancellation, _Mapping]] = ...) -> None: ...

class BackendGetIdentityRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class BackendWatchLogsOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1_1.Row3
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class BackendWatchLogsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BackendWatchLogsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: BackendWatchLogsCancellation
    finish: BackendWatchLogsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[BackendWatchLogsCancellation, _Mapping]] = ..., finish: _Optional[_Union[BackendWatchLogsCancellation, _Mapping]] = ...) -> None: ...

class BackendWatchLogsRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    item: _messages_pb2_1_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class BackendWatchTraceOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1_1_1.Row3
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class BackendWatchTraceCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BackendWatchTraceRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: BackendWatchTraceCancellation
    finish: BackendWatchTraceCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[BackendWatchTraceCancellation, _Mapping]] = ..., finish: _Optional[_Union[BackendWatchTraceCancellation, _Mapping]] = ...) -> None: ...

class BackendWatchTraceRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    item: _messages_pb2_1_1_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...
