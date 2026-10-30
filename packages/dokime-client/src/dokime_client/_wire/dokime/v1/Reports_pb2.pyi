from dokime_client._wire.dokime.v1.ReportsMessages.DownloadCycleReport import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.ReportsMessages.DownloadExecutionReport import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.ReportsMessages.MaterializeReport import messages_pb2 as _messages_pb2_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ReportsDownloadCycleReportOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2.Row3
    response: _messages_pb2.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class ReportsDownloadCycleReportCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReportsDownloadCycleReportRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: ReportsDownloadCycleReportCancellation
    finish: ReportsDownloadCycleReportCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ReportsDownloadCycleReportCancellation, _Mapping]] = ..., finish: _Optional[_Union[ReportsDownloadCycleReportCancellation, _Mapping]] = ...) -> None: ...

class ReportsDownloadCycleReportRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    item: _messages_pb2.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2.Row3, _Mapping]] = ...) -> None: ...

class ReportsDownloadExecutionReportOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1.Row3
    response: _messages_pb2_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class ReportsDownloadExecutionReportCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReportsDownloadExecutionReportRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: ReportsDownloadExecutionReportCancellation
    finish: ReportsDownloadExecutionReportCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ReportsDownloadExecutionReportCancellation, _Mapping]] = ..., finish: _Optional[_Union[ReportsDownloadExecutionReportCancellation, _Mapping]] = ...) -> None: ...

class ReportsDownloadExecutionReportRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    item: _messages_pb2_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ...) -> None: ...

class ReportsMaterializeReportCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ReportsMaterializeReportRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: ReportsMaterializeReportCancellation
    finish: ReportsMaterializeReportCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[ReportsMaterializeReportCancellation, _Mapping]] = ..., finish: _Optional[_Union[ReportsMaterializeReportCancellation, _Mapping]] = ...) -> None: ...

class ReportsMaterializeReportRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...
