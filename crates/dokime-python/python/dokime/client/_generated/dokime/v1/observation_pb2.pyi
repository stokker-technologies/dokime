from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import execution_pb2 as _execution_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class WatchRuntimeRequest(_message.Message):
    __slots__ = ("cursor",)
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    cursor: _common_pb2.Cursor
    def __init__(self, cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class RuntimeStateFrame(_message.Message):
    __slots__ = ("cursor", "resynchronized", "shutting_down", "cycle_count")
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    RESYNCHRONIZED_FIELD_NUMBER: _ClassVar[int]
    SHUTTING_DOWN_FIELD_NUMBER: _ClassVar[int]
    CYCLE_COUNT_FIELD_NUMBER: _ClassVar[int]
    cursor: _common_pb2.Cursor
    resynchronized: bool
    shutting_down: bool
    cycle_count: int
    def __init__(self, cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., resynchronized: _Optional[bool] = ..., shutting_down: _Optional[bool] = ..., cycle_count: _Optional[int] = ...) -> None: ...

class WatchExecutionProgressRequest(_message.Message):
    __slots__ = ("test_execution_id",)
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    test_execution_id: str
    def __init__(self, test_execution_id: _Optional[str] = ...) -> None: ...

class ExecutionProgress(_message.Message):
    __slots__ = ("test_execution_id", "step_id", "phase", "completed", "total", "elapsed_ms", "remaining_ms")
    TEST_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    ELAPSED_MS_FIELD_NUMBER: _ClassVar[int]
    REMAINING_MS_FIELD_NUMBER: _ClassVar[int]
    test_execution_id: str
    step_id: str
    phase: str
    completed: int
    total: int
    elapsed_ms: int
    remaining_ms: int
    def __init__(self, test_execution_id: _Optional[str] = ..., step_id: _Optional[str] = ..., phase: _Optional[str] = ..., completed: _Optional[int] = ..., total: _Optional[int] = ..., elapsed_ms: _Optional[int] = ..., remaining_ms: _Optional[int] = ...) -> None: ...

class ExecutionProgressFrame(_message.Message):
    __slots__ = ("revision", "resynchronized", "progress", "cleared")
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESYNCHRONIZED_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    CLEARED_FIELD_NUMBER: _ClassVar[int]
    revision: int
    resynchronized: bool
    progress: ExecutionProgress
    cleared: bool
    def __init__(self, revision: _Optional[int] = ..., resynchronized: _Optional[bool] = ..., progress: _Optional[_Union[ExecutionProgress, _Mapping]] = ..., cleared: _Optional[bool] = ...) -> None: ...

class GetRuntimeShellRequest(_message.Message):
    __slots__ = ("limit", "after_cycle_id", "cursor")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_cycle_id: str
    cursor: _common_pb2.Cursor
    def __init__(self, limit: _Optional[int] = ..., after_cycle_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class RuntimeShellSnapshot(_message.Message):
    __slots__ = ("session_id", "revision", "shutting_down", "hub_metadata", "test_suite_count", "cycle_summaries", "evidence_counts", "observed_cursor", "next_after_cycle_id")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    SHUTTING_DOWN_FIELD_NUMBER: _ClassVar[int]
    HUB_METADATA_FIELD_NUMBER: _ClassVar[int]
    TEST_SUITE_COUNT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_SUMMARIES_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_COUNTS_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    revision: int
    shutting_down: bool
    hub_metadata: _common_pb2.Metadata
    test_suite_count: int
    cycle_summaries: _containers.RepeatedCompositeFieldContainer[_execution_pb2.CycleSummary]
    evidence_counts: EvidenceCounts
    observed_cursor: _common_pb2.Cursor
    next_after_cycle_id: str
    def __init__(self, session_id: _Optional[str] = ..., revision: _Optional[int] = ..., shutting_down: _Optional[bool] = ..., hub_metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., test_suite_count: _Optional[int] = ..., cycle_summaries: _Optional[_Iterable[_Union[_execution_pb2.CycleSummary, _Mapping]]] = ..., evidence_counts: _Optional[_Union[EvidenceCounts, _Mapping]] = ..., observed_cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., next_after_cycle_id: _Optional[str] = ...) -> None: ...

class EvidenceCounts(_message.Message):
    __slots__ = ("records", "telemetry", "prompts", "artifacts", "health", "traceability", "notes")
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_FIELD_NUMBER: _ClassVar[int]
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    records: int
    telemetry: int
    prompts: int
    artifacts: int
    health: int
    traceability: int
    notes: int
    def __init__(self, records: _Optional[int] = ..., telemetry: _Optional[int] = ..., prompts: _Optional[int] = ..., artifacts: _Optional[int] = ..., health: _Optional[int] = ..., traceability: _Optional[int] = ..., notes: _Optional[int] = ...) -> None: ...

class GetTraceabilityRequest(_message.Message):
    __slots__ = ("scope", "limit", "after_traceability_id", "cursor", "host_traceability_fingerprint")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    HOST_TRACEABILITY_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    scope: _common_pb2.EvidenceScope
    limit: int
    after_traceability_id: str
    cursor: _common_pb2.Cursor
    host_traceability_fingerprint: str
    def __init__(self, scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., limit: _Optional[int] = ..., after_traceability_id: _Optional[str] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., host_traceability_fingerprint: _Optional[str] = ...) -> None: ...

class TraceabilityPage(_message.Message):
    __slots__ = ("scope", "snapshots", "cursor", "next_after_traceability_id", "complete", "host_traceability_fingerprint")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOTS_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    HOST_TRACEABILITY_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    scope: _common_pb2.EvidenceScope
    snapshots: _containers.RepeatedCompositeFieldContainer[_common_pb2.TraceabilitySnapshot]
    cursor: _common_pb2.Cursor
    next_after_traceability_id: str
    complete: bool
    host_traceability_fingerprint: str
    def __init__(self, scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., snapshots: _Optional[_Iterable[_Union[_common_pb2.TraceabilitySnapshot, _Mapping]]] = ..., cursor: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., next_after_traceability_id: _Optional[str] = ..., complete: _Optional[bool] = ..., host_traceability_fingerprint: _Optional[str] = ...) -> None: ...
