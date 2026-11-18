import datetime

from dokime_sdk._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime_sdk._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Termination(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TERMINATION_UNSPECIFIED: _ClassVar[Termination]
    TERMINATION_COMPLETED: _ClassVar[Termination]
    TERMINATION_FAILED: _ClassVar[Termination]
    TERMINATION_CANCELLED: _ClassVar[Termination]
    TERMINATION_INTERRUPTED: _ClassVar[Termination]
TERMINATION_UNSPECIFIED: Termination
TERMINATION_COMPLETED: Termination
TERMINATION_FAILED: Termination
TERMINATION_CANCELLED: Termination
TERMINATION_INTERRUPTED: Termination

class TestCase(_message.Message):
    __slots__ = ("parameters", "steps", "criteria", "streams")
    class ParametersEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _scalar_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_scalar_pb2.Value, _Mapping]] = ...) -> None: ...
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    STREAMS_FIELD_NUMBER: _ClassVar[int]
    parameters: _containers.MessageMap[str, _scalar_pb2.Value]
    steps: _containers.RepeatedCompositeFieldContainer[Step]
    criteria: _containers.RepeatedCompositeFieldContainer[Criterion]
    streams: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, parameters: _Optional[_Mapping[str, _scalar_pb2.Value]] = ..., steps: _Optional[_Iterable[_Union[Step, _Mapping]]] = ..., criteria: _Optional[_Iterable[_Union[Criterion, _Mapping]]] = ..., streams: _Optional[_Iterable[str]] = ...) -> None: ...

class Origin(_message.Message):
    __slots__ = ("plugin_run",)
    PLUGIN_RUN_FIELD_NUMBER: _ClassVar[int]
    IMPORT_FIELD_NUMBER: _ClassVar[int]
    plugin_run: PluginRun
    def __init__(self, plugin_run: _Optional[_Union[PluginRun, _Mapping]] = ..., **kwargs) -> None: ...

class PluginRun(_message.Message):
    __slots__ = ("test_case", "plugin", "template", "digest", "started_at")
    TEST_CASE_FIELD_NUMBER: _ClassVar[int]
    PLUGIN_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    DIGEST_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    test_case: TestCase
    plugin: str
    template: str
    digest: bytes
    started_at: _timestamp_pb2.Timestamp
    def __init__(self, test_case: _Optional[_Union[TestCase, _Mapping]] = ..., plugin: _Optional[str] = ..., template: _Optional[str] = ..., digest: _Optional[bytes] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class Import(_message.Message):
    __slots__ = ("metadata", "sources", "importer", "sdk_version", "processing", "imported_at", "key", "predecessor")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    IMPORTER_FIELD_NUMBER: _ClassVar[int]
    SDK_VERSION_FIELD_NUMBER: _ClassVar[int]
    PROCESSING_FIELD_NUMBER: _ClassVar[int]
    IMPORTED_AT_FIELD_NUMBER: _ClassVar[int]
    KEY_FIELD_NUMBER: _ClassVar[int]
    PREDECESSOR_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    sources: _containers.RepeatedCompositeFieldContainer[ImportSource]
    importer: str
    sdk_version: str
    processing: str
    imported_at: _timestamp_pb2.Timestamp
    key: str
    predecessor: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., sources: _Optional[_Iterable[_Union[ImportSource, _Mapping]]] = ..., importer: _Optional[str] = ..., sdk_version: _Optional[str] = ..., processing: _Optional[str] = ..., imported_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., key: _Optional[str] = ..., predecessor: _Optional[str] = ...) -> None: ...

class ImportSource(_message.Message):
    __slots__ = ("name", "sha256", "attached")
    NAME_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    ATTACHED_FIELD_NUMBER: _ClassVar[int]
    name: str
    sha256: bytes
    attached: bool
    def __init__(self, name: _Optional[str] = ..., sha256: _Optional[bytes] = ..., attached: _Optional[bool] = ...) -> None: ...

class Step(_message.Message):
    __slots__ = ("metadata", "operation", "repeatable")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    OPERATION_FIELD_NUMBER: _ClassVar[int]
    REPEATABLE_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    operation: str
    repeatable: bool
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., operation: _Optional[str] = ..., repeatable: _Optional[bool] = ...) -> None: ...

class Criterion(_message.Message):
    __slots__ = ("metadata", "operation")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    OPERATION_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    operation: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., operation: _Optional[str] = ...) -> None: ...

class ExecutionProgressValue(_message.Message):
    __slots__ = ("completed", "total", "label")
    COMPLETED_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    completed: int
    total: int
    label: str
    def __init__(self, completed: _Optional[int] = ..., total: _Optional[int] = ..., label: _Optional[str] = ...) -> None: ...
