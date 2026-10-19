from dokime._wire.dokime.v1.EvidenceMessages.GetCapturedRecordSchema import messages_pb2 as _messages_pb2
from dokime._wire.dokime.v1.EvidenceMessages.ListRecords import messages_pb2 as _messages_pb2_1
from dokime._wire.dokime.v1.EvidenceMessages.WatchRecords import messages_pb2 as _messages_pb2_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class EvidenceWatchRecordsOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1.Row3
    response: _messages_pb2_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...
