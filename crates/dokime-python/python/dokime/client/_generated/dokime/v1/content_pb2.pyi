import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from dokime.client._generated.dokime.v1 import appearance_pb2 as _appearance_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ArtifactState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ARTIFACT_STATE_UNSPECIFIED: _ClassVar[ArtifactState]
    ARTIFACT_STATE_COMPLETE: _ClassVar[ArtifactState]
    ARTIFACT_STATE_CANCELLED: _ClassVar[ArtifactState]
    ARTIFACT_STATE_REJECTED: _ClassVar[ArtifactState]

class ExecutionFileKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_FILE_KIND_UNSPECIFIED: _ClassVar[ExecutionFileKind]
    EXECUTION_FILE_KIND_TEXT: _ClassVar[ExecutionFileKind]
    EXECUTION_FILE_KIND_BINARY: _ClassVar[ExecutionFileKind]
    EXECUTION_FILE_KIND_MEDIA: _ClassVar[ExecutionFileKind]
    EXECUTION_FILE_KIND_UNKNOWN: _ClassVar[ExecutionFileKind]

class ExecutionFileState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_FILE_STATE_UNSPECIFIED: _ClassVar[ExecutionFileState]
    EXECUTION_FILE_STATE_STREAMING: _ClassVar[ExecutionFileState]
    EXECUTION_FILE_STATE_FINALIZED: _ClassVar[ExecutionFileState]
    EXECUTION_FILE_STATE_INTERRUPTED: _ClassVar[ExecutionFileState]
ARTIFACT_STATE_UNSPECIFIED: ArtifactState
ARTIFACT_STATE_COMPLETE: ArtifactState
ARTIFACT_STATE_CANCELLED: ArtifactState
ARTIFACT_STATE_REJECTED: ArtifactState
EXECUTION_FILE_KIND_UNSPECIFIED: ExecutionFileKind
EXECUTION_FILE_KIND_TEXT: ExecutionFileKind
EXECUTION_FILE_KIND_BINARY: ExecutionFileKind
EXECUTION_FILE_KIND_MEDIA: ExecutionFileKind
EXECUTION_FILE_KIND_UNKNOWN: ExecutionFileKind
EXECUTION_FILE_STATE_UNSPECIFIED: ExecutionFileState
EXECUTION_FILE_STATE_STREAMING: ExecutionFileState
EXECUTION_FILE_STATE_FINALIZED: ExecutionFileState
EXECUTION_FILE_STATE_INTERRUPTED: ExecutionFileState

class ListArtifactsRequest(_message.Message):
    __slots__ = ("limit", "after_artifact_id", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_artifact_id: str
    cursor: ArtifactCursor
    def __init__(self, limit: _Optional[int] = ..., after_artifact_id: _Optional[str] = ..., cursor: _Optional[_Union[ArtifactCursor, _Mapping]] = ...) -> None: ...

class ArtifactCursor(_message.Message):
    __slots__ = ("epoch", "metadata_sha256")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    METADATA_SHA256_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    metadata_sha256: bytes
    def __init__(self, epoch: _Optional[str] = ..., metadata_sha256: _Optional[bytes] = ...) -> None: ...

class ListArtifactsReply(_message.Message):
    __slots__ = ("artifacts", "next_after_artifact_id", "cursor")
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[ArtifactMetadata]
    next_after_artifact_id: str
    cursor: ArtifactCursor
    def __init__(self, artifacts: _Optional[_Iterable[_Union[ArtifactMetadata, _Mapping]]] = ..., next_after_artifact_id: _Optional[str] = ..., cursor: _Optional[_Union[ArtifactCursor, _Mapping]] = ...) -> None: ...

class DownloadArtifactRequest(_message.Message):
    __slots__ = ("artifact_id",)
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    artifact_id: str
    def __init__(self, artifact_id: _Optional[str] = ...) -> None: ...

class ArtifactScope(_message.Message):
    __slots__ = ("cycle_id", "case_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class ArtifactMetadata(_message.Message):
    __slots__ = ("artifact_id", "upload_id", "scope", "filename", "content_type", "size_bytes", "sha256", "state", "created_at")
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    UPLOAD_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    artifact_id: str
    upload_id: str
    scope: ArtifactScope
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    state: ArtifactState
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, artifact_id: _Optional[str] = ..., upload_id: _Optional[str] = ..., scope: _Optional[_Union[ArtifactScope, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ..., state: _Optional[_Union[ArtifactState, str]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ArtifactFrame(_message.Message):
    __slots__ = ("metadata", "chunk", "complete")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    metadata: ArtifactMetadata
    chunk: ArtifactChunk
    complete: ArtifactCompletion
    def __init__(self, metadata: _Optional[_Union[ArtifactMetadata, _Mapping]] = ..., chunk: _Optional[_Union[ArtifactChunk, _Mapping]] = ..., complete: _Optional[_Union[ArtifactCompletion, _Mapping]] = ...) -> None: ...

class ArtifactChunk(_message.Message):
    __slots__ = ("offset", "data")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    offset: int
    data: bytes
    def __init__(self, offset: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class ArtifactCompletion(_message.Message):
    __slots__ = ("size_bytes", "sha256")
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    size_bytes: int
    sha256: str
    def __init__(self, size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class UploadArtifactFrame(_message.Message):
    __slots__ = ("header", "chunk", "complete")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    header: UploadArtifactHeader
    chunk: ArtifactChunk
    complete: ArtifactCompletion
    def __init__(self, header: _Optional[_Union[UploadArtifactHeader, _Mapping]] = ..., chunk: _Optional[_Union[ArtifactChunk, _Mapping]] = ..., complete: _Optional[_Union[ArtifactCompletion, _Mapping]] = ...) -> None: ...

class UploadArtifactHeader(_message.Message):
    __slots__ = ("scope", "filename", "content_type", "size_bytes", "sha256")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    scope: ArtifactScope
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    def __init__(self, scope: _Optional[_Union[ArtifactScope, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class DownloadCycleBundleRequest(_message.Message):
    __slots__ = ("cycle_id",)
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    def __init__(self, cycle_id: _Optional[str] = ...) -> None: ...

class CycleBundleFrame(_message.Message):
    __slots__ = ("metadata", "chunk", "complete")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    metadata: CycleBundleMetadata
    chunk: ArtifactChunk
    complete: ArtifactCompletion
    def __init__(self, metadata: _Optional[_Union[CycleBundleMetadata, _Mapping]] = ..., chunk: _Optional[_Union[ArtifactChunk, _Mapping]] = ..., complete: _Optional[_Union[ArtifactCompletion, _Mapping]] = ...) -> None: ...

class CycleBundleMetadata(_message.Message):
    __slots__ = ("cycle_id", "filename", "content_type", "size_bytes", "sha256")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    def __init__(self, cycle_id: _Optional[str] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class GetCycleFolderPathRequest(_message.Message):
    __slots__ = ("cycle_id",)
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    def __init__(self, cycle_id: _Optional[str] = ...) -> None: ...

class LocalPathResponse(_message.Message):
    __slots__ = ("path",)
    PATH_FIELD_NUMBER: _ClassVar[int]
    path: str
    def __init__(self, path: _Optional[str] = ...) -> None: ...

class ListExecutionFilesRequest(_message.Message):
    __slots__ = ("execution_id", "limit", "after_path", "cursor")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_PATH_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    limit: int
    after_path: str
    cursor: ExecutionFilesCursor
    def __init__(self, execution_id: _Optional[str] = ..., limit: _Optional[int] = ..., after_path: _Optional[str] = ..., cursor: _Optional[_Union[ExecutionFilesCursor, _Mapping]] = ...) -> None: ...

class ExecutionFilesCursor(_message.Message):
    __slots__ = ("epoch", "execution_id", "metadata_sha256")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_SHA256_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    execution_id: str
    metadata_sha256: bytes
    def __init__(self, epoch: _Optional[str] = ..., execution_id: _Optional[str] = ..., metadata_sha256: _Optional[bytes] = ...) -> None: ...

class ListExecutionFilesReply(_message.Message):
    __slots__ = ("files", "next_after_path", "cursor")
    FILES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_PATH_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    files: _containers.RepeatedCompositeFieldContainer[ExecutionFileMetadata]
    next_after_path: str
    cursor: ExecutionFilesCursor
    def __init__(self, files: _Optional[_Iterable[_Union[ExecutionFileMetadata, _Mapping]]] = ..., next_after_path: _Optional[str] = ..., cursor: _Optional[_Union[ExecutionFilesCursor, _Mapping]] = ...) -> None: ...

class ExecutionFileMetadata(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "size_bytes", "created_at")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: ExecutionFileKind
    state: ExecutionFileState
    size_bytes: int
    created_at: _timestamp_pb2.Timestamp
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[ExecutionFileKind, str]] = ..., state: _Optional[_Union[ExecutionFileState, str]] = ..., size_bytes: _Optional[int] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ReadExecutionFileRequest(_message.Message):
    __slots__ = ("execution_id", "relative_path", "offset", "max_bytes")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    relative_path: str
    offset: int
    max_bytes: int
    def __init__(self, execution_id: _Optional[str] = ..., relative_path: _Optional[str] = ..., offset: _Optional[int] = ..., max_bytes: _Optional[int] = ...) -> None: ...

class ExecutionFileWindow(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "offset", "observed_size_bytes", "data")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: ExecutionFileKind
    state: ExecutionFileState
    offset: int
    observed_size_bytes: int
    data: bytes
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[ExecutionFileKind, str]] = ..., state: _Optional[_Union[ExecutionFileState, str]] = ..., offset: _Optional[int] = ..., observed_size_bytes: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...

class DownloadExecutionFileRequest(_message.Message):
    __slots__ = ("execution_id", "relative_path")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    relative_path: str
    def __init__(self, execution_id: _Optional[str] = ..., relative_path: _Optional[str] = ...) -> None: ...

class ExecutionFileDownloadFrame(_message.Message):
    __slots__ = ("metadata", "chunk", "complete")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    metadata: ExecutionFileDownloadMetadata
    chunk: ArtifactChunk
    complete: ArtifactCompletion
    def __init__(self, metadata: _Optional[_Union[ExecutionFileDownloadMetadata, _Mapping]] = ..., chunk: _Optional[_Union[ArtifactChunk, _Mapping]] = ..., complete: _Optional[_Union[ArtifactCompletion, _Mapping]] = ...) -> None: ...

class ExecutionFileDownloadMetadata(_message.Message):
    __slots__ = ("relative_path", "content_type", "kind", "state", "observed_size_bytes")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    content_type: str
    kind: ExecutionFileKind
    state: ExecutionFileState
    observed_size_bytes: int
    def __init__(self, relative_path: _Optional[str] = ..., content_type: _Optional[str] = ..., kind: _Optional[_Union[ExecutionFileKind, str]] = ..., state: _Optional[_Union[ExecutionFileState, str]] = ..., observed_size_bytes: _Optional[int] = ...) -> None: ...

class DownloadAppearanceAssetRequest(_message.Message):
    __slots__ = ("reference",)
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    reference: _appearance_pb2.AppearanceAssetReference
    def __init__(self, reference: _Optional[_Union[_appearance_pb2.AppearanceAssetReference, _Mapping]] = ...) -> None: ...

class AppearanceAssetFrame(_message.Message):
    __slots__ = ("metadata", "chunk", "complete")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHUNK_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    metadata: _appearance_pb2.AppearanceAssetMetadata
    chunk: ArtifactChunk
    complete: ArtifactCompletion
    def __init__(self, metadata: _Optional[_Union[_appearance_pb2.AppearanceAssetMetadata, _Mapping]] = ..., chunk: _Optional[_Union[ArtifactChunk, _Mapping]] = ..., complete: _Optional[_Union[ArtifactCompletion, _Mapping]] = ...) -> None: ...
