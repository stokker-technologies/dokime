from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

import datetime

from dokime_sdk._generated.proto.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._generated.proto.model.v1 import execution_pb2 as _execution_pb2
from dokime_sdk._generated.proto.model.v1 import metadata_pb2 as _metadata_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import failure_pb2 as _failure_pb2
from harmos._generated.proto.model.v1 import file_pb2 as _file_pb2
from harmos._generated.proto.model.v1 import page_pb2 as _page_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from harmos._generated.proto.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CycleOrderKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CYCLE_ORDER_KEY_UNSPECIFIED: _ClassVar[CycleOrderKey]
    CYCLE_ORDER_KEY_UPDATED_AT: _ClassVar[CycleOrderKey]
    CYCLE_ORDER_KEY_CREATED_AT: _ClassVar[CycleOrderKey]
    CYCLE_ORDER_KEY_LABEL: _ClassVar[CycleOrderKey]

class ArchiveScope(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ARCHIVE_SCOPE_UNSPECIFIED: _ClassVar[ArchiveScope]
    ARCHIVE_SCOPE_UNARCHIVED: _ClassVar[ArchiveScope]
    ARCHIVE_SCOPE_ARCHIVED: _ClassVar[ArchiveScope]
    ARCHIVE_SCOPE_ALL: _ClassVar[ArchiveScope]

class CyclePhase(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CYCLE_PHASE_UNSPECIFIED: _ClassVar[CyclePhase]
    CYCLE_PHASE_ACTIVE: _ClassVar[CyclePhase]
    CYCLE_PHASE_SEALING: _ClassVar[CyclePhase]
    CYCLE_PHASE_SEALED: _ClassVar[CyclePhase]
    CYCLE_PHASE_SEAL_FAILED: _ClassVar[CyclePhase]

class CycleStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CYCLE_STATUS_UNSPECIFIED: _ClassVar[CycleStatus]
    CYCLE_STATUS_READY: _ClassVar[CycleStatus]
    CYCLE_STATUS_RUNNING: _ClassVar[CycleStatus]
    CYCLE_STATUS_PAUSED: _ClassVar[CycleStatus]
    CYCLE_STATUS_STOPPING: _ClassVar[CycleStatus]
    CYCLE_STATUS_COMPLETED: _ClassVar[CycleStatus]
    CYCLE_STATUS_FAILED: _ClassVar[CycleStatus]
    CYCLE_STATUS_BLOCKED: _ClassVar[CycleStatus]
    CYCLE_STATUS_STOPPED: _ClassVar[CycleStatus]

class ResultStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RESULT_STATUS_UNSPECIFIED: _ClassVar[ResultStatus]
    RESULT_STATUS_PENDING: _ClassVar[ResultStatus]
    RESULT_STATUS_PASSED: _ClassVar[ResultStatus]
    RESULT_STATUS_FAILED: _ClassVar[ResultStatus]
    RESULT_STATUS_BLOCKED: _ClassVar[ResultStatus]
    RESULT_STATUS_CANCELLED: _ClassVar[ResultStatus]
CYCLE_ORDER_KEY_UNSPECIFIED: CycleOrderKey
CYCLE_ORDER_KEY_UPDATED_AT: CycleOrderKey
CYCLE_ORDER_KEY_CREATED_AT: CycleOrderKey
CYCLE_ORDER_KEY_LABEL: CycleOrderKey
ARCHIVE_SCOPE_UNSPECIFIED: ArchiveScope
ARCHIVE_SCOPE_UNARCHIVED: ArchiveScope
ARCHIVE_SCOPE_ARCHIVED: ArchiveScope
ARCHIVE_SCOPE_ALL: ArchiveScope
CYCLE_PHASE_UNSPECIFIED: CyclePhase
CYCLE_PHASE_ACTIVE: CyclePhase
CYCLE_PHASE_SEALING: CyclePhase
CYCLE_PHASE_SEALED: CyclePhase
CYCLE_PHASE_SEAL_FAILED: CyclePhase
CYCLE_STATUS_UNSPECIFIED: CycleStatus
CYCLE_STATUS_READY: CycleStatus
CYCLE_STATUS_RUNNING: CycleStatus
CYCLE_STATUS_PAUSED: CycleStatus
CYCLE_STATUS_STOPPING: CycleStatus
CYCLE_STATUS_COMPLETED: CycleStatus
CYCLE_STATUS_FAILED: CycleStatus
CYCLE_STATUS_BLOCKED: CycleStatus
CYCLE_STATUS_STOPPED: CycleStatus
RESULT_STATUS_UNSPECIFIED: ResultStatus
RESULT_STATUS_PENDING: ResultStatus
RESULT_STATUS_PASSED: ResultStatus
RESULT_STATUS_FAILED: ResultStatus
RESULT_STATUS_BLOCKED: ResultStatus
RESULT_STATUS_CANCELLED: ResultStatus

@final
class CyclesGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.Position
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class CyclesGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    value: CycleWritable
    revision: bytes
    cycle: CycleSummary
    resolved_at: _position_pb2.Position
    def __init__(self, value: _Optional[_Union[CycleWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., cycle: _Optional[_Union[CycleSummary, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.Position, _Mapping]] = ...) -> None: ...

@final
class CyclesSetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: CycleWritable
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[CycleWritable, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class CyclesSetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: CycleWritable
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[CycleWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class CyclesListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    selection: CycleSelection
    filter: CycleFilter
    order: CycleOrder
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, selection: _Optional[_Union[CycleSelection, _Mapping]] = ..., filter: _Optional[_Union[CycleFilter, _Mapping]] = ..., order: _Optional[_Union[CycleOrder, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class CyclesListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    cycles: _containers.RepeatedCompositeFieldContainer[CycleSummary]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, cycles: _Optional[_Iterable[_Union[CycleSummary, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

@final
class CyclesWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SELECTION_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    selection: CycleSelection
    filter: CycleFilter
    order: CycleOrder
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, selection: _Optional[_Union[CycleSelection, _Mapping]] = ..., filter: _Optional[_Union[CycleFilter, _Mapping]] = ..., order: _Optional[_Union[CycleOrder, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class CyclesWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    cycles: _containers.RepeatedCompositeFieldContainer[CycleSummary]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, cycles: _Optional[_Iterable[_Union[CycleSummary, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

@final
class CyclesCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ORIGINS_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_AUTOMATICALLY_FIELD_NUMBER: _ClassVar[int]
    SOURCE_CYCLE_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    origins: _containers.RepeatedCompositeFieldContainer[_execution_pb2.Origin]
    metadata: _metadata_pb2.Metadata
    continue_automatically: bool
    source_cycle: str
    request_id: str
    def __init__(self, origins: _Optional[_Iterable[_Union[_execution_pb2.Origin, _Mapping]]] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., continue_automatically: _Optional[bool] = ..., source_cycle: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class CyclesCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    DROPPED_PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    cycle: CycleSummary
    skipped: _containers.RepeatedCompositeFieldContainer[RerunCaseNotice]
    dropped_parameters: _containers.RepeatedCompositeFieldContainer[RerunCaseNotice]
    receipt: _receipt_pb2.Receipt
    def __init__(self, cycle: _Optional[_Union[CycleSummary, _Mapping]] = ..., skipped: _Optional[_Iterable[_Union[RerunCaseNotice, _Mapping]]] = ..., dropped_parameters: _Optional[_Iterable[_Union[RerunCaseNotice, _Mapping]]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class CyclesSealRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    request_id: str
    def __init__(self, name: _Optional[str] = ..., request_id: _Optional[str] = ...) -> None: ...

@final
class CyclesSealResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SEAL_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    seal: bytes
    receipt: _receipt_pb2.Receipt
    cycle: CycleSummary
    def __init__(self, seal: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ..., cycle: _Optional[_Union[CycleSummary, _Mapping]] = ...) -> None: ...

@final
class CycleFilter(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AFTER_FIELD_NUMBER: _ClassVar[int]
    CREATED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedScalarFieldContainer[CycleStatus]
    created_after: _timestamp_pb2.Timestamp
    created_before: _timestamp_pb2.Timestamp
    archive: ArchiveScope
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[CycleStatus, str]]] = ..., created_after: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., created_before: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., archive: _Optional[_Union[ArchiveScope, str]] = ...) -> None: ...

@final
class CycleOrder(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: CycleOrderKey
    descending: bool
    def __init__(self, key: _Optional[_Union[CycleOrderKey, str]] = ..., descending: _Optional[bool] = ...) -> None: ...

@final
class CycleSelection(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    PHASES_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    phases: _containers.RepeatedScalarFieldContainer[CyclePhase]
    created_at: _evidence_pb2.Window
    archive: ArchiveScope
    statuses: _containers.RepeatedScalarFieldContainer[CycleStatus]
    def __init__(self, cycle_ids: _Optional[_Iterable[str]] = ..., phases: _Optional[_Iterable[_Union[CyclePhase, str]]] = ..., created_at: _Optional[_Union[_evidence_pb2.Window, _Mapping]] = ..., archive: _Optional[_Union[ArchiveScope, str]] = ..., statuses: _Optional[_Iterable[_Union[CycleStatus, str]]] = ...) -> None: ...

@final
class RerunCaseNotice(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    suite_id: str
    reason: str
    def __init__(self, case_id: _Optional[str] = ..., suite_id: _Optional[str] = ..., reason: _Optional[str] = ...) -> None: ...

@final
class CycleWritable(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    PAUSED_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_AUTOMATICALLY_FIELD_NUMBER: _ClassVar[int]
    label: str
    paused: bool
    archived: bool
    continue_automatically: bool
    def __init__(self, label: _Optional[str] = ..., paused: _Optional[bool] = ..., archived: _Optional[bool] = ..., continue_automatically: _Optional[bool] = ...) -> None: ...

@final
class CycleSummary(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    CASE_COUNT_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    CONTINUE_AUTOMATICALLY_FIELD_NUMBER: _ClassVar[int]
    OBSERVED_CURSOR_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    SEALED_FIELD_NUMBER: _ClassVar[int]
    RUNNING_COUNT_FIELD_NUMBER: _ClassVar[int]
    QUEUED_COUNT_FIELD_NUMBER: _ClassVar[int]
    SEAL_FAILURE_FIELD_NUMBER: _ClassVar[int]
    RERUN_OF_FIELD_NUMBER: _ClassVar[int]
    REPORT_FILE_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_FILE_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: _metadata_pb2.Metadata
    phase: CyclePhase
    status: CycleStatus
    case_count: int
    execution_count: int
    continue_automatically: bool
    observed_cursor: _position_pb2.ReadCut
    created_at: _timestamp_pb2.Timestamp
    started_at: _timestamp_pb2.Timestamp
    updated_at: _timestamp_pb2.Timestamp
    result: ResultSummary
    archived: bool
    sealed: bool
    running_count: int
    queued_count: int
    seal_failure: str
    rerun_of: str
    report_file: _file_pb2.FileReference
    evidence_file: _file_pb2.FileReference
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., phase: _Optional[_Union[CyclePhase, str]] = ..., status: _Optional[_Union[CycleStatus, str]] = ..., case_count: _Optional[int] = ..., execution_count: _Optional[int] = ..., continue_automatically: _Optional[bool] = ..., observed_cursor: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., updated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., result: _Optional[_Union[ResultSummary, _Mapping]] = ..., archived: _Optional[bool] = ..., sealed: _Optional[bool] = ..., running_count: _Optional[int] = ..., queued_count: _Optional[int] = ..., seal_failure: _Optional[str] = ..., rerun_of: _Optional[str] = ..., report_file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., evidence_file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ...) -> None: ...

@final
class ResultSummary(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    DECLARED_FIELD_NUMBER: _ClassVar[int]
    NOT_EVALUATED_FIELD_NUMBER: _ClassVar[int]
    status: ResultStatus
    passed: int
    failed: int
    blocked: int
    skipped: int
    declared: int
    not_evaluated: int
    def __init__(self, status: _Optional[_Union[ResultStatus, str]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ..., declared: _Optional[int] = ..., not_evaluated: _Optional[int] = ...) -> None: ...
