# @generated from protocol descriptors; do not edit.
from harmos.client import (
    ADJACENT,
    BOOL,
    BYTES,
    COMPONENT_STATE_DEGRADED,
    COMPONENT_STATE_HEALTHY,
    COMPONENT_STATE_UNAVAILABLE,
    COMPONENT_STATE_UNSPECIFIED,
    EXTERNAL,
    FLOAT32,
    FLOAT64,
    INT32,
    INT64,
    INTERNAL,
    MESSAGE,
    OBJECT,
    OMIT_EMPTY,
    OMIT_NONE,
    OPTIONAL,
    PAYLOAD_ENCODING_UNSPECIFIED,
    PLUGIN_STATE_CONNECTING,
    PLUGIN_STATE_READY,
    PLUGIN_STATE_STOPPED,
    PLUGIN_STATE_STOPPING,
    PLUGIN_STATE_UNAVAILABLE,
    PLUGIN_STATE_UNSPECIFIED,
    POSTCARD_V1,
    REPEATED,
    REQUIRED,
    STRING,
    TUPLE,
    UINT32,
    UINT64,
    UNIT,
    UNTAGGED,
    VALUE,
    VARIANT,
    ComponentState,
    HealthComponent,
    HostIdentity,
    HostStatus,
    InvocationsCreateResponse,
    PayloadEncoding,
    PluginChange,
    PluginDescription,
    PluginEntry,
    PluginGuest,
    PluginManaged,
    PluginPlacement,
    PluginProcess,
    PluginRemote,
    PluginsCreateResponse,
    PluginsDeleteResponse,
    PluginsGetResponse,
    PluginsListResponse,
    PluginSnapshot,
    PluginsReplaceResponse,
    PluginsRestartResponse,
    PluginsSetResponse,
    PluginState,
    PluginsWatchResponse,
    PluginValue,
    SchemaCardinality,
    SchemaDefinition,
    SchemaDescription,
    SchemaField,
    SchemaForm,
    SchemaGraph,
    SchemaKey,
    SchemaKind,
    SchemaOmission,
    SchemaRepresentation,
    SchemasListResponse,
    StatusGetResponse,
    StatusWatchResponse,
    StreamIdentity,
    StreamsListResponse,
)
from harmos.client import CertificatesCreateResponse as AccessCertificatesCreateResponse
from harmos.client import DiagnosticsGetResponse as StatusDiagnosticsGetResponse
from harmos.client import Event as StatusEvent
from harmos.client import ListenerGetResponse as AccessListenerGetResponse
from harmos.client import ListenerSetResponse as AccessListenerSetResponse
from harmos.client import ListenerValue as AccessListenerValue
from harmos.client import LogsListResponse as StatusLogsListResponse
from harmos.client import LogsWatchResponse as StatusLogsWatchResponse
from harmos.client import Pairing as AccessPairing
from harmos.client import PairingsCreateResponse as AccessPairingsCreateResponse
from harmos.client import PairingsDeleteResponse as AccessPairingsDeleteResponse
from harmos.client import PairingsListResponse as AccessPairingsListResponse
from harmos.client import SettingsSetResponse as PluginsSettingsSetResponse
from harmos.client import TraceEventsListResponse as StatusTraceEventsListResponse
from harmos.client import TraceEventsWatchResponse as StatusTraceEventsWatchResponse

from dokime._wire.host.v1.aggregates_pb2 import (
    QUERY_FILTER_OPERATOR_EQ,
    QUERY_FILTER_OPERATOR_GT,
    QUERY_FILTER_OPERATOR_GTE,
    QUERY_FILTER_OPERATOR_IN,
    QUERY_FILTER_OPERATOR_IS_NULL,
    QUERY_FILTER_OPERATOR_LT,
    QUERY_FILTER_OPERATOR_LTE,
    QUERY_FILTER_OPERATOR_NEQ,
    QUERY_FILTER_OPERATOR_UNSPECIFIED,
    QUERY_SOURCE_CASES,
    QUERY_SOURCE_CYCLES,
    QUERY_SOURCE_EXECUTIONS,
    QUERY_SOURCE_RECORDS,
    QUERY_SOURCE_STEPS,
    QUERY_SOURCE_UNSPECIFIED,
    AggregateReply,
    AggregatesGetResponse,
    QueryAggregate,
    QueryFilter,
    QueryFilterOperator,
    QueryRow,
    QuerySource,
)
from dokime._wire.host.v1.backend.overview_pb2 import EvidenceCounts as BackendEvidenceCounts
from dokime._wire.host.v1.backend.overview_pb2 import OverviewGetResponse as BackendOverviewGetResponse
from dokime._wire.host.v1.backend.overview_pb2 import OverviewWatchResponse as BackendOverviewWatchResponse
from dokime._wire.host.v1.backend.overview_pb2 import RuntimeShellSnapshot as BackendRuntimeShellSnapshot
from dokime._wire.host.v1.backend_pb2 import (
    APPEARANCE_FAILURE_KIND_ASSET_CHANGED,
    APPEARANCE_FAILURE_KIND_ASSET_MISSING,
    APPEARANCE_FAILURE_KIND_ASSET_UNAVAILABLE,
    APPEARANCE_FAILURE_KIND_INVALID_REFERENCE,
    APPEARANCE_FAILURE_KIND_TOO_LARGE,
    APPEARANCE_FAILURE_KIND_UNSPECIFIED,
    LOGO_VARIANT_CONTAINED_ON_DARK,
    LOGO_VARIANT_CONTAINED_ON_LIGHT,
    LOGO_VARIANT_TRANSPARENT_ON_DARK,
    LOGO_VARIANT_TRANSPARENT_ON_LIGHT,
    LOGO_VARIANT_UNSPECIFIED,
    THEME_MODE_POLICY_FORCE_DARK,
    THEME_MODE_POLICY_FORCE_LIGHT,
    THEME_MODE_POLICY_FORCE_SYSTEM,
    THEME_MODE_POLICY_UNSPECIFIED,
    THEME_MODE_POLICY_USER_OR_SYSTEM,
    AppearanceAssetMetadata,
    AppearanceAssetReference,
    AppearanceFailure,
    AppearanceFailureKind,
    AppearanceLogo,
    AppearanceSnapshot,
    BackendAppearanceResponse,
    LogoVariant,
    OrganizationIdentity,
    ThemeCornerRadii,
    ThemeFamily,
    ThemeModePolicy,
    ThemeVariant,
)
from dokime._wire.host.v1.cases_pb2 import (
    CASE_STATUS_BLOCKED,
    CASE_STATUS_CANCELLED,
    CASE_STATUS_FAILED,
    CASE_STATUS_INTERRUPTED,
    CASE_STATUS_INVALIDATED,
    CASE_STATUS_PASSED,
    CASE_STATUS_PLANNED,
    CASE_STATUS_QUEUED,
    CASE_STATUS_RUNNING,
    CASE_STATUS_SKIPPED,
    CASE_STATUS_UNSPECIFIED,
    CaseActions,
    CaseDetail,
    CaseParameter,
    CasesCreateResponse,
    CasesDeleteResponse,
    CasesGetResponse,
    CasesListResponse,
    CasesParametersResponse,
    CaseStatus,
    CaseSummary,
    CasesWatchResponse,
    PlannedCase,
    RegisteredSelection,
    RegisteredSelectionFilesEntry,
    RegisteredSelectionParametersEntry,
    SelectedFile,
    SelectedFileKind,
    SelectionPlan,
    SelectionValues,
)
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsGetResponse as ContentArtifactsGetResponse
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsListResponse as ContentArtifactsListResponse
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsWatchResponse as ContentArtifactsWatchResponse
from dokime._wire.host.v1.content.execution_files_pb2 import (
    EXECUTION_FILE_KIND_BINARY,
    EXECUTION_FILE_KIND_MEDIA,
    EXECUTION_FILE_KIND_TEXT,
    EXECUTION_FILE_KIND_UNKNOWN,
    EXECUTION_FILE_KIND_UNSPECIFIED,
    EXECUTION_FILE_STATE_FINALIZED,
    EXECUTION_FILE_STATE_INTERRUPTED,
    EXECUTION_FILE_STATE_STREAMING,
    EXECUTION_FILE_STATE_UNSPECIFIED,
)
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFileKind as ContentExecutionFileKind
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFileMetadata as ContentExecutionFileMetadata
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesGetResponse as ContentExecutionFilesGetResponse
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesListResponse as ContentExecutionFilesListResponse
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFileState as ContentExecutionFileState
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesWatchResponse as ContentExecutionFilesWatchResponse
from dokime._wire.host.v1.cycles_pb2 import (
    ARCHIVE_SCOPE_ALL,
    ARCHIVE_SCOPE_ARCHIVED,
    ARCHIVE_SCOPE_UNARCHIVED,
    ARCHIVE_SCOPE_UNSPECIFIED,
    CYCLE_ORDER_KEY_CREATED_AT,
    CYCLE_ORDER_KEY_LABEL,
    CYCLE_ORDER_KEY_UNSPECIFIED,
    CYCLE_ORDER_KEY_UPDATED_AT,
    CYCLE_PHASE_ACTIVE,
    CYCLE_PHASE_SEAL_FAILED,
    CYCLE_PHASE_SEALED,
    CYCLE_PHASE_SEALING,
    CYCLE_PHASE_UNSPECIFIED,
    CYCLE_STATUS_BLOCKED,
    CYCLE_STATUS_COMPLETED,
    CYCLE_STATUS_FAILED,
    CYCLE_STATUS_PAUSED,
    CYCLE_STATUS_READY,
    CYCLE_STATUS_RUNNING,
    CYCLE_STATUS_STOPPED,
    CYCLE_STATUS_STOPPING,
    CYCLE_STATUS_UNSPECIFIED,
    RESULT_STATUS_BLOCKED,
    RESULT_STATUS_CANCELLED,
    RESULT_STATUS_FAILED,
    RESULT_STATUS_PASSED,
    RESULT_STATUS_PENDING,
    RESULT_STATUS_UNSPECIFIED,
    ArchiveScope,
    CycleFilter,
    CycleOrder,
    CycleOrderKey,
    CyclePhase,
    CyclesCreateResponse,
    CycleSelection,
    CyclesGetResponse,
    CyclesListResponse,
    CyclesSealResponse,
    CyclesSetResponse,
    CycleStatus,
    CycleSummary,
    CyclesWatchResponse,
    CycleWritable,
    RerunCaseNotice,
    ResultStatus,
    ResultSummary,
)
from dokime._wire.host.v1.documents_pb2 import (
    DOCUMENT_CARD_KIND_ARTIFACTS,
    DOCUMENT_CARD_KIND_CRITERIA,
    DOCUMENT_CARD_KIND_HEALTH,
    DOCUMENT_CARD_KIND_MEDIA,
    DOCUMENT_CARD_KIND_NOTES,
    DOCUMENT_CARD_KIND_PLOT,
    DOCUMENT_CARD_KIND_RECORDS,
    DOCUMENT_CARD_KIND_STEPS,
    DOCUMENT_CARD_KIND_TABLE,
    DOCUMENT_CARD_KIND_TELEMETRY_CONTEXT,
    DOCUMENT_CARD_KIND_TELEMETRY_SUMMARY,
    DOCUMENT_CARD_KIND_TEXT,
    DOCUMENT_CARD_KIND_TRACEABILITY,
    DOCUMENT_CARD_KIND_UNSPECIFIED,
    DOCUMENT_DENSITY_COMPACT,
    DOCUMENT_DENSITY_NORMAL,
    DOCUMENT_DENSITY_SPACIOUS,
    DOCUMENT_DENSITY_UNSPECIFIED,
    DOCUMENT_RAIL_LEFT,
    DOCUMENT_RAIL_MAIN,
    DOCUMENT_RAIL_RIGHT,
    DOCUMENT_RAIL_UNSPECIFIED,
    DOCUMENT_SUMMARY_FORMAT_AUTO,
    DOCUMENT_SUMMARY_FORMAT_DISTANCE,
    DOCUMENT_SUMMARY_FORMAT_DURATION,
    DOCUMENT_SUMMARY_FORMAT_NUMBER,
    DOCUMENT_SUMMARY_FORMAT_PERCENT,
    DOCUMENT_SUMMARY_FORMAT_STATUS,
    DOCUMENT_SUMMARY_FORMAT_TEXT,
    DOCUMENT_SUMMARY_FORMAT_UNSPECIFIED,
    DOCUMENT_SUMMARY_PRESENTATION_BADGE,
    DOCUMENT_SUMMARY_PRESENTATION_PROGRESS,
    DOCUMENT_SUMMARY_PRESENTATION_UNSPECIFIED,
    DOCUMENT_SUMMARY_PRESENTATION_VALUE,
    DOCUMENT_SUMMARY_STAT_LATEST,
    DOCUMENT_SUMMARY_STAT_UNSPECIFIED,
    DOCUMENT_SUMMARY_STATE_CRITICAL,
    DOCUMENT_SUMMARY_STATE_MUTED,
    DOCUMENT_SUMMARY_STATE_OK,
    DOCUMENT_SUMMARY_STATE_UNSPECIFIED,
    DOCUMENT_SUMMARY_STATE_WARNING,
    PLOT_AXIS_KIND_ELAPSED,
    PLOT_AXIS_KIND_UNSPECIFIED,
    PLOT_AXIS_KIND_WALL_TIME,
    DocumentBinding,
    DocumentBindingResult,
    DocumentCard,
    DocumentCardBody,
    DocumentCardBudget,
    DocumentCardFailure,
    DocumentCardKind,
    DocumentCompletion,
    DocumentDensity,
    DocumentEvidenceCut,
    DocumentFrame,
    DocumentFrameFrame,
    DocumentFrontiers,
    DocumentHeader,
    DocumentLayoutNode,
    DocumentLayoutNodeNode,
    DocumentOwnerReference,
    DocumentPage,
    DocumentPlacement,
    DocumentPlotDefinition,
    DocumentRail,
    DocumentRepeat,
    DocumentRepeatInstance,
    DocumentRepeatSelection,
    DocumentScope,
    DocumentSection,
    DocumentSectionIdentity,
    DocumentsGetResponse,
    DocumentSpan,
    DocumentSummary,
    DocumentSummaryCondition,
    DocumentSummaryFormat,
    DocumentSummaryGroup,
    DocumentSummaryGroupDefinition,
    DocumentSummaryPresentation,
    DocumentSummaryStat,
    DocumentSummaryState,
    DocumentSummaryTile,
    DocumentSummaryTileDefinition,
    DocumentsWatchResponse,
    DocumentTable,
    DocumentTableRow,
    DocumentTelemetryCut,
    DocumentWarning,
    PlotAxisKind,
    PlotPresentation,
    PlotReferenceBand,
    PlotReferenceLine,
    PlotReferenceValue,
)
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsListResponse as EvidenceHealthEventsListResponse
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsWatchResponse as EvidenceHealthEventsWatchResponse
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthItem as EvidenceHealthItem
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationOwner as EvidenceAnnotationOwner
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationOwnerOwner as EvidenceAnnotationOwnerOwner
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationSelection as EvidenceAnnotationSelection
from dokime._wire.host.v1.evidence.notes_pb2 import ExecutionNoteOwner as EvidenceExecutionNoteOwner
from dokime._wire.host.v1.evidence.notes_pb2 import NotesCreateResponse as EvidenceNotesCreateResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesGetResponse as EvidenceNotesGetResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesListResponse as EvidenceNotesListResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesSetResponse as EvidenceNotesSetResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NotesWatchResponse as EvidenceNotesWatchResponse
from dokime._wire.host.v1.evidence.notes_pb2 import NoteWritable as EvidenceNoteWritable
from dokime._wire.host.v1.evidence.notes_pb2 import OperatorNote as EvidenceOperatorNote
from dokime._wire.host.v1.evidence.notes_pb2 import PlannedCaseNoteOwner as EvidencePlannedCaseNoteOwner
from dokime._wire.host.v1.evidence.records_pb2 import RecordsWatchResponse as EvidenceRecordsWatchResponse
from dokime._wire.host.v1.evidence.samples_pb2 import Bucket as EvidenceBucket
from dokime._wire.host.v1.evidence.samples_pb2 import BucketRequest as EvidenceBucketRequest
from dokime._wire.host.v1.evidence.samples_pb2 import BucketValue as EvidenceBucketValue
from dokime._wire.host.v1.evidence.samples_pb2 import SampleRow as EvidenceSampleRow
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesListResponse as EvidenceSamplesListResponse
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesWatchResponse as EvidenceSamplesWatchResponse
from dokime._wire.host.v1.evidence.samples_pb2 import StatisticValue as EvidenceStatisticValue
from dokime._wire.host.v1.evidence.samples_pb2 import TelemetrySample as EvidenceTelemetrySample
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityWatchResponse as EvidenceTraceabilityWatchResponse
from dokime._wire.host.v1.evidence_pb2 import EvidenceArchiveResponse, EvidenceSubject
from dokime._wire.host.v1.executions.commands_pb2 import (
    EXECUTION_COMMAND_ACTION_FINISH,
    EXECUTION_COMMAND_ACTION_NEXT,
    EXECUTION_COMMAND_ACTION_RUN_STEP,
    EXECUTION_COMMAND_ACTION_UNSPECIFIED,
    EXECUTION_COMMAND_PHASE_ACCEPTED,
    EXECUTION_COMMAND_PHASE_COMPLETED,
    EXECUTION_COMMAND_PHASE_DISPATCHING,
    EXECUTION_COMMAND_PHASE_FAILED,
    EXECUTION_COMMAND_PHASE_INTERRUPTED,
    EXECUTION_COMMAND_PHASE_REFUSED,
    EXECUTION_COMMAND_PHASE_RUNNING,
    EXECUTION_COMMAND_PHASE_UNSPECIFIED,
)
from dokime._wire.host.v1.executions.commands_pb2 import CommandsCreateResponse as ExecutionsCommandsCreateResponse
from dokime._wire.host.v1.executions.commands_pb2 import CommandsGetResponse as ExecutionsCommandsGetResponse
from dokime._wire.host.v1.executions.commands_pb2 import ExecutionCommandAction as ExecutionsExecutionCommandAction
from dokime._wire.host.v1.executions.commands_pb2 import ExecutionCommandPhase as ExecutionsExecutionCommandPhase
from dokime._wire.host.v1.executions.commands_pb2 import ExecutionCommandReceipt as ExecutionsExecutionCommandReceipt
from dokime._wire.host.v1.executions.progress_pb2 import ProgressWatchResponse as ExecutionsProgressWatchResponse
from dokime._wire.host.v1.executions.schemas_pb2 import SchemasGetResponse as ExecutionsSchemasGetResponse
from dokime._wire.host.v1.executions_pb2 import (
    CRITERION_STATUS_FAILED,
    CRITERION_STATUS_INDETERMINATE,
    CRITERION_STATUS_PASSED,
    CRITERION_STATUS_PENDING,
    CRITERION_STATUS_SKIPPED,
    CRITERION_STATUS_UNSPECIFIED,
    EXECUTION_MODE_AUTOMATIC,
    EXECUTION_MODE_INTERACTIVE,
    EXECUTION_MODE_UNSPECIFIED,
    EXECUTION_ORDER_KEY_FINISHED_AT,
    EXECUTION_ORDER_KEY_STARTED_AT,
    EXECUTION_ORDER_KEY_TITLE,
    EXECUTION_ORDER_KEY_UNSPECIFIED,
    EXECUTION_PARAMETER_SET_INPUTS,
    EXECUTION_PARAMETER_SET_OUTPUTS,
    EXECUTION_PARAMETER_SET_UNSPECIFIED,
    EXECUTION_STATE_CLOSED,
    EXECUTION_STATE_OPEN,
    EXECUTION_STATE_SEALED,
    EXECUTION_STATE_UNSPECIFIED,
    LIVE_EXECUTION_STATE_CANCELLED,
    LIVE_EXECUTION_STATE_CLEANING_UP,
    LIVE_EXECUTION_STATE_EVALUATING,
    LIVE_EXECUTION_STATE_FINISHED,
    LIVE_EXECUTION_STATE_INITIALIZING,
    LIVE_EXECUTION_STATE_INTERRUPTED,
    LIVE_EXECUTION_STATE_QUEUED,
    LIVE_EXECUTION_STATE_STEPPING,
    LIVE_EXECUTION_STATE_UNSPECIFIED,
    PARAMETER_SOURCE_DEFINITION,
    PARAMETER_SOURCE_ENVIRONMENT,
    PARAMETER_SOURCE_OPERATOR,
    PARAMETER_SOURCE_OUTPUT,
    PARAMETER_SOURCE_RECORD,
    PARAMETER_SOURCE_RESOURCE,
    PARAMETER_SOURCE_RUNTIME,
    PARAMETER_SOURCE_TELEMETRY,
    PARAMETER_SOURCE_TRACEABILITY,
    PARAMETER_SOURCE_UNSPECIFIED,
    REPORT_STATE_FAILED,
    REPORT_STATE_PENDING,
    REPORT_STATE_READY,
    REPORT_STATE_UNSPECIFIED,
    STEP_STATUS_CANCELLED,
    STEP_STATUS_FAILED,
    STEP_STATUS_PASSED,
    STEP_STATUS_PENDING,
    STEP_STATUS_RUNNING,
    STEP_STATUS_SKIPPED,
    STEP_STATUS_UNSPECIFIED,
    CriterionStatus,
    DeclaredOutcome,
    Diagnostic,
    EvaluatedOutcome,
    ExecutionBinding,
    ExecutionControls,
    ExecutionDetail,
    ExecutionFilter,
    ExecutionMode,
    ExecutionOrder,
    ExecutionOrderKey,
    ExecutionOutcome,
    ExecutionOutcomeCounts,
    ExecutionParameterSet,
    ExecutionsCancelResponse,
    ExecutionsCreateResponse,
    ExecutionsDeleteResponse,
    ExecutionSelection,
    ExecutionsGetResponse,
    ExecutionsListResponse,
    ExecutionsParametersResponse,
    ExecutionsSetResponse,
    ExecutionsStartResponse,
    ExecutionsStepsResponse,
    ExecutionsStopResponse,
    ExecutionState,
    ExecutionStep,
    ExecutionSummary,
    ExecutionsWatchResponse,
    ExecutionWritable,
    FrozenSchemas,
    LiveExecutionState,
    MaterializedParameter,
    ParameterSource,
    ReportState,
    ReportStatus,
    StepCriterion,
    StepStatus,
    UnevaluatedOutcome,
)
from dokime._wire.host.v1.media.captures_pb2 import (
    CAPTURE_KIND_AUDIO,
    CAPTURE_KIND_CAMERA,
    CAPTURE_KIND_PHOTO,
    CAPTURE_KIND_SCREEN,
    CAPTURE_KIND_SCREENSHOT,
    CAPTURE_KIND_UNSPECIFIED,
)
from dokime._wire.host.v1.media.captures_pb2 import Capture as MediaCapture
from dokime._wire.host.v1.media.captures_pb2 import CaptureKind as MediaCaptureKind
from dokime._wire.host.v1.media.captures_pb2 import CapturesCreateResponse as MediaCapturesCreateResponse
from dokime._wire.host.v1.media.captures_pb2 import CapturesGetResponse as MediaCapturesGetResponse
from dokime._wire.host.v1.media.captures_pb2 import CapturesStopResponse as MediaCapturesStopResponse
from dokime._wire.host.v1.media.captures_pb2 import CapturesWatchResponse as MediaCapturesWatchResponse
from dokime._wire.host.v1.media.captures_pb2 import RecordingAudio as MediaRecordingAudio
from dokime._wire.host.v1.media.captures_pb2 import RecordingProgress as MediaRecordingProgress
from dokime._wire.host.v1.plots.settings_pb2 import PlotSettingsValue as PlotsPlotSettingsValue
from dokime._wire.host.v1.plots.settings_pb2 import SettingsGetResponse as PlotsSettingsGetResponse
from dokime._wire.host.v1.plots.settings_pb2 import SettingsSetResponse as PlotsSettingsSetResponse
from dokime._wire.host.v1.plots.snapshots_pb2 import (
    PLOT_CARTESIAN_MODE_ALIGNED,
    PLOT_CARTESIAN_MODE_FACETED,
    PLOT_CARTESIAN_MODE_UNSPECIFIED,
    PLOT_STREAM_KIND_DELTA,
    PLOT_STREAM_KIND_ERROR,
    PLOT_STREAM_KIND_REPLACE,
    PLOT_STREAM_KIND_SNAPSHOT,
    PLOT_STREAM_KIND_STATISTICS,
    PLOT_STREAM_KIND_UNSPECIFIED,
)
from dokime._wire.host.v1.plots.snapshots_pb2 import ContourGrid as PlotsContourGrid
from dokime._wire.host.v1.plots.snapshots_pb2 import GridCell as PlotsGridCell
from dokime._wire.host.v1.plots.snapshots_pb2 import GridFrame as PlotsGridFrame
from dokime._wire.host.v1.plots.snapshots_pb2 import GridFrames as PlotsGridFrames
from dokime._wire.host.v1.plots.snapshots_pb2 import NullableNumber as PlotsNullableNumber
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotAnalysis as PlotsPlotAnalysis
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotAnalysisError as PlotsPlotAnalysisError
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotAnalysisResult as PlotsPlotAnalysisResult
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotBoxSummary as PlotsPlotBoxSummary
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotBoxSummaryAnalysis as PlotsPlotBoxSummaryAnalysis
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotCartesianMode as PlotsPlotCartesianMode
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotColumn as PlotsPlotColumn
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotDomain as PlotsPlotDomain
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotEnvelope as PlotsPlotEnvelope
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotFrame as PlotsPlotFrame
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramAnalysis as PlotsPlotHistogramAnalysis
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramBin as PlotsPlotHistogramBin
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramSeries as PlotsPlotHistogramSeries
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramStatistics as PlotsPlotHistogramStatistics
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotMarker as PlotsPlotMarker
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotPayload as PlotsPlotPayload
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotPayloadDiagnostics as PlotsPlotPayloadDiagnostics
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotRequest as PlotsPlotRequest
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSeries as PlotsPlotSeries
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSeriesSummary as PlotsPlotSeriesSummary
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSigmaBin as PlotsPlotSigmaBin
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSigmaIntervalAnalysis as PlotsPlotSigmaIntervalAnalysis
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSigmaSeries as PlotsPlotSigmaSeries
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotStreamKind as PlotsPlotStreamKind
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotStreamUpdate as PlotsPlotStreamUpdate
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotWindow as PlotsPlotWindow
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsListResponse as PlotsSnapshotsListResponse
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsWatchResponse as PlotsSnapshotsWatchResponse
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretRequirement as PluginsSecretRequirement
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretsListResponse as PluginsSecretsListResponse
from dokime._wire.host.v1.prompt_definitions_pb2 import PromptDefinitionsGetResponse, PromptDefinitionsListResponse
from dokime._wire.host.v1.prompts.answers_pb2 import AnswersCreateResponse as PromptsAnswersCreateResponse
from dokime._wire.host.v1.prompts_pb2 import (
    PROMPT_KIND_CHOICE,
    PROMPT_KIND_CONFIRMATION,
    PROMPT_KIND_FORM,
    PROMPT_KIND_STATUS,
    PROMPT_KIND_UNSPECIFIED,
    PROMPT_KIND_VERIFICATION,
    PromptInstance,
    PromptKind,
    PromptResponse,
    PromptsGetResponse,
    PromptsListResponse,
    PromptsWatchResponse,
    PromptView,
)
from dokime._wire.host.v1.record_schemas_pb2 import (
    ELAPSED_UNIT_MILLISECONDS,
    ELAPSED_UNIT_MINUTES,
    ELAPSED_UNIT_SECONDS,
    ELAPSED_UNIT_UNSPECIFIED,
    RECORD_ICON_ACTIVITY,
    RECORD_ICON_ALERT_TRIANGLE,
    RECORD_ICON_ARCHIVE,
    RECORD_ICON_BELL,
    RECORD_ICON_CHECK_CIRCLE,
    RECORD_ICON_CIRCLE_DASHED,
    RECORD_ICON_CLIPBOARD_CHECK,
    RECORD_ICON_CLOCK,
    RECORD_ICON_CPU,
    RECORD_ICON_FILE_TEXT,
    RECORD_ICON_FLAG,
    RECORD_ICON_GAUGE,
    RECORD_ICON_HEART_PULSE,
    RECORD_ICON_HELP_CIRCLE,
    RECORD_ICON_MESSAGE_SQUARE_TEXT,
    RECORD_ICON_PACKAGE,
    RECORD_ICON_PLUG,
    RECORD_ICON_RADIO_TOWER,
    RECORD_ICON_ROTATE_CCW,
    RECORD_ICON_SHIELD_ALERT,
    RECORD_ICON_TARGET,
    RECORD_ICON_TIMER_RESET,
    RECORD_ICON_UNSPECIFIED,
    RECORD_ICON_WRENCH,
    RECORD_ICON_ZAP,
    RECORD_TONE_ACCENT,
    RECORD_TONE_DANGER,
    RECORD_TONE_INFO,
    RECORD_TONE_MUTED,
    RECORD_TONE_NEUTRAL,
    RECORD_TONE_SUCCESS,
    RECORD_TONE_UNSPECIFIED,
    RECORD_TONE_WARNING,
    TELEMETRY_ROLE_COUNTER,
    TELEMETRY_ROLE_DISTANCE,
    TELEMETRY_ROLE_LATENCY,
    TELEMETRY_ROLE_STATE,
    TELEMETRY_ROLE_THROUGHPUT,
    TELEMETRY_ROLE_UNSPECIFIED,
    TELEMETRY_ROLE_VALUE,
    UNIT_FAMILY_COUNT,
    UNIT_FAMILY_CUSTOM,
    UNIT_FAMILY_DISTANCE,
    UNIT_FAMILY_DURATION,
    UNIT_FAMILY_FREQUENCY,
    UNIT_FAMILY_POWER,
    UNIT_FAMILY_RATIO,
    UNIT_FAMILY_TEMPERATURE,
    UNIT_FAMILY_UNSPECIFIED,
    UNIT_FAMILY_VOLTAGE,
    VALIDATION_MODE_PROTOTYPE,
    VALIDATION_MODE_STRICT,
    VALIDATION_MODE_UNSPECIFIED,
    AbsoluteTimeFormat,
    ElapsedFormat,
    ElapsedUnit,
    EngineeringFormat,
    FixedFormat,
    MeasurementUnit,
    NormalFormat,
    RecordDisplay,
    RecordField,
    RecordIcon,
    RecordSchema,
    RecordSchemasGetResponse,
    RecordSchemasListResponse,
    RecordTone,
    ScientificFormat,
    SiPrefixFormat,
    TelemetryColumn,
    TelemetryRole,
    TelemetrySchema,
    UnitFamily,
    ValidationMode,
    ValueFormat,
    ValueFormatKind,
)
from dokime._wire.host.v1.reports_pb2 import (
    REPORT_ORIGIN_CURRENT,
    REPORT_ORIGIN_SEALED,
    REPORT_ORIGIN_UNSPECIFIED,
    ReportMetadata,
    ReportOrigin,
    ReportsCreateResponse,
    ReportsGetResponse,
    ReportsPublishResponse,
    ReportSubject,
)
from dokime._wire.host.v1.secrets_pb2 import (
    SECRET_IMPORT_ACTION_REPLACE,
    SECRET_IMPORT_ACTION_SKIP,
    SECRET_IMPORT_ACTION_UNSPECIFIED,
    SecretArchive,
    SecretImportAction,
    SecretImportChoice,
    SecretImportEntry,
    SecretImportPreview,
    SecretsDeleteResponse,
    SecretsExportResponse,
    SecretsImportResponse,
    SecretsListResponse,
    SecretSnapshot,
    SecretsSetResponse,
    SecretSummary,
    SecretsWatchResponse,
)
from dokime._wire.host.v1.settings_pb2 import (
    DENSITY_COMFORTABLE,
    DENSITY_COMPACT,
    DENSITY_TOUCH,
    DENSITY_UNSPECIFIED,
    HOST_CARD_TONE_DANGER,
    HOST_CARD_TONE_INFO,
    HOST_CARD_TONE_NEUTRAL,
    HOST_CARD_TONE_SUCCESS,
    HOST_CARD_TONE_UNSPECIFIED,
    HOST_CARD_TONE_WARNING,
    SETTINGS_CARD_STATE_DISABLED,
    SETTINGS_CARD_STATE_DISCONNECTED,
    SETTINGS_CARD_STATE_ERROR,
    SETTINGS_CARD_STATE_PERMISSION_DENIED,
    SETTINGS_CARD_STATE_READY,
    SETTINGS_CARD_STATE_STALE,
    SETTINGS_CARD_STATE_UNSPECIFIED,
    THEME_MODE_DARK,
    THEME_MODE_LIGHT,
    THEME_MODE_SYSTEM,
    THEME_MODE_UNSPECIFIED,
    ApplicationAppearance,
    ApplicationSettings,
    BootstrapPaths,
    CachedRefresh,
    ClientPreferences,
    Density,
    HostCardBlock,
    HostCardBlockKind,
    HostCardBody,
    HostCardKeyValue,
    HostCardKeyValueList,
    HostCardMetric,
    HostCardMetrics,
    HostCardStatus,
    HostCardTone,
    ManualRefresh,
    OnDemandRefresh,
    RemoteListenerSettings,
    ReportDefaults,
    SettingsCard,
    SettingsCardRefreshPolicy,
    SettingsCardRefreshPolicyPolicy,
    SettingsCardState,
    SettingsFrame,
    SettingsGetResponse,
    SettingsSchemaResponse,
    SettingsSection,
    SettingsSectionOwner,
    SettingsSetResponse,
    TelemetryPreferences,
    ThemeMode,
)
from dokime._wire.host.v1.storage.sources_pb2 import SourcesGetResponse as StorageSourcesGetResponse
from dokime._wire.host.v1.storage.sources_pb2 import SourcesSetResponse as StorageSourcesSetResponse
from dokime._wire.host.v1.storage.sources_pb2 import StorageSource as StorageStorageSource
from dokime._wire.host.v1.storage.sources_pb2 import StorageSources as StorageStorageSources
from dokime._wire.host.v1.stream_schemas_pb2 import StreamSchemasGetResponse, StreamSchemasListResponse
from dokime._wire.host.v1.suites_pb2 import SuitesGetResponse, SuitesListResponse

AggregateReply.__module__ = "dokime.client"
AggregateReply.__qualname__ = "AggregateReply"
AggregatesGetResponse.__module__ = "dokime.client"
AggregatesGetResponse.__qualname__ = "AggregatesGetResponse"
QueryAggregate.__module__ = "dokime.client"
QueryAggregate.__qualname__ = "QueryAggregate"
QueryFilter.__module__ = "dokime.client"
QueryFilter.__qualname__ = "QueryFilter"
QueryRow.__module__ = "dokime.client"
QueryRow.__qualname__ = "QueryRow"
BackendEvidenceCounts.__module__ = "dokime.client"
BackendEvidenceCounts.__qualname__ = "BackendEvidenceCounts"
BackendOverviewGetResponse.__module__ = "dokime.client"
BackendOverviewGetResponse.__qualname__ = "BackendOverviewGetResponse"
BackendOverviewWatchResponse.__module__ = "dokime.client"
BackendOverviewWatchResponse.__qualname__ = "BackendOverviewWatchResponse"
BackendRuntimeShellSnapshot.__module__ = "dokime.client"
BackendRuntimeShellSnapshot.__qualname__ = "BackendRuntimeShellSnapshot"
AppearanceAssetMetadata.__module__ = "dokime.client"
AppearanceAssetMetadata.__qualname__ = "AppearanceAssetMetadata"
AppearanceAssetReference.__module__ = "dokime.client"
AppearanceAssetReference.__qualname__ = "AppearanceAssetReference"
AppearanceFailure.__module__ = "dokime.client"
AppearanceFailure.__qualname__ = "AppearanceFailure"
AppearanceLogo.__module__ = "dokime.client"
AppearanceLogo.__qualname__ = "AppearanceLogo"
AppearanceSnapshot.__module__ = "dokime.client"
AppearanceSnapshot.__qualname__ = "AppearanceSnapshot"
BackendAppearanceResponse.__module__ = "dokime.client"
BackendAppearanceResponse.__qualname__ = "BackendAppearanceResponse"
OrganizationIdentity.__module__ = "dokime.client"
OrganizationIdentity.__qualname__ = "OrganizationIdentity"
ThemeCornerRadii.__module__ = "dokime.client"
ThemeCornerRadii.__qualname__ = "ThemeCornerRadii"
ThemeFamily.__module__ = "dokime.client"
ThemeFamily.__qualname__ = "ThemeFamily"
ThemeVariant.__module__ = "dokime.client"
ThemeVariant.__qualname__ = "ThemeVariant"
CaseActions.__module__ = "dokime.client"
CaseActions.__qualname__ = "CaseActions"
CaseDetail.__module__ = "dokime.client"
CaseDetail.__qualname__ = "CaseDetail"
CaseParameter.__module__ = "dokime.client"
CaseParameter.__qualname__ = "CaseParameter"
CaseSummary.__module__ = "dokime.client"
CaseSummary.__qualname__ = "CaseSummary"
CasesCreateResponse.__module__ = "dokime.client"
CasesCreateResponse.__qualname__ = "CasesCreateResponse"
CasesDeleteResponse.__module__ = "dokime.client"
CasesDeleteResponse.__qualname__ = "CasesDeleteResponse"
CasesGetResponse.__module__ = "dokime.client"
CasesGetResponse.__qualname__ = "CasesGetResponse"
CasesListResponse.__module__ = "dokime.client"
CasesListResponse.__qualname__ = "CasesListResponse"
CasesParametersResponse.__module__ = "dokime.client"
CasesParametersResponse.__qualname__ = "CasesParametersResponse"
CasesWatchResponse.__module__ = "dokime.client"
CasesWatchResponse.__qualname__ = "CasesWatchResponse"
PlannedCase.__module__ = "dokime.client"
PlannedCase.__qualname__ = "PlannedCase"
RegisteredSelection.__module__ = "dokime.client"
RegisteredSelection.__qualname__ = "RegisteredSelection"
RegisteredSelectionFilesEntry.__module__ = "dokime.client"
RegisteredSelectionFilesEntry.__qualname__ = "RegisteredSelectionFilesEntry"
RegisteredSelectionParametersEntry.__module__ = "dokime.client"
RegisteredSelectionParametersEntry.__qualname__ = "RegisteredSelectionParametersEntry"
SelectedFile.__module__ = "dokime.client"
SelectedFile.__qualname__ = "SelectedFile"
SelectedFileKind.__module__ = "dokime.client"
SelectedFileKind.__qualname__ = "SelectedFileKind"
SelectionPlan.__module__ = "dokime.client"
SelectionPlan.__qualname__ = "SelectionPlan"
SelectionValues.__module__ = "dokime.client"
SelectionValues.__qualname__ = "SelectionValues"
ContentArtifactsGetResponse.__module__ = "dokime.client"
ContentArtifactsGetResponse.__qualname__ = "ContentArtifactsGetResponse"
ContentArtifactsListResponse.__module__ = "dokime.client"
ContentArtifactsListResponse.__qualname__ = "ContentArtifactsListResponse"
ContentArtifactsWatchResponse.__module__ = "dokime.client"
ContentArtifactsWatchResponse.__qualname__ = "ContentArtifactsWatchResponse"
ContentExecutionFileMetadata.__module__ = "dokime.client"
ContentExecutionFileMetadata.__qualname__ = "ContentExecutionFileMetadata"
ContentExecutionFilesGetResponse.__module__ = "dokime.client"
ContentExecutionFilesGetResponse.__qualname__ = "ContentExecutionFilesGetResponse"
ContentExecutionFilesListResponse.__module__ = "dokime.client"
ContentExecutionFilesListResponse.__qualname__ = "ContentExecutionFilesListResponse"
ContentExecutionFilesWatchResponse.__module__ = "dokime.client"
ContentExecutionFilesWatchResponse.__qualname__ = "ContentExecutionFilesWatchResponse"
CycleFilter.__module__ = "dokime.client"
CycleFilter.__qualname__ = "CycleFilter"
CycleOrder.__module__ = "dokime.client"
CycleOrder.__qualname__ = "CycleOrder"
CycleSelection.__module__ = "dokime.client"
CycleSelection.__qualname__ = "CycleSelection"
CycleSummary.__module__ = "dokime.client"
CycleSummary.__qualname__ = "CycleSummary"
CycleWritable.__module__ = "dokime.client"
CycleWritable.__qualname__ = "CycleWritable"
CyclesCreateResponse.__module__ = "dokime.client"
CyclesCreateResponse.__qualname__ = "CyclesCreateResponse"
CyclesGetResponse.__module__ = "dokime.client"
CyclesGetResponse.__qualname__ = "CyclesGetResponse"
CyclesListResponse.__module__ = "dokime.client"
CyclesListResponse.__qualname__ = "CyclesListResponse"
CyclesSealResponse.__module__ = "dokime.client"
CyclesSealResponse.__qualname__ = "CyclesSealResponse"
CyclesSetResponse.__module__ = "dokime.client"
CyclesSetResponse.__qualname__ = "CyclesSetResponse"
CyclesWatchResponse.__module__ = "dokime.client"
CyclesWatchResponse.__qualname__ = "CyclesWatchResponse"
RerunCaseNotice.__module__ = "dokime.client"
RerunCaseNotice.__qualname__ = "RerunCaseNotice"
ResultSummary.__module__ = "dokime.client"
ResultSummary.__qualname__ = "ResultSummary"
DocumentBinding.__module__ = "dokime.client"
DocumentBinding.__qualname__ = "DocumentBinding"
DocumentBindingResult.__module__ = "dokime.client"
DocumentBindingResult.__qualname__ = "DocumentBindingResult"
DocumentCard.__module__ = "dokime.client"
DocumentCard.__qualname__ = "DocumentCard"
DocumentCardBody.__module__ = "dokime.client"
DocumentCardBody.__qualname__ = "DocumentCardBody"
DocumentCardBudget.__module__ = "dokime.client"
DocumentCardBudget.__qualname__ = "DocumentCardBudget"
DocumentCardFailure.__module__ = "dokime.client"
DocumentCardFailure.__qualname__ = "DocumentCardFailure"
DocumentCompletion.__module__ = "dokime.client"
DocumentCompletion.__qualname__ = "DocumentCompletion"
DocumentEvidenceCut.__module__ = "dokime.client"
DocumentEvidenceCut.__qualname__ = "DocumentEvidenceCut"
DocumentFrame.__module__ = "dokime.client"
DocumentFrame.__qualname__ = "DocumentFrame"
DocumentFrameFrame.__module__ = "dokime.client"
DocumentFrameFrame.__qualname__ = "DocumentFrameFrame"
DocumentFrontiers.__module__ = "dokime.client"
DocumentFrontiers.__qualname__ = "DocumentFrontiers"
DocumentHeader.__module__ = "dokime.client"
DocumentHeader.__qualname__ = "DocumentHeader"
DocumentLayoutNode.__module__ = "dokime.client"
DocumentLayoutNode.__qualname__ = "DocumentLayoutNode"
DocumentLayoutNodeNode.__module__ = "dokime.client"
DocumentLayoutNodeNode.__qualname__ = "DocumentLayoutNodeNode"
DocumentOwnerReference.__module__ = "dokime.client"
DocumentOwnerReference.__qualname__ = "DocumentOwnerReference"
DocumentPage.__module__ = "dokime.client"
DocumentPage.__qualname__ = "DocumentPage"
DocumentPlacement.__module__ = "dokime.client"
DocumentPlacement.__qualname__ = "DocumentPlacement"
DocumentPlotDefinition.__module__ = "dokime.client"
DocumentPlotDefinition.__qualname__ = "DocumentPlotDefinition"
DocumentRepeat.__module__ = "dokime.client"
DocumentRepeat.__qualname__ = "DocumentRepeat"
DocumentRepeatInstance.__module__ = "dokime.client"
DocumentRepeatInstance.__qualname__ = "DocumentRepeatInstance"
DocumentRepeatSelection.__module__ = "dokime.client"
DocumentRepeatSelection.__qualname__ = "DocumentRepeatSelection"
DocumentScope.__module__ = "dokime.client"
DocumentScope.__qualname__ = "DocumentScope"
DocumentSection.__module__ = "dokime.client"
DocumentSection.__qualname__ = "DocumentSection"
DocumentSectionIdentity.__module__ = "dokime.client"
DocumentSectionIdentity.__qualname__ = "DocumentSectionIdentity"
DocumentSpan.__module__ = "dokime.client"
DocumentSpan.__qualname__ = "DocumentSpan"
DocumentSummary.__module__ = "dokime.client"
DocumentSummary.__qualname__ = "DocumentSummary"
DocumentSummaryCondition.__module__ = "dokime.client"
DocumentSummaryCondition.__qualname__ = "DocumentSummaryCondition"
DocumentSummaryGroup.__module__ = "dokime.client"
DocumentSummaryGroup.__qualname__ = "DocumentSummaryGroup"
DocumentSummaryGroupDefinition.__module__ = "dokime.client"
DocumentSummaryGroupDefinition.__qualname__ = "DocumentSummaryGroupDefinition"
DocumentSummaryTile.__module__ = "dokime.client"
DocumentSummaryTile.__qualname__ = "DocumentSummaryTile"
DocumentSummaryTileDefinition.__module__ = "dokime.client"
DocumentSummaryTileDefinition.__qualname__ = "DocumentSummaryTileDefinition"
DocumentTable.__module__ = "dokime.client"
DocumentTable.__qualname__ = "DocumentTable"
DocumentTableRow.__module__ = "dokime.client"
DocumentTableRow.__qualname__ = "DocumentTableRow"
DocumentTelemetryCut.__module__ = "dokime.client"
DocumentTelemetryCut.__qualname__ = "DocumentTelemetryCut"
DocumentWarning.__module__ = "dokime.client"
DocumentWarning.__qualname__ = "DocumentWarning"
DocumentsGetResponse.__module__ = "dokime.client"
DocumentsGetResponse.__qualname__ = "DocumentsGetResponse"
DocumentsWatchResponse.__module__ = "dokime.client"
DocumentsWatchResponse.__qualname__ = "DocumentsWatchResponse"
PlotPresentation.__module__ = "dokime.client"
PlotPresentation.__qualname__ = "PlotPresentation"
PlotReferenceBand.__module__ = "dokime.client"
PlotReferenceBand.__qualname__ = "PlotReferenceBand"
PlotReferenceLine.__module__ = "dokime.client"
PlotReferenceLine.__qualname__ = "PlotReferenceLine"
PlotReferenceValue.__module__ = "dokime.client"
PlotReferenceValue.__qualname__ = "PlotReferenceValue"
EvidenceHealthEventsListResponse.__module__ = "dokime.client"
EvidenceHealthEventsListResponse.__qualname__ = "EvidenceHealthEventsListResponse"
EvidenceHealthEventsWatchResponse.__module__ = "dokime.client"
EvidenceHealthEventsWatchResponse.__qualname__ = "EvidenceHealthEventsWatchResponse"
EvidenceHealthItem.__module__ = "dokime.client"
EvidenceHealthItem.__qualname__ = "EvidenceHealthItem"
EvidenceAnnotationOwner.__module__ = "dokime.client"
EvidenceAnnotationOwner.__qualname__ = "EvidenceAnnotationOwner"
EvidenceAnnotationOwnerOwner.__module__ = "dokime.client"
EvidenceAnnotationOwnerOwner.__qualname__ = "EvidenceAnnotationOwnerOwner"
EvidenceAnnotationSelection.__module__ = "dokime.client"
EvidenceAnnotationSelection.__qualname__ = "EvidenceAnnotationSelection"
EvidenceExecutionNoteOwner.__module__ = "dokime.client"
EvidenceExecutionNoteOwner.__qualname__ = "EvidenceExecutionNoteOwner"
EvidenceNoteWritable.__module__ = "dokime.client"
EvidenceNoteWritable.__qualname__ = "EvidenceNoteWritable"
EvidenceNotesCreateResponse.__module__ = "dokime.client"
EvidenceNotesCreateResponse.__qualname__ = "EvidenceNotesCreateResponse"
EvidenceNotesGetResponse.__module__ = "dokime.client"
EvidenceNotesGetResponse.__qualname__ = "EvidenceNotesGetResponse"
EvidenceNotesListResponse.__module__ = "dokime.client"
EvidenceNotesListResponse.__qualname__ = "EvidenceNotesListResponse"
EvidenceNotesSetResponse.__module__ = "dokime.client"
EvidenceNotesSetResponse.__qualname__ = "EvidenceNotesSetResponse"
EvidenceNotesWatchResponse.__module__ = "dokime.client"
EvidenceNotesWatchResponse.__qualname__ = "EvidenceNotesWatchResponse"
EvidenceOperatorNote.__module__ = "dokime.client"
EvidenceOperatorNote.__qualname__ = "EvidenceOperatorNote"
EvidencePlannedCaseNoteOwner.__module__ = "dokime.client"
EvidencePlannedCaseNoteOwner.__qualname__ = "EvidencePlannedCaseNoteOwner"
EvidenceRecordsWatchResponse.__module__ = "dokime.client"
EvidenceRecordsWatchResponse.__qualname__ = "EvidenceRecordsWatchResponse"
EvidenceBucket.__module__ = "dokime.client"
EvidenceBucket.__qualname__ = "EvidenceBucket"
EvidenceBucketRequest.__module__ = "dokime.client"
EvidenceBucketRequest.__qualname__ = "EvidenceBucketRequest"
EvidenceBucketValue.__module__ = "dokime.client"
EvidenceBucketValue.__qualname__ = "EvidenceBucketValue"
EvidenceSampleRow.__module__ = "dokime.client"
EvidenceSampleRow.__qualname__ = "EvidenceSampleRow"
EvidenceSamplesListResponse.__module__ = "dokime.client"
EvidenceSamplesListResponse.__qualname__ = "EvidenceSamplesListResponse"
EvidenceSamplesWatchResponse.__module__ = "dokime.client"
EvidenceSamplesWatchResponse.__qualname__ = "EvidenceSamplesWatchResponse"
EvidenceStatisticValue.__module__ = "dokime.client"
EvidenceStatisticValue.__qualname__ = "EvidenceStatisticValue"
EvidenceTelemetrySample.__module__ = "dokime.client"
EvidenceTelemetrySample.__qualname__ = "EvidenceTelemetrySample"
EvidenceTraceabilityWatchResponse.__module__ = "dokime.client"
EvidenceTraceabilityWatchResponse.__qualname__ = "EvidenceTraceabilityWatchResponse"
EvidenceArchiveResponse.__module__ = "dokime.client"
EvidenceArchiveResponse.__qualname__ = "EvidenceArchiveResponse"
EvidenceSubject.__module__ = "dokime.client"
EvidenceSubject.__qualname__ = "EvidenceSubject"
ExecutionsCommandsCreateResponse.__module__ = "dokime.client"
ExecutionsCommandsCreateResponse.__qualname__ = "ExecutionsCommandsCreateResponse"
ExecutionsCommandsGetResponse.__module__ = "dokime.client"
ExecutionsCommandsGetResponse.__qualname__ = "ExecutionsCommandsGetResponse"
ExecutionsExecutionCommandReceipt.__module__ = "dokime.client"
ExecutionsExecutionCommandReceipt.__qualname__ = "ExecutionsExecutionCommandReceipt"
ExecutionsProgressWatchResponse.__module__ = "dokime.client"
ExecutionsProgressWatchResponse.__qualname__ = "ExecutionsProgressWatchResponse"
ExecutionsSchemasGetResponse.__module__ = "dokime.client"
ExecutionsSchemasGetResponse.__qualname__ = "ExecutionsSchemasGetResponse"
DeclaredOutcome.__module__ = "dokime.client"
DeclaredOutcome.__qualname__ = "DeclaredOutcome"
Diagnostic.__module__ = "dokime.client"
Diagnostic.__qualname__ = "Diagnostic"
EvaluatedOutcome.__module__ = "dokime.client"
EvaluatedOutcome.__qualname__ = "EvaluatedOutcome"
ExecutionBinding.__module__ = "dokime.client"
ExecutionBinding.__qualname__ = "ExecutionBinding"
ExecutionControls.__module__ = "dokime.client"
ExecutionControls.__qualname__ = "ExecutionControls"
ExecutionDetail.__module__ = "dokime.client"
ExecutionDetail.__qualname__ = "ExecutionDetail"
ExecutionFilter.__module__ = "dokime.client"
ExecutionFilter.__qualname__ = "ExecutionFilter"
ExecutionOrder.__module__ = "dokime.client"
ExecutionOrder.__qualname__ = "ExecutionOrder"
ExecutionOutcome.__module__ = "dokime.client"
ExecutionOutcome.__qualname__ = "ExecutionOutcome"
ExecutionOutcomeCounts.__module__ = "dokime.client"
ExecutionOutcomeCounts.__qualname__ = "ExecutionOutcomeCounts"
ExecutionSelection.__module__ = "dokime.client"
ExecutionSelection.__qualname__ = "ExecutionSelection"
ExecutionStep.__module__ = "dokime.client"
ExecutionStep.__qualname__ = "ExecutionStep"
ExecutionSummary.__module__ = "dokime.client"
ExecutionSummary.__qualname__ = "ExecutionSummary"
ExecutionWritable.__module__ = "dokime.client"
ExecutionWritable.__qualname__ = "ExecutionWritable"
ExecutionsCancelResponse.__module__ = "dokime.client"
ExecutionsCancelResponse.__qualname__ = "ExecutionsCancelResponse"
ExecutionsCreateResponse.__module__ = "dokime.client"
ExecutionsCreateResponse.__qualname__ = "ExecutionsCreateResponse"
ExecutionsDeleteResponse.__module__ = "dokime.client"
ExecutionsDeleteResponse.__qualname__ = "ExecutionsDeleteResponse"
ExecutionsGetResponse.__module__ = "dokime.client"
ExecutionsGetResponse.__qualname__ = "ExecutionsGetResponse"
ExecutionsListResponse.__module__ = "dokime.client"
ExecutionsListResponse.__qualname__ = "ExecutionsListResponse"
ExecutionsParametersResponse.__module__ = "dokime.client"
ExecutionsParametersResponse.__qualname__ = "ExecutionsParametersResponse"
ExecutionsSetResponse.__module__ = "dokime.client"
ExecutionsSetResponse.__qualname__ = "ExecutionsSetResponse"
ExecutionsStartResponse.__module__ = "dokime.client"
ExecutionsStartResponse.__qualname__ = "ExecutionsStartResponse"
ExecutionsStepsResponse.__module__ = "dokime.client"
ExecutionsStepsResponse.__qualname__ = "ExecutionsStepsResponse"
ExecutionsStopResponse.__module__ = "dokime.client"
ExecutionsStopResponse.__qualname__ = "ExecutionsStopResponse"
ExecutionsWatchResponse.__module__ = "dokime.client"
ExecutionsWatchResponse.__qualname__ = "ExecutionsWatchResponse"
FrozenSchemas.__module__ = "dokime.client"
FrozenSchemas.__qualname__ = "FrozenSchemas"
MaterializedParameter.__module__ = "dokime.client"
MaterializedParameter.__qualname__ = "MaterializedParameter"
ReportStatus.__module__ = "dokime.client"
ReportStatus.__qualname__ = "ReportStatus"
StepCriterion.__module__ = "dokime.client"
StepCriterion.__qualname__ = "StepCriterion"
UnevaluatedOutcome.__module__ = "dokime.client"
UnevaluatedOutcome.__qualname__ = "UnevaluatedOutcome"
MediaCapture.__module__ = "dokime.client"
MediaCapture.__qualname__ = "MediaCapture"
MediaCapturesCreateResponse.__module__ = "dokime.client"
MediaCapturesCreateResponse.__qualname__ = "MediaCapturesCreateResponse"
MediaCapturesGetResponse.__module__ = "dokime.client"
MediaCapturesGetResponse.__qualname__ = "MediaCapturesGetResponse"
MediaCapturesStopResponse.__module__ = "dokime.client"
MediaCapturesStopResponse.__qualname__ = "MediaCapturesStopResponse"
MediaCapturesWatchResponse.__module__ = "dokime.client"
MediaCapturesWatchResponse.__qualname__ = "MediaCapturesWatchResponse"
MediaRecordingAudio.__module__ = "dokime.client"
MediaRecordingAudio.__qualname__ = "MediaRecordingAudio"
MediaRecordingProgress.__module__ = "dokime.client"
MediaRecordingProgress.__qualname__ = "MediaRecordingProgress"
PlotsPlotSettingsValue.__module__ = "dokime.client"
PlotsPlotSettingsValue.__qualname__ = "PlotsPlotSettingsValue"
PlotsSettingsGetResponse.__module__ = "dokime.client"
PlotsSettingsGetResponse.__qualname__ = "PlotsSettingsGetResponse"
PlotsSettingsSetResponse.__module__ = "dokime.client"
PlotsSettingsSetResponse.__qualname__ = "PlotsSettingsSetResponse"
PlotsContourGrid.__module__ = "dokime.client"
PlotsContourGrid.__qualname__ = "PlotsContourGrid"
PlotsGridCell.__module__ = "dokime.client"
PlotsGridCell.__qualname__ = "PlotsGridCell"
PlotsGridFrame.__module__ = "dokime.client"
PlotsGridFrame.__qualname__ = "PlotsGridFrame"
PlotsGridFrames.__module__ = "dokime.client"
PlotsGridFrames.__qualname__ = "PlotsGridFrames"
PlotsNullableNumber.__module__ = "dokime.client"
PlotsNullableNumber.__qualname__ = "PlotsNullableNumber"
PlotsPlotAnalysis.__module__ = "dokime.client"
PlotsPlotAnalysis.__qualname__ = "PlotsPlotAnalysis"
PlotsPlotAnalysisError.__module__ = "dokime.client"
PlotsPlotAnalysisError.__qualname__ = "PlotsPlotAnalysisError"
PlotsPlotAnalysisResult.__module__ = "dokime.client"
PlotsPlotAnalysisResult.__qualname__ = "PlotsPlotAnalysisResult"
PlotsPlotBoxSummary.__module__ = "dokime.client"
PlotsPlotBoxSummary.__qualname__ = "PlotsPlotBoxSummary"
PlotsPlotBoxSummaryAnalysis.__module__ = "dokime.client"
PlotsPlotBoxSummaryAnalysis.__qualname__ = "PlotsPlotBoxSummaryAnalysis"
PlotsPlotColumn.__module__ = "dokime.client"
PlotsPlotColumn.__qualname__ = "PlotsPlotColumn"
PlotsPlotDomain.__module__ = "dokime.client"
PlotsPlotDomain.__qualname__ = "PlotsPlotDomain"
PlotsPlotEnvelope.__module__ = "dokime.client"
PlotsPlotEnvelope.__qualname__ = "PlotsPlotEnvelope"
PlotsPlotFrame.__module__ = "dokime.client"
PlotsPlotFrame.__qualname__ = "PlotsPlotFrame"
PlotsPlotHistogramAnalysis.__module__ = "dokime.client"
PlotsPlotHistogramAnalysis.__qualname__ = "PlotsPlotHistogramAnalysis"
PlotsPlotHistogramBin.__module__ = "dokime.client"
PlotsPlotHistogramBin.__qualname__ = "PlotsPlotHistogramBin"
PlotsPlotHistogramSeries.__module__ = "dokime.client"
PlotsPlotHistogramSeries.__qualname__ = "PlotsPlotHistogramSeries"
PlotsPlotHistogramStatistics.__module__ = "dokime.client"
PlotsPlotHistogramStatistics.__qualname__ = "PlotsPlotHistogramStatistics"
PlotsPlotMarker.__module__ = "dokime.client"
PlotsPlotMarker.__qualname__ = "PlotsPlotMarker"
PlotsPlotPayload.__module__ = "dokime.client"
PlotsPlotPayload.__qualname__ = "PlotsPlotPayload"
PlotsPlotPayloadDiagnostics.__module__ = "dokime.client"
PlotsPlotPayloadDiagnostics.__qualname__ = "PlotsPlotPayloadDiagnostics"
PlotsPlotRequest.__module__ = "dokime.client"
PlotsPlotRequest.__qualname__ = "PlotsPlotRequest"
PlotsPlotSeries.__module__ = "dokime.client"
PlotsPlotSeries.__qualname__ = "PlotsPlotSeries"
PlotsPlotSeriesSummary.__module__ = "dokime.client"
PlotsPlotSeriesSummary.__qualname__ = "PlotsPlotSeriesSummary"
PlotsPlotSigmaBin.__module__ = "dokime.client"
PlotsPlotSigmaBin.__qualname__ = "PlotsPlotSigmaBin"
PlotsPlotSigmaIntervalAnalysis.__module__ = "dokime.client"
PlotsPlotSigmaIntervalAnalysis.__qualname__ = "PlotsPlotSigmaIntervalAnalysis"
PlotsPlotSigmaSeries.__module__ = "dokime.client"
PlotsPlotSigmaSeries.__qualname__ = "PlotsPlotSigmaSeries"
PlotsPlotStreamUpdate.__module__ = "dokime.client"
PlotsPlotStreamUpdate.__qualname__ = "PlotsPlotStreamUpdate"
PlotsPlotWindow.__module__ = "dokime.client"
PlotsPlotWindow.__qualname__ = "PlotsPlotWindow"
PlotsSnapshotsListResponse.__module__ = "dokime.client"
PlotsSnapshotsListResponse.__qualname__ = "PlotsSnapshotsListResponse"
PlotsSnapshotsWatchResponse.__module__ = "dokime.client"
PlotsSnapshotsWatchResponse.__qualname__ = "PlotsSnapshotsWatchResponse"
PluginsSecretRequirement.__module__ = "dokime.client"
PluginsSecretRequirement.__qualname__ = "PluginsSecretRequirement"
PluginsSecretsListResponse.__module__ = "dokime.client"
PluginsSecretsListResponse.__qualname__ = "PluginsSecretsListResponse"
PromptDefinitionsGetResponse.__module__ = "dokime.client"
PromptDefinitionsGetResponse.__qualname__ = "PromptDefinitionsGetResponse"
PromptDefinitionsListResponse.__module__ = "dokime.client"
PromptDefinitionsListResponse.__qualname__ = "PromptDefinitionsListResponse"
PromptsAnswersCreateResponse.__module__ = "dokime.client"
PromptsAnswersCreateResponse.__qualname__ = "PromptsAnswersCreateResponse"
PromptInstance.__module__ = "dokime.client"
PromptInstance.__qualname__ = "PromptInstance"
PromptResponse.__module__ = "dokime.client"
PromptResponse.__qualname__ = "PromptResponse"
PromptView.__module__ = "dokime.client"
PromptView.__qualname__ = "PromptView"
PromptsGetResponse.__module__ = "dokime.client"
PromptsGetResponse.__qualname__ = "PromptsGetResponse"
PromptsListResponse.__module__ = "dokime.client"
PromptsListResponse.__qualname__ = "PromptsListResponse"
PromptsWatchResponse.__module__ = "dokime.client"
PromptsWatchResponse.__qualname__ = "PromptsWatchResponse"
AbsoluteTimeFormat.__module__ = "dokime.client"
AbsoluteTimeFormat.__qualname__ = "AbsoluteTimeFormat"
ElapsedFormat.__module__ = "dokime.client"
ElapsedFormat.__qualname__ = "ElapsedFormat"
EngineeringFormat.__module__ = "dokime.client"
EngineeringFormat.__qualname__ = "EngineeringFormat"
FixedFormat.__module__ = "dokime.client"
FixedFormat.__qualname__ = "FixedFormat"
MeasurementUnit.__module__ = "dokime.client"
MeasurementUnit.__qualname__ = "MeasurementUnit"
NormalFormat.__module__ = "dokime.client"
NormalFormat.__qualname__ = "NormalFormat"
RecordDisplay.__module__ = "dokime.client"
RecordDisplay.__qualname__ = "RecordDisplay"
RecordField.__module__ = "dokime.client"
RecordField.__qualname__ = "RecordField"
RecordSchema.__module__ = "dokime.client"
RecordSchema.__qualname__ = "RecordSchema"
RecordSchemasGetResponse.__module__ = "dokime.client"
RecordSchemasGetResponse.__qualname__ = "RecordSchemasGetResponse"
RecordSchemasListResponse.__module__ = "dokime.client"
RecordSchemasListResponse.__qualname__ = "RecordSchemasListResponse"
ScientificFormat.__module__ = "dokime.client"
ScientificFormat.__qualname__ = "ScientificFormat"
SiPrefixFormat.__module__ = "dokime.client"
SiPrefixFormat.__qualname__ = "SiPrefixFormat"
TelemetryColumn.__module__ = "dokime.client"
TelemetryColumn.__qualname__ = "TelemetryColumn"
TelemetrySchema.__module__ = "dokime.client"
TelemetrySchema.__qualname__ = "TelemetrySchema"
ValueFormat.__module__ = "dokime.client"
ValueFormat.__qualname__ = "ValueFormat"
ValueFormatKind.__module__ = "dokime.client"
ValueFormatKind.__qualname__ = "ValueFormatKind"
ReportMetadata.__module__ = "dokime.client"
ReportMetadata.__qualname__ = "ReportMetadata"
ReportSubject.__module__ = "dokime.client"
ReportSubject.__qualname__ = "ReportSubject"
ReportsCreateResponse.__module__ = "dokime.client"
ReportsCreateResponse.__qualname__ = "ReportsCreateResponse"
ReportsGetResponse.__module__ = "dokime.client"
ReportsGetResponse.__qualname__ = "ReportsGetResponse"
ReportsPublishResponse.__module__ = "dokime.client"
ReportsPublishResponse.__qualname__ = "ReportsPublishResponse"
SecretArchive.__module__ = "dokime.client"
SecretArchive.__qualname__ = "SecretArchive"
SecretImportChoice.__module__ = "dokime.client"
SecretImportChoice.__qualname__ = "SecretImportChoice"
SecretImportEntry.__module__ = "dokime.client"
SecretImportEntry.__qualname__ = "SecretImportEntry"
SecretImportPreview.__module__ = "dokime.client"
SecretImportPreview.__qualname__ = "SecretImportPreview"
SecretSnapshot.__module__ = "dokime.client"
SecretSnapshot.__qualname__ = "SecretSnapshot"
SecretSummary.__module__ = "dokime.client"
SecretSummary.__qualname__ = "SecretSummary"
SecretsDeleteResponse.__module__ = "dokime.client"
SecretsDeleteResponse.__qualname__ = "SecretsDeleteResponse"
SecretsExportResponse.__module__ = "dokime.client"
SecretsExportResponse.__qualname__ = "SecretsExportResponse"
SecretsImportResponse.__module__ = "dokime.client"
SecretsImportResponse.__qualname__ = "SecretsImportResponse"
SecretsListResponse.__module__ = "dokime.client"
SecretsListResponse.__qualname__ = "SecretsListResponse"
SecretsSetResponse.__module__ = "dokime.client"
SecretsSetResponse.__qualname__ = "SecretsSetResponse"
SecretsWatchResponse.__module__ = "dokime.client"
SecretsWatchResponse.__qualname__ = "SecretsWatchResponse"
ApplicationAppearance.__module__ = "dokime.client"
ApplicationAppearance.__qualname__ = "ApplicationAppearance"
ApplicationSettings.__module__ = "dokime.client"
ApplicationSettings.__qualname__ = "ApplicationSettings"
BootstrapPaths.__module__ = "dokime.client"
BootstrapPaths.__qualname__ = "BootstrapPaths"
CachedRefresh.__module__ = "dokime.client"
CachedRefresh.__qualname__ = "CachedRefresh"
ClientPreferences.__module__ = "dokime.client"
ClientPreferences.__qualname__ = "ClientPreferences"
HostCardBlock.__module__ = "dokime.client"
HostCardBlock.__qualname__ = "HostCardBlock"
HostCardBlockKind.__module__ = "dokime.client"
HostCardBlockKind.__qualname__ = "HostCardBlockKind"
HostCardBody.__module__ = "dokime.client"
HostCardBody.__qualname__ = "HostCardBody"
HostCardKeyValue.__module__ = "dokime.client"
HostCardKeyValue.__qualname__ = "HostCardKeyValue"
HostCardKeyValueList.__module__ = "dokime.client"
HostCardKeyValueList.__qualname__ = "HostCardKeyValueList"
HostCardMetric.__module__ = "dokime.client"
HostCardMetric.__qualname__ = "HostCardMetric"
HostCardMetrics.__module__ = "dokime.client"
HostCardMetrics.__qualname__ = "HostCardMetrics"
HostCardStatus.__module__ = "dokime.client"
HostCardStatus.__qualname__ = "HostCardStatus"
ManualRefresh.__module__ = "dokime.client"
ManualRefresh.__qualname__ = "ManualRefresh"
OnDemandRefresh.__module__ = "dokime.client"
OnDemandRefresh.__qualname__ = "OnDemandRefresh"
RemoteListenerSettings.__module__ = "dokime.client"
RemoteListenerSettings.__qualname__ = "RemoteListenerSettings"
ReportDefaults.__module__ = "dokime.client"
ReportDefaults.__qualname__ = "ReportDefaults"
SettingsCard.__module__ = "dokime.client"
SettingsCard.__qualname__ = "SettingsCard"
SettingsCardRefreshPolicy.__module__ = "dokime.client"
SettingsCardRefreshPolicy.__qualname__ = "SettingsCardRefreshPolicy"
SettingsCardRefreshPolicyPolicy.__module__ = "dokime.client"
SettingsCardRefreshPolicyPolicy.__qualname__ = "SettingsCardRefreshPolicyPolicy"
SettingsFrame.__module__ = "dokime.client"
SettingsFrame.__qualname__ = "SettingsFrame"
SettingsGetResponse.__module__ = "dokime.client"
SettingsGetResponse.__qualname__ = "SettingsGetResponse"
SettingsSchemaResponse.__module__ = "dokime.client"
SettingsSchemaResponse.__qualname__ = "SettingsSchemaResponse"
SettingsSection.__module__ = "dokime.client"
SettingsSection.__qualname__ = "SettingsSection"
SettingsSectionOwner.__module__ = "dokime.client"
SettingsSectionOwner.__qualname__ = "SettingsSectionOwner"
SettingsSetResponse.__module__ = "dokime.client"
SettingsSetResponse.__qualname__ = "SettingsSetResponse"
TelemetryPreferences.__module__ = "dokime.client"
TelemetryPreferences.__qualname__ = "TelemetryPreferences"
StorageSourcesGetResponse.__module__ = "dokime.client"
StorageSourcesGetResponse.__qualname__ = "StorageSourcesGetResponse"
StorageSourcesSetResponse.__module__ = "dokime.client"
StorageSourcesSetResponse.__qualname__ = "StorageSourcesSetResponse"
StorageStorageSource.__module__ = "dokime.client"
StorageStorageSource.__qualname__ = "StorageStorageSource"
StorageStorageSources.__module__ = "dokime.client"
StorageStorageSources.__qualname__ = "StorageStorageSources"
StreamSchemasGetResponse.__module__ = "dokime.client"
StreamSchemasGetResponse.__qualname__ = "StreamSchemasGetResponse"
StreamSchemasListResponse.__module__ = "dokime.client"
StreamSchemasListResponse.__qualname__ = "StreamSchemasListResponse"
SuitesGetResponse.__module__ = "dokime.client"
SuitesGetResponse.__qualname__ = "SuitesGetResponse"
SuitesListResponse.__module__ = "dokime.client"
SuitesListResponse.__qualname__ = "SuitesListResponse"

__all__ = [
    "ADJACENT",
    "APPEARANCE_FAILURE_KIND_ASSET_CHANGED",
    "APPEARANCE_FAILURE_KIND_ASSET_MISSING",
    "APPEARANCE_FAILURE_KIND_ASSET_UNAVAILABLE",
    "APPEARANCE_FAILURE_KIND_INVALID_REFERENCE",
    "APPEARANCE_FAILURE_KIND_TOO_LARGE",
    "APPEARANCE_FAILURE_KIND_UNSPECIFIED",
    "ARCHIVE_SCOPE_ALL",
    "ARCHIVE_SCOPE_ARCHIVED",
    "ARCHIVE_SCOPE_UNARCHIVED",
    "ARCHIVE_SCOPE_UNSPECIFIED",
    "BOOL",
    "BYTES",
    "CAPTURE_KIND_AUDIO",
    "CAPTURE_KIND_CAMERA",
    "CAPTURE_KIND_PHOTO",
    "CAPTURE_KIND_SCREEN",
    "CAPTURE_KIND_SCREENSHOT",
    "CAPTURE_KIND_UNSPECIFIED",
    "CASE_STATUS_BLOCKED",
    "CASE_STATUS_CANCELLED",
    "CASE_STATUS_FAILED",
    "CASE_STATUS_INTERRUPTED",
    "CASE_STATUS_INVALIDATED",
    "CASE_STATUS_PASSED",
    "CASE_STATUS_PLANNED",
    "CASE_STATUS_QUEUED",
    "CASE_STATUS_RUNNING",
    "CASE_STATUS_SKIPPED",
    "CASE_STATUS_UNSPECIFIED",
    "COMPONENT_STATE_DEGRADED",
    "COMPONENT_STATE_HEALTHY",
    "COMPONENT_STATE_UNAVAILABLE",
    "COMPONENT_STATE_UNSPECIFIED",
    "CRITERION_STATUS_FAILED",
    "CRITERION_STATUS_INDETERMINATE",
    "CRITERION_STATUS_PASSED",
    "CRITERION_STATUS_PENDING",
    "CRITERION_STATUS_SKIPPED",
    "CRITERION_STATUS_UNSPECIFIED",
    "CYCLE_ORDER_KEY_CREATED_AT",
    "CYCLE_ORDER_KEY_LABEL",
    "CYCLE_ORDER_KEY_UNSPECIFIED",
    "CYCLE_ORDER_KEY_UPDATED_AT",
    "CYCLE_PHASE_ACTIVE",
    "CYCLE_PHASE_SEALED",
    "CYCLE_PHASE_SEALING",
    "CYCLE_PHASE_SEAL_FAILED",
    "CYCLE_PHASE_UNSPECIFIED",
    "CYCLE_STATUS_BLOCKED",
    "CYCLE_STATUS_COMPLETED",
    "CYCLE_STATUS_FAILED",
    "CYCLE_STATUS_PAUSED",
    "CYCLE_STATUS_READY",
    "CYCLE_STATUS_RUNNING",
    "CYCLE_STATUS_STOPPED",
    "CYCLE_STATUS_STOPPING",
    "CYCLE_STATUS_UNSPECIFIED",
    "DENSITY_COMFORTABLE",
    "DENSITY_COMPACT",
    "DENSITY_TOUCH",
    "DENSITY_UNSPECIFIED",
    "DOCUMENT_CARD_KIND_ARTIFACTS",
    "DOCUMENT_CARD_KIND_CRITERIA",
    "DOCUMENT_CARD_KIND_HEALTH",
    "DOCUMENT_CARD_KIND_MEDIA",
    "DOCUMENT_CARD_KIND_NOTES",
    "DOCUMENT_CARD_KIND_PLOT",
    "DOCUMENT_CARD_KIND_RECORDS",
    "DOCUMENT_CARD_KIND_STEPS",
    "DOCUMENT_CARD_KIND_TABLE",
    "DOCUMENT_CARD_KIND_TELEMETRY_CONTEXT",
    "DOCUMENT_CARD_KIND_TELEMETRY_SUMMARY",
    "DOCUMENT_CARD_KIND_TEXT",
    "DOCUMENT_CARD_KIND_TRACEABILITY",
    "DOCUMENT_CARD_KIND_UNSPECIFIED",
    "DOCUMENT_DENSITY_COMPACT",
    "DOCUMENT_DENSITY_NORMAL",
    "DOCUMENT_DENSITY_SPACIOUS",
    "DOCUMENT_DENSITY_UNSPECIFIED",
    "DOCUMENT_RAIL_LEFT",
    "DOCUMENT_RAIL_MAIN",
    "DOCUMENT_RAIL_RIGHT",
    "DOCUMENT_RAIL_UNSPECIFIED",
    "DOCUMENT_SUMMARY_FORMAT_AUTO",
    "DOCUMENT_SUMMARY_FORMAT_DISTANCE",
    "DOCUMENT_SUMMARY_FORMAT_DURATION",
    "DOCUMENT_SUMMARY_FORMAT_NUMBER",
    "DOCUMENT_SUMMARY_FORMAT_PERCENT",
    "DOCUMENT_SUMMARY_FORMAT_STATUS",
    "DOCUMENT_SUMMARY_FORMAT_TEXT",
    "DOCUMENT_SUMMARY_FORMAT_UNSPECIFIED",
    "DOCUMENT_SUMMARY_PRESENTATION_BADGE",
    "DOCUMENT_SUMMARY_PRESENTATION_PROGRESS",
    "DOCUMENT_SUMMARY_PRESENTATION_UNSPECIFIED",
    "DOCUMENT_SUMMARY_PRESENTATION_VALUE",
    "DOCUMENT_SUMMARY_STATE_CRITICAL",
    "DOCUMENT_SUMMARY_STATE_MUTED",
    "DOCUMENT_SUMMARY_STATE_OK",
    "DOCUMENT_SUMMARY_STATE_UNSPECIFIED",
    "DOCUMENT_SUMMARY_STATE_WARNING",
    "DOCUMENT_SUMMARY_STAT_LATEST",
    "DOCUMENT_SUMMARY_STAT_UNSPECIFIED",
    "ELAPSED_UNIT_MILLISECONDS",
    "ELAPSED_UNIT_MINUTES",
    "ELAPSED_UNIT_SECONDS",
    "ELAPSED_UNIT_UNSPECIFIED",
    "EXECUTION_COMMAND_ACTION_FINISH",
    "EXECUTION_COMMAND_ACTION_NEXT",
    "EXECUTION_COMMAND_ACTION_RUN_STEP",
    "EXECUTION_COMMAND_ACTION_UNSPECIFIED",
    "EXECUTION_COMMAND_PHASE_ACCEPTED",
    "EXECUTION_COMMAND_PHASE_COMPLETED",
    "EXECUTION_COMMAND_PHASE_DISPATCHING",
    "EXECUTION_COMMAND_PHASE_FAILED",
    "EXECUTION_COMMAND_PHASE_INTERRUPTED",
    "EXECUTION_COMMAND_PHASE_REFUSED",
    "EXECUTION_COMMAND_PHASE_RUNNING",
    "EXECUTION_COMMAND_PHASE_UNSPECIFIED",
    "EXECUTION_FILE_KIND_BINARY",
    "EXECUTION_FILE_KIND_MEDIA",
    "EXECUTION_FILE_KIND_TEXT",
    "EXECUTION_FILE_KIND_UNKNOWN",
    "EXECUTION_FILE_KIND_UNSPECIFIED",
    "EXECUTION_FILE_STATE_FINALIZED",
    "EXECUTION_FILE_STATE_INTERRUPTED",
    "EXECUTION_FILE_STATE_STREAMING",
    "EXECUTION_FILE_STATE_UNSPECIFIED",
    "EXECUTION_MODE_AUTOMATIC",
    "EXECUTION_MODE_INTERACTIVE",
    "EXECUTION_MODE_UNSPECIFIED",
    "EXECUTION_ORDER_KEY_FINISHED_AT",
    "EXECUTION_ORDER_KEY_STARTED_AT",
    "EXECUTION_ORDER_KEY_TITLE",
    "EXECUTION_ORDER_KEY_UNSPECIFIED",
    "EXECUTION_PARAMETER_SET_INPUTS",
    "EXECUTION_PARAMETER_SET_OUTPUTS",
    "EXECUTION_PARAMETER_SET_UNSPECIFIED",
    "EXECUTION_STATE_CLOSED",
    "EXECUTION_STATE_OPEN",
    "EXECUTION_STATE_SEALED",
    "EXECUTION_STATE_UNSPECIFIED",
    "EXTERNAL",
    "FLOAT32",
    "FLOAT64",
    "HOST_CARD_TONE_DANGER",
    "HOST_CARD_TONE_INFO",
    "HOST_CARD_TONE_NEUTRAL",
    "HOST_CARD_TONE_SUCCESS",
    "HOST_CARD_TONE_UNSPECIFIED",
    "HOST_CARD_TONE_WARNING",
    "INT32",
    "INT64",
    "INTERNAL",
    "LIVE_EXECUTION_STATE_CANCELLED",
    "LIVE_EXECUTION_STATE_CLEANING_UP",
    "LIVE_EXECUTION_STATE_EVALUATING",
    "LIVE_EXECUTION_STATE_FINISHED",
    "LIVE_EXECUTION_STATE_INITIALIZING",
    "LIVE_EXECUTION_STATE_INTERRUPTED",
    "LIVE_EXECUTION_STATE_QUEUED",
    "LIVE_EXECUTION_STATE_STEPPING",
    "LIVE_EXECUTION_STATE_UNSPECIFIED",
    "LOGO_VARIANT_CONTAINED_ON_DARK",
    "LOGO_VARIANT_CONTAINED_ON_LIGHT",
    "LOGO_VARIANT_TRANSPARENT_ON_DARK",
    "LOGO_VARIANT_TRANSPARENT_ON_LIGHT",
    "LOGO_VARIANT_UNSPECIFIED",
    "MESSAGE",
    "OBJECT",
    "OMIT_EMPTY",
    "OMIT_NONE",
    "OPTIONAL",
    "PARAMETER_SOURCE_DEFINITION",
    "PARAMETER_SOURCE_ENVIRONMENT",
    "PARAMETER_SOURCE_OPERATOR",
    "PARAMETER_SOURCE_OUTPUT",
    "PARAMETER_SOURCE_RECORD",
    "PARAMETER_SOURCE_RESOURCE",
    "PARAMETER_SOURCE_RUNTIME",
    "PARAMETER_SOURCE_TELEMETRY",
    "PARAMETER_SOURCE_TRACEABILITY",
    "PARAMETER_SOURCE_UNSPECIFIED",
    "PAYLOAD_ENCODING_UNSPECIFIED",
    "PLOT_AXIS_KIND_ELAPSED",
    "PLOT_AXIS_KIND_UNSPECIFIED",
    "PLOT_AXIS_KIND_WALL_TIME",
    "PLOT_CARTESIAN_MODE_ALIGNED",
    "PLOT_CARTESIAN_MODE_FACETED",
    "PLOT_CARTESIAN_MODE_UNSPECIFIED",
    "PLOT_STREAM_KIND_DELTA",
    "PLOT_STREAM_KIND_ERROR",
    "PLOT_STREAM_KIND_REPLACE",
    "PLOT_STREAM_KIND_SNAPSHOT",
    "PLOT_STREAM_KIND_STATISTICS",
    "PLOT_STREAM_KIND_UNSPECIFIED",
    "PLUGIN_STATE_CONNECTING",
    "PLUGIN_STATE_READY",
    "PLUGIN_STATE_STOPPED",
    "PLUGIN_STATE_STOPPING",
    "PLUGIN_STATE_UNAVAILABLE",
    "PLUGIN_STATE_UNSPECIFIED",
    "POSTCARD_V1",
    "PROMPT_KIND_CHOICE",
    "PROMPT_KIND_CONFIRMATION",
    "PROMPT_KIND_FORM",
    "PROMPT_KIND_STATUS",
    "PROMPT_KIND_UNSPECIFIED",
    "PROMPT_KIND_VERIFICATION",
    "QUERY_FILTER_OPERATOR_EQ",
    "QUERY_FILTER_OPERATOR_GT",
    "QUERY_FILTER_OPERATOR_GTE",
    "QUERY_FILTER_OPERATOR_IN",
    "QUERY_FILTER_OPERATOR_IS_NULL",
    "QUERY_FILTER_OPERATOR_LT",
    "QUERY_FILTER_OPERATOR_LTE",
    "QUERY_FILTER_OPERATOR_NEQ",
    "QUERY_FILTER_OPERATOR_UNSPECIFIED",
    "QUERY_SOURCE_CASES",
    "QUERY_SOURCE_CYCLES",
    "QUERY_SOURCE_EXECUTIONS",
    "QUERY_SOURCE_RECORDS",
    "QUERY_SOURCE_STEPS",
    "QUERY_SOURCE_UNSPECIFIED",
    "RECORD_ICON_ACTIVITY",
    "RECORD_ICON_ALERT_TRIANGLE",
    "RECORD_ICON_ARCHIVE",
    "RECORD_ICON_BELL",
    "RECORD_ICON_CHECK_CIRCLE",
    "RECORD_ICON_CIRCLE_DASHED",
    "RECORD_ICON_CLIPBOARD_CHECK",
    "RECORD_ICON_CLOCK",
    "RECORD_ICON_CPU",
    "RECORD_ICON_FILE_TEXT",
    "RECORD_ICON_FLAG",
    "RECORD_ICON_GAUGE",
    "RECORD_ICON_HEART_PULSE",
    "RECORD_ICON_HELP_CIRCLE",
    "RECORD_ICON_MESSAGE_SQUARE_TEXT",
    "RECORD_ICON_PACKAGE",
    "RECORD_ICON_PLUG",
    "RECORD_ICON_RADIO_TOWER",
    "RECORD_ICON_ROTATE_CCW",
    "RECORD_ICON_SHIELD_ALERT",
    "RECORD_ICON_TARGET",
    "RECORD_ICON_TIMER_RESET",
    "RECORD_ICON_UNSPECIFIED",
    "RECORD_ICON_WRENCH",
    "RECORD_ICON_ZAP",
    "RECORD_TONE_ACCENT",
    "RECORD_TONE_DANGER",
    "RECORD_TONE_INFO",
    "RECORD_TONE_MUTED",
    "RECORD_TONE_NEUTRAL",
    "RECORD_TONE_SUCCESS",
    "RECORD_TONE_UNSPECIFIED",
    "RECORD_TONE_WARNING",
    "REPEATED",
    "REPORT_ORIGIN_CURRENT",
    "REPORT_ORIGIN_SEALED",
    "REPORT_ORIGIN_UNSPECIFIED",
    "REPORT_STATE_FAILED",
    "REPORT_STATE_PENDING",
    "REPORT_STATE_READY",
    "REPORT_STATE_UNSPECIFIED",
    "REQUIRED",
    "RESULT_STATUS_BLOCKED",
    "RESULT_STATUS_CANCELLED",
    "RESULT_STATUS_FAILED",
    "RESULT_STATUS_PASSED",
    "RESULT_STATUS_PENDING",
    "RESULT_STATUS_UNSPECIFIED",
    "SECRET_IMPORT_ACTION_REPLACE",
    "SECRET_IMPORT_ACTION_SKIP",
    "SECRET_IMPORT_ACTION_UNSPECIFIED",
    "SETTINGS_CARD_STATE_DISABLED",
    "SETTINGS_CARD_STATE_DISCONNECTED",
    "SETTINGS_CARD_STATE_ERROR",
    "SETTINGS_CARD_STATE_PERMISSION_DENIED",
    "SETTINGS_CARD_STATE_READY",
    "SETTINGS_CARD_STATE_STALE",
    "SETTINGS_CARD_STATE_UNSPECIFIED",
    "STEP_STATUS_CANCELLED",
    "STEP_STATUS_FAILED",
    "STEP_STATUS_PASSED",
    "STEP_STATUS_PENDING",
    "STEP_STATUS_RUNNING",
    "STEP_STATUS_SKIPPED",
    "STEP_STATUS_UNSPECIFIED",
    "STRING",
    "TELEMETRY_ROLE_COUNTER",
    "TELEMETRY_ROLE_DISTANCE",
    "TELEMETRY_ROLE_LATENCY",
    "TELEMETRY_ROLE_STATE",
    "TELEMETRY_ROLE_THROUGHPUT",
    "TELEMETRY_ROLE_UNSPECIFIED",
    "TELEMETRY_ROLE_VALUE",
    "THEME_MODE_DARK",
    "THEME_MODE_LIGHT",
    "THEME_MODE_POLICY_FORCE_DARK",
    "THEME_MODE_POLICY_FORCE_LIGHT",
    "THEME_MODE_POLICY_FORCE_SYSTEM",
    "THEME_MODE_POLICY_UNSPECIFIED",
    "THEME_MODE_POLICY_USER_OR_SYSTEM",
    "THEME_MODE_SYSTEM",
    "THEME_MODE_UNSPECIFIED",
    "TUPLE",
    "UINT32",
    "UINT64",
    "UNIT",
    "UNIT_FAMILY_COUNT",
    "UNIT_FAMILY_CUSTOM",
    "UNIT_FAMILY_DISTANCE",
    "UNIT_FAMILY_DURATION",
    "UNIT_FAMILY_FREQUENCY",
    "UNIT_FAMILY_POWER",
    "UNIT_FAMILY_RATIO",
    "UNIT_FAMILY_TEMPERATURE",
    "UNIT_FAMILY_UNSPECIFIED",
    "UNIT_FAMILY_VOLTAGE",
    "UNTAGGED",
    "VALIDATION_MODE_PROTOTYPE",
    "VALIDATION_MODE_STRICT",
    "VALIDATION_MODE_UNSPECIFIED",
    "VALUE",
    "VARIANT",
    "AbsoluteTimeFormat",
    "AccessCertificatesCreateResponse",
    "AccessListenerGetResponse",
    "AccessListenerSetResponse",
    "AccessListenerValue",
    "AccessPairing",
    "AccessPairingsCreateResponse",
    "AccessPairingsDeleteResponse",
    "AccessPairingsListResponse",
    "AggregateReply",
    "AggregatesGetResponse",
    "AppearanceAssetMetadata",
    "AppearanceAssetReference",
    "AppearanceFailure",
    "AppearanceFailureKind",
    "AppearanceLogo",
    "AppearanceSnapshot",
    "ApplicationAppearance",
    "ApplicationSettings",
    "ArchiveScope",
    "BackendAppearanceResponse",
    "BackendEvidenceCounts",
    "BackendOverviewGetResponse",
    "BackendOverviewWatchResponse",
    "BackendRuntimeShellSnapshot",
    "BootstrapPaths",
    "CachedRefresh",
    "CaseActions",
    "CaseDetail",
    "CaseParameter",
    "CaseStatus",
    "CaseSummary",
    "CasesCreateResponse",
    "CasesDeleteResponse",
    "CasesGetResponse",
    "CasesListResponse",
    "CasesParametersResponse",
    "CasesWatchResponse",
    "ClientPreferences",
    "ComponentState",
    "ContentArtifactsGetResponse",
    "ContentArtifactsListResponse",
    "ContentArtifactsWatchResponse",
    "ContentExecutionFileKind",
    "ContentExecutionFileMetadata",
    "ContentExecutionFileState",
    "ContentExecutionFilesGetResponse",
    "ContentExecutionFilesListResponse",
    "ContentExecutionFilesWatchResponse",
    "CriterionStatus",
    "CycleFilter",
    "CycleOrder",
    "CycleOrderKey",
    "CyclePhase",
    "CycleSelection",
    "CycleStatus",
    "CycleSummary",
    "CycleWritable",
    "CyclesCreateResponse",
    "CyclesGetResponse",
    "CyclesListResponse",
    "CyclesSealResponse",
    "CyclesSetResponse",
    "CyclesWatchResponse",
    "DeclaredOutcome",
    "Density",
    "Diagnostic",
    "DocumentBinding",
    "DocumentBindingResult",
    "DocumentCard",
    "DocumentCardBody",
    "DocumentCardBudget",
    "DocumentCardFailure",
    "DocumentCardKind",
    "DocumentCompletion",
    "DocumentDensity",
    "DocumentEvidenceCut",
    "DocumentFrame",
    "DocumentFrameFrame",
    "DocumentFrontiers",
    "DocumentHeader",
    "DocumentLayoutNode",
    "DocumentLayoutNodeNode",
    "DocumentOwnerReference",
    "DocumentPage",
    "DocumentPlacement",
    "DocumentPlotDefinition",
    "DocumentRail",
    "DocumentRepeat",
    "DocumentRepeatInstance",
    "DocumentRepeatSelection",
    "DocumentScope",
    "DocumentSection",
    "DocumentSectionIdentity",
    "DocumentSpan",
    "DocumentSummary",
    "DocumentSummaryCondition",
    "DocumentSummaryFormat",
    "DocumentSummaryGroup",
    "DocumentSummaryGroupDefinition",
    "DocumentSummaryPresentation",
    "DocumentSummaryStat",
    "DocumentSummaryState",
    "DocumentSummaryTile",
    "DocumentSummaryTileDefinition",
    "DocumentTable",
    "DocumentTableRow",
    "DocumentTelemetryCut",
    "DocumentWarning",
    "DocumentsGetResponse",
    "DocumentsWatchResponse",
    "ElapsedFormat",
    "ElapsedUnit",
    "EngineeringFormat",
    "EvaluatedOutcome",
    "EvidenceAnnotationOwner",
    "EvidenceAnnotationOwnerOwner",
    "EvidenceAnnotationSelection",
    "EvidenceArchiveResponse",
    "EvidenceBucket",
    "EvidenceBucketRequest",
    "EvidenceBucketValue",
    "EvidenceExecutionNoteOwner",
    "EvidenceHealthEventsListResponse",
    "EvidenceHealthEventsWatchResponse",
    "EvidenceHealthItem",
    "EvidenceNoteWritable",
    "EvidenceNotesCreateResponse",
    "EvidenceNotesGetResponse",
    "EvidenceNotesListResponse",
    "EvidenceNotesSetResponse",
    "EvidenceNotesWatchResponse",
    "EvidenceOperatorNote",
    "EvidencePlannedCaseNoteOwner",
    "EvidenceRecordsWatchResponse",
    "EvidenceSampleRow",
    "EvidenceSamplesListResponse",
    "EvidenceSamplesWatchResponse",
    "EvidenceStatisticValue",
    "EvidenceSubject",
    "EvidenceTelemetrySample",
    "EvidenceTraceabilityWatchResponse",
    "ExecutionBinding",
    "ExecutionControls",
    "ExecutionDetail",
    "ExecutionFilter",
    "ExecutionMode",
    "ExecutionOrder",
    "ExecutionOrderKey",
    "ExecutionOutcome",
    "ExecutionOutcomeCounts",
    "ExecutionParameterSet",
    "ExecutionSelection",
    "ExecutionState",
    "ExecutionStep",
    "ExecutionSummary",
    "ExecutionWritable",
    "ExecutionsCancelResponse",
    "ExecutionsCommandsCreateResponse",
    "ExecutionsCommandsGetResponse",
    "ExecutionsCreateResponse",
    "ExecutionsDeleteResponse",
    "ExecutionsExecutionCommandAction",
    "ExecutionsExecutionCommandPhase",
    "ExecutionsExecutionCommandReceipt",
    "ExecutionsGetResponse",
    "ExecutionsListResponse",
    "ExecutionsParametersResponse",
    "ExecutionsProgressWatchResponse",
    "ExecutionsSchemasGetResponse",
    "ExecutionsSetResponse",
    "ExecutionsStartResponse",
    "ExecutionsStepsResponse",
    "ExecutionsStopResponse",
    "ExecutionsWatchResponse",
    "FixedFormat",
    "FrozenSchemas",
    "HealthComponent",
    "HostCardBlock",
    "HostCardBlockKind",
    "HostCardBody",
    "HostCardKeyValue",
    "HostCardKeyValueList",
    "HostCardMetric",
    "HostCardMetrics",
    "HostCardStatus",
    "HostCardTone",
    "HostIdentity",
    "HostStatus",
    "InvocationsCreateResponse",
    "LiveExecutionState",
    "LogoVariant",
    "ManualRefresh",
    "MaterializedParameter",
    "MeasurementUnit",
    "MediaCapture",
    "MediaCaptureKind",
    "MediaCapturesCreateResponse",
    "MediaCapturesGetResponse",
    "MediaCapturesStopResponse",
    "MediaCapturesWatchResponse",
    "MediaRecordingAudio",
    "MediaRecordingProgress",
    "NormalFormat",
    "OnDemandRefresh",
    "OrganizationIdentity",
    "ParameterSource",
    "PayloadEncoding",
    "PlannedCase",
    "PlotAxisKind",
    "PlotPresentation",
    "PlotReferenceBand",
    "PlotReferenceLine",
    "PlotReferenceValue",
    "PlotsContourGrid",
    "PlotsGridCell",
    "PlotsGridFrame",
    "PlotsGridFrames",
    "PlotsNullableNumber",
    "PlotsPlotAnalysis",
    "PlotsPlotAnalysisError",
    "PlotsPlotAnalysisResult",
    "PlotsPlotBoxSummary",
    "PlotsPlotBoxSummaryAnalysis",
    "PlotsPlotCartesianMode",
    "PlotsPlotColumn",
    "PlotsPlotDomain",
    "PlotsPlotEnvelope",
    "PlotsPlotFrame",
    "PlotsPlotHistogramAnalysis",
    "PlotsPlotHistogramBin",
    "PlotsPlotHistogramSeries",
    "PlotsPlotHistogramStatistics",
    "PlotsPlotMarker",
    "PlotsPlotPayload",
    "PlotsPlotPayloadDiagnostics",
    "PlotsPlotRequest",
    "PlotsPlotSeries",
    "PlotsPlotSeriesSummary",
    "PlotsPlotSettingsValue",
    "PlotsPlotSigmaBin",
    "PlotsPlotSigmaIntervalAnalysis",
    "PlotsPlotSigmaSeries",
    "PlotsPlotStreamKind",
    "PlotsPlotStreamUpdate",
    "PlotsPlotWindow",
    "PlotsSettingsGetResponse",
    "PlotsSettingsSetResponse",
    "PlotsSnapshotsListResponse",
    "PlotsSnapshotsWatchResponse",
    "PluginChange",
    "PluginDescription",
    "PluginEntry",
    "PluginGuest",
    "PluginManaged",
    "PluginPlacement",
    "PluginProcess",
    "PluginRemote",
    "PluginSnapshot",
    "PluginState",
    "PluginValue",
    "PluginsCreateResponse",
    "PluginsDeleteResponse",
    "PluginsGetResponse",
    "PluginsListResponse",
    "PluginsReplaceResponse",
    "PluginsRestartResponse",
    "PluginsSecretRequirement",
    "PluginsSecretsListResponse",
    "PluginsSetResponse",
    "PluginsSettingsSetResponse",
    "PluginsWatchResponse",
    "PromptDefinitionsGetResponse",
    "PromptDefinitionsListResponse",
    "PromptInstance",
    "PromptKind",
    "PromptResponse",
    "PromptView",
    "PromptsAnswersCreateResponse",
    "PromptsGetResponse",
    "PromptsListResponse",
    "PromptsWatchResponse",
    "QueryAggregate",
    "QueryFilter",
    "QueryFilterOperator",
    "QueryRow",
    "QuerySource",
    "RecordDisplay",
    "RecordField",
    "RecordIcon",
    "RecordSchema",
    "RecordSchemasGetResponse",
    "RecordSchemasListResponse",
    "RecordTone",
    "RegisteredSelection",
    "RegisteredSelectionFilesEntry",
    "RegisteredSelectionParametersEntry",
    "RemoteListenerSettings",
    "ReportDefaults",
    "ReportMetadata",
    "ReportOrigin",
    "ReportState",
    "ReportStatus",
    "ReportSubject",
    "ReportsCreateResponse",
    "ReportsGetResponse",
    "ReportsPublishResponse",
    "RerunCaseNotice",
    "ResultStatus",
    "ResultSummary",
    "SchemaCardinality",
    "SchemaDefinition",
    "SchemaDescription",
    "SchemaField",
    "SchemaForm",
    "SchemaGraph",
    "SchemaKey",
    "SchemaKind",
    "SchemaOmission",
    "SchemaRepresentation",
    "SchemasListResponse",
    "ScientificFormat",
    "SecretArchive",
    "SecretImportAction",
    "SecretImportChoice",
    "SecretImportEntry",
    "SecretImportPreview",
    "SecretSnapshot",
    "SecretSummary",
    "SecretsDeleteResponse",
    "SecretsExportResponse",
    "SecretsImportResponse",
    "SecretsListResponse",
    "SecretsSetResponse",
    "SecretsWatchResponse",
    "SelectedFile",
    "SelectedFileKind",
    "SelectionPlan",
    "SelectionValues",
    "SettingsCard",
    "SettingsCardRefreshPolicy",
    "SettingsCardRefreshPolicyPolicy",
    "SettingsCardState",
    "SettingsFrame",
    "SettingsGetResponse",
    "SettingsSchemaResponse",
    "SettingsSection",
    "SettingsSectionOwner",
    "SettingsSetResponse",
    "SiPrefixFormat",
    "StatusDiagnosticsGetResponse",
    "StatusEvent",
    "StatusGetResponse",
    "StatusLogsListResponse",
    "StatusLogsWatchResponse",
    "StatusTraceEventsListResponse",
    "StatusTraceEventsWatchResponse",
    "StatusWatchResponse",
    "StepCriterion",
    "StepStatus",
    "StorageSourcesGetResponse",
    "StorageSourcesSetResponse",
    "StorageStorageSource",
    "StorageStorageSources",
    "StreamIdentity",
    "StreamSchemasGetResponse",
    "StreamSchemasListResponse",
    "StreamsListResponse",
    "SuitesGetResponse",
    "SuitesListResponse",
    "TelemetryColumn",
    "TelemetryPreferences",
    "TelemetryRole",
    "TelemetrySchema",
    "ThemeCornerRadii",
    "ThemeFamily",
    "ThemeMode",
    "ThemeModePolicy",
    "ThemeVariant",
    "UnevaluatedOutcome",
    "UnitFamily",
    "ValidationMode",
    "ValueFormat",
    "ValueFormatKind",
]
