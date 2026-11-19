from google.protobuf.internal.python_message import GeneratedProtocolMessageType
from typing import final

import datetime

from dokime_sdk._wire.host.v1 import record_schemas_pb2 as _record_schemas_pb2
from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import page_pb2 as _page_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class RecordsListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: RecordScope
    record_type: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    between: _evidence_pb2.Window
    step: str
    selection: CapturedRecordSelection
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, scope: _Optional[_Union[RecordScope, _Mapping]] = ..., record_type: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., between: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., step: _Optional[str] = ..., selection: _Optional[_Union[CapturedRecordSelection, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class RecordsListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    LAST_FIELD_NUMBER: _ClassVar[int]
    records: _containers.RepeatedCompositeFieldContainer[CapturedRecord]
    schema: _record_schemas_pb2.Schema
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    count: int
    last: CapturedRecord
    def __init__(self, records: _Optional[_Iterable[_Union[CapturedRecord, _Mapping]]] = ..., schema: _Optional[_Union[_record_schemas_pb2.Schema, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ..., count: _Optional[int] = ..., last: _Optional[_Union[CapturedRecord, _Mapping]] = ...) -> None: ...

@final
class RecordsWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPE_FIELD_NUMBER: _ClassVar[int]
    VERSIONS_FIELD_NUMBER: _ClassVar[int]
    BETWEEN_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: RecordScope
    record_type: str
    versions: _containers.RepeatedScalarFieldContainer[int]
    between: _evidence_pb2.Window
    step: str
    selection: CapturedRecordSelection
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, scope: _Optional[_Union[RecordScope, _Mapping]] = ..., record_type: _Optional[str] = ..., versions: _Optional[_Iterable[int]] = ..., between: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., step: _Optional[str] = ..., selection: _Optional[_Union[CapturedRecordSelection, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class RecordsWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    records: _containers.RepeatedCompositeFieldContainer[CapturedRecord]
    schema: _record_schemas_pb2.Schema
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, records: _Optional[_Iterable[_Union[CapturedRecord, _Mapping]]] = ..., schema: _Optional[_Union[_record_schemas_pb2.Schema, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

@final
class RecordsAppendRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    SOURCE_AT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    execution: str
    kind: str
    payload: bytes
    source_at: _timestamp_pb2.Timestamp
    request_id: str
    def __init__(self, execution: _Optional[str] = ..., kind: _Optional[str] = ..., payload: _Optional[bytes] = ..., source_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class RecordsAppendResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    ordinal: int
    receipt: _receipt_pb2.Receipt
    def __init__(self, ordinal: _Optional[int] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class CapturedErrorCause(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ...) -> None: ...

@final
class CapturedRecord(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
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
    payload: bytes
    record_id: str
    sequence: int
    timestamp: _timestamp_pb2.Timestamp
    record_type: str
    schema_version: int
    source: str
    severity: _evidence_pb2.Severity
    message: str
    error: CapturedRecordError
    details: _scalar_pb2.Value
    scope: _evidence_pb2.EvidenceScope
    related_sample_id: str
    related_stream_id: str
    related_artifact_id: str
    received_at: _timestamp_pb2.Timestamp
    def __init__(self, payload: _Optional[bytes] = ..., record_id: _Optional[str] = ..., sequence: _Optional[int] = ..., timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., record_type: _Optional[str] = ..., schema_version: _Optional[int] = ..., source: _Optional[str] = ..., severity: _Optional[_Union[_evidence_pb2.Severity, str]] = ..., message: _Optional[str] = ..., error: _Optional[_Union[CapturedRecordError, _Mapping]] = ..., details: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., related_sample_id: _Optional[str] = ..., related_stream_id: _Optional[str] = ..., related_artifact_id: _Optional[str] = ..., received_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

@final
class CapturedRecordError(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CODE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    CAUSE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    STACKTRACE_ATTACHMENT_FIELD_NUMBER: _ClassVar[int]
    TRACE_ID_FIELD_NUMBER: _ClassVar[int]
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    DIAGNOSTICS_FIELD_NUMBER: _ClassVar[int]
    message: str
    code: str
    source: str
    cause_chain: _containers.RepeatedCompositeFieldContainer[CapturedErrorCause]
    stacktrace_attachment: str
    trace_id: str
    correlation_id: str
    diagnostics: str
    def __init__(self, message: _Optional[str] = ..., code: _Optional[str] = ..., source: _Optional[str] = ..., cause_chain: _Optional[_Iterable[_Union[CapturedErrorCause, _Mapping]]] = ..., stacktrace_attachment: _Optional[str] = ..., trace_id: _Optional[str] = ..., correlation_id: _Optional[str] = ..., diagnostics: _Optional[str] = ...) -> None: ...

@final
class CycleScope(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    execution_id: str
    def __init__(self, cycle_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

@final
class RecordScope(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    execution: str
    cycle: CycleScope
    def __init__(self, execution: _Optional[str] = ..., cycle: _Optional[_Union[CycleScope, _Mapping]] = ...) -> None: ...

@final
class CapturedRecordSelection(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__ = ()
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    RECORD_TYPES_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    SEVERITIES_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.EvidenceScope
    record_types: _containers.RepeatedScalarFieldContainer[str]
    sources: _containers.RepeatedScalarFieldContainer[str]
    severities: _containers.RepeatedScalarFieldContainer[_evidence_pb2.Severity]
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., record_types: _Optional[_Iterable[str]] = ..., sources: _Optional[_Iterable[str]] = ..., severities: _Optional[_Iterable[_Union[_evidence_pb2.Severity, str]]] = ...) -> None: ...
