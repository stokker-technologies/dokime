from dokime_client._wire.dokime.v1.SettingsMessages.GetPluginSettings import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.SettingsMessages.GetSettings import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.SettingsMessages.ListStorageSources import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.SettingsMessages.ReadApplicationSettings import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.SettingsMessages.ReorderStorageSources import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.SettingsMessages.SetStorageSourceEnabled import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime_client._wire.dokime.v1.SettingsMessages.UpdateApplicationSettings import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.SettingsMessages.UpdatePluginSecret import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1
from dokime_client._wire.dokime.v1.SettingsMessages.UpdatePluginSettings import messages_pb2 as _messages_pb2_1_1_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SettingsGetPluginSettingsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsGetPluginSettingsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: SettingsGetPluginSettingsCancellation
    finish: SettingsGetPluginSettingsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsGetPluginSettingsCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsGetPluginSettingsCancellation, _Mapping]] = ...) -> None: ...

class SettingsGetPluginSettingsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class SettingsGetSettingsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsGetSettingsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: SettingsGetSettingsCancellation
    finish: SettingsGetSettingsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsGetSettingsCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsGetSettingsCancellation, _Mapping]] = ...) -> None: ...

class SettingsGetSettingsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsListStorageSourcesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsListStorageSourcesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: SettingsListStorageSourcesCancellation
    finish: SettingsListStorageSourcesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsListStorageSourcesCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsListStorageSourcesCancellation, _Mapping]] = ...) -> None: ...

class SettingsListStorageSourcesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsReadApplicationSettingsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsReadApplicationSettingsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: SettingsReadApplicationSettingsCancellation
    finish: SettingsReadApplicationSettingsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsReadApplicationSettingsCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsReadApplicationSettingsCancellation, _Mapping]] = ...) -> None: ...

class SettingsReadApplicationSettingsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsReorderStorageSourcesCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsReorderStorageSourcesRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: SettingsReorderStorageSourcesCancellation
    finish: SettingsReorderStorageSourcesCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsReorderStorageSourcesCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsReorderStorageSourcesCancellation, _Mapping]] = ...) -> None: ...

class SettingsReorderStorageSourcesRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsSetStorageSourceEnabledCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsSetStorageSourceEnabledRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: SettingsSetStorageSourceEnabledCancellation
    finish: SettingsSetStorageSourceEnabledCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsSetStorageSourceEnabledCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsSetStorageSourceEnabledCancellation, _Mapping]] = ...) -> None: ...

class SettingsSetStorageSourceEnabledRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsUpdateApplicationSettingsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsUpdateApplicationSettingsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    cancel: SettingsUpdateApplicationSettingsCancellation
    finish: SettingsUpdateApplicationSettingsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsUpdateApplicationSettingsCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsUpdateApplicationSettingsCancellation, _Mapping]] = ...) -> None: ...

class SettingsUpdateApplicationSettingsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsUpdatePluginSecretCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsUpdatePluginSecretRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1.Row1
    cancel: SettingsUpdatePluginSecretCancellation
    finish: SettingsUpdatePluginSecretCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsUpdatePluginSecretCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsUpdatePluginSecretCancellation, _Mapping]] = ...) -> None: ...

class SettingsUpdatePluginSecretRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SettingsUpdatePluginSettingsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SettingsUpdatePluginSettingsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1_1_1.Row1
    cancel: SettingsUpdatePluginSettingsCancellation
    finish: SettingsUpdatePluginSettingsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SettingsUpdatePluginSettingsCancellation, _Mapping]] = ..., finish: _Optional[_Union[SettingsUpdatePluginSettingsCancellation, _Mapping]] = ...) -> None: ...

class SettingsUpdatePluginSettingsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
