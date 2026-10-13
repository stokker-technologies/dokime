from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Row1(_message.Message):
    __slots__ = ("page_size", "after")
    PAGE_SIZE_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    page_size: int
    after: Row3
    def __init__(self, page_size: _Optional[int] = ..., after: _Optional[_Union[Row3, _Mapping]] = ...) -> None: ...

class Row2(_message.Message):
    __slots__ = ("plugins", "next", "total_count", "epoch", "revision")
    PLUGINS_FIELD_NUMBER: _ClassVar[int]
    NEXT_FIELD_NUMBER: _ClassVar[int]
    TOTAL_COUNT_FIELD_NUMBER: _ClassVar[int]
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    plugins: _containers.RepeatedCompositeFieldContainer[Row4]
    next: Row3
    total_count: int
    epoch: str
    revision: int
    def __init__(self, plugins: _Optional[_Iterable[_Union[Row4, _Mapping]]] = ..., next: _Optional[_Union[Row3, _Mapping]] = ..., total_count: _Optional[int] = ..., epoch: _Optional[str] = ..., revision: _Optional[int] = ...) -> None: ...

class Row3(_message.Message):
    __slots__ = ("epoch", "revision", "after_plugin_id")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    AFTER_PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    revision: int
    after_plugin_id: str
    def __init__(self, epoch: _Optional[str] = ..., revision: _Optional[int] = ..., after_plugin_id: _Optional[str] = ...) -> None: ...

class Row4(_message.Message):
    __slots__ = ("plugin_id", "label", "version", "enabled", "owner")
    PLUGIN_ID_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    ENABLED_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    plugin_id: str
    label: str
    version: str
    enabled: bool
    owner: Row5
    def __init__(self, plugin_id: _Optional[str] = ..., label: _Optional[str] = ..., version: _Optional[str] = ..., enabled: _Optional[bool] = ..., owner: _Optional[_Union[Row5, _Mapping]] = ...) -> None: ...

class Row5(_message.Message):
    __slots__ = ("Host", "Bundle", "Embedded")
    HOST_FIELD_NUMBER: _ClassVar[int]
    BUNDLE_FIELD_NUMBER: _ClassVar[int]
    EMBEDDED_FIELD_NUMBER: _ClassVar[int]
    Host: Row6
    Bundle: Row7
    Embedded: Row8
    def __init__(self, Host: _Optional[_Union[Row6, _Mapping]] = ..., Bundle: _Optional[_Union[Row7, _Mapping]] = ..., Embedded: _Optional[_Union[Row8, _Mapping]] = ...) -> None: ...

class Row6(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: bool
    def __init__(self, field_0: _Optional[bool] = ...) -> None: ...

class Row7(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row9
    def __init__(self, field_0: _Optional[_Union[Row9, _Mapping]] = ...) -> None: ...

class Row8(_message.Message):
    __slots__ = ("field_0",)
    FIELD_0_FIELD_NUMBER: _ClassVar[int]
    field_0: Row10
    def __init__(self, field_0: _Optional[_Union[Row10, _Mapping]] = ...) -> None: ...

class Row9(_message.Message):
    __slots__ = ("package", "path", "installation_id", "content_digest")
    PACKAGE_FIELD_NUMBER: _ClassVar[int]
    PATH_FIELD_NUMBER: _ClassVar[int]
    INSTALLATION_ID_FIELD_NUMBER: _ClassVar[int]
    CONTENT_DIGEST_FIELD_NUMBER: _ClassVar[int]
    package: str
    path: str
    installation_id: str
    content_digest: str
    def __init__(self, package: _Optional[str] = ..., path: _Optional[str] = ..., installation_id: _Optional[str] = ..., content_digest: _Optional[str] = ...) -> None: ...

class Row10(_message.Message):
    __slots__ = ("generation",)
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    generation: str
    def __init__(self, generation: _Optional[str] = ...) -> None: ...

class Row0(_message.Message):
    __slots__ = ("request", "response")
    REQUEST_FIELD_NUMBER: _ClassVar[int]
    RESPONSE_FIELD_NUMBER: _ClassVar[int]
    request: Row1
    response: Row2
    def __init__(self, request: _Optional[_Union[Row1, _Mapping]] = ..., response: _Optional[_Union[Row2, _Mapping]] = ...) -> None: ...
