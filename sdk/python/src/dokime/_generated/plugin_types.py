# @generated from protocol descriptors; do not edit.
from dokime._wire.host.v1.backend_pb2 import BackendEnvironmentResponse
from dokime._wire.host.v1.documents_pb2 import DocumentsAppendPartsResponse
from dokime._wire.host.v1.effects_pb2 import EffectsCreateResponse, EffectsGetResponse, EffectsSetResponse, EffectWritable
from dokime._wire.host.v1.evidence.logs_pb2 import LogsAppendResponse as EvidenceLogsAppendResponse
from dokime._wire.host.v1.evidence.records_pb2 import RecordsAppendResponse as EvidenceRecordsAppendResponse
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilitySetResponse as EvidenceTraceabilitySetResponse
from dokime._wire.host.v1.executions.progress_pb2 import ProgressSetResponse as ExecutionsProgressSetResponse
from dokime._wire.host.v1.files.captures_pb2 import CaptureMode as FilesCaptureMode
from dokime._wire.host.v1.files.captures_pb2 import CapturesStartResponse as FilesCapturesStartResponse
from dokime._wire.host.v1.files.captures_pb2 import CapturesStopResponse as FilesCapturesStopResponse
from dokime._wire.host.v1.files.captures_pb2 import LineField as FilesLineField
from dokime._wire.host.v1.files.captures_pb2 import LineMarker as FilesLineMarker
from dokime._wire.host.v1.files.captures_pb2 import LineMatcher as FilesLineMatcher
from dokime._wire.host.v1.files.captures_pb2 import MirrorCapture as FilesMirrorCapture
from dokime._wire.host.v1.files.captures_pb2 import StreamCapture as FilesStreamCapture
from dokime._wire.host.v1.files_pb2 import FilesCaptureResponse
from dokime._wire.host.v1.prompts_pb2 import PromptsCreateResponse
from dokime._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateRequest, CriteriaEvaluateResponse
from dokime._wire.plugin.v1.executions_pb2 import (
    ExecutionsCleanupRequest,
    ExecutionsCleanupResponse,
    ExecutionsInitializeRequest,
    ExecutionsInitializeResponse,
)
from dokime._wire.plugin.v1.steps_pb2 import StepsExecuteRequest, StepsExecuteResponse
from dokime._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeRequest, TestCasesMaterializeResponse

BackendEnvironmentResponse.__module__ = "dokime.plugin"
BackendEnvironmentResponse.__qualname__ = "BackendEnvironmentResponse"
DocumentsAppendPartsResponse.__module__ = "dokime.plugin"
DocumentsAppendPartsResponse.__qualname__ = "DocumentsAppendPartsResponse"
EffectWritable.__module__ = "dokime.plugin"
EffectWritable.__qualname__ = "EffectWritable"
EffectsCreateResponse.__module__ = "dokime.plugin"
EffectsCreateResponse.__qualname__ = "EffectsCreateResponse"
EffectsGetResponse.__module__ = "dokime.plugin"
EffectsGetResponse.__qualname__ = "EffectsGetResponse"
EffectsSetResponse.__module__ = "dokime.plugin"
EffectsSetResponse.__qualname__ = "EffectsSetResponse"
EvidenceLogsAppendResponse.__module__ = "dokime.plugin"
EvidenceLogsAppendResponse.__qualname__ = "EvidenceLogsAppendResponse"
EvidenceRecordsAppendResponse.__module__ = "dokime.plugin"
EvidenceRecordsAppendResponse.__qualname__ = "EvidenceRecordsAppendResponse"
EvidenceTraceabilitySetResponse.__module__ = "dokime.plugin"
EvidenceTraceabilitySetResponse.__qualname__ = "EvidenceTraceabilitySetResponse"
ExecutionsProgressSetResponse.__module__ = "dokime.plugin"
ExecutionsProgressSetResponse.__qualname__ = "ExecutionsProgressSetResponse"
FilesCaptureMode.__module__ = "dokime.plugin"
FilesCaptureMode.__qualname__ = "FilesCaptureMode"
FilesCapturesStartResponse.__module__ = "dokime.plugin"
FilesCapturesStartResponse.__qualname__ = "FilesCapturesStartResponse"
FilesCapturesStopResponse.__module__ = "dokime.plugin"
FilesCapturesStopResponse.__qualname__ = "FilesCapturesStopResponse"
FilesLineField.__module__ = "dokime.plugin"
FilesLineField.__qualname__ = "FilesLineField"
FilesLineMarker.__module__ = "dokime.plugin"
FilesLineMarker.__qualname__ = "FilesLineMarker"
FilesLineMatcher.__module__ = "dokime.plugin"
FilesLineMatcher.__qualname__ = "FilesLineMatcher"
FilesMirrorCapture.__module__ = "dokime.plugin"
FilesMirrorCapture.__qualname__ = "FilesMirrorCapture"
FilesStreamCapture.__module__ = "dokime.plugin"
FilesStreamCapture.__qualname__ = "FilesStreamCapture"
FilesCaptureResponse.__module__ = "dokime.plugin"
FilesCaptureResponse.__qualname__ = "FilesCaptureResponse"
PromptsCreateResponse.__module__ = "dokime.plugin"
PromptsCreateResponse.__qualname__ = "PromptsCreateResponse"
CriteriaEvaluateRequest.__module__ = "dokime.plugin"
CriteriaEvaluateRequest.__qualname__ = "CriteriaEvaluateRequest"
CriteriaEvaluateResponse.__module__ = "dokime.plugin"
CriteriaEvaluateResponse.__qualname__ = "CriteriaEvaluateResponse"
ExecutionsCleanupRequest.__module__ = "dokime.plugin"
ExecutionsCleanupRequest.__qualname__ = "ExecutionsCleanupRequest"
ExecutionsCleanupResponse.__module__ = "dokime.plugin"
ExecutionsCleanupResponse.__qualname__ = "ExecutionsCleanupResponse"
ExecutionsInitializeRequest.__module__ = "dokime.plugin"
ExecutionsInitializeRequest.__qualname__ = "ExecutionsInitializeRequest"
ExecutionsInitializeResponse.__module__ = "dokime.plugin"
ExecutionsInitializeResponse.__qualname__ = "ExecutionsInitializeResponse"
StepsExecuteRequest.__module__ = "dokime.plugin"
StepsExecuteRequest.__qualname__ = "StepsExecuteRequest"
StepsExecuteResponse.__module__ = "dokime.plugin"
StepsExecuteResponse.__qualname__ = "StepsExecuteResponse"
TestCasesMaterializeRequest.__module__ = "dokime.plugin"
TestCasesMaterializeRequest.__qualname__ = "TestCasesMaterializeRequest"
TestCasesMaterializeResponse.__module__ = "dokime.plugin"
TestCasesMaterializeResponse.__qualname__ = "TestCasesMaterializeResponse"

__all__ = [
    "BackendEnvironmentResponse",
    "CriteriaEvaluateRequest",
    "CriteriaEvaluateResponse",
    "DocumentsAppendPartsResponse",
    "EffectWritable",
    "EffectsCreateResponse",
    "EffectsGetResponse",
    "EffectsSetResponse",
    "EvidenceLogsAppendResponse",
    "EvidenceRecordsAppendResponse",
    "EvidenceTraceabilitySetResponse",
    "ExecutionsCleanupRequest",
    "ExecutionsCleanupResponse",
    "ExecutionsInitializeRequest",
    "ExecutionsInitializeResponse",
    "ExecutionsProgressSetResponse",
    "FilesCaptureMode",
    "FilesCaptureResponse",
    "FilesCapturesStartResponse",
    "FilesCapturesStopResponse",
    "FilesLineField",
    "FilesLineMarker",
    "FilesLineMatcher",
    "FilesMirrorCapture",
    "FilesStreamCapture",
    "PromptsCreateResponse",
    "StepsExecuteRequest",
    "StepsExecuteResponse",
    "TestCasesMaterializeRequest",
    "TestCasesMaterializeResponse",
]
