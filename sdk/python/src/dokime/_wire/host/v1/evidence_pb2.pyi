from dokime._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import file_pb2 as _file_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class EvidenceStatisticsRequest(_message.Message):
    __slots__ = ("scope", "stream", "versions", "between", "outputs", "field", "step", "at")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.EvidenceScope
    stream: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    between: _evidence_pb2.Window
    outputs: _containers.RepeatedCompositeFieldContainer[Output]
    field: str
    step: str
    at: _position_pb2.ReadCut
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., stream: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., between: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., outputs: _Optional[_Iterable[_Union[Output, _Mapping]]] = ..., field: _Optional[str] = ..., step: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class EvidenceStatisticsResponse(_message.Message):
    __slots__ = ("statistics", "resolved_at")
    STATISTICS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    statistics: Statistics
    resolved_at: _position_pb2.ReadCut
    def __init__(self, statistics: _Optional[_Union[Statistics, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class EvidenceArchiveRequest(_message.Message):
    __slots__ = ("subject", "at")
    SUBJECT_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    subject: EvidenceSubject
    at: _position_pb2.ReadCut
    def __init__(self, subject: _Optional[_Union[EvidenceSubject, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class EvidenceArchiveResponse(_message.Message):
    __slots__ = ("file", "resolved_at")
    FILE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    resolved_at: _position_pb2.ReadCut
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class Statistics(_message.Message):
    __slots__ = ("count", "min", "max", "mean", "stddev", "median", "p05", "p95", "p99", "first", "last", "sum")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    MIN_FIELD_NUMBER: _ClassVar[int]
    MAX_FIELD_NUMBER: _ClassVar[int]
    MEAN_FIELD_NUMBER: _ClassVar[int]
    STDDEV_FIELD_NUMBER: _ClassVar[int]
    MEDIAN_FIELD_NUMBER: _ClassVar[int]
    P05_FIELD_NUMBER: _ClassVar[int]
    P95_FIELD_NUMBER: _ClassVar[int]
    P99_FIELD_NUMBER: _ClassVar[int]
    FIRST_FIELD_NUMBER: _ClassVar[int]
    LAST_FIELD_NUMBER: _ClassVar[int]
    SUM_FIELD_NUMBER: _ClassVar[int]
    count: int
    min: _scalar_pb2.Value
    max: _scalar_pb2.Value
    mean: _scalar_pb2.Value
    stddev: float
    median: _scalar_pb2.Value
    p05: _scalar_pb2.Value
    p95: _scalar_pb2.Value
    p99: _scalar_pb2.Value
    first: _scalar_pb2.Value
    last: _scalar_pb2.Value
    sum: _scalar_pb2.Value
    def __init__(self, count: _Optional[int] = ..., min: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., max: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., mean: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., stddev: _Optional[float] = ..., median: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., p05: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., p95: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., p99: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., first: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., last: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., sum: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...

class EvidenceSubject(_message.Message):
    __slots__ = ("execution", "cycle")
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    execution: str
    cycle: str
    def __init__(self, execution: _Optional[str] = ..., cycle: _Optional[str] = ...) -> None: ...

class Output(_message.Message):
    __slots__ = ("log", "artifact")
    LOG_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    log: _scalar_pb2.Null
    artifact: str
    def __init__(self, log: _Optional[_Union[_scalar_pb2.Null, _Mapping]] = ..., artifact: _Optional[str] = ...) -> None: ...
