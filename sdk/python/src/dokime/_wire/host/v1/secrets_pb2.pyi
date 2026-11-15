from dokime._wire.model.v1 import metadata_pb2 as _metadata_pb2
from harmos._wire.model.v1 import callers_pb2 as _callers_pb2
from harmos._wire.model.v1 import failure_pb2 as _failure_pb2
from harmos._wire.model.v1 import position_pb2 as _position_pb2
from harmos._wire.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SecretImportAction(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SECRET_IMPORT_ACTION_UNSPECIFIED: _ClassVar[SecretImportAction]
    SECRET_IMPORT_ACTION_REPLACE: _ClassVar[SecretImportAction]
    SECRET_IMPORT_ACTION_SKIP: _ClassVar[SecretImportAction]
SECRET_IMPORT_ACTION_UNSPECIFIED: SecretImportAction
SECRET_IMPORT_ACTION_REPLACE: SecretImportAction
SECRET_IMPORT_ACTION_SKIP: SecretImportAction

class SecretsGetRequest(_message.Message):
    __slots__ = ("name", "at")
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsGetResponse(_message.Message):
    __slots__ = ("value", "revision", "resolved_at", "secret")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    SECRET_FIELD_NUMBER: _ClassVar[int]
    value: SecretWritable
    revision: bytes
    resolved_at: _position_pb2.ReadCut
    secret: bytes
    def __init__(self, value: _Optional[_Union[SecretWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., secret: _Optional[bytes] = ...) -> None: ...

class SecretsSetRequest(_message.Message):
    __slots__ = ("name", "value", "expected_revision", "request_id")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: SecretWritable
    expected_revision: bytes
    request_id: str
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[SecretWritable, _Mapping]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class SecretsSetResponse(_message.Message):
    __slots__ = ("value", "revision", "receipt")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    value: SecretWritable
    revision: bytes
    receipt: _receipt_pb2.Receipt
    def __init__(self, value: _Optional[_Union[SecretWritable, _Mapping]] = ..., revision: _Optional[bytes] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class SecretsListRequest(_message.Message):
    __slots__ = ("at",)
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsListResponse(_message.Message):
    __slots__ = ("entries", "generation", "resolved_at")
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[SecretSummary]
    generation: int
    resolved_at: _position_pb2.ReadCut
    def __init__(self, entries: _Optional[_Iterable[_Union[SecretSummary, _Mapping]]] = ..., generation: _Optional[int] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsWatchRequest(_message.Message):
    __slots__ = ("at",)
    AT_FIELD_NUMBER: _ClassVar[int]
    at: _position_pb2.ReadCut
    def __init__(self, at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsWatchResponse(_message.Message):
    __slots__ = ("entries", "generation", "resolved_at", "opening", "resync_required")
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[SecretSummary]
    generation: int
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, entries: _Optional[_Iterable[_Union[SecretSummary, _Mapping]]] = ..., generation: _Optional[int] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...

class SecretsDeleteRequest(_message.Message):
    __slots__ = ("names", "expected_revision", "request_id")
    NAMES_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    names: _containers.RepeatedScalarFieldContainer[str]
    expected_revision: bytes
    request_id: str
    def __init__(self, names: _Optional[_Iterable[str]] = ..., expected_revision: _Optional[bytes] = ..., request_id: _Optional[str] = ...) -> None: ...

class SecretsDeleteResponse(_message.Message):
    __slots__ = ("receipt", "cleanup_pending")
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_PENDING_FIELD_NUMBER: _ClassVar[int]
    receipt: _receipt_pb2.Receipt
    cleanup_pending: bool
    def __init__(self, receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ..., cleanup_pending: _Optional[bool] = ...) -> None: ...

class SecretsExportRequest(_message.Message):
    __slots__ = ("names", "generation", "at")
    NAMES_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    names: _containers.RepeatedScalarFieldContainer[str]
    generation: int
    at: _position_pb2.ReadCut
    def __init__(self, names: _Optional[_Iterable[str]] = ..., generation: _Optional[int] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsExportResponse(_message.Message):
    __slots__ = ("archive", "resolved_at")
    ARCHIVE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    archive: SecretArchive
    resolved_at: _position_pb2.ReadCut
    def __init__(self, archive: _Optional[_Union[SecretArchive, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

class SecretsImportRequest(_message.Message):
    __slots__ = ("data", "choices", "generation", "validate_only", "request_id")
    DATA_FIELD_NUMBER: _ClassVar[int]
    CHOICES_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    VALIDATE_ONLY_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    choices: _containers.RepeatedCompositeFieldContainer[SecretImportChoice]
    generation: int
    validate_only: bool
    request_id: str
    def __init__(self, data: _Optional[bytes] = ..., choices: _Optional[_Iterable[_Union[SecretImportChoice, _Mapping]]] = ..., generation: _Optional[int] = ..., validate_only: _Optional[bool] = ..., request_id: _Optional[str] = ...) -> None: ...

class SecretsImportResponse(_message.Message):
    __slots__ = ("preview", "snapshot", "receipt")
    PREVIEW_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    preview: SecretImportPreview
    snapshot: SecretSnapshot
    receipt: _receipt_pb2.Receipt
    def __init__(self, preview: _Optional[_Union[SecretImportPreview, _Mapping]] = ..., snapshot: _Optional[_Union[SecretSnapshot, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

class SecretArchive(_message.Message):
    __slots__ = ("data",)
    DATA_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    def __init__(self, data: _Optional[bytes] = ...) -> None: ...

class SecretImportChoice(_message.Message):
    __slots__ = ("identifier", "action")
    IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    identifier: str
    action: SecretImportAction
    def __init__(self, identifier: _Optional[str] = ..., action: _Optional[_Union[SecretImportAction, str]] = ...) -> None: ...

class SecretImportEntry(_message.Message):
    __slots__ = ("metadata", "exists")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    exists: bool
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., exists: _Optional[bool] = ...) -> None: ...

class SecretImportPreview(_message.Message):
    __slots__ = ("host_id", "host_label", "generation", "entries")
    HOST_ID_FIELD_NUMBER: _ClassVar[int]
    HOST_LABEL_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    host_id: str
    host_label: str
    generation: int
    entries: _containers.RepeatedCompositeFieldContainer[SecretImportEntry]
    def __init__(self, host_id: _Optional[str] = ..., host_label: _Optional[str] = ..., generation: _Optional[int] = ..., entries: _Optional[_Iterable[_Union[SecretImportEntry, _Mapping]]] = ...) -> None: ...

class SecretSnapshot(_message.Message):
    __slots__ = ("host_id", "host_label", "generation", "entries", "cleanup_pending")
    HOST_ID_FIELD_NUMBER: _ClassVar[int]
    HOST_LABEL_FIELD_NUMBER: _ClassVar[int]
    GENERATION_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_PENDING_FIELD_NUMBER: _ClassVar[int]
    host_id: str
    host_label: str
    generation: int
    entries: _containers.RepeatedCompositeFieldContainer[SecretSummary]
    cleanup_pending: bool
    def __init__(self, host_id: _Optional[str] = ..., host_label: _Optional[str] = ..., generation: _Optional[int] = ..., entries: _Optional[_Iterable[_Union[SecretSummary, _Mapping]]] = ..., cleanup_pending: _Optional[bool] = ...) -> None: ...

class SecretSummary(_message.Message):
    __slots__ = ("metadata", "known_uses")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    KNOWN_USES_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    known_uses: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., known_uses: _Optional[_Iterable[str]] = ...) -> None: ...

class SecretWritable(_message.Message):
    __slots__ = ("metadata", "value")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    value: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., value: _Optional[str] = ...) -> None: ...
