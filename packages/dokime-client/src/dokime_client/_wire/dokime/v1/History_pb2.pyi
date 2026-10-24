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

class HistoryDownloadCheckpointArtifactOutput(_message.Message):
    __slots__ = ("item", "response")
    ITEM_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    item: _messages_pb2_1_1_1.Row3
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, item: _Optional[_Union[_messages_pb2_1_1_1.Row3, _Mapping]] = ..., response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...
