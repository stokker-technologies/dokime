from dokime._wire.model.v1 import analyses_pb2 as _analyses_pb2
from dokime._wire.model.v1 import scalar_pb2 as _scalar_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class AnalysesComputeRequest(_message.Message):
    __slots__ = ("samples", "parameters")
    SAMPLES_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    samples: AnalysisSamples
    parameters: _scalar_pb2.MapValue
    def __init__(self, samples: _Optional[_Union[AnalysisSamples, _Mapping]] = ..., parameters: _Optional[_Union[_scalar_pb2.MapValue, _Mapping]] = ...) -> None: ...

class AnalysesComputeResponse(_message.Message):
    __slots__ = ("outputs",)
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    outputs: _containers.RepeatedCompositeFieldContainer[_analyses_pb2.AnalysisValues]
    def __init__(self, outputs: _Optional[_Iterable[_Union[_analyses_pb2.AnalysisValues, _Mapping]]] = ...) -> None: ...

class AnalysisSamples(_message.Message):
    __slots__ = ("axis", "values", "axis_unit", "unit")
    AXIS_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    AXIS_UNIT_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    axis: _containers.RepeatedScalarFieldContainer[float]
    values: _containers.RepeatedScalarFieldContainer[float]
    axis_unit: str
    unit: str
    def __init__(self, axis: _Optional[_Iterable[float]] = ..., values: _Optional[_Iterable[float]] = ..., axis_unit: _Optional[str] = ..., unit: _Optional[str] = ...) -> None: ...
