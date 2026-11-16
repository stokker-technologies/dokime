# @generated from protocol descriptors; do not edit.
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
    FailureValue,
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
from harmos import Section as HarmosSection
from harmos import SettingsListResponse as PluginsSettingsListResponse

from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateResponse as ContentArtifactsCreateResponse
from dokime._wire.host.v1.evidence.records_pb2 import CapturedErrorCause as EvidenceCapturedErrorCause
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecord as EvidenceCapturedRecord
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecordError as EvidenceCapturedRecordError
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecordSelection as EvidenceCapturedRecordSelection
from dokime._wire.host.v1.evidence.records_pb2 import CycleScope as EvidenceCycleScope
from dokime._wire.host.v1.evidence.records_pb2 import RecordScope as EvidenceRecordScope
from dokime._wire.host.v1.evidence.records_pb2 import RecordsListResponse as EvidenceRecordsListResponse
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityGetResponse as EvidenceTraceabilityGetResponse
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilitySnapshot as EvidenceTraceabilitySnapshot
from dokime._wire.host.v1.evidence_pb2 import EvidenceStatisticsResponse, Output, Statistics
from dokime._wire.host.v1.executions.progress_pb2 import ProgressGetResponse as ExecutionsProgressGetResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsAppendResponse as FilesUploadsAppendResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsCreateResponse as FilesUploadsCreateResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsDeleteResponse as FilesUploadsDeleteResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsGetResponse as FilesUploadsGetResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsPublishResponse as FilesUploadsPublishResponse
from dokime._wire.host.v1.files.uploads_pb2 import UploadsResumeResponse as FilesUploadsResumeResponse
from dokime._wire.host.v1.files_pb2 import (
    FileBatch,
    FileCompletion,
    FilesDeleteResponse,
    FilesGetResponse,
    FilesReadResponse,
    FilesWatchResponse,
)
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
    FieldType,
    Schema,
)
from dokime._wire.host.v1.secrets_pb2 import SecretsGetResponse, SecretWritable
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
    Card,
    CardLayout,
    Document,
    EvidenceCard,
    Section,
    SectionRepeat,
    Summary,
    SummaryTile,
    Table,
    TableRow,
)
from dokime._wire.model.v1.evidence_pb2 import (
    ARTIFACT_STATE_CANCELLED,
    ARTIFACT_STATE_COMPLETE,
    ARTIFACT_STATE_FAILED,
    ARTIFACT_STATE_REJECTED,
    ARTIFACT_STATE_UNSPECIFIED,
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
    ArtifactMetadata,
    ArtifactScope,
    ArtifactState,
    EvidenceCondition,
    EvidenceConditions,
    EvidenceOrigin,
    EvidenceOriginKind,
    EvidenceScope,
    RecordMatch,
    Severity,
    StreamMatch,
    StreamMatchOperator,
    Window,
)
from dokime._wire.model.v1.execution_pb2 import (
    TERMINATION_CANCELLED,
    TERMINATION_COMPLETED,
    TERMINATION_FAILED,
    TERMINATION_INTERRUPTED,
    TERMINATION_UNSPECIFIED,
    Criterion,
    ExecutionProgressValue,
    Import,
    ImportSource,
    Origin,
    PluginRun,
    Step,
    Termination,
    TestCase,
)
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
    FILTER_OPERATOR_CONTAINS,
    FILTER_OPERATOR_EQ,
    FILTER_OPERATOR_EXISTS,
    FILTER_OPERATOR_GE,
    FILTER_OPERATOR_GT,
    FILTER_OPERATOR_LE,
    FILTER_OPERATOR_LT,
    FILTER_OPERATOR_NE,
    FILTER_OPERATOR_UNSPECIFIED,
    Aggregate,
    AggregateExpression,
    DataExpression,
    DocumentFilter,
    FieldReference,
    FilterOperator,
    TransformExpression,
)
from dokime._wire.model.v1.files_pb2 import (
    CAPTURE_GAP_REASON_REPLACED,
    CAPTURE_GAP_REASON_RESUME_UNSUPPORTED,
    CAPTURE_GAP_REASON_TRUNCATED,
    CAPTURE_GAP_REASON_UNSPECIFIED,
    CAPTURE_GAP_REASON_VANISHED,
    FILE_RULE_ACTION_EXCLUDE,
    FILE_RULE_ACTION_INCLUDE,
    FILE_RULE_ACTION_UNSPECIFIED,
    CapturedArtifact,
    CaptureGap,
    CaptureGapReason,
    FileLocation,
    FileRule,
    FileRuleAction,
    FileSelection,
    FileSource,
    FtpLocation,
)
from dokime._wire.model.v1.metadata_pb2 import Metadata
from dokime._wire.model.v1.parameters_pb2 import (
    DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL,
    DYNAMIC_DEFAULT_KEY_OPERATOR_NAME,
    DYNAMIC_DEFAULT_KEY_UNSPECIFIED,
    PARAMETER_BINARY_OPERATOR_ADD,
    PARAMETER_BINARY_OPERATOR_DIVIDE,
    PARAMETER_BINARY_OPERATOR_MULTIPLY,
    PARAMETER_BINARY_OPERATOR_SUBTRACT,
    PARAMETER_BINARY_OPERATOR_UNSPECIFIED,
    PARAMETER_UNARY_OPERATOR_NEGATE,
    PARAMETER_UNARY_OPERATOR_UNSPECIFIED,
    DynamicDefault,
    DynamicDefaultKey,
    FileParameter,
    IntegerRange,
    LinearSpace,
    NumberRange,
    Parameter,
    ParameterBinary,
    ParameterBinaryOperator,
    ParameterExpansion,
    ParameterExpression,
    ParameterExpressions,
    ParameterUnary,
    ParameterUnaryOperator,
)
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
    REDUCTION_MAX,
    REDUCTION_MEAN,
    REDUCTION_MIN,
    REDUCTION_UNSPECIFIED,
    XY_INTERVAL_SIGMA,
    XY_INTERVAL_UNSPECIFIED,
    AxisScale,
    BarArrangement,
    ContourRender,
    Downsample,
    FftDetrend,
    FftWindow,
    FrequencyLevelUnit,
    GridAxisEntry,
    GridChart,
    GridDefinition,
    GridHighlight,
    LineStyle,
    Plot,
    PlotFftOptions,
    PlotFilter,
    PlotHistogramOptions,
    PlotIntervalOptions,
    PlotKind,
    PlotStyle,
    Reduction,
    ReferenceBand,
    ReferenceLine,
    XyInterval,
)
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
    AnswerBinaryPredicate,
    AnswerOperator,
    AnswerPredicate,
    AnswerRule,
    Prompt,
    PromptAction,
    PromptActionEmphasis,
    PromptArchetype,
    PromptDefinitions,
    PromptField,
    PromptFieldKind,
    PromptFieldValidation,
    PromptValues,
)
from dokime._wire.model.v1.record_contracts_pb2 import Listeners, RecordContract, RecordContracts, RecordListener
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
    ExactNumber,
    MapValue,
    Null,
    Value,
    ValueList,
    ValueType,
)
from dokime._wire.model.v1.secrets_pb2 import SecretDeclaration, SecretsDeclarations
from dokime._wire.model.v1.suites_pb2 import Suite, Suites, TestCaseTemplate
from dokime._wire.model.v1.traceability_pb2 import TraceabilityEntry

ContentArtifactsCreateResponse.__module__ = "dokime"
ContentArtifactsCreateResponse.__qualname__ = "ContentArtifactsCreateResponse"
EvidenceCapturedErrorCause.__module__ = "dokime"
EvidenceCapturedErrorCause.__qualname__ = "EvidenceCapturedErrorCause"
EvidenceCapturedRecord.__module__ = "dokime"
EvidenceCapturedRecord.__qualname__ = "EvidenceCapturedRecord"
EvidenceCapturedRecordError.__module__ = "dokime"
EvidenceCapturedRecordError.__qualname__ = "EvidenceCapturedRecordError"
EvidenceCapturedRecordSelection.__module__ = "dokime"
EvidenceCapturedRecordSelection.__qualname__ = "EvidenceCapturedRecordSelection"
EvidenceCycleScope.__module__ = "dokime"
EvidenceCycleScope.__qualname__ = "EvidenceCycleScope"
EvidenceRecordScope.__module__ = "dokime"
EvidenceRecordScope.__qualname__ = "EvidenceRecordScope"
EvidenceRecordsListResponse.__module__ = "dokime"
EvidenceRecordsListResponse.__qualname__ = "EvidenceRecordsListResponse"
EvidenceTraceabilityGetResponse.__module__ = "dokime"
EvidenceTraceabilityGetResponse.__qualname__ = "EvidenceTraceabilityGetResponse"
EvidenceTraceabilitySnapshot.__module__ = "dokime"
EvidenceTraceabilitySnapshot.__qualname__ = "EvidenceTraceabilitySnapshot"
EvidenceStatisticsResponse.__module__ = "dokime"
EvidenceStatisticsResponse.__qualname__ = "EvidenceStatisticsResponse"
Output.__module__ = "dokime"
Output.__qualname__ = "Output"
Statistics.__module__ = "dokime"
Statistics.__qualname__ = "Statistics"
ExecutionsProgressGetResponse.__module__ = "dokime"
ExecutionsProgressGetResponse.__qualname__ = "ExecutionsProgressGetResponse"
FilesUploadsAppendResponse.__module__ = "dokime"
FilesUploadsAppendResponse.__qualname__ = "FilesUploadsAppendResponse"
FilesUploadsCreateResponse.__module__ = "dokime"
FilesUploadsCreateResponse.__qualname__ = "FilesUploadsCreateResponse"
FilesUploadsDeleteResponse.__module__ = "dokime"
FilesUploadsDeleteResponse.__qualname__ = "FilesUploadsDeleteResponse"
FilesUploadsGetResponse.__module__ = "dokime"
FilesUploadsGetResponse.__qualname__ = "FilesUploadsGetResponse"
FilesUploadsPublishResponse.__module__ = "dokime"
FilesUploadsPublishResponse.__qualname__ = "FilesUploadsPublishResponse"
FilesUploadsResumeResponse.__module__ = "dokime"
FilesUploadsResumeResponse.__qualname__ = "FilesUploadsResumeResponse"
FileBatch.__module__ = "dokime"
FileBatch.__qualname__ = "FileBatch"
FileCompletion.__module__ = "dokime"
FileCompletion.__qualname__ = "FileCompletion"
FilesDeleteResponse.__module__ = "dokime"
FilesDeleteResponse.__qualname__ = "FilesDeleteResponse"
FilesGetResponse.__module__ = "dokime"
FilesGetResponse.__qualname__ = "FilesGetResponse"
FilesReadResponse.__module__ = "dokime"
FilesReadResponse.__qualname__ = "FilesReadResponse"
FilesWatchResponse.__module__ = "dokime"
FilesWatchResponse.__qualname__ = "FilesWatchResponse"
Field.__module__ = "dokime"
Field.__qualname__ = "Field"
Schema.__module__ = "dokime"
Schema.__qualname__ = "Schema"
SecretWritable.__module__ = "dokime"
SecretWritable.__qualname__ = "SecretWritable"
SecretsGetResponse.__module__ = "dokime"
SecretsGetResponse.__qualname__ = "SecretsGetResponse"
Card.__module__ = "dokime"
Card.__qualname__ = "Card"
CardLayout.__module__ = "dokime"
CardLayout.__qualname__ = "CardLayout"
Document.__module__ = "dokime"
Document.__qualname__ = "Document"
Section.__module__ = "dokime"
Section.__qualname__ = "Section"
SectionRepeat.__module__ = "dokime"
SectionRepeat.__qualname__ = "SectionRepeat"
Summary.__module__ = "dokime"
Summary.__qualname__ = "Summary"
SummaryTile.__module__ = "dokime"
SummaryTile.__qualname__ = "SummaryTile"
Table.__module__ = "dokime"
Table.__qualname__ = "Table"
TableRow.__module__ = "dokime"
TableRow.__qualname__ = "TableRow"
ArtifactMetadata.__module__ = "dokime"
ArtifactMetadata.__qualname__ = "ArtifactMetadata"
ArtifactScope.__module__ = "dokime"
ArtifactScope.__qualname__ = "ArtifactScope"
EvidenceCondition.__module__ = "dokime"
EvidenceCondition.__qualname__ = "EvidenceCondition"
EvidenceConditions.__module__ = "dokime"
EvidenceConditions.__qualname__ = "EvidenceConditions"
EvidenceOrigin.__module__ = "dokime"
EvidenceOrigin.__qualname__ = "EvidenceOrigin"
EvidenceOriginKind.__module__ = "dokime"
EvidenceOriginKind.__qualname__ = "EvidenceOriginKind"
EvidenceScope.__module__ = "dokime"
EvidenceScope.__qualname__ = "EvidenceScope"
RecordMatch.__module__ = "dokime"
RecordMatch.__qualname__ = "RecordMatch"
StreamMatch.__module__ = "dokime"
StreamMatch.__qualname__ = "StreamMatch"
Window.__module__ = "dokime"
Window.__qualname__ = "Window"
Criterion.__module__ = "dokime"
Criterion.__qualname__ = "Criterion"
ExecutionProgressValue.__module__ = "dokime"
ExecutionProgressValue.__qualname__ = "ExecutionProgressValue"
Import.__module__ = "dokime"
Import.__qualname__ = "Import"
ImportSource.__module__ = "dokime"
ImportSource.__qualname__ = "ImportSource"
Origin.__module__ = "dokime"
Origin.__qualname__ = "Origin"
PluginRun.__module__ = "dokime"
PluginRun.__qualname__ = "PluginRun"
Step.__module__ = "dokime"
Step.__qualname__ = "Step"
TestCase.__module__ = "dokime"
TestCase.__qualname__ = "TestCase"
AggregateExpression.__module__ = "dokime"
AggregateExpression.__qualname__ = "AggregateExpression"
DataExpression.__module__ = "dokime"
DataExpression.__qualname__ = "DataExpression"
DocumentFilter.__module__ = "dokime"
DocumentFilter.__qualname__ = "DocumentFilter"
FieldReference.__module__ = "dokime"
FieldReference.__qualname__ = "FieldReference"
TransformExpression.__module__ = "dokime"
TransformExpression.__qualname__ = "TransformExpression"
CaptureGap.__module__ = "dokime"
CaptureGap.__qualname__ = "CaptureGap"
CapturedArtifact.__module__ = "dokime"
CapturedArtifact.__qualname__ = "CapturedArtifact"
FileLocation.__module__ = "dokime"
FileLocation.__qualname__ = "FileLocation"
FileRule.__module__ = "dokime"
FileRule.__qualname__ = "FileRule"
FileSelection.__module__ = "dokime"
FileSelection.__qualname__ = "FileSelection"
FileSource.__module__ = "dokime"
FileSource.__qualname__ = "FileSource"
FtpLocation.__module__ = "dokime"
FtpLocation.__qualname__ = "FtpLocation"
Metadata.__module__ = "dokime"
Metadata.__qualname__ = "Metadata"
DynamicDefault.__module__ = "dokime"
DynamicDefault.__qualname__ = "DynamicDefault"
FileParameter.__module__ = "dokime"
FileParameter.__qualname__ = "FileParameter"
IntegerRange.__module__ = "dokime"
IntegerRange.__qualname__ = "IntegerRange"
LinearSpace.__module__ = "dokime"
LinearSpace.__qualname__ = "LinearSpace"
NumberRange.__module__ = "dokime"
NumberRange.__qualname__ = "NumberRange"
Parameter.__module__ = "dokime"
Parameter.__qualname__ = "Parameter"
ParameterBinary.__module__ = "dokime"
ParameterBinary.__qualname__ = "ParameterBinary"
ParameterExpansion.__module__ = "dokime"
ParameterExpansion.__qualname__ = "ParameterExpansion"
ParameterExpression.__module__ = "dokime"
ParameterExpression.__qualname__ = "ParameterExpression"
ParameterExpressions.__module__ = "dokime"
ParameterExpressions.__qualname__ = "ParameterExpressions"
ParameterUnary.__module__ = "dokime"
ParameterUnary.__qualname__ = "ParameterUnary"
GridAxisEntry.__module__ = "dokime"
GridAxisEntry.__qualname__ = "GridAxisEntry"
GridChart.__module__ = "dokime"
GridChart.__qualname__ = "GridChart"
GridDefinition.__module__ = "dokime"
GridDefinition.__qualname__ = "GridDefinition"
GridHighlight.__module__ = "dokime"
GridHighlight.__qualname__ = "GridHighlight"
Plot.__module__ = "dokime"
Plot.__qualname__ = "Plot"
PlotFftOptions.__module__ = "dokime"
PlotFftOptions.__qualname__ = "PlotFftOptions"
PlotFilter.__module__ = "dokime"
PlotFilter.__qualname__ = "PlotFilter"
PlotHistogramOptions.__module__ = "dokime"
PlotHistogramOptions.__qualname__ = "PlotHistogramOptions"
PlotIntervalOptions.__module__ = "dokime"
PlotIntervalOptions.__qualname__ = "PlotIntervalOptions"
ReferenceBand.__module__ = "dokime"
ReferenceBand.__qualname__ = "ReferenceBand"
ReferenceLine.__module__ = "dokime"
ReferenceLine.__qualname__ = "ReferenceLine"
AnswerBinaryPredicate.__module__ = "dokime"
AnswerBinaryPredicate.__qualname__ = "AnswerBinaryPredicate"
AnswerPredicate.__module__ = "dokime"
AnswerPredicate.__qualname__ = "AnswerPredicate"
AnswerRule.__module__ = "dokime"
AnswerRule.__qualname__ = "AnswerRule"
Prompt.__module__ = "dokime"
Prompt.__qualname__ = "Prompt"
PromptAction.__module__ = "dokime"
PromptAction.__qualname__ = "PromptAction"
PromptDefinitions.__module__ = "dokime"
PromptDefinitions.__qualname__ = "PromptDefinitions"
PromptField.__module__ = "dokime"
PromptField.__qualname__ = "PromptField"
PromptFieldValidation.__module__ = "dokime"
PromptFieldValidation.__qualname__ = "PromptFieldValidation"
PromptValues.__module__ = "dokime"
PromptValues.__qualname__ = "PromptValues"
Listeners.__module__ = "dokime"
Listeners.__qualname__ = "Listeners"
RecordContract.__module__ = "dokime"
RecordContract.__qualname__ = "RecordContract"
RecordContracts.__module__ = "dokime"
RecordContracts.__qualname__ = "RecordContracts"
RecordListener.__module__ = "dokime"
RecordListener.__qualname__ = "RecordListener"
ExactNumber.__module__ = "dokime"
ExactNumber.__qualname__ = "ExactNumber"
MapValue.__module__ = "dokime"
MapValue.__qualname__ = "MapValue"
Null.__module__ = "dokime"
Null.__qualname__ = "Null"
Value.__module__ = "dokime"
Value.__qualname__ = "Value"
ValueList.__module__ = "dokime"
ValueList.__qualname__ = "ValueList"
SecretDeclaration.__module__ = "dokime"
SecretDeclaration.__qualname__ = "SecretDeclaration"
SecretsDeclarations.__module__ = "dokime"
SecretsDeclarations.__qualname__ = "SecretsDeclarations"
Suite.__module__ = "dokime"
Suite.__qualname__ = "Suite"
Suites.__module__ = "dokime"
Suites.__qualname__ = "Suites"
TestCaseTemplate.__module__ = "dokime"
TestCaseTemplate.__qualname__ = "TestCaseTemplate"
TraceabilityEntry.__module__ = "dokime"
TraceabilityEntry.__qualname__ = "TraceabilityEntry"

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
    "AxisScale",
    "BarArrangement",
    "Callers",
    "CaptureGap",
    "CaptureGapReason",
    "CapturedArtifact",
    "Card",
    "CardLayout",
    "Classification",
    "ContentArtifactsCreateResponse",
    "ContourRender",
    "Criterion",
    "DataExpression",
    "Declarations",
    "DiscoveredContract",
    "DiscoveredOperation",
    "Document",
    "DocumentFilter",
    "Downsample",
    "DynamicDefault",
    "DynamicDefaultKey",
    "EvidenceCapturedErrorCause",
    "EvidenceCapturedRecord",
    "EvidenceCapturedRecordError",
    "EvidenceCapturedRecordSelection",
    "EvidenceCard",
    "EvidenceCondition",
    "EvidenceConditions",
    "EvidenceCycleScope",
    "EvidenceOrigin",
    "EvidenceOriginKind",
    "EvidenceRecordScope",
    "EvidenceRecordsListResponse",
    "EvidenceScope",
    "EvidenceStatisticsResponse",
    "EvidenceTraceabilityGetResponse",
    "EvidenceTraceabilitySnapshot",
    "ExactNumber",
    "ExecutionProgressValue",
    "ExecutionsProgressGetResponse",
    "FailureValue",
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
    "FilesUploadsAppendResponse",
    "FilesUploadsCreateResponse",
    "FilesUploadsDeleteResponse",
    "FilesUploadsGetResponse",
    "FilesUploadsPublishResponse",
    "FilesUploadsResumeResponse",
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
    "HarmosSection",
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
    "PluginsSettingsListResponse",
    "Position",
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
    "Section",
    "SectionRepeat",
    "SettingDeclaration",
    "SettingValue",
    "SettingValues",
    "SettingsDeclarations",
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
    "TransferFraming",
    "TransformExpression",
    "Upload",
    "UploadState",
    "Value",
    "ValueList",
    "ValueType",
    "Visibility",
    "Window",
    "XyInterval",
]
