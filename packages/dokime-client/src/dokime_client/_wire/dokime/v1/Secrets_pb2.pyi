from dokime_client._wire.dokime.v1.SecretsMessages.Export import messages_pb2 as _messages_pb2
from dokime_client._wire.dokime.v1.SecretsMessages.Import import messages_pb2 as _messages_pb2_1
from dokime_client._wire.dokime.v1.SecretsMessages.List import messages_pb2 as _messages_pb2_1_1
from dokime_client._wire.dokime.v1.SecretsMessages.Lookup import messages_pb2 as _messages_pb2_1_1_1
from dokime_client._wire.dokime.v1.SecretsMessages.Preview import messages_pb2 as _messages_pb2_1_1_1_1
from dokime_client._wire.dokime.v1.SecretsMessages.Put import messages_pb2 as _messages_pb2_1_1_1_1_1
from dokime_client._wire.dokime.v1.SecretsMessages.Remove import messages_pb2 as _messages_pb2_1_1_1_1_1_1
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SecretsExportCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsExportRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2.Row1
    cancel: SecretsExportCancellation
    finish: SecretsExportCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsExportCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsExportCancellation, _Mapping]] = ...) -> None: ...

class SecretsExportRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2.Row2, _Mapping]] = ...) -> None: ...

class SecretsImportCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsImportRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1.Row1
    cancel: SecretsImportCancellation
    finish: SecretsImportCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsImportCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsImportCancellation, _Mapping]] = ...) -> None: ...

class SecretsImportRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1.Row2, _Mapping]] = ...) -> None: ...

class SecretsListCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsListRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1.Row1
    cancel: SecretsListCancellation
    finish: SecretsListCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsListCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsListCancellation, _Mapping]] = ...) -> None: ...

class SecretsListRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1.Row2, _Mapping]] = ...) -> None: ...

class SecretsLookupCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsLookupRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1.Row1
    cancel: SecretsLookupCancellation
    finish: SecretsLookupCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsLookupCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsLookupCancellation, _Mapping]] = ...) -> None: ...

class SecretsLookupRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SecretsPreviewCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsPreviewRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1.Row1
    cancel: SecretsPreviewCancellation
    finish: SecretsPreviewCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsPreviewCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsPreviewCancellation, _Mapping]] = ...) -> None: ...

class SecretsPreviewRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SecretsPutCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsPutRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1.Row1
    cancel: SecretsPutCancellation
    finish: SecretsPutCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsPutCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsPutCancellation, _Mapping]] = ...) -> None: ...

class SecretsPutRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...

class SecretsRemoveCancellation(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SecretsRemoveRunInput(_message.Message):
    __slots__ = ("request", "cancel", "finish")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    FINISH_FIELD_NUMBER: _ClassVar[int]
    request: _messages_pb2_1_1_1_1_1_1.Row1
    cancel: SecretsRemoveCancellation
    finish: SecretsRemoveCancellation
    def __init__(self, request: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row1, _Mapping]] = ..., cancel: _Optional[_Union[SecretsRemoveCancellation, _Mapping]] = ..., finish: _Optional[_Union[SecretsRemoveCancellation, _Mapping]] = ...) -> None: ...

class SecretsRemoveRunOutput(_message.Message):
    __slots__ = ("response",)
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    response: _messages_pb2_1_1_1_1_1_1.Row2
    def __init__(self, response: _Optional[_Union[_messages_pb2_1_1_1_1_1_1.Row2, _Mapping]] = ...) -> None: ...
