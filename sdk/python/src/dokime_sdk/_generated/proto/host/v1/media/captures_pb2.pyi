from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import evidence_pb2 as _evidence_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import failure_pb2 as _failure_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from harmos._generated.proto.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CaptureKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CAPTURE_KIND_UNSPECIFIED: _ClassVar[CaptureKind]
    CAPTURE_KIND_AUDIO: _ClassVar[CaptureKind]
    CAPTURE_KIND_CAMERA: _ClassVar[CaptureKind]
    CAPTURE_KIND_PHOTO: _ClassVar[CaptureKind]
    CAPTURE_KIND_SCREEN: _ClassVar[CaptureKind]
    CAPTURE_KIND_SCREENSHOT: _ClassVar[CaptureKind]
CAPTURE_KIND_UNSPECIFIED: CaptureKind
CAPTURE_KIND_AUDIO: CaptureKind
CAPTURE_KIND_CAMERA: CaptureKind
CAPTURE_KIND_PHOTO: CaptureKind
CAPTURE_KIND_SCREEN: CaptureKind
CAPTURE_KIND_SCREENSHOT: CaptureKind

@final
class CapturesCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    INPUT_FIELD_NUMBER: _ClassVar[int]
    CAMERA_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_FIELD_NUMBER: _ClassVar[int]
    AUDIO_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_SECONDS_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.ArtifactScope
    kind: CaptureKind
    input: int
    camera: int
    display: int
    audio: RecordingAudio
    maximum_seconds: int
    request_id: str
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.ArtifactScope, _Mapping]] = ..., kind: _Optional[_Union[CaptureKind, str]] = ..., input: _Optional[int] = ..., camera: _Optional[int] = ..., display: _Optional[int] = ..., audio: _Optional[_Union[RecordingAudio, _Mapping]] = ..., maximum_seconds: _Optional[int] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class CapturesCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CAPTURE_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    capture: Capture
    receipt: _receipt_pb2.Receipt
    def __init__(self, capture: _Optional[_Union[Capture, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class CapturesGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class CapturesGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CAPTURE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    capture: Capture
    resolved_at: _position_pb2.ReadCut
    def __init__(self, capture: _Optional[_Union[Capture, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class CapturesWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class CapturesWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CAPTURE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    capture: Capture
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, capture: _Optional[_Union[Capture, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

@final
class CapturesStopRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class CapturesStopResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CAPTURE_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    capture: Capture
    receipt: _receipt_pb2.Receipt
    def __init__(self, capture: _Optional[_Union[Capture, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class RecordingAudio(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    MICROPHONE_FIELD_NUMBER: _ClassVar[int]
    SYSTEM_FIELD_NUMBER: _ClassVar[int]
    microphone: bool
    system: bool
    def __init__(self, microphone: _Optional[bool] = ..., system: _Optional[bool] = ...) -> None: ...

@final
class RecordingProgress(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ELAPSED_MS_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    elapsed_ms: int
    size_bytes: int
    def __init__(self, elapsed_ms: _Optional[int] = ..., size_bytes: _Optional[int] = ...) -> None: ...

@final
class Capture(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_FIELD_NUMBER: _ClassVar[int]
    FAILURE_FIELD_NUMBER: _ClassVar[int]
    name: str
    kind: CaptureKind
    progress: RecordingProgress
    artifact: _evidence_pb2.ArtifactMetadata
    finished: bool
    failure: _failure_pb2.Failure
    def __init__(self, name: _Optional[str] = ..., kind: _Optional[_Union[CaptureKind, str]] = ..., progress: _Optional[_Union[RecordingProgress, _Mapping]] = ..., artifact: _Optional[_Union[_evidence_pb2.ArtifactMetadata, _Mapping]] = ..., finished: _Optional[bool] = ..., failure: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...
