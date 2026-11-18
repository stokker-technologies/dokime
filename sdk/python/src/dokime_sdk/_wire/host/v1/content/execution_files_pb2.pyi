import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import file_pb2 as _file_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

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
EXECUTION_FILE_KIND_UNSPECIFIED: ExecutionFileKind
EXECUTION_FILE_KIND_TEXT: ExecutionFileKind
EXECUTION_FILE_KIND_BINARY: ExecutionFileKind
EXECUTION_FILE_KIND_MEDIA: ExecutionFileKind
EXECUTION_FILE_KIND_UNKNOWN: ExecutionFileKind
EXECUTION_FILE_STATE_UNSPECIFIED: ExecutionFileState
EXECUTION_FILE_STATE_STREAMING: ExecutionFileState
EXECUTION_FILE_STATE_FINALIZED: ExecutionFileState
EXECUTION_FILE_STATE_INTERRUPTED: ExecutionFileState

class ExecutionFilesGetRequest(_message.Message):
    __slots__ = ("parent", "name", "at")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    parent: str
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, parent: _Optional[str] = ..., name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class ExecutionFilesGetResponse(_message.Message):
    __slots__ = ("metadata", "file", "resolved_at")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    metadata: ExecutionFileMetadata
    file: _file_pb2.FileReference
    resolved_at: _position_pb2.ReadCut
    def __init__(self, metadata: _Optional[_Union[ExecutionFileMetadata, _Mapping]] = ..., file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class ExecutionFilesListRequest(_message.Message):
    __slots__ = ("parent", "at", "page")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    parent: str
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class ExecutionFilesListResponse(_message.Message):
    __slots__ = ("files", "resolved_at", "next_page_token")
    FILES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    files: _containers.RepeatedCompositeFieldContainer[ExecutionFileMetadata]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, files: _Optional[_Iterable[_Union[ExecutionFileMetadata, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

class ExecutionFilesWatchRequest(_message.Message):
    __slots__ = ("parent", "at", "page")
    PARENT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    parent: str
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, parent: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

class ExecutionFilesWatchResponse(_message.Message):
    __slots__ = ("files", "resolved_at", "opening", "resync_required")
    FILES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    files: _containers.RepeatedCompositeFieldContainer[ExecutionFileMetadata]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, files: _Optional[_Iterable[_Union[ExecutionFileMetadata, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

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
