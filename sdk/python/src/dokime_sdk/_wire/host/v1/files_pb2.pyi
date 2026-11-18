from dokime_sdk._wire.model.v1 import files_pb2 as _files_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import file_pb2 as _file_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class FilesGetRequest(_message.Message):
    __slots__ = ("file", "at")
    FILE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    at: _position_pb2.ReadCut
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class FilesGetResponse(_message.Message):
    __slots__ = ("file", "chunk_bytes", "message_bytes", "resolved_at")
    FILE_FIELD_NUMBER: _ClassVar[int]
    CHUNK_BYTES_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_BYTES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    chunk_bytes: int
    message_bytes: int
    resolved_at: _position_pb2.ReadCut
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., chunk_bytes: _Optional[int] = ..., message_bytes: _Optional[int] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class FilesReadRequest(_message.Message):
    __slots__ = ("file", "offset", "max_bytes", "at")
    FILE_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    offset: int
    max_bytes: int
    at: _position_pb2.ReadCut
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., offset: _Optional[int] = ..., max_bytes: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class FilesReadResponse(_message.Message):
    __slots__ = ("data", "total_length", "sha256", "resolved_at")
    DATA_FIELD_NUMBER: _ClassVar[int]
    TOTAL_LENGTH_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    total_length: int
    sha256: bytes
    resolved_at: _position_pb2.ReadCut
    def __init__(self, data: _Optional[bytes] = ..., total_length: _Optional[int] = ..., sha256: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class FilesWatchRequest(_message.Message):
    __slots__ = ("file", "upload", "offset", "at")
    FILE_FIELD_NUMBER: _ClassVar[int]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    upload: _file_pb2.Upload
    offset: int
    at: _position_pb2.ReadCut
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., offset: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class FilesWatchResponse(_message.Message):
    __slots__ = ("data", "completion", "resolved_at")
    DATA_FIELD_NUMBER: _ClassVar[int]
    COMPLETION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    data: FileBatch
    completion: FileCompletion
    resolved_at: _position_pb2.ReadCut
    def __init__(self, data: _Optional[_Union[FileBatch, _Mapping]] = ..., completion: _Optional[_Union[FileCompletion, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class FilesDeleteRequest(_message.Message):
    __slots__ = ("file", "request_id")
    FILE_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    request_id: str
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

class FilesDeleteResponse(_message.Message):
    __slots__ = ("receipt",)
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class FilesCaptureRequest(_message.Message):
    __slots__ = ("source", "output")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    source: _files_pb2.FileSource
    output: str
    def __init__(self, source: _Optional[_Union[_files_pb2.FileSource, _Mapping]] = ..., output: _Optional[str] = ...) -> None: ...

class FilesCaptureResponse(_message.Message):
    __slots__ = ("artifacts", "gaps")
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    GAPS_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[_files_pb2.CapturedArtifact]
    gaps: _containers.RepeatedCompositeFieldContainer[_files_pb2.CaptureGap]
    def __init__(self, artifacts: _Optional[_Iterable[_Union[_files_pb2.CapturedArtifact, _Mapping]]] = ..., gaps: _Optional[_Iterable[_Union[_files_pb2.CaptureGap, _Mapping]]] = ...) -> None: ...

class FileBatch(_message.Message):
    __slots__ = ("offset", "bytes")
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    BYTES_FIELD_NUMBER: _ClassVar[int]
    offset: int
    bytes: bytes
    def __init__(self, offset: _Optional[int] = ..., bytes: _Optional[bytes] = ...) -> None: ...

class FileCompletion(_message.Message):
    __slots__ = ("bytes",)
    BYTES_FIELD_NUMBER: _ClassVar[int]
    bytes: int
    def __init__(self, bytes: _Optional[int] = ...) -> None: ...
