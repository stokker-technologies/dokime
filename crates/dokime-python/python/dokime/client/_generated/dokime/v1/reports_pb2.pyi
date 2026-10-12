import datetime

from dokime.client._generated.dokime.v1 import checkpoint_pb2 as _checkpoint_pb2
from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class DownloadCycleReportRequest(_message.Message):
    __slots__ = ("cycle_id", "identifier")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    identifier: str
    def __init__(self, cycle_id: _Optional[str] = ..., identifier: _Optional[str] = ...) -> None: ...

class RenderCheckpointReportRequest(_message.Message):
    __slots__ = ("context", "checkpoints", "identifier")
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINTS_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    context: _checkpoint_pb2.ExecutionCheckpoint
    checkpoints: _containers.RepeatedCompositeFieldContainer[_checkpoint_pb2.ExecutionCheckpoint]
    identifier: str
    def __init__(self, context: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., checkpoints: _Optional[_Iterable[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]]] = ..., identifier: _Optional[str] = ...) -> None: ...

class RenderedReport(_message.Message):
    __slots__ = ("identifier", "metadata", "generated_at", "warnings", "html")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    HTML_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    metadata: _common_pb2.Metadata
    generated_at: _timestamp_pb2.Timestamp
    warnings: _containers.RepeatedScalarFieldContainer[str]
    html: str
    def __init__(self, identifier: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., generated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., warnings: _Optional[_Iterable[str]] = ..., html: _Optional[str] = ...) -> None: ...

class ReportMetadata(_message.Message):
    __slots__ = ("test_cycle_id", "identifier", "metadata", "generated_at", "warnings")
    TEST_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    GENERATED_AT_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    test_cycle_id: str
    identifier: str
    metadata: _common_pb2.Metadata
    generated_at: _timestamp_pb2.Timestamp
    warnings: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, test_cycle_id: _Optional[str] = ..., identifier: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., generated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., warnings: _Optional[_Iterable[str]] = ...) -> None: ...

class ReportHtmlFrame(_message.Message):
    __slots__ = ("report", "chunk", "complete")
    REPORT_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    report: ReportMetadata
    chunk: ReportHtmlChunk
    complete: ReportHtmlCompletion
    def __init__(self, report: _Optional[_Union[ReportMetadata, _Mapping]] = ..., chunk: _Optional[_Union[ReportHtmlChunk, _Mapping]] = ..., complete: _Optional[_Union[ReportHtmlCompletion, _Mapping]] = ...) -> None: ...

class ReportHtmlChunk(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class ReportHtmlCompletion(_message.Message):
    __slots__ = ("size_bytes", "sha256")
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    size_bytes: int
    sha256: str
    def __init__(self, size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...
