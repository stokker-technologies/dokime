from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

import datetime

from dokime_sdk._wire.host.v1 import record_schemas_pb2 as _record_schemas_pb2
from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._wire.model.v1 import expressions_pb2 as _expressions_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class SamplesListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SIGNALS_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    BUCKET_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.EvidenceScope
    streams: _containers.RepeatedScalarFieldContainer[str]
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedScalarFieldContainer[str]
    sources: _containers.RepeatedScalarFieldContainer[str]
    signals: _containers.RepeatedScalarFieldContainer[str]
    between: _evidence_pb2.Window
    bucket: BucketRequest
    latest: bool
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., streams: _Optional[_Iterable[str]] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[str]] = ..., sources: _Optional[_Iterable[str]] = ..., signals: _Optional[_Iterable[str]] = ..., between: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., bucket: _Optional[_Union[BucketRequest, _Mapping]] = ..., latest: _Optional[bool] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class SamplesListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SAMPLES_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    samples: _containers.RepeatedCompositeFieldContainer[SampleRow]
    schema: _record_schemas_pb2.Schema
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, samples: _Optional[_Iterable[_Union[SampleRow, _Mapping]]] = ..., schema: _Optional[_Union[_record_schemas_pb2.Schema, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

@final
class SamplesWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SIGNALS_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    BUCKET_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.EvidenceScope
    streams: _containers.RepeatedScalarFieldContainer[str]
    versions: _containers.RepeatedScalarFieldContainer[int]
    fields: _containers.RepeatedScalarFieldContainer[str]
    sources: _containers.RepeatedScalarFieldContainer[str]
    signals: _containers.RepeatedScalarFieldContainer[str]
    between: _evidence_pb2.Window
    bucket: BucketRequest
    latest: bool
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., streams: _Optional[_Iterable[str]] = ..., versions: _Optional[_Iterable[int]] = ..., fields: _Optional[_Iterable[str]] = ..., sources: _Optional[_Iterable[str]] = ..., signals: _Optional[_Iterable[str]] = ..., between: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., bucket: _Optional[_Union[BucketRequest, _Mapping]] = ..., latest: _Optional[bool] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class SamplesWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SAMPLES_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    samples: _containers.RepeatedCompositeFieldContainer[SampleRow]
    schema: _record_schemas_pb2.Schema
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, samples: _Optional[_Iterable[_Union[SampleRow, _Mapping]]] = ..., schema: _Optional[_Union[_record_schemas_pb2.Schema, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

@final
class Bucket(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_COUNT_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    start: _timestamp_pb2.Timestamp
    end: _timestamp_pb2.Timestamp
    execution_id: str
    stream: str
    source: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    sample_count: int
    fields: _containers.RepeatedCompositeFieldContainer[BucketValue]
    def __init__(self, start: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., end: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., execution_id: _Optional[str] = ..., stream: _Optional[str] = ..., source: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., sample_count: _Optional[int] = ..., fields: _Optional[_Iterable[_Union[BucketValue, _Mapping]]] = ...) -> None: ...

@final
class BucketRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    EVERY_NANOS_FIELD_NUMBER: _ClassVar[int]
    AGGREGATE_FIELD_NUMBER: _ClassVar[int]
    every_nanos: int
    aggregate: _expressions_pb2.Aggregate
    def __init__(self, every_nanos: _Optional[int] = ..., aggregate: _Optional[_Union[_expressions_pb2.Aggregate, str]] = ...) -> None: ...

@final
class BucketValue(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    FIELD_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    REDUCER_FIELD_NUMBER: _ClassVar[int]
    field: str
    count: int
    value: StatisticValue
    unit: str
    reducer: _expressions_pb2.Aggregate
    def __init__(self, field: _Optional[str] = ..., count: _Optional[int] = ..., value: _Optional[_Union[StatisticValue, _Mapping]] = ..., unit: _Optional[str] = ..., reducer: _Optional[_Union[_expressions_pb2.Aggregate, str]] = ...) -> None: ...

@final
class SampleRow(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    RAW_FIELD_NUMBER: _ClassVar[int]
    BUCKET_FIELD_NUMBER: _ClassVar[int]
    raw: TelemetrySample
    bucket: Bucket
    def __init__(self, raw: _Optional[_Union[TelemetrySample, _Mapping]] = ..., bucket: _Optional[_Union[Bucket, _Mapping]] = ...) -> None: ...

@final
class StatisticValue(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    NUMBER_FIELD_NUMBER: _ClassVar[int]
    count: int
    number: float
    def __init__(self, count: _Optional[int] = ..., number: _Optional[float] = ...) -> None: ...

@final
class TelemetrySample(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_ID_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    RECEIVED_AT_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    payload: bytes
    sample_id: str
    stream: str
    schema_version: int
    sequence: int
    timestamp: _timestamp_pb2.Timestamp
    received_at: _timestamp_pb2.Timestamp
    source: str
    scope: _evidence_pb2.EvidenceScope
    values: _scalar_pb2.MapValue
    summary: _scalar_pb2.MapValue
    metadata: _scalar_pb2.MapValue
    def __init__(self, payload: _Optional[bytes] = ..., sample_id: _Optional[str] = ..., stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., received_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., source: _Optional[str] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., values: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ..., summary: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ..., metadata: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ...) -> None: ...
