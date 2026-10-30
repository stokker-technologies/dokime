from dokime_client._wire.dokime.v1.PluginsMessages.GetInstallation import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.PluginsMessages.Install import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.PluginsMessages.ListAdmittedPlugins import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.PluginsMessages.ListInstallations import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.PluginsMessages.Remove import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.PluginsMessages.SetEnabled import messages_pb2 as _messages_pb2_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PluginsGetInstallationCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PluginsGetInstallationRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: PluginsGetInstallationCancellation
    finish: PluginsGetInstallationCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PluginsGetInstallationCancellation, _Mapping]] = ..., finish: _Optional[_Union[PluginsGetInstallationCancellation, _Mapping]] = ...) -> None: ...

class PluginsGetInstallationRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class PluginsInstallInput(_message.Message):
    __slots__ = ("request", "item")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    item: _messages_pb2_1.Row3
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ...) -> None: ...

class PluginsInstallCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PluginsInstallRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish", "item")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    ITEM_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: PluginsInstallCancellation
    finish: PluginsInstallCancellation
    item: _messages_pb2_1.Row3
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PluginsInstallCancellation, _Mapping]] = ..., finish: _Optional[_Union[PluginsInstallCancellation, _Mapping]] = ..., item: _Optional[_Union[_messages_pb2_1.Row3, _Mapping]] = ...) -> None: ...

class PluginsInstallRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class PluginsListAdmittedPluginsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PluginsListAdmittedPluginsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: PluginsListAdmittedPluginsCancellation
    finish: PluginsListAdmittedPluginsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PluginsListAdmittedPluginsCancellation, _Mapping]] = ..., finish: _Optional[_Union[PluginsListAdmittedPluginsCancellation, _Mapping]] = ...) -> None: ...

class PluginsListAdmittedPluginsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class PluginsListInstallationsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PluginsListInstallationsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: PluginsListInstallationsCancellation
    finish: PluginsListInstallationsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PluginsListInstallationsCancellation, _Mapping]] = ..., finish: _Optional[_Union[PluginsListInstallationsCancellation, _Mapping]] = ...) -> None: ...

class PluginsListInstallationsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class PluginsRemoveCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PluginsRemoveRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: PluginsRemoveCancellation
    finish: PluginsRemoveCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PluginsRemoveCancellation, _Mapping]] = ..., finish: _Optional[_Union[PluginsRemoveCancellation, _Mapping]] = ...) -> None: ...

class PluginsRemoveRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class PluginsSetEnabledCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PluginsSetEnabledRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: PluginsSetEnabledCancellation
    finish: PluginsSetEnabledCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[PluginsSetEnabledCancellation, _Mapping]] = ..., finish: _Optional[_Union[PluginsSetEnabledCancellation, _Mapping]] = ...) -> None: ...

class PluginsSetEnabledRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
