from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import file_pb2 as _file_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from harmos._generated.proto.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class UploadsCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    metadata: _file_pb2.FileMetadata
    request_id: str
    def __init__(self, metadata: _Optional[_Union[_file_pb2.FileMetadata, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class UploadsCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    CHUNK_BYTES_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_BYTES_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    receipt: _receipt_pb2.Receipt
    chunk_bytes: int
    message_bytes: int
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ..., chunk_bytes: _Optional[int] = ..., message_bytes: _Optional[int] = ...) -> None: ...

@final
class UploadsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    at: _position_pb2.ReadCut
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class UploadsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    CHUNK_BYTES_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_BYTES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    chunk_bytes: int
    message_bytes: int
    resolved_at: _position_pb2.ReadCut
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., chunk_bytes: _Optional[int] = ..., message_bytes: _Optional[int] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class UploadsResumeRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    request_id: str
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class UploadsResumeResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    CHUNK_BYTES_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_BYTES_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    receipt: _receipt_pb2.Receipt
    chunk_bytes: int
    message_bytes: int
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ..., chunk_bytes: _Optional[int] = ..., message_bytes: _Optional[int] = ...) -> None: ...

@final
class UploadsAppendRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    offset: int
    data: bytes
    request_id: str
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., offset: _Optional[int] = ..., data: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class UploadsAppendResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    receipt: _receipt_pb2.Receipt
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class UploadsPublishRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    LENGTH_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    length: int
    sha256: bytes
    request_id: str
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., length: _Optional[int] = ..., sha256: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class UploadsPublishResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.FileReference
    receipt: _receipt_pb2.Receipt
    def __init__(self, upload: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class UploadsDeleteRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    request_id: str
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class UploadsDeleteResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    UPLOAD_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    upload: _file_pb2.Upload
    receipt: _receipt_pb2.Receipt
    def __init__(self, upload: _Optional[_Union[_file_pb2.Upload, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...
