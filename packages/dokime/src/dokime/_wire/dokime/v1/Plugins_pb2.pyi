from dokime._wire.dokime.v1.PluginsMessages.Consent import messages_pb2 as _messages_pb2
from dokime._wire.dokime.v1.PluginsMessages.GetInstallation import messages_pb2 as _messages_pb2_1
from dokime._wire.dokime.v1.PluginsMessages.ListAdmittedPlugins import messages_pb2 as _messages_pb2_1_1
from dokime._wire.dokime.v1.PluginsMessages.ListInstallations import messages_pb2 as _messages_pb2_1_1_1
from dokime._wire.dokime.v1.PluginsMessages.Remove import messages_pb2 as _messages_pb2_1_1_1_1
from dokime._wire.dokime.v1.PluginsMessages.SetEnabled import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime._wire.dokime.v1.PluginsMessages.Stage import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PluginsStageInput(_message.Message):
    __slots__ = ("request", "item")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    item: _messages_pb2_1_1_1_1_1_1.Row3
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row3, _Mapping]] = ...) -> None: ...
