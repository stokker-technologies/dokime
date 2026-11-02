from dokime_client._wire.dokime.v1.ObservationMessages.GetRuntimeShell import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.ObservationMessages.GetTraceability import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.ObservationMessages.WatchExecutionProgress import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.ObservationMessages.WatchRuntime import messages_pb2 as _messages_pb2_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ObservationGetRuntimeShellCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ObservationGetRuntimeShellRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: ObservationGetRuntimeShellCancellation
    finish: ObservationGetRuntimeShellCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ObservationGetRuntimeShellCancellation, _Mapping]] = ..., finish: _Optional[_Union[ObservationGetRuntimeShellCancellation, _Mapping]] = ...) -> None: ...

class ObservationGetRuntimeShellRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class ObservationGetTraceabilityCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ObservationGetTraceabilityRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: ObservationGetTraceabilityCancellation
    finish: ObservationGetTraceabilityCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ObservationGetTraceabilityCancellation, _Mapping]] = ..., finish: _Optional[_Union[ObservationGetTraceabilityCancellation, _Mapping]] = ...) -> None: ...

class ObservationGetTraceabilityRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class ObservationWatchExecutionProgressOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1.Row3
    response: _messages_pb2_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class ObservationWatchExecutionProgressCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ObservationWatchExecutionProgressRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: ObservationWatchExecutionProgressCancellation
    finish: ObservationWatchExecutionProgressCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ObservationWatchExecutionProgressCancellation, _Mapping]] = ..., finish: _Optional[_Union[ObservationWatchExecutionProgressCancellation, _Mapping]] = ...) -> None: ...

class ObservationWatchExecutionProgressRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    item: _messages_pb2_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ...) -> None: ...

class ObservationWatchRuntimeOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1.Row3
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ObservationWatchRuntimeCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ObservationWatchRuntimeRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: ObservationWatchRuntimeCancellation
    finish: ObservationWatchRuntimeCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ObservationWatchRuntimeCancellation, _Mapping]] = ..., finish: _Optional[_Union[ObservationWatchRuntimeCancellation, _Mapping]] = ...) -> None: ...

class ObservationWatchRuntimeRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    item: _messages_pb2_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ...) -> None: ...
