from google.protobuf.reflection import GeneratedProtocolMessageType
from typing import final

from dokime_sdk._generated.proto.model.v1 import documents_pb2 as _documents_pb2
from dokime_sdk._generated.proto.model.v1 import execution_pb2 as _execution_pb2
from dokime_sdk._generated.proto.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._generated.proto.model.v1 import parameters_pb2 as _parameters_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

@final
class Suites(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    SUITES_FIELD_NUMBER: _ClassVar[int]
    suites: _containers.RepeatedCompositeFieldContainer[Suite]
    def __init__(self, suites: _Optional[_Iterable[_Union[Suite, _Mapping]]] = ...) -> None: ...

@final
class Suite(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TEMPLATES_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_GROUPS_FIELD_NUMBER: _ClassVar[int]
    UNAVAILABLE_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    templates: _containers.RepeatedCompositeFieldContainer[TestCaseTemplate]
    tags: _containers.RepeatedScalarFieldContainer[str]
    parameter_groups: _containers.RepeatedCompositeFieldContainer[_metadata_pb2.Metadata]
    unavailable: bool
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., templates: _Optional[_Iterable[_Union[TestCaseTemplate, _Mapping]]] = ..., tags: _Optional[_Iterable[str]] = ..., parameter_groups: _Optional[_Iterable[_Union[_metadata_pb2.Metadata, _Mapping]]] = ..., unavailable: _Optional[bool] = ...) -> None: ...

@final
class TestCaseTemplate(_message.Message, metaclass=GeneratedProtocolMessageType):
    __slots__: tuple[str, ...]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    FILES_FIELD_NUMBER: _ClassVar[int]
    DOCUMENTS_FIELD_NUMBER: _ClassVar[int]
    TAGS_FIELD_NUMBER: _ClassVar[int]
    MATERIALIZE_FIELD_NUMBER: _ClassVar[int]
    INITIALIZE_FIELD_NUMBER: _ClassVar[int]
    CLEANUP_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_GROUPS_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    parameters: _containers.RepeatedCompositeFieldContainer[_parameters_pb2.Parameter]
    files: _containers.RepeatedCompositeFieldContainer[_parameters_pb2.FileParameter]
    documents: _containers.RepeatedCompositeFieldContainer[_documents_pb2.Document]
    tags: _containers.RepeatedScalarFieldContainer[str]
    materialize: str
    initialize: str
    cleanup: str
    parameter_groups: _containers.RepeatedCompositeFieldContainer[_metadata_pb2.Metadata]
    steps: _containers.RepeatedCompositeFieldContainer[_execution_pb2.Step]
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., parameters: _Optional[_Iterable[_Union[_parameters_pb2.Parameter, _Mapping]]] = ..., files: _Optional[_Iterable[_Union[_parameters_pb2.FileParameter, _Mapping]]] = ..., documents: _Optional[_Iterable[_Union[_documents_pb2.Document, _Mapping]]] = ..., tags: _Optional[_Iterable[str]] = ..., materialize: _Optional[str] = ..., initialize: _Optional[str] = ..., cleanup: _Optional[str] = ..., parameter_groups: _Optional[_Iterable[_Union[_metadata_pb2.Metadata, _Mapping]]] = ..., steps: _Optional[_Iterable[_Union[_execution_pb2.Step, _Mapping]]] = ...) -> None: ...
