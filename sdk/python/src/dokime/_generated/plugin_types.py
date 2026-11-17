# @generated from protocol descriptors; do not edit.
from dokime._wire.host.v1.backend_pb2 import BackendEnvironmentResponse

BackendEnvironmentResponse.__module__ = "dokime.plugin"
BackendEnvironmentResponse.__qualname__ = "BackendEnvironmentResponse"
from dokime._wire.host.v1.documents_pb2 import DocumentsAppendPartsResponse

DocumentsAppendPartsResponse.__module__ = "dokime.plugin"
DocumentsAppendPartsResponse.__qualname__ = "DocumentsAppendPartsResponse"
from dokime._wire.host.v1.effects_pb2 import EffectWritable

EffectWritable.__module__ = "dokime.plugin"
EffectWritable.__qualname__ = "EffectWritable"
from dokime._wire.host.v1.effects_pb2 import EffectsCreateResponse

EffectsCreateResponse.__module__ = "dokime.plugin"
EffectsCreateResponse.__qualname__ = "EffectsCreateResponse"
from dokime._wire.host.v1.effects_pb2 import EffectsGetResponse

EffectsGetResponse.__module__ = "dokime.plugin"
EffectsGetResponse.__qualname__ = "EffectsGetResponse"
from dokime._wire.host.v1.effects_pb2 import EffectsSetResponse

EffectsSetResponse.__module__ = "dokime.plugin"
EffectsSetResponse.__qualname__ = "EffectsSetResponse"
from dokime._wire.host.v1.evidence.logs_pb2 import LogsAppendResponse

LogsAppendResponse.__module__ = "dokime.plugin"
LogsAppendResponse.__qualname__ = "LogsAppendResponse"
from dokime._wire.host.v1.evidence.records_pb2 import RecordsAppendResponse

RecordsAppendResponse.__module__ = "dokime.plugin"
RecordsAppendResponse.__qualname__ = "RecordsAppendResponse"
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilitySetResponse

TraceabilitySetResponse.__module__ = "dokime.plugin"
TraceabilitySetResponse.__qualname__ = "TraceabilitySetResponse"
from dokime._wire.host.v1.executions.progress_pb2 import ProgressSetResponse

ProgressSetResponse.__module__ = "dokime.plugin"
ProgressSetResponse.__qualname__ = "ProgressSetResponse"
from dokime._wire.host.v1.files.captures_pb2 import CaptureMode

CaptureMode.__module__ = "dokime.plugin"
CaptureMode.__qualname__ = "CaptureMode"
from dokime._wire.host.v1.files.captures_pb2 import CapturesStartResponse

CapturesStartResponse.__module__ = "dokime.plugin"
CapturesStartResponse.__qualname__ = "CapturesStartResponse"
from dokime._wire.host.v1.files.captures_pb2 import LineField

from ._exports.plugin_types import host

LineField.__module__ = "dokime.plugin"
LineField.__qualname__ = "LineField"
from dokime._wire.host.v1.files.captures_pb2 import LineMarker

LineMarker.__module__ = "dokime.plugin"
LineMarker.__qualname__ = "LineMarker"
from dokime._wire.host.v1.files.captures_pb2 import LineMatcher

LineMatcher.__module__ = "dokime.plugin"
LineMatcher.__qualname__ = "LineMatcher"
from dokime._wire.host.v1.files.captures_pb2 import MirrorCapture

MirrorCapture.__module__ = "dokime.plugin"
MirrorCapture.__qualname__ = "MirrorCapture"
from dokime._wire.host.v1.files.captures_pb2 import StreamCapture

StreamCapture.__module__ = "dokime.plugin"
StreamCapture.__qualname__ = "StreamCapture"
from dokime._wire.host.v1.files_pb2 import FilesCaptureResponse

FilesCaptureResponse.__module__ = "dokime.plugin"
FilesCaptureResponse.__qualname__ = "FilesCaptureResponse"
from dokime._wire.host.v1.prompts_pb2 import PromptsCreateResponse

PromptsCreateResponse.__module__ = "dokime.plugin"
PromptsCreateResponse.__qualname__ = "PromptsCreateResponse"
from dokime._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateRequest

CriteriaEvaluateRequest.__module__ = "dokime.plugin"
CriteriaEvaluateRequest.__qualname__ = "CriteriaEvaluateRequest"
from dokime._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateResponse

CriteriaEvaluateResponse.__module__ = "dokime.plugin"
CriteriaEvaluateResponse.__qualname__ = "CriteriaEvaluateResponse"
from dokime._wire.plugin.v1.executions_pb2 import ExecutionsCleanupRequest

ExecutionsCleanupRequest.__module__ = "dokime.plugin"
ExecutionsCleanupRequest.__qualname__ = "ExecutionsCleanupRequest"
from dokime._wire.plugin.v1.executions_pb2 import ExecutionsCleanupResponse

ExecutionsCleanupResponse.__module__ = "dokime.plugin"
ExecutionsCleanupResponse.__qualname__ = "ExecutionsCleanupResponse"
from dokime._wire.plugin.v1.executions_pb2 import ExecutionsInitializeRequest

ExecutionsInitializeRequest.__module__ = "dokime.plugin"
ExecutionsInitializeRequest.__qualname__ = "ExecutionsInitializeRequest"
from dokime._wire.plugin.v1.executions_pb2 import ExecutionsInitializeResponse

ExecutionsInitializeResponse.__module__ = "dokime.plugin"
ExecutionsInitializeResponse.__qualname__ = "ExecutionsInitializeResponse"
from dokime._wire.plugin.v1.steps_pb2 import StepsExecuteRequest

StepsExecuteRequest.__module__ = "dokime.plugin"
StepsExecuteRequest.__qualname__ = "StepsExecuteRequest"
from dokime._wire.plugin.v1.steps_pb2 import StepsExecuteResponse

StepsExecuteResponse.__module__ = "dokime.plugin"
StepsExecuteResponse.__qualname__ = "StepsExecuteResponse"
from dokime._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeRequest

TestCasesMaterializeRequest.__module__ = "dokime.plugin"
TestCasesMaterializeRequest.__qualname__ = "TestCasesMaterializeRequest"
from dokime._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeResponse

TestCasesMaterializeResponse.__module__ = "dokime.plugin"
TestCasesMaterializeResponse.__qualname__ = "TestCasesMaterializeResponse"

__all__ = [
    "BackendEnvironmentResponse",
    "CaptureMode",
    "CapturesStartResponse",
    "CriteriaEvaluateRequest",
    "CriteriaEvaluateResponse",
    "DocumentsAppendPartsResponse",
    "EffectWritable",
    "EffectsCreateResponse",
    "EffectsGetResponse",
    "EffectsSetResponse",
    "ExecutionsCleanupRequest",
    "ExecutionsCleanupResponse",
    "ExecutionsInitializeRequest",
    "ExecutionsInitializeResponse",
    "FilesCaptureResponse",
    "LineField",
    "LineMarker",
    "LineMatcher",
    "LogsAppendResponse",
    "MirrorCapture",
    "ProgressSetResponse",
    "PromptsCreateResponse",
    "RecordsAppendResponse",
    "StepsExecuteRequest",
    "StepsExecuteResponse",
    "StreamCapture",
    "TestCasesMaterializeRequest",
    "TestCasesMaterializeResponse",
    "TraceabilitySetResponse",
    "host",
]
