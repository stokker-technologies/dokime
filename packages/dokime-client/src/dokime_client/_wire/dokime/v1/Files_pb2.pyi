from dokime_client._wire.dokime.v1.FilesMessages.FileWatch import messages_pb2 as _messages_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class FilesFileWatchOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2.Row3
    response: _messages_pb2.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class FilesFileWatchCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class FilesFileWatchRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: FilesFileWatchCancellation
    finish: FilesFileWatchCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[FilesFileWatchCancellation, _Mapping]] = ..., finish: _Optional[_Union[FilesFileWatchCancellation, _Mapping]] = ...) -> None: ...

class FilesFileWatchRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    item: _messages_pb2.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ...) -> None: ...
