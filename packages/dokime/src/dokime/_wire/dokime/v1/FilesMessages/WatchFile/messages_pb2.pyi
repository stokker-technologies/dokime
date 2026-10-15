from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "source", "output", "artifact", "policy", "poll_interval_ms", "maximum_seconds", "maximum_bytes", "matchers", "markers")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    POLICY_FIELD_NUMBER: _ClassVar[int]
    POLL_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_SECONDS_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_BYTES_FIELD_NUMBER: _ClassVar[int]
    MATCHERS_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    source: Row3
    output: str
    artifact: str
    policy: Row4
    poll_interval_ms: int
    maximum_seconds: int
    maximum_bytes: int
    matchers: _containers.RepeatedCompositeFieldContainer[Row5]
    markers: _containers.RepeatedCompositeFieldContainer[Row6]
    def __init__(self, execution_id: _Optional[str] = ..., source: _Optional[_Union[Row3, _Mapping]] = ..., output: _Optional[str] = ..., artifact: _Optional[str] = ..., policy: _Optional[_Union[Row4, _Mapping]] = ..., poll_interval_ms: _Optional[int] = ..., maximum_seconds: _Optional[int] = ..., maximum_bytes: _Optional[int] = ..., matchers: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., markers: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("artifacts", "gaps")
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    GAPS_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[Row7]
    gaps: _containers.RepeatedCompositeFieldContainer[Row8]
    def __init__(self, artifacts: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., gaps: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row9
    def __init__(self, payload: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("Unspecified", "Stream", "Mirror", "SnapshotAtStart", "SnapshotAtStop")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    MIRROR_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOTATSTART_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOTATSTOP_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row10
    Stream: Row11
    Mirror: Row12
    SnapshotAtStart: Row13
    SnapshotAtStop: Row14
    def __init__(self, Unspecified: _Optional[_Union[Row10, _Mapping]] = ..., Stream: _Optional[_Union[Row11, _Mapping]] = ..., Mirror: _Optional[_Union[Row12, _Mapping]] = ..., SnapshotAtStart: _Optional[_Union[Row13, _Mapping]] = ..., SnapshotAtStop: _Optional[_Union[Row14, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("pattern", "stream", "fields")
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    stream: str
    fields: _containers.RepeatedCompositeFieldContainer[Row15]
    def __init__(self, pattern: _Optional[str] = ..., stream: _Optional[str] = ..., fields: _Optional[_Iterable[_Union[Row15, _Mapping]]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("pattern", "record_type")
    PATTERN_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    pattern: str
    record_type: str
    def __init__(self, pattern: _Optional[str] = ..., record_type: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("artifact", "size_bytes")
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    artifact: str
    size_bytes: int
    def __init__(self, artifact: _Optional[str] = ..., size_bytes: _Optional[int] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("path", "offset", "reason")
    PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    path: str
    offset: int
    reason: Row16
    def __init__(self, path: _Optional[str] = ..., offset: _Optional[int] = ..., reason: _Optional[_Union[Row16, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("Local", "Ftp")
    LOCAL_FIELD_NUMBER: _ClassVar[int]
    FTP_FIELD_NUMBER: _ClassVar[int]
    Local: Row17
    Ftp: Row18
    def __init__(self, Local: _Optional[_Union[Row17, _Mapping]] = ..., Ftp: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row11(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row12(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ("name", "kind")
    NAME_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    name: str
    kind: Row19
    def __init__(self, name: _Optional[str] = ..., kind: _Optional[_Union[Row19, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Unspecified", "Truncated", "Replaced", "Vanished", "ResumeUnsupported")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRUNCATED_FIELD_NUMBER: _ClassVar[int]
    REPLACED_FIELD_NUMBER: _ClassVar[int]
    VANISHED_FIELD_NUMBER: _ClassVar[int]
    RESUMEUNSUPPORTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row20
    Truncated: Row21
    Replaced: Row22
    Vanished: Row23
    ResumeUnsupported: Row24
    def __init__(self, Unspecified: _Optional[_Union[Row20, _Mapping]] = ..., Truncated: _Optional[_Union[Row21, _Mapping]] = ..., Replaced: _Optional[_Union[Row22, _Mapping]] = ..., Vanished: _Optional[_Union[Row23, _Mapping]] = ..., ResumeUnsupported: _Optional[_Union[Row24, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row25
    def __init__(self, field_0: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row18(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row26
    def __init__(self, field_0: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row19(_message.Message):
    __slots__ = ("Unspecified", "Text", "Integer", "Number", "Boolean")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    INTEGER_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    BOOLEAN_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row27
    Text: Row28
    Integer: Row29
    Number: Row30
    Boolean: Row31
    def __init__(self, Unspecified: _Optional[_Union[Row27, _Mapping]] = ..., Text: _Optional[_Union[Row28, _Mapping]] = ..., Integer: _Optional[_Union[Row29, _Mapping]] = ..., Number: _Optional[_Union[Row30, _Mapping]] = ..., Boolean: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row23(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row24(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row25(_message.Message):
    __slots__ = ("root",)
    ROOT_FIELD_NUMBER: _ClassVar[int]
    root: str
    def __init__(self, root: _Optional[str] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ("url", "credential_id")
    URL_FIELD_NUMBER: _ClassVar[int]
    CREDENTIAL_ID_FIELD_NUMBER: _ClassVar[int]
    url: str
    credential_id: str
    def __init__(self, url: _Optional[str] = ..., credential_id: _Optional[str] = ...) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row29(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
