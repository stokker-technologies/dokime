from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import evidence_pb2 as _evidence_pb2
from dokime_sdk._generated.proto.model.v1 import metadata_pb2 as _metadata_pb2
from harmos._generated.proto.model.v1 import callers_pb2 as _callers_pb2
from harmos._generated.proto.model.v1 import failure_pb2 as _failure_pb2
from harmos._generated.proto.model.v1 import file_pb2 as _file_pb2
from harmos._generated.proto.model.v1 import page_pb2 as _page_pb2
from harmos._generated.proto.model.v1 import position_pb2 as _position_pb2
from harmos._generated.proto.model.v1 import receipt_pb2 as _receipt_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class ArtifactsGetRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    NAME_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    name: str
    at: _position_pb2.ReadCut
    def __init__(self, name: _Optional[str] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class ArtifactsGetResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    metadata: _evidence_pb2.ArtifactMetadata
    file: _file_pb2.FileReference
    resolved_at: _position_pb2.ReadCut
    def __init__(self, metadata: _Optional[_Union[_evidence_pb2.ArtifactMetadata, _Mapping]] = ..., file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ...) -> None: ...

@final
class ArtifactsCreateRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    FILE_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    CONTENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    file: _file_pb2.FileReference
    scope: _evidence_pb2.ArtifactScope
    filename: str
    content_type: str
    request_id: str
    metadata: _metadata_pb2.Metadata
    def __init__(self, file: _Optional[_Union[_file_pb2.FileReference, _Mapping]] = ..., scope: _Optional[_Union[_evidence_pb2.ArtifactScope, _Mapping]] = ..., filename: _Optional[str] = ..., content_type: _Optional[str] = ..., request_id: _Optional[str] = ..., metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ...) -> None: ...

@final
class ArtifactsCreateResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ARTIFACT_FIELD_NUMBER: _ClassVar[int]
    RECEIPT_FIELD_NUMBER: _ClassVar[int]
    artifact: _evidence_pb2.ArtifactMetadata
    receipt: _receipt_pb2.Receipt
    def __init__(self, artifact: _Optional[_Union[_evidence_pb2.ArtifactMetadata, _Mapping]] = ..., receipt: _Optional[_Union[_receipt_pb2.Receipt, _Mapping]] = ...) -> None: ...

@final
class ArtifactsListRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.ArtifactScope
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.ArtifactScope, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class ArtifactsListResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    NEXT_PAGE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[_evidence_pb2.ArtifactMetadata]
    resolved_at: _position_pb2.ReadCut
    next_page_token: str
    def __init__(self, artifacts: _Optional[_Iterable[_Union[_evidence_pb2.ArtifactMetadata, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., next_page_token: _Optional[str] = ...) -> None: ...

@final
class ArtifactsWatchRequest(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    AT_FIELD_NUMBER: _ClassVar[int]
    PAGE_FIELD_NUMBER: _ClassVar[int]
    scope: _evidence_pb2.ArtifactScope
    at: _position_pb2.ReadCut
    page: _page_pb2.PageRequest
    def __init__(self, scope: _Optional[_Union[_evidence_pb2.ArtifactScope, _Mapping]] = ..., at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., page: _Optional[_Union[_page_pb2.PageRequest, _Mapping]] = ...) -> None: ...

@final
class ArtifactsWatchResponse(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_AT_FIELD_NUMBER: _ClassVar[int]
    OPENING_FIELD_NUMBER: _ClassVar[int]
    RESYNC_REQUIRED_FIELD_NUMBER: _ClassVar[int]
    artifacts: _containers.RepeatedCompositeFieldContainer[_evidence_pb2.ArtifactMetadata]
    resolved_at: _position_pb2.ReadCut
    opening: bool
    resync_required: _failure_pb2.Failure
    def __init__(self, artifacts: _Optional[_Iterable[_Union[_evidence_pb2.ArtifactMetadata, _Mapping]]] = ..., resolved_at: _Optional[_Union[_position_pb2.ReadCut, _Mapping]] = ..., opening: _Optional[bool] = ..., resync_required: _Optional[_Union[_failure_pb2.Failure, _Mapping]] = ...) -> None: ...
