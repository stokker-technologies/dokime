from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("limit", "after_execution_id", "filter", "order")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_execution_id: str
    filter: Row3
    order: Row4
    def __init__(self, limit: _Optional[int] = ..., after_execution_id: _Optional[str] = ..., filter: _Optional[_Union[Row3, _Mapping]] = ..., order: _Optional[_Union[Row4, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("entries", "next_after_execution_id", "discovery_complete", "conflicts", "omitted_conflicts", "issues", "omitted_issues", "cycles")
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    DISCOVERY_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CONFLICTS_FIELD_NUMBER: _ClassVar[int]
    OMITTED_CONFLICTS_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    OMITTED_ISSUES_FIELD_NUMBER: _ClassVar[int]
    CYCLES_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[Row5]
    next_after_execution_id: str
    discovery_complete: bool
    conflicts: _containers.RepeatedScalarFieldContainer[str]
    omitted_conflicts: int
    issues: _containers.RepeatedCompositeFieldContainer[Row6]
    omitted_issues: int
    cycles: _containers.RepeatedCompositeFieldContainer[Row7]
    def __init__(self, entries: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., next_after_execution_id: _Optional[str] = ..., discovery_complete: _Optional[bool] = ..., conflicts: _Optional[_Iterable[str]] = ..., omitted_conflicts: _Optional[int] = ..., issues: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., omitted_issues: _Optional[int] = ..., cycles: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("text", "statuses", "template_ids", "cycle_ids", "started_after", "started_before", "archive")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AFTER_FIELD_NUMBER: _ClassVar[int]
    STARTED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedCompositeFieldContainer[Row8]
    template_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    started_after: Row9
    started_before: Row9
    archive: Row10
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[Row8, _Mapping]]] = ..., template_ids: _Optional[_Iterable[str]] = ..., cycle_ids: _Optional[_Iterable[str]] = ..., started_after: _Optional[_Union[Row9, _Mapping]] = ..., started_before: _Optional[_Union[Row9, _Mapping]] = ..., archive: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("key", "descending")
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: Row11
    descending: bool
    def __init__(self, key: _Optional[_Union[Row11, _Mapping]] = ..., descending: _Optional[bool] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("metadata", "checkpoint", "availability", "archived")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    AVAILABILITY_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    metadata: Row12
    checkpoint: Row13
    availability: Row14
    archived: bool
    def __init__(self, metadata: _Optional[_Union[Row12, _Mapping]] = ..., checkpoint: _Optional[_Union[Row13, _Mapping]] = ..., availability: _Optional[_Union[Row14, _Mapping]] = ..., archived: _Optional[bool] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("source", "dataset", "message")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    DATASET_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    source: str
    dataset: str
    message: str
    def __init__(self, source: _Optional[str] = ..., dataset: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("cycle_id", "lifecycle", "result_status", "case_count", "execution_count", "unrun_count", "queued_count", "running_count", "passed_count", "failed_count", "blocked_count", "started_at", "finished_at")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    LIFECYCLE_FIELD_NUMBER: _ClassVar[int]
    RESULT_STATUS_FIELD_NUMBER: _ClassVar[int]
    CASE_COUNT_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_COUNT_FIELD_NUMBER: _ClassVar[int]
    UNRUN_COUNT_FIELD_NUMBER: _ClassVar[int]
    QUEUED_COUNT_FIELD_NUMBER: _ClassVar[int]
    RUNNING_COUNT_FIELD_NUMBER: _ClassVar[int]
    PASSED_COUNT_FIELD_NUMBER: _ClassVar[int]
    FAILED_COUNT_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_COUNT_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    lifecycle: Row15
    result_status: Row16
    case_count: int
    execution_count: int
    unrun_count: int
    queued_count: int
    running_count: int
    passed_count: int
    failed_count: int
    blocked_count: int
    started_at: Row9
    finished_at: Row9
    def __init__(self, cycle_id: _Optional[str] = ..., lifecycle: _Optional[_Union[Row15, _Mapping]] = ..., result_status: _Optional[_Union[Row16, _Mapping]] = ..., case_count: _Optional[int] = ..., execution_count: _Optional[int] = ..., unrun_count: _Optional[int] = ..., queued_count: _Optional[int] = ..., running_count: _Optional[int] = ..., passed_count: _Optional[int] = ..., failed_count: _Optional[int] = ..., blocked_count: _Optional[int] = ..., started_at: _Optional[_Union[Row9, _Mapping]] = ..., finished_at: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("Unspecified", "Invalidated", "Pending", "Running", "Passed", "Failed", "Blocked", "Cancelled", "Interrupted", "Skipped")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    INVALIDATED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row17
    Invalidated: Row18
    Pending: Row19
    Running: Row20
    Passed: Row21
    Failed: Row22
    Blocked: Row23
    Cancelled: Row24
    Interrupted: Row25
    Skipped: Row26
    def __init__(self, Unspecified: _Optional[_Union[Row17, _Mapping]] = ..., Invalidated: _Optional[_Union[Row18, _Mapping]] = ..., Pending: _Optional[_Union[Row19, _Mapping]] = ..., Running: _Optional[_Union[Row20, _Mapping]] = ..., Passed: _Optional[_Union[Row21, _Mapping]] = ..., Failed: _Optional[_Union[Row22, _Mapping]] = ..., Blocked: _Optional[_Union[Row23, _Mapping]] = ..., Cancelled: _Optional[_Union[Row24, _Mapping]] = ..., Interrupted: _Optional[_Union[Row25, _Mapping]] = ..., Skipped: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("Unarchived", "Archived", "All")
    UNARCHIVED_FIELD_NUMBER: _ClassVar[int]
    ARCHIVED_FIELD_NUMBER: _ClassVar[int]
    ALL_FIELD_NUMBER: _ClassVar[int]
    Unarchived: Row27
    Archived: Row28
    All: Row29
    def __init__(self, Unarchived: _Optional[_Union[Row27, _Mapping]] = ..., Archived: _Optional[_Union[Row28, _Mapping]] = ..., All: _Optional[_Union[Row29, _Mapping]] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("Unspecified", "StartedAt", "FinishedAt", "Title")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STARTEDAT_FIELD_NUMBER: _ClassVar[int]
    FINISHEDAT_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row30
    StartedAt: Row31
    FinishedAt: Row32
    Title: Row33
    def __init__(self, Unspecified: _Optional[_Union[Row30, _Mapping]] = ..., StartedAt: _Optional[_Union[Row31, _Mapping]] = ..., FinishedAt: _Optional[_Union[Row32, _Mapping]] = ..., Title: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("execution_id", "cycle_id", "case_id", "title", "status", "started_at", "finished_at", "result", "template_id", "suite_label")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_LABEL_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    cycle_id: str
    case_id: str
    title: str
    status: Row8
    started_at: Row9
    finished_at: Row9
    result: Row34
    template_id: str
    suite_label: str
    def __init__(self, execution_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., title: _Optional[str] = ..., status: _Optional[_Union[Row8, _Mapping]] = ..., started_at: _Optional[_Union[Row9, _Mapping]] = ..., finished_at: _Optional[_Union[Row9, _Mapping]] = ..., result: _Optional[_Union[Row34, _Mapping]] = ..., template_id: _Optional[str] = ..., suite_label: _Optional[str] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row35
    version: bytes
    evidence_through: Row35
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row35, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row35, _Mapping]] = ...) -> None: ...

class Row14(_message.Message):
    __slots__ = ("Unspecified", "Live", "Stored", "Interrupted", "Uncheckpointed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LIVE_FIELD_NUMBER: _ClassVar[int]
    STORED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    UNCHECKPOINTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row36
    Live: Row37
    Stored: Row38
    Interrupted: Row39
    Uncheckpointed: Row40
    def __init__(self, Unspecified: _Optional[_Union[Row36, _Mapping]] = ..., Live: _Optional[_Union[Row37, _Mapping]] = ..., Stored: _Optional[_Union[Row38, _Mapping]] = ..., Interrupted: _Optional[_Union[Row39, _Mapping]] = ..., Uncheckpointed: _Optional[_Union[Row40, _Mapping]] = ...) -> None: ...

class Row15(_message.Message):
    __slots__ = ("Unspecified", "Running", "Interrupted", "Pending", "Uncheckpointed", "Completed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    UNCHECKPOINTED_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row41
    Running: Row42
    Interrupted: Row43
    Pending: Row44
    Uncheckpointed: Row45
    Completed: Row46
    def __init__(self, Unspecified: _Optional[_Union[Row41, _Mapping]] = ..., Running: _Optional[_Union[Row42, _Mapping]] = ..., Interrupted: _Optional[_Union[Row43, _Mapping]] = ..., Pending: _Optional[_Union[Row44, _Mapping]] = ..., Uncheckpointed: _Optional[_Union[Row45, _Mapping]] = ..., Completed: _Optional[_Union[Row46, _Mapping]] = ...) -> None: ...

class Row16(_message.Message):
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row47
    Pending: Row48
    Passed: Row49
    Failed: Row50
    Blocked: Row51
    Cancelled: Row52
    def __init__(self, Unspecified: _Optional[_Union[Row47, _Mapping]] = ..., Pending: _Optional[_Union[Row48, _Mapping]] = ..., Passed: _Optional[_Union[Row49, _Mapping]] = ..., Failed: _Optional[_Union[Row50, _Mapping]] = ..., Blocked: _Optional[_Union[Row51, _Mapping]] = ..., Cancelled: _Optional[_Union[Row52, _Mapping]] = ...) -> None: ...

class Row17(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row18(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row19(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row20(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row21(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row22(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row23(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row24(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row25(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row26(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row27(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row28(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row29(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row30(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row31(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row32(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row33(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row34(_message.Message):
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row16
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row16, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row53
    sequence: int
    def __init__(self, history: _Optional[_Union[Row53, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

class Row36(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row37(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row38(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row39(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row40(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row41(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row42(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row43(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row44(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row45(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row46(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row47(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row48(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row49(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row50(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row51(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row52(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row53(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
