from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("limit", "after_artifact_id", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_artifact_id: str
    cursor: Row3
    def __init__(self, limit: _Optional[int] = ..., after_artifact_id: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("artifacts", "next_after_artifact_id", "cursor")
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[Row4]
    next_after_artifact_id: str
    cursor: Row3
    def __init__(self, artifacts: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next_after_artifact_id: _Optional[str] = ..., cursor: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "metadata_sha256")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    METADATA_SHA256_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    metadata_sha256: bytes
    def __init__(self, epoch: _Optional[str] = ..., metadata_sha256: _Optional[bytes] = ...) -> None: ...

class Row4(_message.Message):
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
    scope: Row5
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    state: Row6
    created_at: Row7
    def __init__(self, artifact_id: _Optional[str] = ..., upload_id: _Optional[str] = ..., scope: _Optional[_Union[Row5, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ..., state: _Optional[_Union[Row6, _Mapping]] = ..., created_at: _Optional[_Union[Row7, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("cycle_id", "case_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "Complete", "Cancelled", "Rejected")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row8
    Complete: Row9
    Cancelled: Row10
    Rejected: Row11
    def __init__(self, Unspecified: _Optional[_Union[Row8, _Mapping]] = ..., Complete: _Optional[_Union[Row9, _Mapping]] = ..., Cancelled: _Optional[_Union[Row10, _Mapping]] = ..., Rejected: _Optional[_Union[Row11, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row9(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
