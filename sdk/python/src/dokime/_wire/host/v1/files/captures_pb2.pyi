from dokime._wire.model.v1 import files_pb2 as _files_pb2
from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CapturesStartRequest(_message.Message):
    __slots__ = ("source", "output", "mode")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    source: _files_pb2.FileSource
    output: str
    mode: CaptureMode
    def __init__(self, source: _Optional[_Union[_files_pb2.FileSource, _Mapping]] = ..., output: _Optional[str] = ..., mode: _Optional[_Union[CaptureMode, _Mapping]] = ...) -> None: ...

class CapturesStartResponse(_message.Message):
    __slots__ = ("capture_id",)
    CAPTURE_ID_FIELD_NUMBER: _ClassVar[int]
    capture_id: int
    def __init__(self, capture_id: _Optional[int] = ...) -> None: ...

class CapturesStopRequest(_message.Message):
    __slots__ = ("capture_id",)
    CAPTURE_ID_FIELD_NUMBER: _ClassVar[int]
    capture_id: int
    def __init__(self, capture_id: _Optional[int] = ...) -> None: ...

class CapturesStopResponse(_message.Message):
    __slots__ = ("artifacts", "gaps")
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    GAPS_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[_files_pb2.CapturedArtifact]
    gaps: _containers.RepeatedCompositeFieldContainer[_files_pb2.CaptureGap]
    def __init__(self, artifacts: _Optional[_Iterable[_Union[_files_pb2.CapturedArtifact, _Mapping]]] = ..., gaps: _Optional[_Iterable[_Union[_files_pb2.CaptureGap, _Mapping]]] = ...) -> None: ...

class CaptureMode(_message.Message):
    __slots__ = ("stream", "mirror")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    MIRROR_FIELD_NUMBER: _ClassVar[int]
    stream: StreamCapture
    mirror: MirrorCapture
    def __init__(self, stream: _Optional[_Union[StreamCapture, _Mapping]] = ..., mirror: _Optional[_Union[MirrorCapture, _Mapping]] = ...) -> None: ...

class StreamCapture(_message.Message):
    __slots__ = ("matchers", "markers")
    MATCHERS_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    matchers: _containers.RepeatedCompositeFieldContainer[LineMatcher]
    markers: _containers.RepeatedCompositeFieldContainer[LineMarker]
    def __init__(self, matchers: _Optional[_Iterable[_Union[LineMatcher, _Mapping]]] = ..., markers: _Optional[_Iterable[_Union[LineMarker, _Mapping]]] = ...) -> None: ...

class MirrorCapture(_message.Message):
    __slots__ = ("interval_s",)
    INTERVAL_S_FIELD_NUMBER: _ClassVar[int]
    interval_s: int
    def __init__(self, interval_s: _Optional[int] = ...) -> None: ...

class LineMatcher(_message.Message):
    __slots__ = ("pattern", "stream", "fields")
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    stream: str
    fields: _containers.RepeatedCompositeFieldContainer[LineField]
    def __init__(self, pattern: _Optional[str] = ..., stream: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[LineField, _Mapping]]] = ...) -> None: ...

class LineField(_message.Message):
    __slots__ = ("name", "type")
    NAME_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    name: str
    type: _scalar_pb2.ValueType
    def __init__(self, name: _Optional[str] = ..., type: _Optional[_Union[_scalar_pb2.ValueType, str]] = ...) -> None: ...

class LineMarker(_message.Message):
    __slots__ = ("pattern", "record_type")
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    record_type: str
    def __init__(self, pattern: _Optional[str] = ..., record_type: _Optional[str] = ...) -> None: ...
