# @generated from protocol descriptors; do not edit.
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateResponse

ArtifactsCreateResponse.__module__ = "dokime"
ArtifactsCreateResponse.__qualname__ = "ArtifactsCreateResponse"
from dokime._wire.host.v1.evidence.records_pb2 import CapturedErrorCause

CapturedErrorCause.__module__ = "dokime"
CapturedErrorCause.__qualname__ = "CapturedErrorCause"
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecord

CapturedRecord.__module__ = "dokime"
CapturedRecord.__qualname__ = "CapturedRecord"
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecordError

CapturedRecordError.__module__ = "dokime"
CapturedRecordError.__qualname__ = "CapturedRecordError"
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecordSelection

CapturedRecordSelection.__module__ = "dokime"
CapturedRecordSelection.__qualname__ = "CapturedRecordSelection"
from dokime._wire.host.v1.evidence.records_pb2 import CycleScope

CycleScope.__module__ = "dokime"
CycleScope.__qualname__ = "CycleScope"
from dokime._wire.host.v1.evidence.records_pb2 import RecordScope

RecordScope.__module__ = "dokime"
RecordScope.__qualname__ = "RecordScope"
from dokime._wire.host.v1.evidence.records_pb2 import RecordsListResponse

RecordsListResponse.__module__ = "dokime"
RecordsListResponse.__qualname__ = "RecordsListResponse"
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityGetResponse

TraceabilityGetResponse.__module__ = "dokime"
TraceabilityGetResponse.__qualname__ = "TraceabilityGetResponse"
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilitySnapshot

TraceabilitySnapshot.__module__ = "dokime"
TraceabilitySnapshot.__qualname__ = "TraceabilitySnapshot"
from dokime._wire.host.v1.evidence_pb2 import EvidenceStatisticsResponse

EvidenceStatisticsResponse.__module__ = "dokime"
EvidenceStatisticsResponse.__qualname__ = "EvidenceStatisticsResponse"
from dokime._wire.host.v1.evidence_pb2 import Output

Output.__module__ = "dokime"
Output.__qualname__ = "Output"
from dokime._wire.host.v1.evidence_pb2 import Statistics

Statistics.__module__ = "dokime"
Statistics.__qualname__ = "Statistics"
from dokime._wire.host.v1.executions.progress_pb2 import ProgressGetResponse

ProgressGetResponse.__module__ = "dokime"
ProgressGetResponse.__qualname__ = "ProgressGetResponse"
from dokime._wire.host.v1.files.uploads_pb2 import UploadsAppendResponse

UploadsAppendResponse.__module__ = "dokime"
UploadsAppendResponse.__qualname__ = "UploadsAppendResponse"
from dokime._wire.host.v1.files.uploads_pb2 import UploadsCreateResponse

UploadsCreateResponse.__module__ = "dokime"
UploadsCreateResponse.__qualname__ = "UploadsCreateResponse"
from dokime._wire.host.v1.files.uploads_pb2 import UploadsDeleteResponse

UploadsDeleteResponse.__module__ = "dokime"
UploadsDeleteResponse.__qualname__ = "UploadsDeleteResponse"
from dokime._wire.host.v1.files.uploads_pb2 import UploadsGetResponse

UploadsGetResponse.__module__ = "dokime"
UploadsGetResponse.__qualname__ = "UploadsGetResponse"
from dokime._wire.host.v1.files.uploads_pb2 import UploadsPublishResponse

UploadsPublishResponse.__module__ = "dokime"
UploadsPublishResponse.__qualname__ = "UploadsPublishResponse"
from dokime._wire.host.v1.files.uploads_pb2 import UploadsResumeResponse

UploadsResumeResponse.__module__ = "dokime"
UploadsResumeResponse.__qualname__ = "UploadsResumeResponse"
from dokime._wire.host.v1.files_pb2 import FileBatch

FileBatch.__module__ = "dokime"
FileBatch.__qualname__ = "FileBatch"
from dokime._wire.host.v1.files_pb2 import FileCompletion

FileCompletion.__module__ = "dokime"
FileCompletion.__qualname__ = "FileCompletion"
from dokime._wire.host.v1.files_pb2 import FilesDeleteResponse

FilesDeleteResponse.__module__ = "dokime"
FilesDeleteResponse.__qualname__ = "FilesDeleteResponse"
from dokime._wire.host.v1.files_pb2 import FilesGetResponse

FilesGetResponse.__module__ = "dokime"
FilesGetResponse.__qualname__ = "FilesGetResponse"
from dokime._wire.host.v1.files_pb2 import FilesReadResponse

FilesReadResponse.__module__ = "dokime"
FilesReadResponse.__qualname__ = "FilesReadResponse"
from dokime._wire.host.v1.files_pb2 import FilesWatchResponse

FilesWatchResponse.__module__ = "dokime"
FilesWatchResponse.__qualname__ = "FilesWatchResponse"
from dokime._wire.host.v1.record_schemas_pb2 import (
    FIELD_TYPE_ARRAY,
    FIELD_TYPE_BOOLEAN,
    FIELD_TYPE_DATETIME,
    FIELD_TYPE_FLOAT64,
    FIELD_TYPE_INT64,
    FIELD_TYPE_INTEGER,
    FIELD_TYPE_OBJECT,
    FIELD_TYPE_SEVERITY,
    FIELD_TYPE_TEXT,
    FIELD_TYPE_UINT64,
    FIELD_TYPE_UNSPECIFIED,
    Field,
)

Field.__module__ = "dokime"
Field.__qualname__ = "Field"
from dokime._wire.host.v1.record_schemas_pb2 import FieldType, Schema

Schema.__module__ = "dokime"
Schema.__qualname__ = "Schema"
from dokime._wire.host.v1.secrets_pb2 import SecretWritable

SecretWritable.__module__ = "dokime"
SecretWritable.__qualname__ = "SecretWritable"
from dokime._wire.host.v1.secrets_pb2 import SecretsGetResponse

SecretsGetResponse.__module__ = "dokime"
SecretsGetResponse.__qualname__ = "SecretsGetResponse"
from dokime._wire.model.v1.documents_pb2 import Card

Card.__module__ = "dokime"
Card.__qualname__ = "Card"
from dokime._wire.model.v1.documents_pb2 import CardLayout

CardLayout.__module__ = "dokime"
CardLayout.__qualname__ = "CardLayout"
from dokime._wire.model.v1.documents_pb2 import Document

Document.__module__ = "dokime"
Document.__qualname__ = "Document"
from dokime._wire.model.v1.documents_pb2 import (
    EVIDENCE_CARD_ARTIFACTS,
    EVIDENCE_CARD_CRITERIA,
    EVIDENCE_CARD_HEALTH,
    EVIDENCE_CARD_MEDIA,
    EVIDENCE_CARD_NOTES,
    EVIDENCE_CARD_RECORDS,
    EVIDENCE_CARD_STEPS,
    EVIDENCE_CARD_STREAM_CONTEXT,
    EVIDENCE_CARD_TRACEABILITY,
    EVIDENCE_CARD_UNSPECIFIED,
    EvidenceCard,
    SectionRepeat,
)

from ._exports.shared import model

SectionRepeat.__module__ = "dokime"
SectionRepeat.__qualname__ = "SectionRepeat"
from dokime._wire.model.v1.documents_pb2 import Summary

Summary.__module__ = "dokime"
Summary.__qualname__ = "Summary"
from dokime._wire.model.v1.documents_pb2 import SummaryTile

SummaryTile.__module__ = "dokime"
SummaryTile.__qualname__ = "SummaryTile"
from dokime._wire.model.v1.documents_pb2 import Table

Table.__module__ = "dokime"
Table.__qualname__ = "Table"
from dokime._wire.model.v1.documents_pb2 import TableRow

TableRow.__module__ = "dokime"
TableRow.__qualname__ = "TableRow"
from dokime._wire.model.v1.evidence_pb2 import (
    ARTIFACT_STATE_CANCELLED,
    ARTIFACT_STATE_COMPLETE,
    ARTIFACT_STATE_FAILED,
    ARTIFACT_STATE_REJECTED,
    ARTIFACT_STATE_UNSPECIFIED,
    ArtifactMetadata,
)

ArtifactMetadata.__module__ = "dokime"
ArtifactMetadata.__qualname__ = "ArtifactMetadata"
from dokime._wire.model.v1.evidence_pb2 import ArtifactScope

ArtifactScope.__module__ = "dokime"
ArtifactScope.__qualname__ = "ArtifactScope"
from dokime._wire.model.v1.evidence_pb2 import ArtifactState, EvidenceCondition

EvidenceCondition.__module__ = "dokime"
EvidenceCondition.__qualname__ = "EvidenceCondition"
from dokime._wire.model.v1.evidence_pb2 import EvidenceConditions

EvidenceConditions.__module__ = "dokime"
EvidenceConditions.__qualname__ = "EvidenceConditions"
from dokime._wire.model.v1.evidence_pb2 import EvidenceOrigin

EvidenceOrigin.__module__ = "dokime"
EvidenceOrigin.__qualname__ = "EvidenceOrigin"
from dokime._wire.model.v1.evidence_pb2 import EvidenceOriginKind

EvidenceOriginKind.__module__ = "dokime"
EvidenceOriginKind.__qualname__ = "EvidenceOriginKind"
from dokime._wire.model.v1.evidence_pb2 import EvidenceScope

EvidenceScope.__module__ = "dokime"
EvidenceScope.__qualname__ = "EvidenceScope"
from dokime._wire.model.v1.evidence_pb2 import RecordMatch

RecordMatch.__module__ = "dokime"
RecordMatch.__qualname__ = "RecordMatch"
from dokime._wire.model.v1.evidence_pb2 import (
    SEVERITY_DEBUG,
    SEVERITY_FAILURE,
    SEVERITY_INFO,
    SEVERITY_UNSPECIFIED,
    SEVERITY_WARNING,
    STREAM_MATCH_OPERATOR_EQ,
    STREAM_MATCH_OPERATOR_EXISTS,
    STREAM_MATCH_OPERATOR_GE,
    STREAM_MATCH_OPERATOR_GT,
    STREAM_MATCH_OPERATOR_LE,
    STREAM_MATCH_OPERATOR_LT,
    STREAM_MATCH_OPERATOR_UNSPECIFIED,
    STREAM_MATCH_OPERATOR_WITHIN,
    Severity,
    StreamMatch,
)

StreamMatch.__module__ = "dokime"
StreamMatch.__qualname__ = "StreamMatch"
from dokime._wire.model.v1.evidence_pb2 import StreamMatchOperator
from dokime._wire.model.v1.execution_pb2 import Criterion

Criterion.__module__ = "dokime"
Criterion.__qualname__ = "Criterion"
from dokime._wire.model.v1.execution_pb2 import ExecutionProgressValue

ExecutionProgressValue.__module__ = "dokime"
ExecutionProgressValue.__qualname__ = "ExecutionProgressValue"
from dokime._wire.model.v1.execution_pb2 import Import

Import.__module__ = "dokime"
Import.__qualname__ = "Import"
from dokime._wire.model.v1.execution_pb2 import ImportSource

ImportSource.__module__ = "dokime"
ImportSource.__qualname__ = "ImportSource"
from dokime._wire.model.v1.execution_pb2 import Origin

Origin.__module__ = "dokime"
Origin.__qualname__ = "Origin"
from dokime._wire.model.v1.execution_pb2 import PluginRun

PluginRun.__module__ = "dokime"
PluginRun.__qualname__ = "PluginRun"
from dokime._wire.model.v1.execution_pb2 import Step

Step.__module__ = "dokime"
Step.__qualname__ = "Step"
from dokime._wire.model.v1.execution_pb2 import (
    TERMINATION_CANCELLED,
    TERMINATION_COMPLETED,
    TERMINATION_FAILED,
    TERMINATION_INTERRUPTED,
    TERMINATION_UNSPECIFIED,
    Termination,
    TestCase,
)

TestCase.__module__ = "dokime"
TestCase.__qualname__ = "TestCase"
from dokime._wire.model.v1.expressions_pb2 import (
    AGGREGATE_COUNT,
    AGGREGATE_FIRST,
    AGGREGATE_LAST,
    AGGREGATE_MAX,
    AGGREGATE_MEAN,
    AGGREGATE_MEDIAN,
    AGGREGATE_MIN,
    AGGREGATE_PERCENTILE,
    AGGREGATE_RMS,
    AGGREGATE_STDDEV,
    AGGREGATE_SUM,
    AGGREGATE_UNSPECIFIED,
    Aggregate,
    AggregateExpression,
)

AggregateExpression.__module__ = "dokime"
AggregateExpression.__qualname__ = "AggregateExpression"
from dokime._wire.model.v1.expressions_pb2 import DataExpression

DataExpression.__module__ = "dokime"
DataExpression.__qualname__ = "DataExpression"
from dokime._wire.model.v1.expressions_pb2 import DocumentFilter

DocumentFilter.__module__ = "dokime"
DocumentFilter.__qualname__ = "DocumentFilter"
from dokime._wire.model.v1.expressions_pb2 import (
    FILTER_OPERATOR_CONTAINS,
    FILTER_OPERATOR_EQ,
    FILTER_OPERATOR_EXISTS,
    FILTER_OPERATOR_GE,
    FILTER_OPERATOR_GT,
    FILTER_OPERATOR_LE,
    FILTER_OPERATOR_LT,
    FILTER_OPERATOR_NE,
    FILTER_OPERATOR_UNSPECIFIED,
    FieldReference,
)

FieldReference.__module__ = "dokime"
FieldReference.__qualname__ = "FieldReference"
from dokime._wire.model.v1.expressions_pb2 import FilterOperator, TransformExpression

TransformExpression.__module__ = "dokime"
TransformExpression.__qualname__ = "TransformExpression"
from dokime._wire.model.v1.files_pb2 import (
    CAPTURE_GAP_REASON_REPLACED,
    CAPTURE_GAP_REASON_RESUME_UNSUPPORTED,
    CAPTURE_GAP_REASON_TRUNCATED,
    CAPTURE_GAP_REASON_UNSPECIFIED,
    CAPTURE_GAP_REASON_VANISHED,
    CaptureGap,
)

CaptureGap.__module__ = "dokime"
CaptureGap.__qualname__ = "CaptureGap"
from dokime._wire.model.v1.files_pb2 import CapturedArtifact, CaptureGapReason

CapturedArtifact.__module__ = "dokime"
CapturedArtifact.__qualname__ = "CapturedArtifact"
from dokime._wire.model.v1.files_pb2 import FILE_RULE_ACTION_EXCLUDE, FILE_RULE_ACTION_INCLUDE, FILE_RULE_ACTION_UNSPECIFIED, FileLocation

FileLocation.__module__ = "dokime"
FileLocation.__qualname__ = "FileLocation"
from dokime._wire.model.v1.files_pb2 import FileRule

FileRule.__module__ = "dokime"
FileRule.__qualname__ = "FileRule"
from dokime._wire.model.v1.files_pb2 import FileRuleAction, FileSelection

FileSelection.__module__ = "dokime"
FileSelection.__qualname__ = "FileSelection"
from dokime._wire.model.v1.files_pb2 import FileSource

FileSource.__module__ = "dokime"
FileSource.__qualname__ = "FileSource"
from dokime._wire.model.v1.files_pb2 import FtpLocation

FtpLocation.__module__ = "dokime"
FtpLocation.__qualname__ = "FtpLocation"
from dokime._wire.model.v1.metadata_pb2 import Metadata

Metadata.__module__ = "dokime"
Metadata.__qualname__ = "Metadata"
from dokime._wire.model.v1.parameters_pb2 import (
    DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL,
    DYNAMIC_DEFAULT_KEY_OPERATOR_NAME,
    DYNAMIC_DEFAULT_KEY_UNSPECIFIED,
    DynamicDefault,
)

DynamicDefault.__module__ = "dokime"
DynamicDefault.__qualname__ = "DynamicDefault"
from dokime._wire.model.v1.parameters_pb2 import DynamicDefaultKey, FileParameter

FileParameter.__module__ = "dokime"
FileParameter.__qualname__ = "FileParameter"
from dokime._wire.model.v1.parameters_pb2 import IntegerRange

IntegerRange.__module__ = "dokime"
IntegerRange.__qualname__ = "IntegerRange"
from dokime._wire.model.v1.parameters_pb2 import LinearSpace

LinearSpace.__module__ = "dokime"
LinearSpace.__qualname__ = "LinearSpace"
from dokime._wire.model.v1.parameters_pb2 import NumberRange

NumberRange.__module__ = "dokime"
NumberRange.__qualname__ = "NumberRange"
from dokime._wire.model.v1.parameters_pb2 import (
    PARAMETER_BINARY_OPERATOR_ADD,
    PARAMETER_BINARY_OPERATOR_DIVIDE,
    PARAMETER_BINARY_OPERATOR_MULTIPLY,
    PARAMETER_BINARY_OPERATOR_SUBTRACT,
    PARAMETER_BINARY_OPERATOR_UNSPECIFIED,
    PARAMETER_UNARY_OPERATOR_NEGATE,
    PARAMETER_UNARY_OPERATOR_UNSPECIFIED,
    Parameter,
)

Parameter.__module__ = "dokime"
Parameter.__qualname__ = "Parameter"
from dokime._wire.model.v1.parameters_pb2 import ParameterBinary

ParameterBinary.__module__ = "dokime"
ParameterBinary.__qualname__ = "ParameterBinary"
from dokime._wire.model.v1.parameters_pb2 import ParameterBinaryOperator, ParameterExpansion

ParameterExpansion.__module__ = "dokime"
ParameterExpansion.__qualname__ = "ParameterExpansion"
from dokime._wire.model.v1.parameters_pb2 import ParameterExpression

ParameterExpression.__module__ = "dokime"
ParameterExpression.__qualname__ = "ParameterExpression"
from dokime._wire.model.v1.parameters_pb2 import ParameterExpressions

ParameterExpressions.__module__ = "dokime"
ParameterExpressions.__qualname__ = "ParameterExpressions"
from dokime._wire.model.v1.parameters_pb2 import ParameterUnary

ParameterUnary.__module__ = "dokime"
ParameterUnary.__qualname__ = "ParameterUnary"
from dokime._wire.model.v1.parameters_pb2 import ParameterUnaryOperator
from dokime._wire.model.v1.plots_pb2 import (
    AXIS_SCALE_LINEAR,
    AXIS_SCALE_LOG,
    AXIS_SCALE_UNSPECIFIED,
    BAR_ARRANGEMENT_GROUPED,
    BAR_ARRANGEMENT_STACKED,
    BAR_ARRANGEMENT_UNSPECIFIED,
    CONTOUR_RENDER_BANDS,
    CONTOUR_RENDER_CELLS,
    CONTOUR_RENDER_UNSPECIFIED,
    DOWNSAMPLE_AUTO,
    DOWNSAMPLE_LTTB,
    DOWNSAMPLE_MIN_MAX,
    DOWNSAMPLE_RAW,
    DOWNSAMPLE_UNSPECIFIED,
    FFT_DETREND_MEAN,
    FFT_DETREND_NONE,
    FFT_DETREND_UNSPECIFIED,
    FFT_WINDOW_HANN,
    FFT_WINDOW_RECTANGULAR,
    FFT_WINDOW_UNSPECIFIED,
    FREQUENCY_LEVEL_UNIT_DB,
    FREQUENCY_LEVEL_UNIT_DBM,
    FREQUENCY_LEVEL_UNIT_DBU,
    FREQUENCY_LEVEL_UNIT_LINEAR,
    FREQUENCY_LEVEL_UNIT_UNSPECIFIED,
    AxisScale,
    BarArrangement,
    ContourRender,
    Downsample,
    FftDetrend,
    FftWindow,
    FrequencyLevelUnit,
    GridAxisEntry,
)

GridAxisEntry.__module__ = "dokime"
GridAxisEntry.__qualname__ = "GridAxisEntry"
from dokime._wire.model.v1.plots_pb2 import GridChart

GridChart.__module__ = "dokime"
GridChart.__qualname__ = "GridChart"
from dokime._wire.model.v1.plots_pb2 import GridDefinition

GridDefinition.__module__ = "dokime"
GridDefinition.__qualname__ = "GridDefinition"
from dokime._wire.model.v1.plots_pb2 import GridHighlight

GridHighlight.__module__ = "dokime"
GridHighlight.__qualname__ = "GridHighlight"
from dokime._wire.model.v1.plots_pb2 import (
    LINE_STYLE_LINE,
    LINE_STYLE_LINE_POINTS,
    LINE_STYLE_POINTS,
    LINE_STYLE_STEP,
    LINE_STYLE_UNSPECIFIED,
    PLOT_KIND_BAR,
    PLOT_KIND_BOX_PLOT,
    PLOT_KIND_CONTOUR,
    PLOT_KIND_FREQUENCY_SERIES,
    PLOT_KIND_GRID_CHART,
    PLOT_KIND_HISTOGRAM,
    PLOT_KIND_PIE,
    PLOT_KIND_TIME_SERIES,
    PLOT_KIND_TRAJECTORY,
    PLOT_KIND_UNSPECIFIED,
    PLOT_KIND_XY,
    PLOT_STYLE_LINE,
    PLOT_STYLE_LINE_POINTS,
    PLOT_STYLE_POINTS,
    PLOT_STYLE_STEP,
    PLOT_STYLE_UNSPECIFIED,
    LineStyle,
    Plot,
)

Plot.__module__ = "dokime"
Plot.__qualname__ = "Plot"
from dokime._wire.model.v1.plots_pb2 import PlotFftOptions

PlotFftOptions.__module__ = "dokime"
PlotFftOptions.__qualname__ = "PlotFftOptions"
from dokime._wire.model.v1.plots_pb2 import PlotFilter

PlotFilter.__module__ = "dokime"
PlotFilter.__qualname__ = "PlotFilter"
from dokime._wire.model.v1.plots_pb2 import PlotHistogramOptions

PlotHistogramOptions.__module__ = "dokime"
PlotHistogramOptions.__qualname__ = "PlotHistogramOptions"
from dokime._wire.model.v1.plots_pb2 import PlotIntervalOptions

PlotIntervalOptions.__module__ = "dokime"
PlotIntervalOptions.__qualname__ = "PlotIntervalOptions"
from dokime._wire.model.v1.plots_pb2 import (
    REDUCTION_MAX,
    REDUCTION_MEAN,
    REDUCTION_MIN,
    REDUCTION_UNSPECIFIED,
    PlotKind,
    PlotStyle,
    Reduction,
    ReferenceBand,
)

ReferenceBand.__module__ = "dokime"
ReferenceBand.__qualname__ = "ReferenceBand"
from dokime._wire.model.v1.plots_pb2 import ReferenceLine

ReferenceLine.__module__ = "dokime"
ReferenceLine.__qualname__ = "ReferenceLine"
from dokime._wire.model.v1.plots_pb2 import XY_INTERVAL_SIGMA, XY_INTERVAL_UNSPECIFIED, XyInterval
from dokime._wire.model.v1.prompts_pb2 import (
    ANSWER_OPERATOR_AND,
    ANSWER_OPERATOR_CONTAINS,
    ANSWER_OPERATOR_ENDS_WITH,
    ANSWER_OPERATOR_EQ,
    ANSWER_OPERATOR_GE,
    ANSWER_OPERATOR_GT,
    ANSWER_OPERATOR_LE,
    ANSWER_OPERATOR_LT,
    ANSWER_OPERATOR_NE,
    ANSWER_OPERATOR_OR,
    ANSWER_OPERATOR_STARTS_WITH,
    ANSWER_OPERATOR_UNSPECIFIED,
    AnswerBinaryPredicate,
)

AnswerBinaryPredicate.__module__ = "dokime"
AnswerBinaryPredicate.__qualname__ = "AnswerBinaryPredicate"
from dokime._wire.model.v1.prompts_pb2 import AnswerOperator, AnswerPredicate

AnswerPredicate.__module__ = "dokime"
AnswerPredicate.__qualname__ = "AnswerPredicate"
from dokime._wire.model.v1.prompts_pb2 import AnswerRule

AnswerRule.__module__ = "dokime"
AnswerRule.__qualname__ = "AnswerRule"
from dokime._wire.model.v1.prompts_pb2 import (
    PROMPT_ACTION_EMPHASIS_DANGER,
    PROMPT_ACTION_EMPHASIS_PRIMARY,
    PROMPT_ACTION_EMPHASIS_SECONDARY,
    PROMPT_ACTION_EMPHASIS_UNSPECIFIED,
    PROMPT_ARCHETYPE_CHOICE,
    PROMPT_ARCHETYPE_CONFIRMATION,
    PROMPT_ARCHETYPE_FORM,
    PROMPT_ARCHETYPE_STATUS,
    PROMPT_ARCHETYPE_UNSPECIFIED,
    PROMPT_FIELD_KIND_BOOLEAN,
    PROMPT_FIELD_KIND_CHOICE,
    PROMPT_FIELD_KIND_FLOAT,
    PROMPT_FIELD_KIND_INTEGER,
    PROMPT_FIELD_KIND_TEXT,
    PROMPT_FIELD_KIND_UNSPECIFIED,
    Prompt,
)

Prompt.__module__ = "dokime"
Prompt.__qualname__ = "Prompt"
from dokime._wire.model.v1.prompts_pb2 import PromptAction

PromptAction.__module__ = "dokime"
PromptAction.__qualname__ = "PromptAction"
from dokime._wire.model.v1.prompts_pb2 import PromptActionEmphasis, PromptArchetype, PromptDefinitions

PromptDefinitions.__module__ = "dokime"
PromptDefinitions.__qualname__ = "PromptDefinitions"
from dokime._wire.model.v1.prompts_pb2 import PromptField

PromptField.__module__ = "dokime"
PromptField.__qualname__ = "PromptField"
from dokime._wire.model.v1.prompts_pb2 import PromptFieldKind, PromptFieldValidation

PromptFieldValidation.__module__ = "dokime"
PromptFieldValidation.__qualname__ = "PromptFieldValidation"
from dokime._wire.model.v1.prompts_pb2 import PromptValues

PromptValues.__module__ = "dokime"
PromptValues.__qualname__ = "PromptValues"
from dokime._wire.model.v1.record_contracts_pb2 import Listeners

Listeners.__module__ = "dokime"
Listeners.__qualname__ = "Listeners"
from dokime._wire.model.v1.record_contracts_pb2 import RecordContract

RecordContract.__module__ = "dokime"
RecordContract.__qualname__ = "RecordContract"
from dokime._wire.model.v1.record_contracts_pb2 import RecordContracts

RecordContracts.__module__ = "dokime"
RecordContracts.__qualname__ = "RecordContracts"
from dokime._wire.model.v1.record_contracts_pb2 import RecordListener

RecordListener.__module__ = "dokime"
RecordListener.__qualname__ = "RecordListener"
from dokime._wire.model.v1.scalar_pb2 import ExactNumber

ExactNumber.__module__ = "dokime"
ExactNumber.__qualname__ = "ExactNumber"
from dokime._wire.model.v1.scalar_pb2 import MapValue

MapValue.__module__ = "dokime"
MapValue.__qualname__ = "MapValue"
from dokime._wire.model.v1.scalar_pb2 import Null

Null.__module__ = "dokime"
Null.__qualname__ = "Null"
from dokime._wire.model.v1.scalar_pb2 import (
    VALUE_TYPE_BOOLEAN,
    VALUE_TYPE_BYTES,
    VALUE_TYPE_DATETIME,
    VALUE_TYPE_INTEGER,
    VALUE_TYPE_LIST,
    VALUE_TYPE_NUMBER,
    VALUE_TYPE_TEXT,
    VALUE_TYPE_UNSIGNED_INTEGER,
    VALUE_TYPE_UNSPECIFIED,
    Value,
)

Value.__module__ = "dokime"
Value.__qualname__ = "Value"
from dokime._wire.model.v1.scalar_pb2 import ValueList

ValueList.__module__ = "dokime"
ValueList.__qualname__ = "ValueList"
from dokime._wire.model.v1.scalar_pb2 import ValueType
from dokime._wire.model.v1.secrets_pb2 import SecretDeclaration

SecretDeclaration.__module__ = "dokime"
SecretDeclaration.__qualname__ = "SecretDeclaration"
from dokime._wire.model.v1.secrets_pb2 import SecretsDeclarations

SecretsDeclarations.__module__ = "dokime"
SecretsDeclarations.__qualname__ = "SecretsDeclarations"
from dokime._wire.model.v1.suites_pb2 import Suite

Suite.__module__ = "dokime"
Suite.__qualname__ = "Suite"
from dokime._wire.model.v1.suites_pb2 import Suites

Suites.__module__ = "dokime"
Suites.__qualname__ = "Suites"
from dokime._wire.model.v1.suites_pb2 import TestCaseTemplate

TestCaseTemplate.__module__ = "dokime"
TestCaseTemplate.__qualname__ = "TestCaseTemplate"
from dokime._wire.model.v1.traceability_pb2 import TraceabilityEntry

TraceabilityEntry.__module__ = "dokime"
TraceabilityEntry.__qualname__ = "TraceabilityEntry"
from harmos import (
    ALL,
    CALLERS_UNSPECIFIED,
    CLASSIFICATION_CANCELLED,
    CLASSIFICATION_PERMANENT,
    CLASSIFICATION_REFUSED,
    CLASSIFICATION_TIMED_OUT,
    CLASSIFICATION_TRANSIENT,
    CLASSIFICATION_UNSPECIFIED,
    CLIENTS,
    PLUGINS,
    TRANSFER_FRAMING_BYTES,
    TRANSFER_FRAMING_UNSPECIFIED,
    UPLOAD_STATE_ABANDONED,
    UPLOAD_STATE_FINISHED,
    UPLOAD_STATE_PRODUCING,
    UPLOAD_STATE_UNSPECIFIED,
    VISIBILITY_PRIVATE,
    VISIBILITY_PROTECTED,
    VISIBILITY_PUBLIC,
    VISIBILITY_UNSPECIFIED,
    Callers,
    Classification,
    Declarations,
    DiscoveredContract,
    DiscoveredOperation,
    File,
    FileMetadata,
    FileReference,
    Grant,
    Grpc,
    History,
    InstanceState,
    InvocationsCancelResponse,
    InvocationsGetResponse,
    InvocationsWatchResponse,
    MessageSettings,
    OperationContract,
    OperationPolicy,
    OperationsGetResponse,
    OperationsListResponse,
    PageRequest,
    Position,
    Protobuf,
    ReadCut,
    Receipt,
    Requirement,
    ResourceRevision,
    RetryPolicy,
    SettingDeclaration,
    SettingsDeclarations,
    SettingsListResponse,
    SettingValue,
    SettingValues,
    StorageTransferSettings,
    StreamDeclaration,
    TimeGetResponse,
    TimeSleepResponse,
    TransferFraming,
    Upload,
    UploadState,
    Visibility,
)

from ._exports.shared import harmos

__all__ = [
    "AGGREGATE_COUNT",
    "AGGREGATE_FIRST",
    "AGGREGATE_LAST",
    "AGGREGATE_MAX",
    "AGGREGATE_MEAN",
    "AGGREGATE_MEDIAN",
    "AGGREGATE_MIN",
    "AGGREGATE_PERCENTILE",
    "AGGREGATE_RMS",
    "AGGREGATE_STDDEV",
    "AGGREGATE_SUM",
    "AGGREGATE_UNSPECIFIED",
    "ALL",
    "ANSWER_OPERATOR_AND",
    "ANSWER_OPERATOR_CONTAINS",
    "ANSWER_OPERATOR_ENDS_WITH",
    "ANSWER_OPERATOR_EQ",
    "ANSWER_OPERATOR_GE",
    "ANSWER_OPERATOR_GT",
    "ANSWER_OPERATOR_LE",
    "ANSWER_OPERATOR_LT",
    "ANSWER_OPERATOR_NE",
    "ANSWER_OPERATOR_OR",
    "ANSWER_OPERATOR_STARTS_WITH",
    "ANSWER_OPERATOR_UNSPECIFIED",
    "ARTIFACT_STATE_CANCELLED",
    "ARTIFACT_STATE_COMPLETE",
    "ARTIFACT_STATE_FAILED",
    "ARTIFACT_STATE_REJECTED",
    "ARTIFACT_STATE_UNSPECIFIED",
    "AXIS_SCALE_LINEAR",
    "AXIS_SCALE_LOG",
    "AXIS_SCALE_UNSPECIFIED",
    "BAR_ARRANGEMENT_GROUPED",
    "BAR_ARRANGEMENT_STACKED",
    "BAR_ARRANGEMENT_UNSPECIFIED",
    "CALLERS_UNSPECIFIED",
    "CAPTURE_GAP_REASON_REPLACED",
    "CAPTURE_GAP_REASON_RESUME_UNSUPPORTED",
    "CAPTURE_GAP_REASON_TRUNCATED",
    "CAPTURE_GAP_REASON_UNSPECIFIED",
    "CAPTURE_GAP_REASON_VANISHED",
    "CLASSIFICATION_CANCELLED",
    "CLASSIFICATION_PERMANENT",
    "CLASSIFICATION_REFUSED",
    "CLASSIFICATION_TIMED_OUT",
    "CLASSIFICATION_TRANSIENT",
    "CLASSIFICATION_UNSPECIFIED",
    "CLIENTS",
    "CONTOUR_RENDER_BANDS",
    "CONTOUR_RENDER_CELLS",
    "CONTOUR_RENDER_UNSPECIFIED",
    "DOWNSAMPLE_AUTO",
    "DOWNSAMPLE_LTTB",
    "DOWNSAMPLE_MIN_MAX",
    "DOWNSAMPLE_RAW",
    "DOWNSAMPLE_UNSPECIFIED",
    "DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL",
    "DYNAMIC_DEFAULT_KEY_OPERATOR_NAME",
    "DYNAMIC_DEFAULT_KEY_UNSPECIFIED",
    "EVIDENCE_CARD_ARTIFACTS",
    "EVIDENCE_CARD_CRITERIA",
    "EVIDENCE_CARD_HEALTH",
    "EVIDENCE_CARD_MEDIA",
    "EVIDENCE_CARD_NOTES",
    "EVIDENCE_CARD_RECORDS",
    "EVIDENCE_CARD_STEPS",
    "EVIDENCE_CARD_STREAM_CONTEXT",
    "EVIDENCE_CARD_TRACEABILITY",
    "EVIDENCE_CARD_UNSPECIFIED",
    "FFT_DETREND_MEAN",
    "FFT_DETREND_NONE",
    "FFT_DETREND_UNSPECIFIED",
    "FFT_WINDOW_HANN",
    "FFT_WINDOW_RECTANGULAR",
    "FFT_WINDOW_UNSPECIFIED",
    "FIELD_TYPE_ARRAY",
    "FIELD_TYPE_BOOLEAN",
    "FIELD_TYPE_DATETIME",
    "FIELD_TYPE_FLOAT64",
    "FIELD_TYPE_INT64",
    "FIELD_TYPE_INTEGER",
    "FIELD_TYPE_OBJECT",
    "FIELD_TYPE_SEVERITY",
    "FIELD_TYPE_TEXT",
    "FIELD_TYPE_UINT64",
    "FIELD_TYPE_UNSPECIFIED",
    "FILE_RULE_ACTION_EXCLUDE",
    "FILE_RULE_ACTION_INCLUDE",
    "FILE_RULE_ACTION_UNSPECIFIED",
    "FILTER_OPERATOR_CONTAINS",
    "FILTER_OPERATOR_EQ",
    "FILTER_OPERATOR_EXISTS",
    "FILTER_OPERATOR_GE",
    "FILTER_OPERATOR_GT",
    "FILTER_OPERATOR_LE",
    "FILTER_OPERATOR_LT",
    "FILTER_OPERATOR_NE",
    "FILTER_OPERATOR_UNSPECIFIED",
    "FREQUENCY_LEVEL_UNIT_DB",
    "FREQUENCY_LEVEL_UNIT_DBM",
    "FREQUENCY_LEVEL_UNIT_DBU",
    "FREQUENCY_LEVEL_UNIT_LINEAR",
    "FREQUENCY_LEVEL_UNIT_UNSPECIFIED",
    "LINE_STYLE_LINE",
    "LINE_STYLE_LINE_POINTS",
    "LINE_STYLE_POINTS",
    "LINE_STYLE_STEP",
    "LINE_STYLE_UNSPECIFIED",
    "PARAMETER_BINARY_OPERATOR_ADD",
    "PARAMETER_BINARY_OPERATOR_DIVIDE",
    "PARAMETER_BINARY_OPERATOR_MULTIPLY",
    "PARAMETER_BINARY_OPERATOR_SUBTRACT",
    "PARAMETER_BINARY_OPERATOR_UNSPECIFIED",
    "PARAMETER_UNARY_OPERATOR_NEGATE",
    "PARAMETER_UNARY_OPERATOR_UNSPECIFIED",
    "PLOT_KIND_BAR",
    "PLOT_KIND_BOX_PLOT",
    "PLOT_KIND_CONTOUR",
    "PLOT_KIND_FREQUENCY_SERIES",
    "PLOT_KIND_GRID_CHART",
    "PLOT_KIND_HISTOGRAM",
    "PLOT_KIND_PIE",
    "PLOT_KIND_TIME_SERIES",
    "PLOT_KIND_TRAJECTORY",
    "PLOT_KIND_UNSPECIFIED",
    "PLOT_KIND_XY",
    "PLOT_STYLE_LINE",
    "PLOT_STYLE_LINE_POINTS",
    "PLOT_STYLE_POINTS",
    "PLOT_STYLE_STEP",
    "PLOT_STYLE_UNSPECIFIED",
    "PLUGINS",
    "PROMPT_ACTION_EMPHASIS_DANGER",
    "PROMPT_ACTION_EMPHASIS_PRIMARY",
    "PROMPT_ACTION_EMPHASIS_SECONDARY",
    "PROMPT_ACTION_EMPHASIS_UNSPECIFIED",
    "PROMPT_ARCHETYPE_CHOICE",
    "PROMPT_ARCHETYPE_CONFIRMATION",
    "PROMPT_ARCHETYPE_FORM",
    "PROMPT_ARCHETYPE_STATUS",
    "PROMPT_ARCHETYPE_UNSPECIFIED",
    "PROMPT_FIELD_KIND_BOOLEAN",
    "PROMPT_FIELD_KIND_CHOICE",
    "PROMPT_FIELD_KIND_FLOAT",
    "PROMPT_FIELD_KIND_INTEGER",
    "PROMPT_FIELD_KIND_TEXT",
    "PROMPT_FIELD_KIND_UNSPECIFIED",
    "REDUCTION_MAX",
    "REDUCTION_MEAN",
    "REDUCTION_MIN",
    "REDUCTION_UNSPECIFIED",
    "SEVERITY_DEBUG",
    "SEVERITY_FAILURE",
    "SEVERITY_INFO",
    "SEVERITY_UNSPECIFIED",
    "SEVERITY_WARNING",
    "STREAM_MATCH_OPERATOR_EQ",
    "STREAM_MATCH_OPERATOR_EXISTS",
    "STREAM_MATCH_OPERATOR_GE",
    "STREAM_MATCH_OPERATOR_GT",
    "STREAM_MATCH_OPERATOR_LE",
    "STREAM_MATCH_OPERATOR_LT",
    "STREAM_MATCH_OPERATOR_UNSPECIFIED",
    "STREAM_MATCH_OPERATOR_WITHIN",
    "TERMINATION_CANCELLED",
    "TERMINATION_COMPLETED",
    "TERMINATION_FAILED",
    "TERMINATION_INTERRUPTED",
    "TERMINATION_UNSPECIFIED",
    "TRANSFER_FRAMING_BYTES",
    "TRANSFER_FRAMING_UNSPECIFIED",
    "UPLOAD_STATE_ABANDONED",
    "UPLOAD_STATE_FINISHED",
    "UPLOAD_STATE_PRODUCING",
    "UPLOAD_STATE_UNSPECIFIED",
    "VALUE_TYPE_BOOLEAN",
    "VALUE_TYPE_BYTES",
    "VALUE_TYPE_DATETIME",
    "VALUE_TYPE_INTEGER",
    "VALUE_TYPE_LIST",
    "VALUE_TYPE_NUMBER",
    "VALUE_TYPE_TEXT",
    "VALUE_TYPE_UNSIGNED_INTEGER",
    "VALUE_TYPE_UNSPECIFIED",
    "VISIBILITY_PRIVATE",
    "VISIBILITY_PROTECTED",
    "VISIBILITY_PUBLIC",
    "VISIBILITY_UNSPECIFIED",
    "XY_INTERVAL_SIGMA",
    "XY_INTERVAL_UNSPECIFIED",
    "Aggregate",
    "AggregateExpression",
    "AnswerBinaryPredicate",
    "AnswerOperator",
    "AnswerPredicate",
    "AnswerRule",
    "ArtifactMetadata",
    "ArtifactScope",
    "ArtifactState",
    "ArtifactsCreateResponse",
    "AxisScale",
    "BarArrangement",
    "Callers",
    "CaptureGap",
    "CaptureGapReason",
    "CapturedArtifact",
    "CapturedErrorCause",
    "CapturedRecord",
    "CapturedRecordError",
    "CapturedRecordSelection",
    "Card",
    "CardLayout",
    "Classification",
    "ContourRender",
    "Criterion",
    "CycleScope",
    "DataExpression",
    "Declarations",
    "DiscoveredContract",
    "DiscoveredOperation",
    "Document",
    "DocumentFilter",
    "Downsample",
    "DynamicDefault",
    "DynamicDefaultKey",
    "EvidenceCard",
    "EvidenceCondition",
    "EvidenceConditions",
    "EvidenceOrigin",
    "EvidenceOriginKind",
    "EvidenceScope",
    "EvidenceStatisticsResponse",
    "ExactNumber",
    "ExecutionProgressValue",
    "FftDetrend",
    "FftWindow",
    "Field",
    "FieldReference",
    "FieldType",
    "File",
    "FileBatch",
    "FileCompletion",
    "FileLocation",
    "FileMetadata",
    "FileParameter",
    "FileReference",
    "FileRule",
    "FileRuleAction",
    "FileSelection",
    "FileSource",
    "FilesDeleteResponse",
    "FilesGetResponse",
    "FilesReadResponse",
    "FilesWatchResponse",
    "FilterOperator",
    "FrequencyLevelUnit",
    "FtpLocation",
    "Grant",
    "GridAxisEntry",
    "GridChart",
    "GridDefinition",
    "GridHighlight",
    "Grpc",
    "History",
    "Import",
    "ImportSource",
    "InstanceState",
    "IntegerRange",
    "InvocationsCancelResponse",
    "InvocationsGetResponse",
    "InvocationsWatchResponse",
    "LineStyle",
    "LinearSpace",
    "Listeners",
    "MapValue",
    "MessageSettings",
    "Metadata",
    "Null",
    "NumberRange",
    "OperationContract",
    "OperationPolicy",
    "OperationsGetResponse",
    "OperationsListResponse",
    "Origin",
    "Output",
    "PageRequest",
    "Parameter",
    "ParameterBinary",
    "ParameterBinaryOperator",
    "ParameterExpansion",
    "ParameterExpression",
    "ParameterExpressions",
    "ParameterUnary",
    "ParameterUnaryOperator",
    "Plot",
    "PlotFftOptions",
    "PlotFilter",
    "PlotHistogramOptions",
    "PlotIntervalOptions",
    "PlotKind",
    "PlotStyle",
    "PluginRun",
    "Position",
    "ProgressGetResponse",
    "Prompt",
    "PromptAction",
    "PromptActionEmphasis",
    "PromptArchetype",
    "PromptDefinitions",
    "PromptField",
    "PromptFieldKind",
    "PromptFieldValidation",
    "PromptValues",
    "Protobuf",
    "ReadCut",
    "Receipt",
    "RecordContract",
    "RecordContracts",
    "RecordListener",
    "RecordMatch",
    "RecordScope",
    "RecordsListResponse",
    "Reduction",
    "ReferenceBand",
    "ReferenceLine",
    "Requirement",
    "ResourceRevision",
    "RetryPolicy",
    "Schema",
    "SecretDeclaration",
    "SecretWritable",
    "SecretsDeclarations",
    "SecretsGetResponse",
    "SectionRepeat",
    "SettingDeclaration",
    "SettingValue",
    "SettingValues",
    "SettingsDeclarations",
    "SettingsListResponse",
    "Severity",
    "Statistics",
    "Step",
    "StorageTransferSettings",
    "StreamDeclaration",
    "StreamMatch",
    "StreamMatchOperator",
    "Suite",
    "Suites",
    "Summary",
    "SummaryTile",
    "Table",
    "TableRow",
    "Termination",
    "TestCase",
    "TestCaseTemplate",
    "TimeGetResponse",
    "TimeSleepResponse",
    "TraceabilityEntry",
    "TraceabilityGetResponse",
    "TraceabilitySnapshot",
    "TransferFraming",
    "TransformExpression",
    "Upload",
    "UploadState",
    "UploadsAppendResponse",
    "UploadsCreateResponse",
    "UploadsDeleteResponse",
    "UploadsGetResponse",
    "UploadsPublishResponse",
    "UploadsResumeResponse",
    "Value",
    "ValueList",
    "ValueType",
    "Visibility",
    "XyInterval",
    "harmos",
    "model",
]
