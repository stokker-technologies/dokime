from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("execution_id", "source", "rules", "output", "policy", "selection", "wait_for_source_seconds", "poll_interval_ms", "maximum_seconds", "maximum_files", "maximum_bytes")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    RULES_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    POLICY_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    WAIT_FOR_SOURCE_SECONDS_FIELD_NUMBER: _ClassVar[int]
    POLL_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_SECONDS_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_FILES_FIELD_NUMBER: _ClassVar[int]
    MAXIMUM_BYTES_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    source: Row4
    rules: _containers.RepeatedCompositeFieldContainer[Row5]
    output: str
    policy: Row6
    selection: Row7
    wait_for_source_seconds: int
    poll_interval_ms: int
    maximum_seconds: int
    maximum_files: int
    maximum_bytes: int
    def __init__(self, execution_id: _Optional[str] = ..., source: _Optional[_Union[Row4, _Mapping]] = ..., rules: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., output: _Optional[str] = ..., policy: _Optional[_Union[Row6, _Mapping]] = ..., selection: _Optional[_Union[Row7, _Mapping]] = ..., wait_for_source_seconds: _Optional[int] = ..., poll_interval_ms: _Optional[int] = ..., maximum_seconds: _Optional[int] = ..., maximum_files: _Optional[int] = ..., maximum_bytes: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row3(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row8
    def __init__(self, payload: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("payload",)
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    payload: Row9
    def __init__(self, payload: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("glob", "action")
    GLOB_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    glob: str
    action: Row10
    def __init__(self, glob: _Optional[str] = ..., action: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("Unspecified", "Stream", "Mirror", "SnapshotAtStart", "SnapshotAtStop")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    MIRROR_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOTATSTART_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOTATSTOP_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row11
    Stream: Row12
    Mirror: Row13
    SnapshotAtStart: Row14
    SnapshotAtStop: Row15
    def __init__(self, Unspecified: _Optional[_Union[Row11, _Mapping]] = ..., Stream: _Optional[_Union[Row12, _Mapping]] = ..., Mirror: _Optional[_Union[Row13, _Mapping]] = ..., SnapshotAtStart: _Optional[_Union[Row14, _Mapping]] = ..., SnapshotAtStop: _Optional[_Union[Row15, _Mapping]] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("All", "Latest", "New")
    ALL_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    NEW_FIELD_NUMBER: _ClassVar[int]
    All: Row16
    Latest: Row17
    New: Row18
    def __init__(self, All: _Optional[_Union[Row16, _Mapping]] = ..., Latest: _Optional[_Union[Row17, _Mapping]] = ..., New: _Optional[_Union[Row18, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Progress", "Gap", "Sealed", "Completed", "Armed")
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    GAP_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    ARMED_FIELD_NUMBER: _ClassVar[int]
    Progress: Row19
    Gap: Row20
    Sealed: Row21
    Completed: Row22
    Armed: Row23
    def __init__(self, Progress: _Optional[_Union[Row19, _Mapping]] = ..., Gap: _Optional[_Union[Row20, _Mapping]] = ..., Sealed: _Optional[_Union[Row21, _Mapping]] = ..., Completed: _Optional[_Union[Row22, _Mapping]] = ..., Armed: _Optional[_Union[Row23, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("Local", "Ftp")
    LOCAL_FIELD_NUMBER: _ClassVar[int]
    FTP_FIELD_NUMBER: _ClassVar[int]
    Local: Row24
    Ftp: Row25
    def __init__(self, Local: _Optional[_Union[Row24, _Mapping]] = ..., Ftp: _Optional[_Union[Row25, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Unspecified", "Include", "Exclude")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_FIELD_NUMBER: _ClassVar[int]
    EXCLUDE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row26
    Include: Row27
    Exclude: Row28
    def __init__(self, Unspecified: _Optional[_Union[Row26, _Mapping]] = ..., Include: _Optional[_Union[Row27, _Mapping]] = ..., Exclude: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

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
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row19(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row29
    def __init__(self, field_0: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row20(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row30
    def __init__(self, field_0: _Optional[_Union[Row30, _Mapping]] = ...) -> None: ...

class Row21(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row31
    def __init__(self, field_0: _Optional[_Union[Row31, _Mapping]] = ...) -> None: ...

class Row22(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row32
    def __init__(self, field_0: _Optional[_Union[Row32, _Mapping]] = ...) -> None: ...

class Row23(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row33
    def __init__(self, field_0: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row24(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row34
    def __init__(self, field_0: _Optional[_Union[Row34, _Mapping]] = ...) -> None: ...

class Row25(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row35
    def __init__(self, field_0: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row26(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row29(_message.Message):
    __slots__ = ("path", "offset", "phase")
    PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    path: str
    offset: int
    phase: Row36
    def __init__(self, path: _Optional[str] = ..., offset: _Optional[int] = ..., phase: _Optional[_Union[Row36, _Mapping]] = ...) -> None: ...

class Row30(_message.Message):
    __slots__ = ("path", "offset", "reason")
    PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    path: str
    offset: int
    reason: Row37
    def __init__(self, path: _Optional[str] = ..., offset: _Optional[int] = ..., reason: _Optional[_Union[Row37, _Mapping]] = ...) -> None: ...

class Row31(_message.Message):
    __slots__ = ("path", "artifact")
    PATH_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    path: str
    artifact: Row38
    def __init__(self, path: _Optional[str] = ..., artifact: _Optional[_Union[Row38, _Mapping]] = ...) -> None: ...

class Row32(_message.Message):
    __slots__ = ("artifacts",)
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[Row38]
    def __init__(self, artifacts: _Optional[_Iterable[_Union[Row38, _Mapping]]] = ...) -> None: ...

class Row33(_message.Message):
    __slots__ = ("baseline",)
    BASELINE_FIELD_NUMBER: _ClassVar[int]
    baseline: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, baseline: _Optional[_Iterable[str]] = ...) -> None: ...

class Row34(_message.Message):
    __slots__ = ("root",)
    ROOT_FIELD_NUMBER: _ClassVar[int]
    root: str
    def __init__(self, root: _Optional[str] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("url", "secret_id")
    URL_FIELD_NUMBER: _ClassVar[int]
    SECRET_ID_FIELD_NUMBER: _ClassVar[int]
    url: str
    secret_id: str
    def __init__(self, url: _Optional[str] = ..., secret_id: _Optional[str] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ("Unspecified", "Baseline", "Capturing", "Synced")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    BASELINE_FIELD_NUMBER: _ClassVar[int]
    CAPTURING_FIELD_NUMBER: _ClassVar[int]
    SYNCED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row39
    Baseline: Row40
    Capturing: Row41
    Synced: Row42
    def __init__(self, Unspecified: _Optional[_Union[Row39, _Mapping]] = ..., Baseline: _Optional[_Union[Row40, _Mapping]] = ..., Capturing: _Optional[_Union[Row41, _Mapping]] = ..., Synced: _Optional[_Union[Row42, _Mapping]] = ...) -> None: ...

class Row37(_message.Message):
    __slots__ = ("Unspecified", "Truncated", "Replaced", "Vanished", "ResumeUnsupported")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    TRUNCATED_FIELD_NUMBER: _ClassVar[int]
    REPLACED_FIELD_NUMBER: _ClassVar[int]
    VANISHED_FIELD_NUMBER: _ClassVar[int]
    RESUMEUNSUPPORTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row43
    Truncated: Row44
    Replaced: Row45
    Vanished: Row46
    ResumeUnsupported: Row47
    def __init__(self, Unspecified: _Optional[_Union[Row43, _Mapping]] = ..., Truncated: _Optional[_Union[Row44, _Mapping]] = ..., Replaced: _Optional[_Union[Row45, _Mapping]] = ..., Vanished: _Optional[_Union[Row46, _Mapping]] = ..., ResumeUnsupported: _Optional[_Union[Row47, _Mapping]] = ...) -> None: ...

class Row38(_message.Message):
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
    scope: Row48
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    state: Row49
    created_at: Row50
    def __init__(self, artifact_id: _Optional[str] = ..., upload_id: _Optional[str] = ..., scope: _Optional[_Union[Row48, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ..., state: _Optional[_Union[Row49, _Mapping]] = ..., created_at: _Optional[_Union[Row50, _Mapping]] = ...) -> None: ...

class Row39(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row43(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ("cycle_id", "case_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class Row49(_message.Message):
    __slots__ = ("Unspecified", "Complete", "Cancelled", "Rejected")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row51
    Complete: Row52
    Cancelled: Row53
    Rejected: Row54
    def __init__(self, Unspecified: _Optional[_Union[Row51, _Mapping]] = ..., Complete: _Optional[_Union[Row52, _Mapping]] = ..., Cancelled: _Optional[_Union[Row53, _Mapping]] = ..., Rejected: _Optional[_Union[Row54, _Mapping]] = ...) -> None: ...

class Row50(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row53(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row54(_message.Message):
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
