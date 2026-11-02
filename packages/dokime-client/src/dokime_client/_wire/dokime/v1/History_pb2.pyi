from dokime_client._wire.dokime.v1.HistoryMessages.BrowseCycles import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.HistoryMessages.BrowseExecutions import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.HistoryMessages.DeleteExecution import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.DownloadCheckpointArtifact import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.GetCheckpointContext import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.GetCheckpointDocument import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.GetCheckpointEvidence import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.GetExecutionCheckpoint import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.ListCheckpointRecords import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.ListCheckpointTelemetry import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.HistoryMessages.ReadCheckpointOutput import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class HistoryBrowseCyclesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryBrowseCyclesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: HistoryBrowseCyclesCancellation
    finish: HistoryBrowseCyclesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryBrowseCyclesCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryBrowseCyclesCancellation, _Mapping]] = ...) -> None: ...

class HistoryBrowseCyclesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class HistoryBrowseExecutionsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryBrowseExecutionsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: HistoryBrowseExecutionsCancellation
    finish: HistoryBrowseExecutionsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryBrowseExecutionsCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryBrowseExecutionsCancellation, _Mapping]] = ...) -> None: ...

class HistoryBrowseExecutionsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryDeleteExecutionCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryDeleteExecutionRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: HistoryDeleteExecutionCancellation
    finish: HistoryDeleteExecutionCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryDeleteExecutionCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryDeleteExecutionCancellation, _Mapping]] = ...) -> None: ...

class HistoryDeleteExecutionRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryDownloadCheckpointArtifactOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1.Row3
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryDownloadCheckpointArtifactCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryDownloadCheckpointArtifactRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: HistoryDownloadCheckpointArtifactCancellation
    finish: HistoryDownloadCheckpointArtifactCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryDownloadCheckpointArtifactCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryDownloadCheckpointArtifactCancellation, _Mapping]] = ...) -> None: ...

class HistoryDownloadCheckpointArtifactRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    item: _messages_pb2_1_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ...) -> None: ...

class HistoryGetCheckpointContextCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryGetCheckpointContextRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: HistoryGetCheckpointContextCancellation
    finish: HistoryGetCheckpointContextCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryGetCheckpointContextCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryGetCheckpointContextCancellation, _Mapping]] = ...) -> None: ...

class HistoryGetCheckpointContextRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryGetCheckpointDocumentCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryGetCheckpointDocumentRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: HistoryGetCheckpointDocumentCancellation
    finish: HistoryGetCheckpointDocumentCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryGetCheckpointDocumentCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryGetCheckpointDocumentCancellation, _Mapping]] = ...) -> None: ...

class HistoryGetCheckpointDocumentRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryGetCheckpointEvidenceCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryGetCheckpointEvidenceRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    cancel: HistoryGetCheckpointEvidenceCancellation
    finish: HistoryGetCheckpointEvidenceCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryGetCheckpointEvidenceCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryGetCheckpointEvidenceCancellation, _Mapping]] = ...) -> None: ...

class HistoryGetCheckpointEvidenceRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryGetExecutionCheckpointCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryGetExecutionCheckpointRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1.Row1
    cancel: HistoryGetExecutionCheckpointCancellation
    finish: HistoryGetExecutionCheckpointCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryGetExecutionCheckpointCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryGetExecutionCheckpointCancellation, _Mapping]] = ...) -> None: ...

class HistoryGetExecutionCheckpointRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryListCheckpointRecordsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryListCheckpointRecordsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1.Row1
    cancel: HistoryListCheckpointRecordsCancellation
    finish: HistoryListCheckpointRecordsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryListCheckpointRecordsCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryListCheckpointRecordsCancellation, _Mapping]] = ...) -> None: ...

class HistoryListCheckpointRecordsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryListCheckpointTelemetryCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryListCheckpointTelemetryRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1.Row1
    cancel: HistoryListCheckpointTelemetryCancellation
    finish: HistoryListCheckpointTelemetryCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryListCheckpointTelemetryCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryListCheckpointTelemetryCancellation, _Mapping]] = ...) -> None: ...

class HistoryListCheckpointTelemetryRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class HistoryReadCheckpointOutputCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HistoryReadCheckpointOutputRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1_1_1.Row1
    cancel: HistoryReadCheckpointOutputCancellation
    finish: HistoryReadCheckpointOutputCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[HistoryReadCheckpointOutputCancellation, _Mapping]] = ..., finish: _Optional[_Union[HistoryReadCheckpointOutputCancellation, _Mapping]] = ...) -> None: ...

class HistoryReadCheckpointOutputRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
