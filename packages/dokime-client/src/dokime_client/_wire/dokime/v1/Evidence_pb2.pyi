from dokime_client._wire.dokime.v1.EvidenceMessages.GetCapturedRecordSchema import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.EvidenceMessages.ListRecords import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.EvidenceMessages.WatchRecords import messages_pb2 as _messages_pb2_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class EvidenceGetCapturedRecordSchemaCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class EvidenceGetCapturedRecordSchemaRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: EvidenceGetCapturedRecordSchemaCancellation
    finish: EvidenceGetCapturedRecordSchemaCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[EvidenceGetCapturedRecordSchemaCancellation, _Mapping]] = ..., finish: _Optional[_Union[EvidenceGetCapturedRecordSchemaCancellation, _Mapping]] = ...) -> None: ...

class EvidenceGetCapturedRecordSchemaRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class EvidenceListRecordsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class EvidenceListRecordsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: EvidenceListRecordsCancellation
    finish: EvidenceListRecordsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[EvidenceListRecordsCancellation, _Mapping]] = ..., finish: _Optional[_Union[EvidenceListRecordsCancellation, _Mapping]] = ...) -> None: ...

class EvidenceListRecordsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class EvidenceWatchRecordsOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1.Row3
    response: _messages_pb2_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class EvidenceWatchRecordsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class EvidenceWatchRecordsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: EvidenceWatchRecordsCancellation
    finish: EvidenceWatchRecordsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[EvidenceWatchRecordsCancellation, _Mapping]] = ..., finish: _Optional[_Union[EvidenceWatchRecordsCancellation, _Mapping]] = ...) -> None: ...

class EvidenceWatchRecordsRunOutput(_message.Message):
    __slots__ = ("response", "item")
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    item: _messages_pb2_1_1.Row3
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ...) -> None: ...
