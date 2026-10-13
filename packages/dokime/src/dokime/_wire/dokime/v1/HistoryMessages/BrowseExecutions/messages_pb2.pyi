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
    __slots__ = ("entries", "next_after_execution_id", "discovery_complete", "conflicts", "omitted_conflicts", "issues", "omitted_issues")
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    DISCOVERY_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CONFLICTS_FIELD_NUMBER: _ClassVar[int]
    OMITTED_CONFLICTS_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    OMITTED_ISSUES_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[Row5]
    next_after_execution_id: str
    discovery_complete: bool
    conflicts: _containers.RepeatedScalarFieldContainer[str]
    omitted_conflicts: int
    issues: _containers.RepeatedCompositeFieldContainer[Row6]
    omitted_issues: int
    def __init__(self, entries: _Optional[_Iterable[_Union[Row5, _Mapping]]] = ..., next_after_execution_id: _Optional[str] = ..., discovery_complete: _Optional[bool] = ..., conflicts: _Optional[_Iterable[str]] = ..., omitted_conflicts: _Optional[int] = ..., issues: _Optional[_Iterable[_Union[Row6, _Mapping]]] = ..., omitted_issues: _Optional[int] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("text", "statuses", "template_ids", "cycle_ids", "started_after", "started_before")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AFTER_FIELD_NUMBER: _ClassVar[int]
    STARTED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedCompositeFieldContainer[Row7]
    template_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    started_after: Row8
    started_before: Row8
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[Row7, _Mapping]]] = ..., template_ids: _Optional[_Iterable[str]] = ..., cycle_ids: _Optional[_Iterable[str]] = ..., started_after: _Optional[_Union[Row8, _Mapping]] = ..., started_before: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("key", "descending")
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: Row9
    descending: bool
    def __init__(self, key: _Optional[_Union[Row9, _Mapping]] = ..., descending: _Optional[bool] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("metadata", "checkpoint", "availability")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    AVAILABILITY_FIELD_NUMBER: _ClassVar[int]
    metadata: Row10
    checkpoint: Row11
    availability: Row12
    def __init__(self, metadata: _Optional[_Union[Row10, _Mapping]] = ..., checkpoint: _Optional[_Union[Row11, _Mapping]] = ..., availability: _Optional[_Union[Row12, _Mapping]] = ...) -> None: ...

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
    Unspecified: Row13
    Invalidated: Row14
    Pending: Row15
    Running: Row16
    Passed: Row17
    Failed: Row18
    Blocked: Row19
    Cancelled: Row20
    Interrupted: Row21
    Skipped: Row22
    def __init__(self, Unspecified: _Optional[_Union[Row13, _Mapping]] = ..., Invalidated: _Optional[_Union[Row14, _Mapping]] = ..., Pending: _Optional[_Union[Row15, _Mapping]] = ..., Running: _Optional[_Union[Row16, _Mapping]] = ..., Passed: _Optional[_Union[Row17, _Mapping]] = ..., Failed: _Optional[_Union[Row18, _Mapping]] = ..., Blocked: _Optional[_Union[Row19, _Mapping]] = ..., Cancelled: _Optional[_Union[Row20, _Mapping]] = ..., Interrupted: _Optional[_Union[Row21, _Mapping]] = ..., Skipped: _Optional[_Union[Row22, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("seconds", "nanos")
    SECONDS_FIELD_NUMBER: _ClassVar[int]
    NANOS_FIELD_NUMBER: _ClassVar[int]
    seconds: int
    nanos: int
    def __init__(self, seconds: _Optional[int] = ..., nanos: _Optional[int] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("Unspecified", "StartedAt", "FinishedAt", "Title")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    STARTEDAT_FIELD_NUMBER: _ClassVar[int]
    FINISHEDAT_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row23
    StartedAt: Row24
    FinishedAt: Row25
    Title: Row26
    def __init__(self, Unspecified: _Optional[_Union[Row23, _Mapping]] = ..., StartedAt: _Optional[_Union[Row24, _Mapping]] = ..., FinishedAt: _Optional[_Union[Row25, _Mapping]] = ..., Title: _Optional[_Union[Row26, _Mapping]] = ...) -> None: ...

class Row10(_message.Message):
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
    status: Row7
    started_at: Row8
    finished_at: Row8
    result: Row27
    template_id: str
    suite_label: str
    def __init__(self, execution_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., title: _Optional[str] = ..., status: _Optional[_Union[Row7, _Mapping]] = ..., started_at: _Optional[_Union[Row8, _Mapping]] = ..., finished_at: _Optional[_Union[Row8, _Mapping]] = ..., result: _Optional[_Union[Row27, _Mapping]] = ..., template_id: _Optional[str] = ..., suite_label: _Optional[str] = ...) -> None: ...

class Row11(_message.Message):
    __slots__ = ("execution_identifier", "position", "version", "evidence_through")
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    EVIDENCE_THROUGH_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    position: Row28
    version: bytes
    evidence_through: Row28
    def __init__(self, execution_identifier: _Optional[str] = ..., position: _Optional[_Union[Row28, _Mapping]] = ..., version: _Optional[bytes] = ..., evidence_through: _Optional[_Union[Row28, _Mapping]] = ...) -> None: ...

class Row12(_message.Message):
    __slots__ = ("Unspecified", "Live", "Stored", "Interrupted", "Uncheckpointed")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    LIVE_FIELD_NUMBER: _ClassVar[int]
    STORED_FIELD_NUMBER: _ClassVar[int]
    INTERRUPTED_FIELD_NUMBER: _ClassVar[int]
    UNCHECKPOINTED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row29
    Live: Row30
    Stored: Row31
    Interrupted: Row32
    Uncheckpointed: Row33
    def __init__(self, Unspecified: _Optional[_Union[Row29, _Mapping]] = ..., Live: _Optional[_Union[Row30, _Mapping]] = ..., Stored: _Optional[_Union[Row31, _Mapping]] = ..., Interrupted: _Optional[_Union[Row32, _Mapping]] = ..., Uncheckpointed: _Optional[_Union[Row33, _Mapping]] = ...) -> None: ...

class Row13(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row14(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row15(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row16(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

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
    __slots__ = ("status", "passed", "failed", "blocked", "skipped")
    STATUS_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    SKIPPED_FIELD_NUMBER: _ClassVar[int]
    status: Row34
    passed: int
    failed: int
    blocked: int
    skipped: int
    def __init__(self, status: _Optional[_Union[Row34, _Mapping]] = ..., passed: _Optional[int] = ..., failed: _Optional[int] = ..., blocked: _Optional[int] = ..., skipped: _Optional[int] = ...) -> None: ...

class Row28(_message.Message):
    __slots__ = ("history", "sequence")
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    history: Row35
    sequence: int
    def __init__(self, history: _Optional[_Union[Row35, _Mapping]] = ..., sequence: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("Unspecified", "Pending", "Passed", "Failed", "Blocked", "Cancelled")
    UNSPECIFIED_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    PASSED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    BLOCKED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    Unspecified: Row36
    Pending: Row37
    Passed: Row38
    Failed: Row39
    Blocked: Row40
    Cancelled: Row41
    def __init__(self, Unspecified: _Optional[_Union[Row36, _Mapping]] = ..., Pending: _Optional[_Union[Row37, _Mapping]] = ..., Passed: _Optional[_Union[Row38, _Mapping]] = ..., Failed: _Optional[_Union[Row39, _Mapping]] = ..., Blocked: _Optional[_Union[Row40, _Mapping]] = ..., Cancelled: _Optional[_Union[Row41, _Mapping]] = ...) -> None: ...

class Row35(_message.Message):
    __slots__ = ("store", "epoch")
    STORE_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    store: str
    epoch: str
    def __init__(self, store: _Optional[str] = ..., epoch: _Optional[str] = ...) -> None: ...

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

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
