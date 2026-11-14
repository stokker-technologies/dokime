from dokime._wire.model.v1 import metadata_pb2 as _metadata_pb2
from dokime._wire.model.v1 import parameters_pb2 as _parameters_pb2
from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Analyses(_message.Message):
    __slots__ = ("analyses",)
    ANALYSES_FIELD_NUMBER: _ClassVar[int]
    analyses: _containers.RepeatedCompositeFieldContainer[Analysis]
    def __init__(self, analyses: _Optional[_Iterable[_Union[Analysis, _Mapping]]] = ...) -> None: ...

class Analysis(_message.Message):
    __slots__ = ("metadata", "version", "parameters", "outputs", "max_samples", "max_output_values")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    MAX_SAMPLES_FIELD_NUMBER: _ClassVar[int]
    MAX_OUTPUT_VALUES_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    version: int
    parameters: _containers.RepeatedCompositeFieldContainer[_parameters_pb2.Parameter]
    outputs: _containers.RepeatedCompositeFieldContainer[AnalysisOutput]
    max_samples: int
    max_output_values: int
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., version: _Optional[int] = ..., parameters: _Optional[_Iterable[_Union[_parameters_pb2.Parameter, _Mapping]]] = ..., outputs: _Optional[_Iterable[_Union[AnalysisOutput, _Mapping]]] = ..., max_samples: _Optional[int] = ..., max_output_values: _Optional[int] = ...) -> None: ...

class AnalysisOutput(_message.Message):
    __slots__ = ("metadata", "unit", "axis", "axis_unit")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_FIELD_NUMBER: _ClassVar[int]
    AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    metadata: _metadata_pb2.Metadata
    unit: str
    axis: str
    axis_unit: str
    def __init__(self, metadata: _Optional[_Union[_metadata_pb2.Metadata, _Mapping]] = ..., unit: _Optional[str] = ..., axis: _Optional[str] = ..., axis_unit: _Optional[str] = ...) -> None: ...

class AnalysisSelection(_message.Message):
    __slots__ = ("id", "version", "parameters")
    ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    id: str
    version: int
    parameters: _scalar_pb2.MapValue
    def __init__(self, id: _Optional[str] = ..., version: _Optional[int] = ..., parameters: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ...) -> None: ...

class AnalysisValues(_message.Message):
    __slots__ = ("key", "unit", "axis_unit", "axis", "values", "display_indices")
    KEY_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_INDICES_FIELD_NUMBER: _ClassVar[int]
    key: str
    unit: str
    axis_unit: str
    axis: _containers.RepeatedScalarFieldContainer[float]
    values: _containers.RepeatedScalarFieldContainer[float]
    display_indices: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, key: _Optional[str] = ..., unit: _Optional[str] = ..., axis_unit: _Optional[str] = ..., axis: _Optional[_Iterable[float]] = ..., values: _Optional[_Iterable[float]] = ..., display_indices: _Optional[_Iterable[int]] = ...) -> None: ...
