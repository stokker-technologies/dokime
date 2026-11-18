# @generated from protocol descriptors; do not edit.
from dokime_sdk._wire.host.v1.backend_pb2 import BackendEnvironmentResponse

BackendEnvironmentResponse.__module__ = "dokime_sdk.plugin"
BackendEnvironmentResponse.__qualname__ = "BackendEnvironmentResponse"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentsAppendPartsResponse

DocumentsAppendPartsResponse.__module__ = "dokime_sdk.plugin"
DocumentsAppendPartsResponse.__qualname__ = "DocumentsAppendPartsResponse"
from dokime_sdk._wire.host.v1.effects_pb2 import EffectWritable

EffectWritable.__module__ = "dokime_sdk.plugin"
EffectWritable.__qualname__ = "EffectWritable"
from dokime_sdk._wire.host.v1.effects_pb2 import EffectsCreateResponse

EffectsCreateResponse.__module__ = "dokime_sdk.plugin"
EffectsCreateResponse.__qualname__ = "EffectsCreateResponse"
from dokime_sdk._wire.host.v1.effects_pb2 import EffectsGetResponse

EffectsGetResponse.__module__ = "dokime_sdk.plugin"
EffectsGetResponse.__qualname__ = "EffectsGetResponse"
from dokime_sdk._wire.host.v1.effects_pb2 import EffectsSetResponse

EffectsSetResponse.__module__ = "dokime_sdk.plugin"
EffectsSetResponse.__qualname__ = "EffectsSetResponse"
from dokime_sdk._wire.host.v1.evidence.logs_pb2 import LogsAppendResponse

LogsAppendResponse.__module__ = "dokime_sdk.plugin"
LogsAppendResponse.__qualname__ = "LogsAppendResponse"
from dokime_sdk._wire.host.v1.evidence.records_pb2 import RecordsAppendResponse

RecordsAppendResponse.__module__ = "dokime_sdk.plugin"
RecordsAppendResponse.__qualname__ = "RecordsAppendResponse"
from dokime_sdk._wire.host.v1.evidence.traceability_pb2 import TraceabilitySetResponse

TraceabilitySetResponse.__module__ = "dokime_sdk.plugin"
TraceabilitySetResponse.__qualname__ = "TraceabilitySetResponse"
from dokime_sdk._wire.host.v1.executions.progress_pb2 import ProgressSetResponse

ProgressSetResponse.__module__ = "dokime_sdk.plugin"
ProgressSetResponse.__qualname__ = "ProgressSetResponse"
from dokime_sdk._wire.host.v1.files.captures_pb2 import CaptureMode

CaptureMode.__module__ = "dokime_sdk.plugin"
CaptureMode.__qualname__ = "CaptureMode"
from dokime_sdk._wire.host.v1.files.captures_pb2 import CapturesStartResponse

CapturesStartResponse.__module__ = "dokime_sdk.plugin"
CapturesStartResponse.__qualname__ = "CapturesStartResponse"
from dokime_sdk._wire.host.v1.files.captures_pb2 import LineField

from ._exports.plugin_types import host

LineField.__module__ = "dokime_sdk.plugin"
LineField.__qualname__ = "LineField"
from dokime_sdk._wire.host.v1.files.captures_pb2 import LineMarker

LineMarker.__module__ = "dokime_sdk.plugin"
LineMarker.__qualname__ = "LineMarker"
from dokime_sdk._wire.host.v1.files.captures_pb2 import LineMatcher

LineMatcher.__module__ = "dokime_sdk.plugin"
LineMatcher.__qualname__ = "LineMatcher"
from dokime_sdk._wire.host.v1.files.captures_pb2 import MirrorCapture

MirrorCapture.__module__ = "dokime_sdk.plugin"
MirrorCapture.__qualname__ = "MirrorCapture"
from dokime_sdk._wire.host.v1.files.captures_pb2 import StreamCapture

StreamCapture.__module__ = "dokime_sdk.plugin"
StreamCapture.__qualname__ = "StreamCapture"
from dokime_sdk._wire.host.v1.files_pb2 import FilesCaptureResponse

FilesCaptureResponse.__module__ = "dokime_sdk.plugin"
FilesCaptureResponse.__qualname__ = "FilesCaptureResponse"
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptsCreateResponse

PromptsCreateResponse.__module__ = "dokime_sdk.plugin"
PromptsCreateResponse.__qualname__ = "PromptsCreateResponse"
from dokime_sdk._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateRequest

CriteriaEvaluateRequest.__module__ = "dokime_sdk.plugin"
CriteriaEvaluateRequest.__qualname__ = "CriteriaEvaluateRequest"
from dokime_sdk._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateResponse

CriteriaEvaluateResponse.__module__ = "dokime_sdk.plugin"
CriteriaEvaluateResponse.__qualname__ = "CriteriaEvaluateResponse"
from dokime_sdk._wire.plugin.v1.executions_pb2 import ExecutionsCleanupRequest

ExecutionsCleanupRequest.__module__ = "dokime_sdk.plugin"
ExecutionsCleanupRequest.__qualname__ = "ExecutionsCleanupRequest"
from dokime_sdk._wire.plugin.v1.executions_pb2 import ExecutionsCleanupResponse

ExecutionsCleanupResponse.__module__ = "dokime_sdk.plugin"
ExecutionsCleanupResponse.__qualname__ = "ExecutionsCleanupResponse"
from dokime_sdk._wire.plugin.v1.executions_pb2 import ExecutionsInitializeRequest

ExecutionsInitializeRequest.__module__ = "dokime_sdk.plugin"
ExecutionsInitializeRequest.__qualname__ = "ExecutionsInitializeRequest"
from dokime_sdk._wire.plugin.v1.executions_pb2 import ExecutionsInitializeResponse

ExecutionsInitializeResponse.__module__ = "dokime_sdk.plugin"
ExecutionsInitializeResponse.__qualname__ = "ExecutionsInitializeResponse"
from dokime_sdk._wire.plugin.v1.steps_pb2 import StepsExecuteRequest

StepsExecuteRequest.__module__ = "dokime_sdk.plugin"
StepsExecuteRequest.__qualname__ = "StepsExecuteRequest"
from dokime_sdk._wire.plugin.v1.steps_pb2 import StepsExecuteResponse

StepsExecuteResponse.__module__ = "dokime_sdk.plugin"
StepsExecuteResponse.__qualname__ = "StepsExecuteResponse"
from dokime_sdk._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeRequest

TestCasesMaterializeRequest.__module__ = "dokime_sdk.plugin"
TestCasesMaterializeRequest.__qualname__ = "TestCasesMaterializeRequest"
from dokime_sdk._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeResponse

TestCasesMaterializeResponse.__module__ = "dokime_sdk.plugin"
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
