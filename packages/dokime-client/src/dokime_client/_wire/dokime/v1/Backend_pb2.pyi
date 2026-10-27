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

class BackendWatchLogsOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1_1.Row3
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class BackendWatchTraceOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1_1_1.Row3
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
