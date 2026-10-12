import datetime

from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CapturedRecordSelection(_message.Message):
    __slots__ = ("scope", "record_types", "sources", "severities")
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPES_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SEVERITIES_FIELD_NUMBER: _ClassVar[int]
    scope: _common_pb2.EvidenceScope
    record_types: _containers.RepeatedScalarFieldContainer[str]
    sources: _containers.RepeatedScalarFieldContainer[str]
    severities: _containers.RepeatedScalarFieldContainer[_common_pb2.Severity]
    def __init__(self, scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., record_types: _Optional[_Iterable[str]] = ..., sources: _Optional[_Iterable[str]] = ..., severities: _Optional[_Iterable[_Union[_common_pb2.Severity, str]]] = ...) -> None: ...

class ListRecordsRequest(_message.Message):
    __slots__ = ("selection", "after", "through")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    THROUGH_FIELD_NUMBER: _ClassVar[int]
    selection: CapturedRecordSelection
    after: _common_pb2.Cursor
    through: _common_pb2.Cursor
    def __init__(self, selection: _Optional[_Union[CapturedRecordSelection, _Mapping]] = ..., after: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., through: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class WatchRecordsRequest(_message.Message):
    __slots__ = ("selection", "after")
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    selection: CapturedRecordSelection
    after: _common_pb2.Cursor
    def __init__(self, selection: _Optional[_Union[CapturedRecordSelection, _Mapping]] = ..., after: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ...) -> None: ...

class RecordPage(_message.Message):
    __slots__ = ("records", "scanned", "through", "replay_complete", "retained_after")
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    SCANNED_FIELD_NUMBER: _ClassVar[int]
    THROUGH_FIELD_NUMBER: _ClassVar[int]
    REPLAY_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    RETAINED_AFTER_FIELD_NUMBER: _ClassVar[int]
    records: _containers.RepeatedCompositeFieldContainer[CapturedRecord]
    scanned: _common_pb2.Cursor
    through: _common_pb2.Cursor
    replay_complete: bool
    retained_after: int
    def __init__(self, records: _Optional[_Iterable[_Union[CapturedRecord, _Mapping]]] = ..., scanned: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., through: _Optional[_Union[_common_pb2.Cursor, _Mapping]] = ..., replay_complete: _Optional[bool] = ..., retained_after: _Optional[int] = ...) -> None: ...

class CapturedRecord(_message.Message):
    __slots__ = ("record_id", "sequence", "timestamp", "record_type", "schema_version", "source", "severity", "message", "error", "details", "scope", "related_sample_id", "related_stream_id", "related_artifact_id", "received_at")
    RECORD_ID_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    DETAILS_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RELATED_SAMPLE_ID_FIELD_NUMBER: _ClassVar[int]
    RELATED_STREAM_ID_FIELD_NUMBER: _ClassVar[int]
    RELATED_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    RECEIVED_AT_FIELD_NUMBER: _ClassVar[int]
    record_id: str
    sequence: int
    timestamp: _timestamp_pb2.Timestamp
    record_type: str
    schema_version: int
    source: str
    severity: _common_pb2.Severity
    message: str
    error: CapturedRecordError
    details: _value_pb2.ValueObject
    scope: _common_pb2.EvidenceScope
    related_sample_id: str
    related_stream_id: str
    related_artifact_id: str
    received_at: _timestamp_pb2.Timestamp
    def __init__(self, record_id: _Optional[str] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., source: _Optional[str] = ..., severity: _Optional[_Union[_common_pb2.Severity, str]] = ..., message: _Optional[str] = ..., error: _Optional[_Union[CapturedRecordError, _Mapping]] = ..., details: _Optional[_Union[_value_pb2.ValueObject, _Mapping]] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., related_sample_id: _Optional[str] = ..., related_stream_id: _Optional[str] = ..., related_artifact_id: _Optional[str] = ..., received_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class CapturedRecordError(_message.Message):
    __slots__ = ("message", "code", "source", "cause_chain", "stacktrace_attachment", "trace_id", "correlation_id")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    CAUSE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    STACKTRACE_ATTACHMENT_FIELD_NUMBER: _ClassVar[int]
    TRACE_ID_FIELD_NUMBER: _ClassVar[int]
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    cause_chain: _containers.RepeatedCompositeFieldContainer[CapturedErrorCause]
    stacktrace_attachment: str
    trace_id: str
    correlation_id: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ..., cause_chain: _Optional[_Iterable[_Union[CapturedErrorCause, _Mapping]]] = ..., stacktrace_attachment: _Optional[str] = ..., trace_id: _Optional[str] = ..., correlation_id: _Optional[str] = ...) -> None: ...

class CapturedErrorCause(_message.Message):
    __slots__ = ("message", "code", "source")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ...) -> None: ...

class GetCapturedRecordSchemaRequest(_message.Message):
    __slots__ = ("record_type", "schema_version", "epoch")
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    record_type: str
    schema_version: int
    epoch: str
    def __init__(self, record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., epoch: _Optional[str] = ...) -> None: ...
