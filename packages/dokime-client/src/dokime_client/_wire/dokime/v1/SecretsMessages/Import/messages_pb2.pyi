from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("ciphertext", "password", "choices", "generation")
    CIPHERTEXT_FIELD_NUMBER: _ClassVar[int]
    PASSWORD_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    ciphertext: bytes
    password: str
    choices: _containers.RepeatedCompositeFieldContainer[Row3]
    generation: int
    def __init__(self, ciphertext: _Optional[bytes] = ..., password: _Optional[str] = ..., choices: _Optional[_Iterable[_Union[Row3, _Mapping]]] = ..., generation: _Optional[int] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("host_id", "host_label", "generation", "entries", "cleanup_pending")
    HOST_ID_FIELD_NUMBER: _ClassVar[int]
    HOST_LABEL_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_PENDING_FIELD_NUMBER: _ClassVar[int]
    host_id: str
    host_label: str
    generation: int
    entries: _containers.RepeatedCompositeFieldContainer[Row4]
    cleanup_pending: bool
    def __init__(self, host_id: _Optional[str] = ..., host_label: _Optional[str] = ..., generation: _Optional[int] = ..., entries: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., cleanup_pending: _Optional[bool] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("name", "action")
    NAME_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    name: str
    action: Row5
    def __init__(self, name: _Optional[str] = ..., action: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("metadata", "known_uses")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    KNOWN_USES_FIELD_NUMBER: _ClassVar[int]
    metadata: Row6
    known_uses: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, metadata: _Optional[_Union[Row6, _Mapping]] = ..., known_uses: _Optional[_Iterable[str]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Replace", "Skip")
    REPLACE_FIELD_NUMBER: _ClassVar[int]
    SKIP_FIELD_NUMBER: _ClassVar[int]
    Replace: Row7
    Skip: Row8
    def __init__(self, Replace: _Optional[_Union[Row7, _Mapping]] = ..., Skip: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("name", "label", "description")
    NAME_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    name: str
    label: str
    description: str
    def __init__(self, name: _Optional[str] = ..., label: _Optional[str] = ..., description: _Optional[str] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row8(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
