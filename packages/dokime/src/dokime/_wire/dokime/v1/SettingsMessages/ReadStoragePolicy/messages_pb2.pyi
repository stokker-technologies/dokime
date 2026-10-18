from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row2(_message.Message):
    __slots__ = ("revision", "configured", "effective", "restart_required")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    CONFIGURED_FIELD_NUMBER: _ClassVar[int]
    EFFECTIVE_FIELD_NUMBER: _ClassVar[int]
    RESTART_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    revision: int
    configured: Row3
    effective: Row3
    restart_required: bool
    def __init__(self, revision: _Optional[int] = ..., configured: _Optional[_Union[Row3, _Mapping]] = ..., effective: _Optional[_Union[Row3, _Mapping]] = ..., restart_required: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("compaction_interval_ms", "local_retention_ms")
    COMPACTION_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    LOCAL_RETENTION_MS_FIELD_NUMBER: _ClassVar[int]
    compaction_interval_ms: int
    local_retention_ms: int
    def __init__(self, compaction_interval_ms: _Optional[int] = ..., local_retention_ms: _Optional[int] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
