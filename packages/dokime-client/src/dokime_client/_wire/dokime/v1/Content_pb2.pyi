from dokime_client._wire.dokime.v1.ContentMessages.DownloadAppearanceAsset import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.ContentMessages.DownloadArtifact import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.ContentMessages.DownloadEvidence import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.ContentMessages.DownloadExecutionFile import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.ContentMessages.FollowExecutionFile import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.ContentMessages.ListArtifacts import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime_client._wire.dokime.v1.ContentMessages.ListExecutionFiles import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ContentMessages.ReadExecutionFile import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.ContentMessages.UploadArtifact import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1
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

class ContentDownloadAppearanceAssetCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentDownloadAppearanceAssetRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: ContentDownloadAppearanceAssetCancellation
    finish: ContentDownloadAppearanceAssetCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentDownloadAppearanceAssetCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentDownloadAppearanceAssetCancellation, _Mapping]] = ...) -> None: ...

class ContentDownloadAppearanceAssetRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    item: _messages_pb2.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ...) -> None: ...

class ContentDownloadArtifactOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1.Row3
    response: _messages_pb2_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class ContentDownloadArtifactCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentDownloadArtifactRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: ContentDownloadArtifactCancellation
    finish: ContentDownloadArtifactCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentDownloadArtifactCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentDownloadArtifactCancellation, _Mapping]] = ...) -> None: ...

class ContentDownloadArtifactRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    item: _messages_pb2_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ...) -> None: ...

class ContentDownloadEvidenceOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1.Row3
    response: _messages_pb2_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentDownloadEvidenceCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentDownloadEvidenceRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: ContentDownloadEvidenceCancellation
    finish: ContentDownloadEvidenceCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentDownloadEvidenceCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentDownloadEvidenceCancellation, _Mapping]] = ...) -> None: ...

class ContentDownloadEvidenceRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    item: _messages_pb2_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ...) -> None: ...

class ContentDownloadExecutionFileOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1.Row3
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentDownloadExecutionFileCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentDownloadExecutionFileRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: ContentDownloadExecutionFileCancellation
    finish: ContentDownloadExecutionFileCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentDownloadExecutionFileCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentDownloadExecutionFileCancellation, _Mapping]] = ...) -> None: ...

class ContentDownloadExecutionFileRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    item: _messages_pb2_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class ContentFollowExecutionFileOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1_1.Row3
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentFollowExecutionFileCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentFollowExecutionFileRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: ContentFollowExecutionFileCancellation
    finish: ContentFollowExecutionFileCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentFollowExecutionFileCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentFollowExecutionFileCancellation, _Mapping]] = ...) -> None: ...

class ContentFollowExecutionFileRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    item: _messages_pb2_1_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class ContentListArtifactsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentListArtifactsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: ContentListArtifactsCancellation
    finish: ContentListArtifactsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentListArtifactsCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentListArtifactsCancellation, _Mapping]] = ...) -> None: ...

class ContentListArtifactsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentListExecutionFilesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentListExecutionFilesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    cancel: ContentListExecutionFilesCancellation
    finish: ContentListExecutionFilesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentListExecutionFilesCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentListExecutionFilesCancellation, _Mapping]] = ...) -> None: ...

class ContentListExecutionFilesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentReadExecutionFileCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentReadExecutionFileRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1.Row1
    cancel: ContentReadExecutionFileCancellation
    finish: ContentReadExecutionFileCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentReadExecutionFileCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentReadExecutionFileCancellation, _Mapping]] = ...) -> None: ...

class ContentReadExecutionFileRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class ContentUploadArtifactInput(_message.Message):
    __slots__ = ("request", "item")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1.Row1
    item: _messages_pb2_1_1_1_1_1_1_1_1.Row3
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class ContentUploadArtifactCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ContentUploadArtifactRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish", "item")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1.Row1
    cancel: ContentUploadArtifactCancellation
    finish: ContentUploadArtifactCancellation
    item: _messages_pb2_1_1_1_1_1_1_1_1.Row3
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ContentUploadArtifactCancellation, _Mapping]] = ..., finish: _Optional[_Union[ContentUploadArtifactCancellation, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class ContentUploadArtifactRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
