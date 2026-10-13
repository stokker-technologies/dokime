from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("scope", "camera", "audio", "maximum_seconds")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CAMERA_FIELD_NUMBER: _ClassVar[int]
    AUDIO_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_SECONDS_FIELD_NUMBER: _ClassVar[int]
    scope: Row4
    camera: int
    audio: Row5
    maximum_seconds: int
    def __init__(self, scope: _Optional[_Union[Row4, _Mapping]] = ..., camera: _Optional[int] = ..., audio: _Optional[_Union[Row5, _Mapping]] = ..., maximum_seconds: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row6
    def __init__(self, payload: _Optional[_Union[Row6, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("cycle_id", "case_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("microphone", "system")
    MICROPHONE_FIELD_NUMBER: _ClassVar[int]
    SYSTEM_FIELD_NUMBER: _ClassVar[int]
    microphone: bool
    system: bool
    def __init__(self, microphone: _Optional[bool] = ..., system: _Optional[bool] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Progress", "Completed")
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    Progress: Row7
    Completed: Row8
    def __init__(self, Progress: _Optional[_Union[Row7, _Mapping]] = ..., Completed: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("elapsed_ms", "size_bytes")
    ELAPSED_MS_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    elapsed_ms: int
    size_bytes: int
    def __init__(self, elapsed_ms: _Optional[int] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
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
    scope: Row4
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    state: Row11
    created_at: Row12
    def __init__(self, artifact_id: _Optional[str] = ..., upload_id: _Optional[str] = ..., scope: _Optional[_Union[Row4, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ..., state: _Optional[_Union[Row11, _Mapping]] = ..., created_at: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Unspecified", "Complete", "Cancelled", "Rejected")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row13
    Complete: Row14
    Cancelled: Row15
    Rejected: Row16
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., Complete: _Optional[_Union[Row14, _Mapping]] = ..., Cancelled: _Optional[_Union[Row15, _Mapping]] = ..., Rejected: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response", "output")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    output: Row3
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ..., output: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...
