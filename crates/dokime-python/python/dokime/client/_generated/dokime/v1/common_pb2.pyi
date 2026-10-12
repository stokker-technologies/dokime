import datetime

from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
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
    SEVERITY_TRACE: _ClassVar[Severity]
    SEVERITY_DEBUG: _ClassVar[Severity]
    SEVERITY_INFO: _ClassVar[Severity]
    SEVERITY_WARN: _ClassVar[Severity]
    SEVERITY_ERROR: _ClassVar[Severity]
SEVERITY_UNSPECIFIED: Severity
SEVERITY_TRACE: Severity
SEVERITY_DEBUG: Severity
SEVERITY_INFO: Severity
SEVERITY_WARN: Severity
SEVERITY_ERROR: Severity

class Metadata(_message.Message):
    __slots__ = ("identifier", "label", "description", "tags", "attributes", "created_at", "updated_at")
    class AttributesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _value_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ...) -> None: ...
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    label: str
    description: str
    tags: _containers.RepeatedScalarFieldContainer[str]
    attributes: _containers.MessageMap[str, _value_pb2.Value]
    created_at: _timestamp_pb2.Timestamp
    updated_at: _timestamp_pb2.Timestamp
    def __init__(self, identifier: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ..., tags: _Optional[_Iterable[str]] = ..., attributes: _Optional[_Mapping[str, _value_pb2.Value]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., updated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class Cursor(_message.Message):
    __slots__ = ("epoch", "stream", "position")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    STREAM_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    stream: str
    position: int
    def __init__(self, epoch: _Optional[str] = ..., stream: _Optional[str] = ..., position: _Optional[int] = ...) -> None: ...

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

class EvidenceOrigin(_message.Message):
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

class TraceabilitySnapshot(_message.Message):
    __slots__ = ("traceability_id", "scope", "created_at", "entries", "metadata")
    TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    traceability_id: str
    scope: EvidenceScope
    created_at: _timestamp_pb2.Timestamp
    entries: _containers.RepeatedCompositeFieldContainer[TraceabilityEntry]
    metadata: Metadata
    def __init__(self, traceability_id: _Optional[str] = ..., scope: _Optional[_Union[EvidenceScope, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., entries: _Optional[_Iterable[_Union[TraceabilityEntry, _Mapping]]] = ..., metadata: _Optional[_Union[Metadata, _Mapping]] = ...) -> None: ...

class TraceabilityEntry(_message.Message):
    __slots__ = ("metadata", "value", "source", "unit")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    metadata: Metadata
    value: _value_pb2.Value
    source: str
    unit: str
    def __init__(self, metadata: _Optional[_Union[Metadata, _Mapping]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., source: _Optional[str] = ..., unit: _Optional[str] = ...) -> None: ...

class Diagnostic(_message.Message):
    __slots__ = ("message", "chain", "location", "span_trace", "backtrace")
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    CHAIN_FIELD_NUMBER: _ClassVar[int]
    LOCATION_FIELD_NUMBER: _ClassVar[int]
    SPAN_TRACE_FIELD_NUMBER: _ClassVar[int]
    BACKTRACE_FIELD_NUMBER: _ClassVar[int]
    message: str
    chain: _containers.RepeatedScalarFieldContainer[str]
    location: str
    span_trace: _containers.RepeatedScalarFieldContainer[str]
    backtrace: str
    def __init__(self, message: _Optional[str] = ..., chain: _Optional[_Iterable[str]] = ..., location: _Optional[str] = ..., span_trace: _Optional[_Iterable[str]] = ..., backtrace: _Optional[str] = ...) -> None: ...
