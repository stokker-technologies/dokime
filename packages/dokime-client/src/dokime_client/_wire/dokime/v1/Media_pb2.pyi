from dokime_client._wire.dokime.v1.MediaMessages.AudioRecording import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.MediaMessages.CameraRecording import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.MediaMessages.Photo import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.MediaMessages.ScreenRecording import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.MediaMessages.Screenshot import messages_pb2 as _messages_pb2_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MediaAudioRecordingOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2.Row3
    response: _messages_pb2.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class MediaAudioRecordingCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class MediaAudioRecordingRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: MediaAudioRecordingCancellation
    finish: MediaAudioRecordingCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[MediaAudioRecordingCancellation, _Mapping]] = ..., finish: _Optional[_Union[MediaAudioRecordingCancellation, _Mapping]] = ...) -> None: ...

class MediaAudioRecordingRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    item: _messages_pb2.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ...) -> None: ...

class MediaCameraRecordingOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1.Row3
    response: _messages_pb2_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class MediaCameraRecordingCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class MediaCameraRecordingRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: MediaCameraRecordingCancellation
    finish: MediaCameraRecordingCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[MediaCameraRecordingCancellation, _Mapping]] = ..., finish: _Optional[_Union[MediaCameraRecordingCancellation, _Mapping]] = ...) -> None: ...

class MediaCameraRecordingRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    item: _messages_pb2_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ...) -> None: ...

class MediaPhotoCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class MediaPhotoRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: MediaPhotoCancellation
    finish: MediaPhotoCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[MediaPhotoCancellation, _Mapping]] = ..., finish: _Optional[_Union[MediaPhotoCancellation, _Mapping]] = ...) -> None: ...

class MediaPhotoRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class MediaScreenRecordingOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1.Row3
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class MediaScreenRecordingCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class MediaScreenRecordingRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: MediaScreenRecordingCancellation
    finish: MediaScreenRecordingCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[MediaScreenRecordingCancellation, _Mapping]] = ..., finish: _Optional[_Union[MediaScreenRecordingCancellation, _Mapping]] = ...) -> None: ...

class MediaScreenRecordingRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    item: _messages_pb2_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class MediaScreenshotCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class MediaScreenshotRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: MediaScreenshotCancellation
    finish: MediaScreenshotCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[MediaScreenshotCancellation, _Mapping]] = ..., finish: _Optional[_Union[MediaScreenshotCancellation, _Mapping]] = ...) -> None: ...

class MediaScreenshotRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
