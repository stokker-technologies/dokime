from dokime._wire.dokime.v1.ContentMessages.DownloadAppearanceAsset import messages_pb2 as _messages_pb2
from dokime._wire.dokime.v1.ContentMessages.DownloadArtifact import messages_pb2 as _messages_pb2_1
from dokime._wire.dokime.v1.ContentMessages.DownloadEvidence import messages_pb2 as _messages_pb2_1_1
from dokime._wire.dokime.v1.ContentMessages.DownloadExecutionFile import messages_pb2 as _messages_pb2_1_1_1
from dokime._wire.dokime.v1.ContentMessages.FollowExecutionFile import messages_pb2 as _messages_pb2_1_1_1_1
from dokime._wire.dokime.v1.ContentMessages.ListArtifacts import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime._wire.dokime.v1.ContentMessages.ListExecutionFiles import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from dokime._wire.dokime.v1.ContentMessages.ReadExecutionFile import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1
from dokime._wire.dokime.v1.ContentMessages.UploadArtifact import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ContentDownloadAppearanceAssetOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2.Row3
    response: _messages_pb2.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class ContentDownloadArtifactOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1.Row3
    response: _messages_pb2_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class ContentDownloadEvidenceOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1.Row3
    response: _messages_pb2_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentDownloadExecutionFileOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1.Row3
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentFollowExecutionFileOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1_1.Row3
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentUploadArtifactInput(_message.Message):
    __slots__ = ("request", "item")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1.Row1
    item: _messages_pb2_1_1_1_1_1_1_1_1.Row3
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...
