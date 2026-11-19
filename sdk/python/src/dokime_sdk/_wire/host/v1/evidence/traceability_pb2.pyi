from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

import datetime

from dokime_sdk._wire.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._wire.model.v1 import traceability_pb2 as _traceability_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
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
class TraceabilityGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.EvidenceScope
    page: _page_pb2.PageRequest
    at: _position_pb2.ReadCut
    entry: str
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., entry: _Optional[str] = ...) -> None: ...

@final
class TraceabilityGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SNAPSHOTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    snapshots: _containers.RepeatedCompositeFieldContainer[TraceabilitySnapshot]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    value: _traceability_pb2.TraceabilityEntry
    revision: bytes
    def __init__(self, snapshots: _Optional[_Iterable[_Union[TraceabilitySnapshot, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ..., value: _Optional[_Union[_traceability_pb2.TraceabilityEntry, _Mapping]] = ..., revision: _Optional[bytes] = ...) -> None: ...

@final
class TraceabilityWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.EvidenceScope
    page: _page_pb2.PageRequest
    at: _position_pb2.ReadCut
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class TraceabilityWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SNAPSHOTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    snapshots: _containers.RepeatedCompositeFieldContainer[TraceabilitySnapshot]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    def __init__(self, snapshots: _Optional[_Iterable[_Union[TraceabilitySnapshot, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ...) -> None: ...

@final
class TraceabilitySetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    ENTRY_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: _traceability_pb2.TraceabilityEntry
    expected_revision: bytes
    request_id: str
    entry: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[_traceability_pb2.TraceabilityEntry, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ..., entry: _Optional[str] = ...) -> None: ...

@final
class TraceabilitySetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: _traceability_pb2.TraceabilityEntry
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[_traceability_pb2.TraceabilityEntry, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class TraceabilitySnapshot(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_ID_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    revision: bytes
    resolved_at: _position_pb2.Position
    traceability_id: str
    scope: _evidence_pb2.EvidenceScope
    created_at: _timestamp_pb2.Timestamp
    entries: _containers.RepeatedCompositeFieldContainer[_traceability_pb2.TraceabilityEntry]
    metadata: _metadata_pb2.Metadata
    def __init__(self, revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ..., traceability_id: _Optional[str] = ..., scope: _Optional[_Union[_evidence_pb2.EvidenceScope, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., entries: _Optional[_Iterable[_Union[_traceability_pb2.TraceabilityEntry, _Mapping]]] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ...) -> None: ...
