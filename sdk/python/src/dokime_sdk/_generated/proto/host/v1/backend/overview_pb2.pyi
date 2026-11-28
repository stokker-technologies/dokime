from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.host.v1 import cycles_pb2 as _cycles_pb2
from dokime_sdk._generated.proto.model.v1 import metadata_pb2 as _metadata_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import failure_pb2 as _failure_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class OverviewGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class OverviewGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    OVERVIEW_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    overview: RuntimeShellSnapshot
    resolved_at: _position_pb2.ReadCut
    def __init__(self, overview: _Optional[_Union[RuntimeShellSnapshot, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class OverviewWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class OverviewWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    OVERVIEW_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    overview: RuntimeShellSnapshot
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, overview: _Optional[_Union[RuntimeShellSnapshot, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

@final
class EvidenceCounts(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
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

@final
class RuntimeShellSnapshot(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
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
    hub_metadata: _metadata_pb2.Metadata
    test_suite_count: int
    cycle_summaries: _containers.RepeatedCompositeFieldContainer[_cycles_pb2.CycleSummary]
    evidence_counts: EvidenceCounts
    observed_cursor: _position_pb2.ReadCut
    next_after_cycle_id: str
    def __init__(self, session_id: _Optional[str] = ..., revision: _Optional[int] = ..., shutting_down: _Optional[bool] = ..., hub_metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., test_suite_count: _Optional[int] = ..., cycle_summaries: _Optional[_Iterable[_Union[_cycles_pb2.CycleSummary, _Mapping]]] = ..., evidence_counts: _Optional[_Union[EvidenceCounts, _Mapping]] = ..., observed_cursor: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_after_cycle_id: _Optional[str] = ...) -> None: ...
