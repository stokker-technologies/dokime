from dokime_client._wire.dokime.v1.PlotsMessages.ListAnalyses import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.PlotsMessages.QuerySnapshots import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.PlotsMessages.Watch import messages_pb2 as _messages_pb2_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PlotsListAnalysesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PlotsListAnalysesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: PlotsListAnalysesCancellation
    finish: PlotsListAnalysesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PlotsListAnalysesCancellation, _Mapping]] = ..., finish: _Optional[_Union[PlotsListAnalysesCancellation, _Mapping]] = ...) -> None: ...

class PlotsListAnalysesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class PlotsQuerySnapshotsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PlotsQuerySnapshotsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: PlotsQuerySnapshotsCancellation
    finish: PlotsQuerySnapshotsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PlotsQuerySnapshotsCancellation, _Mapping]] = ..., finish: _Optional[_Union[PlotsQuerySnapshotsCancellation, _Mapping]] = ...) -> None: ...

class PlotsQuerySnapshotsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class PlotsWatchOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1.Row3
    response: _messages_pb2_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class PlotsWatchCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PlotsWatchRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: PlotsWatchCancellation
    finish: PlotsWatchCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PlotsWatchCancellation, _Mapping]] = ..., finish: _Optional[_Union[PlotsWatchCancellation, _Mapping]] = ...) -> None: ...

class PlotsWatchRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    item: _messages_pb2_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ...) -> None: ...
