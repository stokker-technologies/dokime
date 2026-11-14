import datetime

from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Severity(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SEVERITY_UNSPECIFIED: _ClassVar[Severity]
    SEVERITY_DEBUG: _ClassVar[Severity]
    SEVERITY_INFO: _ClassVar[Severity]
    SEVERITY_WARNING: _ClassVar[Severity]
    SEVERITY_ERROR: _ClassVar[Severity]

class StreamMatchOperator(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STREAM_MATCH_OPERATOR_UNSPECIFIED: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_EXISTS: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_EQ: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_GT: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_GE: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_LT: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_LE: _ClassVar[StreamMatchOperator]
    STREAM_MATCH_OPERATOR_WITHIN: _ClassVar[StreamMatchOperator]

class ArtifactState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ARTIFACT_STATE_UNSPECIFIED: _ClassVar[ArtifactState]
    ARTIFACT_STATE_COMPLETE: _ClassVar[ArtifactState]
    ARTIFACT_STATE_CANCELLED: _ClassVar[ArtifactState]
    ARTIFACT_STATE_REJECTED: _ClassVar[ArtifactState]
    ARTIFACT_STATE_FAILED: _ClassVar[ArtifactState]
SEVERITY_UNSPECIFIED: Severity
SEVERITY_DEBUG: Severity
SEVERITY_INFO: Severity
SEVERITY_WARNING: Severity
SEVERITY_ERROR: Severity
STREAM_MATCH_OPERATOR_UNSPECIFIED: StreamMatchOperator
STREAM_MATCH_OPERATOR_EXISTS: StreamMatchOperator
STREAM_MATCH_OPERATOR_EQ: StreamMatchOperator
STREAM_MATCH_OPERATOR_GT: StreamMatchOperator
STREAM_MATCH_OPERATOR_GE: StreamMatchOperator
STREAM_MATCH_OPERATOR_LT: StreamMatchOperator
STREAM_MATCH_OPERATOR_LE: StreamMatchOperator
STREAM_MATCH_OPERATOR_WITHIN: StreamMatchOperator
ARTIFACT_STATE_UNSPECIFIED: ArtifactState
ARTIFACT_STATE_COMPLETE: ArtifactState
ARTIFACT_STATE_CANCELLED: ArtifactState
ARTIFACT_STATE_REJECTED: ArtifactState
ARTIFACT_STATE_FAILED: ArtifactState

class EvidenceCondition(_message.Message):
    __slots__ = ("record", "stream", "all", "any")
    RECORD_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    ANY_FIELD_NUMBER: _ClassVar[int]
    record: RecordMatch
    stream: StreamMatch
    all: EvidenceConditions
    any: EvidenceConditions
    def __init__(self, record: _Optional[_Union[RecordMatch, _Mapping]] = ..., stream: _Optional[_Union[StreamMatch, _Mapping]] = ..., all: _Optional[_Union[EvidenceConditions, _Mapping]] = ..., any: _Optional[_Union[EvidenceConditions, _Mapping]] = ...) -> None: ...

class EvidenceConditions(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedCompositeFieldContainer[EvidenceCondition]
    def __init__(self, conditions: _Optional[_Iterable[_Union[EvidenceCondition, _Mapping]]] = ...) -> None: ...

class RecordMatch(_message.Message):
    __slots__ = ("kind", "fields")
    class FieldsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    KIND_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    kind: str
    fields: _containers.MessageMap[str, _scalar_pb2.Value]
    def __init__(self, kind: _Optional[str] = ..., fields: _Optional[_Mapping[str, _scalar_pb2.Value]] = ...) -> None: ...

class StreamMatch(_message.Message):
    __slots__ = ("stream", "field", "operator", "target", "tolerance")
    STREAM_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    stream: str
    field: str
    operator: StreamMatchOperator
    target: _scalar_pb2.Value
    tolerance: float
    def __init__(self, stream: _Optional[str] = ..., field: _Optional[str] = ..., operator: _Optional[_Union[StreamMatchOperator, str]] = ..., target: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., tolerance: _Optional[float] = ...) -> None: ...

class ArtifactMetadata(_message.Message):
    __slots__ = ("artifact_id", "upload_id", "scope", "filename", "content_type", "size_bytes", "sha256", "state", "created_at", "error")
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    UPLOAD_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    SIZE_BYTES_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    artifact_id: str
    upload_id: str
    scope: ArtifactScope
    filename: str
    content_type: str
    size_bytes: int
    sha256: str
    state: ArtifactState
    created_at: _timestamp_pb2.Timestamp
    error: str
    def __init__(self, artifact_id: _Optional[str] = ..., upload_id: _Optional[str] = ..., scope: _Optional[_Union[ArtifactScope, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., size_bytes: _Optional[int] = ..., sha256: _Optional[str] = ..., state: _Optional[_Union[ArtifactState, str]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., error: _Optional[str] = ...) -> None: ...

class ArtifactScope(_message.Message):
    __slots__ = ("cycle_id", "case_id", "execution_id")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    case_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class EvidenceOrigin(_message.Message):
    __slots__ = ("kind",)
    KIND_FIELD_NUMBER: _ClassVar[int]
    kind: EvidenceOriginKind
    def __init__(self, kind: _Optional[_Union[EvidenceOriginKind, _Mapping]] = ...) -> None: ...

class EvidenceOriginKind(_message.Message):
    __slots__ = ("default", "initialization", "step_attempt_id", "cleanup")
    DEFAULT_FIELD_NUMBER: _ClassVar[int]
    INITIALIZATION_FIELD_NUMBER: _ClassVar[int]
    STEP_ATTEMPT_ID_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    default: bool
    initialization: bool
    step_attempt_id: str
    cleanup: bool
    def __init__(self, default: _Optional[bool] = ..., initialization: _Optional[bool] = ..., step_attempt_id: _Optional[str] = ..., cleanup: _Optional[bool] = ...) -> None: ...

class EvidenceScope(_message.Message):
    __slots__ = ("session_id", "cycle_id", "case_id", "execution_id", "step_id", "criterion_id", "template_id", "suite_id", "origin")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    CRITERION_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    cycle_id: str
    case_id: str
    execution_id: str
    step_id: str
    criterion_id: str
    template_id: str
    suite_id: str
    origin: EvidenceOrigin
    def __init__(self, session_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., criterion_id: _Optional[str] = ..., template_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., origin: _Optional[_Union[EvidenceOrigin, _Mapping]] = ...) -> None: ...

class Window(_message.Message):
    __slots__ = ("start", "end")
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    start: _timestamp_pb2.Timestamp
    end: _timestamp_pb2.Timestamp
    def __init__(self, start: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., end: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...
