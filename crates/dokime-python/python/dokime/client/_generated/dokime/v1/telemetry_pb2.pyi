import datetime

from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ListTelemetrySamplesRequest(_message.Message):
    __slots__ = ("streams", "limit", "schema_versions", "sources", "signals", "scope", "after_stream_cursors", "start", "end", "latest")
    class AfterStreamCursorsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: int
        def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSIONS_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SIGNALS_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AFTER_STREAM_CURSORS_FIELD_NUMBER: _ClassVar[int]
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    LATEST_FIELD_NUMBER: _ClassVar[int]
    streams: _containers.RepeatedScalarFieldContainer[str]
    limit: int
    schema_versions: _containers.RepeatedScalarFieldContainer[int]
    sources: _containers.RepeatedScalarFieldContainer[str]
    signals: _containers.RepeatedScalarFieldContainer[str]
    scope: _common_pb2.EvidenceScope
    after_stream_cursors: _containers.ScalarMap[str, int]
    start: _timestamp_pb2.Timestamp
    end: _timestamp_pb2.Timestamp
    latest: bool
    def __init__(self, streams: _Optional[_Iterable[str]] = ..., limit: _Optional[int] = ..., schema_versions: _Optional[_Iterable[int]] = ..., sources: _Optional[_Iterable[str]] = ..., signals: _Optional[_Iterable[str]] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., after_stream_cursors: _Optional[_Mapping[str, int]] = ..., start: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., end: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., latest: _Optional[bool] = ...) -> None: ...

class TelemetrySamplesReply(_message.Message):
    __slots__ = ("samples", "through_stream_cursors", "complete")
    class ThroughStreamCursorsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: int
        def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...
    SAMPLES_FIELD_NUMBER: _ClassVar[int]
    THROUGH_STREAM_CURSORS_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    samples: _containers.RepeatedCompositeFieldContainer[TelemetrySample]
    through_stream_cursors: _containers.ScalarMap[str, int]
    complete: bool
    def __init__(self, samples: _Optional[_Iterable[_Union[TelemetrySample, _Mapping]]] = ..., through_stream_cursors: _Optional[_Mapping[str, int]] = ..., complete: _Optional[bool] = ...) -> None: ...

class TelemetrySample(_message.Message):
    __slots__ = ("sample_id", "stream", "schema_version", "sequence", "timestamp", "received_at", "source", "scope", "values", "summary", "metadata")
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
    sample_id: str
    stream: str
    schema_version: int
    sequence: int
    timestamp: _timestamp_pb2.Timestamp
    received_at: _timestamp_pb2.Timestamp
    source: str
    scope: _common_pb2.EvidenceScope
    values: _value_pb2.ValueObject
    summary: _value_pb2.ValueObject
    metadata: _value_pb2.ValueObject
    def __init__(self, sample_id: _Optional[str] = ..., stream: _Optional[str] = ..., schema_version: _Optional[int] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., received_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., source: _Optional[str] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., values: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., summary: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., metadata: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ...) -> None: ...
