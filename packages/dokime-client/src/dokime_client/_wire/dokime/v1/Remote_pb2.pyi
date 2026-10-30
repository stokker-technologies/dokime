from dokime_client._wire.dokime.v1.RemoteMessages.CreatePairing import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.RemoteMessages.ListPairedClients import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.RemoteMessages.ListSessions import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.RemoteMessages.ReadListener import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.RemoteMessages.RegenerateCertificate import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.RemoteMessages.RevokePairedClient import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime_client._wire.dokime.v1.RemoteMessages.UpdateListener import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RemoteCreatePairingCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteCreatePairingRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: RemoteCreatePairingCancellation
    finish: RemoteCreatePairingCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteCreatePairingCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteCreatePairingCancellation, _Mapping]] = ...) -> None: ...

class RemoteCreatePairingRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class RemoteListPairedClientsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteListPairedClientsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: RemoteListPairedClientsCancellation
    finish: RemoteListPairedClientsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteListPairedClientsCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteListPairedClientsCancellation, _Mapping]] = ...) -> None: ...

class RemoteListPairedClientsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class RemoteListSessionsCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteListSessionsRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: RemoteListSessionsCancellation
    finish: RemoteListSessionsCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteListSessionsCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteListSessionsCancellation, _Mapping]] = ...) -> None: ...

class RemoteListSessionsRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class RemoteReadListenerCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteReadListenerRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: RemoteReadListenerCancellation
    finish: RemoteReadListenerCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteReadListenerCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteReadListenerCancellation, _Mapping]] = ...) -> None: ...

class RemoteReadListenerRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class RemoteRegenerateCertificateCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteRegenerateCertificateRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: RemoteRegenerateCertificateCancellation
    finish: RemoteRegenerateCertificateCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteRegenerateCertificateCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteRegenerateCertificateCancellation, _Mapping]] = ...) -> None: ...

class RemoteRegenerateCertificateRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class RemoteRevokePairedClientCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteRevokePairedClientRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: RemoteRevokePairedClientCancellation
    finish: RemoteRevokePairedClientCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteRevokePairedClientCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteRevokePairedClientCancellation, _Mapping]] = ...) -> None: ...

class RemoteRevokePairedClientRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class RemoteUpdateListenerCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class RemoteUpdateListenerRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    cancel: RemoteUpdateListenerCancellation
    finish: RemoteUpdateListenerCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[RemoteUpdateListenerCancellation, _Mapping]] = ..., finish: _Optional[_Union[RemoteUpdateListenerCancellation, _Mapping]] = ...) -> None: ...

class RemoteUpdateListenerRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
