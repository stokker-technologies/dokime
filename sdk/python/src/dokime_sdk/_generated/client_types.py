# @generated from protocol descriptors; do not edit.
from harmos._wire.host.v1.access.certificates_pb2 import CertificatesCreateResponse
from harmos._wire.host.v1.access.listener_pb2 import ListenerGetResponse, ListenerSetResponse, ListenerValue
from harmos._wire.host.v1.access.pairings_pb2 import Pairing, PairingsCreateResponse, PairingsDeleteResponse, PairingsListResponse
from harmos._wire.host.v1.invocations_pb2 import InvocationsCreateResponse
from harmos._wire.host.v1.schema_pb2 import (
    ADJACENT,
    BOOL,
    BYTES,
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
    PayloadEncoding,
    SchemaCardinality,
    SchemaDefinition,
    SchemaField,
    SchemaForm,
    SchemaGraph,
    SchemaKey,
    SchemaKind,
    SchemaOmission,
    SchemaRepresentation,
)
from harmos._wire.host.v1.schemas_pb2 import SchemaDescription, SchemasListResponse
from harmos._wire.host.v1.status.diagnostics_pb2 import DiagnosticsGetResponse
from harmos._wire.host.v1.status.logs_pb2 import Event, LogsListResponse, LogsWatchResponse
from harmos._wire.host.v1.status.trace_events_pb2 import TraceEventsListResponse, TraceEventsWatchResponse
from harmos._wire.host.v1.status_pb2 import (
    COMPONENT_STATE_DEGRADED,
    COMPONENT_STATE_HEALTHY,
    COMPONENT_STATE_UNAVAILABLE,
    COMPONENT_STATE_UNSPECIFIED,
    ComponentState,
    HealthComponent,
    HostIdentity,
    HostStatus,
)
from harmos._wire.host.v1.streams_pb2 import StreamIdentity, StreamsListResponse

from dokime_sdk._wire.host.v1.aggregates_pb2 import (
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
from dokime_sdk._wire.host.v1.backend.overview_pb2 import EvidenceCounts, OverviewGetResponse, OverviewWatchResponse, RuntimeShellSnapshot
from dokime_sdk._wire.host.v1.backend_pb2 import (
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
from dokime_sdk._wire.host.v1.cases_pb2 import (
    CASE_STATUS_BLOCKED,
    CASE_STATUS_CANCELLED,
    CASE_STATUS_FAILED,
    CASE_STATUS_INTERRUPTED,
    CASE_STATUS_INVALIDATED,
    CASE_STATUS_NOT_RUN,
    CASE_STATUS_PASSED,
    CASE_STATUS_PLANNED,
    CASE_STATUS_QUEUED,
    CASE_STATUS_RUNNING,
    CASE_STATUS_SKIPPED,
    CASE_STATUS_STOPPED,
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
from dokime_sdk._wire.host.v1.content.artifacts_pb2 import ArtifactsGetResponse, ArtifactsListResponse, ArtifactsWatchResponse
from dokime_sdk._wire.host.v1.content.execution_files_pb2 import (
    EXECUTION_FILE_KIND_BINARY,
    EXECUTION_FILE_KIND_MEDIA,
    EXECUTION_FILE_KIND_TEXT,
    EXECUTION_FILE_KIND_UNKNOWN,
    EXECUTION_FILE_KIND_UNSPECIFIED,
    EXECUTION_FILE_STATE_FINALIZED,
    EXECUTION_FILE_STATE_INTERRUPTED,
    EXECUTION_FILE_STATE_STREAMING,
    EXECUTION_FILE_STATE_UNSPECIFIED,
    ExecutionFileKind,
    ExecutionFileMetadata,
    ExecutionFilesGetResponse,
    ExecutionFilesListResponse,
    ExecutionFileState,
    ExecutionFilesWatchResponse,
)
from dokime_sdk._wire.host.v1.cycles_pb2 import (
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
from dokime_sdk._wire.host.v1.documents_pb2 import (
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
from dokime_sdk._wire.host.v1.evidence.health_events_pb2 import HealthEventsListResponse, HealthEventsWatchResponse, HealthItem
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import (
    AnnotationOwner,
    AnnotationOwnerOwner,
    AnnotationSelection,
    ExecutionNoteOwner,
    NotesCreateResponse,
    NotesGetResponse,
    NotesListResponse,
    NotesSetResponse,
    NotesWatchResponse,
    NoteWritable,
    OperatorNote,
    PlannedCaseNoteOwner,
)
from dokime_sdk._wire.host.v1.evidence.records_pb2 import RecordsWatchResponse
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import (
    Bucket,
    BucketRequest,
    BucketValue,
    SampleRow,
    SamplesListResponse,
    SamplesWatchResponse,
    StatisticValue,
    TelemetrySample,
)
from dokime_sdk._wire.host.v1.evidence.traceability_pb2 import TraceabilityWatchResponse
from dokime_sdk._wire.host.v1.evidence_pb2 import EvidenceArchiveResponse, EvidenceSubject
from dokime_sdk._wire.host.v1.executions.commands_pb2 import (
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
    CommandsCreateResponse,
    CommandsGetResponse,
    ExecutionCommandAction,
    ExecutionCommandPhase,
    ExecutionCommandReceipt,
)
from dokime_sdk._wire.host.v1.executions.progress_pb2 import ProgressWatchResponse
from dokime_sdk._wire.host.v1.executions.schemas_pb2 import SchemasGetResponse
from dokime_sdk._wire.host.v1.executions_pb2 import (
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
    ExecutionsSealResponse,
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
from dokime_sdk._wire.host.v1.media.captures_pb2 import (
    CAPTURE_KIND_AUDIO,
    CAPTURE_KIND_CAMERA,
    CAPTURE_KIND_PHOTO,
    CAPTURE_KIND_SCREEN,
    CAPTURE_KIND_SCREENSHOT,
    CAPTURE_KIND_UNSPECIFIED,
    Capture,
    CaptureKind,
    CapturesCreateResponse,
    CapturesGetResponse,
    CapturesWatchResponse,
    RecordingAudio,
    RecordingProgress,
)
from dokime_sdk._wire.host.v1.plots.settings_pb2 import PlotSettingsValue
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import (
    PLOT_CARTESIAN_MODE_ALIGNED,
    PLOT_CARTESIAN_MODE_FACETED,
    PLOT_CARTESIAN_MODE_UNSPECIFIED,
    PLOT_STREAM_KIND_DELTA,
    PLOT_STREAM_KIND_ERROR,
    PLOT_STREAM_KIND_REPLACE,
    PLOT_STREAM_KIND_SNAPSHOT,
    PLOT_STREAM_KIND_STATISTICS,
    PLOT_STREAM_KIND_UNSPECIFIED,
    ContourGrid,
    GridCell,
    GridFrame,
    GridFrames,
    NullableNumber,
    PlotAnalysis,
    PlotAnalysisError,
    PlotAnalysisResult,
    PlotBoxSummary,
    PlotBoxSummaryAnalysis,
    PlotCartesianMode,
    PlotColumn,
    PlotDomain,
    PlotEnvelope,
    PlotFrame,
    PlotHistogramAnalysis,
    PlotHistogramBin,
    PlotHistogramSeries,
    PlotHistogramStatistics,
    PlotMarker,
    PlotPayload,
    PlotPayloadDiagnostics,
    PlotRequest,
    PlotSeries,
    PlotSeriesSummary,
    PlotSigmaBin,
    PlotSigmaIntervalAnalysis,
    PlotSigmaSeries,
    PlotStreamKind,
    PlotStreamUpdate,
    PlotWindow,
    SnapshotsListResponse,
    SnapshotsWatchResponse,
)
from dokime_sdk._wire.host.v1.plugins.secrets_pb2 import SecretRequirement
from dokime_sdk._wire.host.v1.prompt_definitions_pb2 import PromptDefinitionsGetResponse, PromptDefinitionsListResponse
from dokime_sdk._wire.host.v1.prompts.answers_pb2 import AnswersCreateResponse
from dokime_sdk._wire.host.v1.prompts_pb2 import (
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
from dokime_sdk._wire.host.v1.record_schemas_pb2 import (
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
from dokime_sdk._wire.host.v1.reports_pb2 import (
    REPORT_ORIGIN_CURRENT,
    REPORT_ORIGIN_SEALED,
    REPORT_ORIGIN_UNSPECIFIED,
    ReportCapture,
    ReportCase,
    ReportExecution,
    ReportFile,
    ReportMetadata,
    ReportOrigin,
    ReportsCreateResponse,
    ReportsGetResponse,
    ReportsPublishResponse,
    ReportSubject,
)
from dokime_sdk._wire.host.v1.secrets_pb2 import (
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
    SecretSnapshot,
    SecretsSetResponse,
    SecretSummary,
    SecretsWatchResponse,
)
from dokime_sdk._wire.host.v1.settings_pb2 import (
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
    SettingsSchemaResponse,
    SettingsSection,
    SettingsSectionOwner,
    TelemetryPreferences,
    ThemeMode,
)
from dokime_sdk._wire.host.v1.status_pb2 import HostBuild
from dokime_sdk._wire.host.v1.storage.sources_pb2 import SourcesGetResponse, SourcesSetResponse, StorageSource, StorageSources
from dokime_sdk._wire.host.v1.stream_schemas_pb2 import StreamSchemasGetResponse, StreamSchemasListResponse
from dokime_sdk._wire.host.v1.suites_pb2 import SuitesGetResponse, SuitesListResponse

from ._exports.client_types import harmos, host

AggregateReply.__module__ = "dokime_sdk.client"
AggregateReply.__qualname__ = "AggregateReply"
AggregatesGetResponse.__module__ = "dokime_sdk.client"
AggregatesGetResponse.__qualname__ = "AggregatesGetResponse"
QueryAggregate.__module__ = "dokime_sdk.client"
QueryAggregate.__qualname__ = "QueryAggregate"
QueryFilter.__module__ = "dokime_sdk.client"
QueryFilter.__qualname__ = "QueryFilter"
QueryRow.__module__ = "dokime_sdk.client"
QueryRow.__qualname__ = "QueryRow"
EvidenceCounts.__module__ = "dokime_sdk.client"
EvidenceCounts.__qualname__ = "EvidenceCounts"
OverviewGetResponse.__module__ = "dokime_sdk.client"
OverviewGetResponse.__qualname__ = "OverviewGetResponse"
OverviewWatchResponse.__module__ = "dokime_sdk.client"
OverviewWatchResponse.__qualname__ = "OverviewWatchResponse"
RuntimeShellSnapshot.__module__ = "dokime_sdk.client"
RuntimeShellSnapshot.__qualname__ = "RuntimeShellSnapshot"
AppearanceAssetMetadata.__module__ = "dokime_sdk.client"
AppearanceAssetMetadata.__qualname__ = "AppearanceAssetMetadata"
AppearanceAssetReference.__module__ = "dokime_sdk.client"
AppearanceAssetReference.__qualname__ = "AppearanceAssetReference"
AppearanceFailure.__module__ = "dokime_sdk.client"
AppearanceFailure.__qualname__ = "AppearanceFailure"
AppearanceLogo.__module__ = "dokime_sdk.client"
AppearanceLogo.__qualname__ = "AppearanceLogo"
AppearanceSnapshot.__module__ = "dokime_sdk.client"
AppearanceSnapshot.__qualname__ = "AppearanceSnapshot"
BackendAppearanceResponse.__module__ = "dokime_sdk.client"
BackendAppearanceResponse.__qualname__ = "BackendAppearanceResponse"
OrganizationIdentity.__module__ = "dokime_sdk.client"
OrganizationIdentity.__qualname__ = "OrganizationIdentity"
ThemeCornerRadii.__module__ = "dokime_sdk.client"
ThemeCornerRadii.__qualname__ = "ThemeCornerRadii"
ThemeFamily.__module__ = "dokime_sdk.client"
ThemeFamily.__qualname__ = "ThemeFamily"
ThemeVariant.__module__ = "dokime_sdk.client"
ThemeVariant.__qualname__ = "ThemeVariant"
CaseActions.__module__ = "dokime_sdk.client"
CaseActions.__qualname__ = "CaseActions"
CaseDetail.__module__ = "dokime_sdk.client"
CaseDetail.__qualname__ = "CaseDetail"
CaseParameter.__module__ = "dokime_sdk.client"
CaseParameter.__qualname__ = "CaseParameter"
CaseSummary.__module__ = "dokime_sdk.client"
CaseSummary.__qualname__ = "CaseSummary"
CasesCreateResponse.__module__ = "dokime_sdk.client"
CasesCreateResponse.__qualname__ = "CasesCreateResponse"
CasesDeleteResponse.__module__ = "dokime_sdk.client"
CasesDeleteResponse.__qualname__ = "CasesDeleteResponse"
CasesGetResponse.__module__ = "dokime_sdk.client"
CasesGetResponse.__qualname__ = "CasesGetResponse"
CasesListResponse.__module__ = "dokime_sdk.client"
CasesListResponse.__qualname__ = "CasesListResponse"
CasesParametersResponse.__module__ = "dokime_sdk.client"
CasesParametersResponse.__qualname__ = "CasesParametersResponse"
CasesWatchResponse.__module__ = "dokime_sdk.client"
CasesWatchResponse.__qualname__ = "CasesWatchResponse"
PlannedCase.__module__ = "dokime_sdk.client"
PlannedCase.__qualname__ = "PlannedCase"
RegisteredSelection.__module__ = "dokime_sdk.client"
RegisteredSelection.__qualname__ = "RegisteredSelection"
RegisteredSelectionFilesEntry.__module__ = "dokime_sdk.client"
RegisteredSelectionFilesEntry.__qualname__ = "RegisteredSelectionFilesEntry"
RegisteredSelectionParametersEntry.__module__ = "dokime_sdk.client"
RegisteredSelectionParametersEntry.__qualname__ = "RegisteredSelectionParametersEntry"
SelectedFile.__module__ = "dokime_sdk.client"
SelectedFile.__qualname__ = "SelectedFile"
SelectedFileKind.__module__ = "dokime_sdk.client"
SelectedFileKind.__qualname__ = "SelectedFileKind"
SelectionPlan.__module__ = "dokime_sdk.client"
SelectionPlan.__qualname__ = "SelectionPlan"
SelectionValues.__module__ = "dokime_sdk.client"
SelectionValues.__qualname__ = "SelectionValues"
ArtifactsGetResponse.__module__ = "dokime_sdk.client"
ArtifactsGetResponse.__qualname__ = "ArtifactsGetResponse"
ArtifactsListResponse.__module__ = "dokime_sdk.client"
ArtifactsListResponse.__qualname__ = "ArtifactsListResponse"
ArtifactsWatchResponse.__module__ = "dokime_sdk.client"
ArtifactsWatchResponse.__qualname__ = "ArtifactsWatchResponse"
ExecutionFileMetadata.__module__ = "dokime_sdk.client"
ExecutionFileMetadata.__qualname__ = "ExecutionFileMetadata"
ExecutionFilesGetResponse.__module__ = "dokime_sdk.client"
ExecutionFilesGetResponse.__qualname__ = "ExecutionFilesGetResponse"
ExecutionFilesListResponse.__module__ = "dokime_sdk.client"
ExecutionFilesListResponse.__qualname__ = "ExecutionFilesListResponse"
ExecutionFilesWatchResponse.__module__ = "dokime_sdk.client"
ExecutionFilesWatchResponse.__qualname__ = "ExecutionFilesWatchResponse"
CycleFilter.__module__ = "dokime_sdk.client"
CycleFilter.__qualname__ = "CycleFilter"
CycleOrder.__module__ = "dokime_sdk.client"
CycleOrder.__qualname__ = "CycleOrder"
CycleSelection.__module__ = "dokime_sdk.client"
CycleSelection.__qualname__ = "CycleSelection"
CycleSummary.__module__ = "dokime_sdk.client"
CycleSummary.__qualname__ = "CycleSummary"
CycleWritable.__module__ = "dokime_sdk.client"
CycleWritable.__qualname__ = "CycleWritable"
CyclesCreateResponse.__module__ = "dokime_sdk.client"
CyclesCreateResponse.__qualname__ = "CyclesCreateResponse"
CyclesGetResponse.__module__ = "dokime_sdk.client"
CyclesGetResponse.__qualname__ = "CyclesGetResponse"
CyclesListResponse.__module__ = "dokime_sdk.client"
CyclesListResponse.__qualname__ = "CyclesListResponse"
CyclesSealResponse.__module__ = "dokime_sdk.client"
CyclesSealResponse.__qualname__ = "CyclesSealResponse"
CyclesSetResponse.__module__ = "dokime_sdk.client"
CyclesSetResponse.__qualname__ = "CyclesSetResponse"
CyclesWatchResponse.__module__ = "dokime_sdk.client"
CyclesWatchResponse.__qualname__ = "CyclesWatchResponse"
RerunCaseNotice.__module__ = "dokime_sdk.client"
RerunCaseNotice.__qualname__ = "RerunCaseNotice"
ResultSummary.__module__ = "dokime_sdk.client"
ResultSummary.__qualname__ = "ResultSummary"
DocumentBinding.__module__ = "dokime_sdk.client"
DocumentBinding.__qualname__ = "DocumentBinding"
DocumentBindingResult.__module__ = "dokime_sdk.client"
DocumentBindingResult.__qualname__ = "DocumentBindingResult"
DocumentCard.__module__ = "dokime_sdk.client"
DocumentCard.__qualname__ = "DocumentCard"
DocumentCardBody.__module__ = "dokime_sdk.client"
DocumentCardBody.__qualname__ = "DocumentCardBody"
DocumentCardBudget.__module__ = "dokime_sdk.client"
DocumentCardBudget.__qualname__ = "DocumentCardBudget"
DocumentCardFailure.__module__ = "dokime_sdk.client"
DocumentCardFailure.__qualname__ = "DocumentCardFailure"
DocumentCompletion.__module__ = "dokime_sdk.client"
DocumentCompletion.__qualname__ = "DocumentCompletion"
DocumentEvidenceCut.__module__ = "dokime_sdk.client"
DocumentEvidenceCut.__qualname__ = "DocumentEvidenceCut"
DocumentFrame.__module__ = "dokime_sdk.client"
DocumentFrame.__qualname__ = "DocumentFrame"
DocumentFrameFrame.__module__ = "dokime_sdk.client"
DocumentFrameFrame.__qualname__ = "DocumentFrameFrame"
DocumentFrontiers.__module__ = "dokime_sdk.client"
DocumentFrontiers.__qualname__ = "DocumentFrontiers"
DocumentHeader.__module__ = "dokime_sdk.client"
DocumentHeader.__qualname__ = "DocumentHeader"
DocumentLayoutNode.__module__ = "dokime_sdk.client"
DocumentLayoutNode.__qualname__ = "DocumentLayoutNode"
DocumentLayoutNodeNode.__module__ = "dokime_sdk.client"
DocumentLayoutNodeNode.__qualname__ = "DocumentLayoutNodeNode"
DocumentOwnerReference.__module__ = "dokime_sdk.client"
DocumentOwnerReference.__qualname__ = "DocumentOwnerReference"
DocumentPage.__module__ = "dokime_sdk.client"
DocumentPage.__qualname__ = "DocumentPage"
DocumentPlacement.__module__ = "dokime_sdk.client"
DocumentPlacement.__qualname__ = "DocumentPlacement"
DocumentPlotDefinition.__module__ = "dokime_sdk.client"
DocumentPlotDefinition.__qualname__ = "DocumentPlotDefinition"
DocumentRepeat.__module__ = "dokime_sdk.client"
DocumentRepeat.__qualname__ = "DocumentRepeat"
DocumentRepeatInstance.__module__ = "dokime_sdk.client"
DocumentRepeatInstance.__qualname__ = "DocumentRepeatInstance"
DocumentRepeatSelection.__module__ = "dokime_sdk.client"
DocumentRepeatSelection.__qualname__ = "DocumentRepeatSelection"
DocumentScope.__module__ = "dokime_sdk.client"
DocumentScope.__qualname__ = "DocumentScope"
DocumentSection.__module__ = "dokime_sdk.client"
DocumentSection.__qualname__ = "DocumentSection"
DocumentSectionIdentity.__module__ = "dokime_sdk.client"
DocumentSectionIdentity.__qualname__ = "DocumentSectionIdentity"
DocumentSpan.__module__ = "dokime_sdk.client"
DocumentSpan.__qualname__ = "DocumentSpan"
DocumentSummary.__module__ = "dokime_sdk.client"
DocumentSummary.__qualname__ = "DocumentSummary"
DocumentSummaryCondition.__module__ = "dokime_sdk.client"
DocumentSummaryCondition.__qualname__ = "DocumentSummaryCondition"
DocumentSummaryGroup.__module__ = "dokime_sdk.client"
DocumentSummaryGroup.__qualname__ = "DocumentSummaryGroup"
DocumentSummaryGroupDefinition.__module__ = "dokime_sdk.client"
DocumentSummaryGroupDefinition.__qualname__ = "DocumentSummaryGroupDefinition"
DocumentSummaryTile.__module__ = "dokime_sdk.client"
DocumentSummaryTile.__qualname__ = "DocumentSummaryTile"
DocumentSummaryTileDefinition.__module__ = "dokime_sdk.client"
DocumentSummaryTileDefinition.__qualname__ = "DocumentSummaryTileDefinition"
DocumentTable.__module__ = "dokime_sdk.client"
DocumentTable.__qualname__ = "DocumentTable"
DocumentTableRow.__module__ = "dokime_sdk.client"
DocumentTableRow.__qualname__ = "DocumentTableRow"
DocumentTelemetryCut.__module__ = "dokime_sdk.client"
DocumentTelemetryCut.__qualname__ = "DocumentTelemetryCut"
DocumentWarning.__module__ = "dokime_sdk.client"
DocumentWarning.__qualname__ = "DocumentWarning"
DocumentsGetResponse.__module__ = "dokime_sdk.client"
DocumentsGetResponse.__qualname__ = "DocumentsGetResponse"
DocumentsWatchResponse.__module__ = "dokime_sdk.client"
DocumentsWatchResponse.__qualname__ = "DocumentsWatchResponse"
PlotPresentation.__module__ = "dokime_sdk.client"
PlotPresentation.__qualname__ = "PlotPresentation"
PlotReferenceBand.__module__ = "dokime_sdk.client"
PlotReferenceBand.__qualname__ = "PlotReferenceBand"
PlotReferenceLine.__module__ = "dokime_sdk.client"
PlotReferenceLine.__qualname__ = "PlotReferenceLine"
PlotReferenceValue.__module__ = "dokime_sdk.client"
PlotReferenceValue.__qualname__ = "PlotReferenceValue"
HealthEventsListResponse.__module__ = "dokime_sdk.client"
HealthEventsListResponse.__qualname__ = "HealthEventsListResponse"
HealthEventsWatchResponse.__module__ = "dokime_sdk.client"
HealthEventsWatchResponse.__qualname__ = "HealthEventsWatchResponse"
HealthItem.__module__ = "dokime_sdk.client"
HealthItem.__qualname__ = "HealthItem"
AnnotationOwner.__module__ = "dokime_sdk.client"
AnnotationOwner.__qualname__ = "AnnotationOwner"
AnnotationOwnerOwner.__module__ = "dokime_sdk.client"
AnnotationOwnerOwner.__qualname__ = "AnnotationOwnerOwner"
AnnotationSelection.__module__ = "dokime_sdk.client"
AnnotationSelection.__qualname__ = "AnnotationSelection"
ExecutionNoteOwner.__module__ = "dokime_sdk.client"
ExecutionNoteOwner.__qualname__ = "ExecutionNoteOwner"
NoteWritable.__module__ = "dokime_sdk.client"
NoteWritable.__qualname__ = "NoteWritable"
NotesCreateResponse.__module__ = "dokime_sdk.client"
NotesCreateResponse.__qualname__ = "NotesCreateResponse"
NotesGetResponse.__module__ = "dokime_sdk.client"
NotesGetResponse.__qualname__ = "NotesGetResponse"
NotesListResponse.__module__ = "dokime_sdk.client"
NotesListResponse.__qualname__ = "NotesListResponse"
NotesSetResponse.__module__ = "dokime_sdk.client"
NotesSetResponse.__qualname__ = "NotesSetResponse"
NotesWatchResponse.__module__ = "dokime_sdk.client"
NotesWatchResponse.__qualname__ = "NotesWatchResponse"
OperatorNote.__module__ = "dokime_sdk.client"
OperatorNote.__qualname__ = "OperatorNote"
PlannedCaseNoteOwner.__module__ = "dokime_sdk.client"
PlannedCaseNoteOwner.__qualname__ = "PlannedCaseNoteOwner"
RecordsWatchResponse.__module__ = "dokime_sdk.client"
RecordsWatchResponse.__qualname__ = "RecordsWatchResponse"
Bucket.__module__ = "dokime_sdk.client"
Bucket.__qualname__ = "Bucket"
BucketRequest.__module__ = "dokime_sdk.client"
BucketRequest.__qualname__ = "BucketRequest"
BucketValue.__module__ = "dokime_sdk.client"
BucketValue.__qualname__ = "BucketValue"
SampleRow.__module__ = "dokime_sdk.client"
SampleRow.__qualname__ = "SampleRow"
SamplesListResponse.__module__ = "dokime_sdk.client"
SamplesListResponse.__qualname__ = "SamplesListResponse"
SamplesWatchResponse.__module__ = "dokime_sdk.client"
SamplesWatchResponse.__qualname__ = "SamplesWatchResponse"
StatisticValue.__module__ = "dokime_sdk.client"
StatisticValue.__qualname__ = "StatisticValue"
TelemetrySample.__module__ = "dokime_sdk.client"
TelemetrySample.__qualname__ = "TelemetrySample"
TraceabilityWatchResponse.__module__ = "dokime_sdk.client"
TraceabilityWatchResponse.__qualname__ = "TraceabilityWatchResponse"
EvidenceArchiveResponse.__module__ = "dokime_sdk.client"
EvidenceArchiveResponse.__qualname__ = "EvidenceArchiveResponse"
EvidenceSubject.__module__ = "dokime_sdk.client"
EvidenceSubject.__qualname__ = "EvidenceSubject"
CommandsCreateResponse.__module__ = "dokime_sdk.client"
CommandsCreateResponse.__qualname__ = "CommandsCreateResponse"
CommandsGetResponse.__module__ = "dokime_sdk.client"
CommandsGetResponse.__qualname__ = "CommandsGetResponse"
ExecutionCommandReceipt.__module__ = "dokime_sdk.client"
ExecutionCommandReceipt.__qualname__ = "ExecutionCommandReceipt"
ProgressWatchResponse.__module__ = "dokime_sdk.client"
ProgressWatchResponse.__qualname__ = "ProgressWatchResponse"
SchemasGetResponse.__module__ = "dokime_sdk.client"
SchemasGetResponse.__qualname__ = "SchemasGetResponse"
DeclaredOutcome.__module__ = "dokime_sdk.client"
DeclaredOutcome.__qualname__ = "DeclaredOutcome"
Diagnostic.__module__ = "dokime_sdk.client"
Diagnostic.__qualname__ = "Diagnostic"
EvaluatedOutcome.__module__ = "dokime_sdk.client"
EvaluatedOutcome.__qualname__ = "EvaluatedOutcome"
ExecutionBinding.__module__ = "dokime_sdk.client"
ExecutionBinding.__qualname__ = "ExecutionBinding"
ExecutionControls.__module__ = "dokime_sdk.client"
ExecutionControls.__qualname__ = "ExecutionControls"
ExecutionDetail.__module__ = "dokime_sdk.client"
ExecutionDetail.__qualname__ = "ExecutionDetail"
ExecutionFilter.__module__ = "dokime_sdk.client"
ExecutionFilter.__qualname__ = "ExecutionFilter"
ExecutionOrder.__module__ = "dokime_sdk.client"
ExecutionOrder.__qualname__ = "ExecutionOrder"
ExecutionOutcome.__module__ = "dokime_sdk.client"
ExecutionOutcome.__qualname__ = "ExecutionOutcome"
ExecutionOutcomeCounts.__module__ = "dokime_sdk.client"
ExecutionOutcomeCounts.__qualname__ = "ExecutionOutcomeCounts"
ExecutionSelection.__module__ = "dokime_sdk.client"
ExecutionSelection.__qualname__ = "ExecutionSelection"
ExecutionStep.__module__ = "dokime_sdk.client"
ExecutionStep.__qualname__ = "ExecutionStep"
ExecutionSummary.__module__ = "dokime_sdk.client"
ExecutionSummary.__qualname__ = "ExecutionSummary"
ExecutionWritable.__module__ = "dokime_sdk.client"
ExecutionWritable.__qualname__ = "ExecutionWritable"
ExecutionsCancelResponse.__module__ = "dokime_sdk.client"
ExecutionsCancelResponse.__qualname__ = "ExecutionsCancelResponse"
ExecutionsCreateResponse.__module__ = "dokime_sdk.client"
ExecutionsCreateResponse.__qualname__ = "ExecutionsCreateResponse"
ExecutionsDeleteResponse.__module__ = "dokime_sdk.client"
ExecutionsDeleteResponse.__qualname__ = "ExecutionsDeleteResponse"
ExecutionsGetResponse.__module__ = "dokime_sdk.client"
ExecutionsGetResponse.__qualname__ = "ExecutionsGetResponse"
ExecutionsListResponse.__module__ = "dokime_sdk.client"
ExecutionsListResponse.__qualname__ = "ExecutionsListResponse"
ExecutionsParametersResponse.__module__ = "dokime_sdk.client"
ExecutionsParametersResponse.__qualname__ = "ExecutionsParametersResponse"
ExecutionsSealResponse.__module__ = "dokime_sdk.client"
ExecutionsSealResponse.__qualname__ = "ExecutionsSealResponse"
ExecutionsSetResponse.__module__ = "dokime_sdk.client"
ExecutionsSetResponse.__qualname__ = "ExecutionsSetResponse"
ExecutionsStartResponse.__module__ = "dokime_sdk.client"
ExecutionsStartResponse.__qualname__ = "ExecutionsStartResponse"
ExecutionsStepsResponse.__module__ = "dokime_sdk.client"
ExecutionsStepsResponse.__qualname__ = "ExecutionsStepsResponse"
ExecutionsStopResponse.__module__ = "dokime_sdk.client"
ExecutionsStopResponse.__qualname__ = "ExecutionsStopResponse"
ExecutionsWatchResponse.__module__ = "dokime_sdk.client"
ExecutionsWatchResponse.__qualname__ = "ExecutionsWatchResponse"
FrozenSchemas.__module__ = "dokime_sdk.client"
FrozenSchemas.__qualname__ = "FrozenSchemas"
MaterializedParameter.__module__ = "dokime_sdk.client"
MaterializedParameter.__qualname__ = "MaterializedParameter"
ReportStatus.__module__ = "dokime_sdk.client"
ReportStatus.__qualname__ = "ReportStatus"
StepCriterion.__module__ = "dokime_sdk.client"
StepCriterion.__qualname__ = "StepCriterion"
UnevaluatedOutcome.__module__ = "dokime_sdk.client"
UnevaluatedOutcome.__qualname__ = "UnevaluatedOutcome"
Capture.__module__ = "dokime_sdk.client"
Capture.__qualname__ = "Capture"
CapturesCreateResponse.__module__ = "dokime_sdk.client"
CapturesCreateResponse.__qualname__ = "CapturesCreateResponse"
CapturesGetResponse.__module__ = "dokime_sdk.client"
CapturesGetResponse.__qualname__ = "CapturesGetResponse"
CapturesWatchResponse.__module__ = "dokime_sdk.client"
CapturesWatchResponse.__qualname__ = "CapturesWatchResponse"
RecordingAudio.__module__ = "dokime_sdk.client"
RecordingAudio.__qualname__ = "RecordingAudio"
RecordingProgress.__module__ = "dokime_sdk.client"
RecordingProgress.__qualname__ = "RecordingProgress"
PlotSettingsValue.__module__ = "dokime_sdk.client"
PlotSettingsValue.__qualname__ = "PlotSettingsValue"
ContourGrid.__module__ = "dokime_sdk.client"
ContourGrid.__qualname__ = "ContourGrid"
GridCell.__module__ = "dokime_sdk.client"
GridCell.__qualname__ = "GridCell"
GridFrame.__module__ = "dokime_sdk.client"
GridFrame.__qualname__ = "GridFrame"
GridFrames.__module__ = "dokime_sdk.client"
GridFrames.__qualname__ = "GridFrames"
NullableNumber.__module__ = "dokime_sdk.client"
NullableNumber.__qualname__ = "NullableNumber"
PlotAnalysis.__module__ = "dokime_sdk.client"
PlotAnalysis.__qualname__ = "PlotAnalysis"
PlotAnalysisError.__module__ = "dokime_sdk.client"
PlotAnalysisError.__qualname__ = "PlotAnalysisError"
PlotAnalysisResult.__module__ = "dokime_sdk.client"
PlotAnalysisResult.__qualname__ = "PlotAnalysisResult"
PlotBoxSummary.__module__ = "dokime_sdk.client"
PlotBoxSummary.__qualname__ = "PlotBoxSummary"
PlotBoxSummaryAnalysis.__module__ = "dokime_sdk.client"
PlotBoxSummaryAnalysis.__qualname__ = "PlotBoxSummaryAnalysis"
PlotColumn.__module__ = "dokime_sdk.client"
PlotColumn.__qualname__ = "PlotColumn"
PlotDomain.__module__ = "dokime_sdk.client"
PlotDomain.__qualname__ = "PlotDomain"
PlotEnvelope.__module__ = "dokime_sdk.client"
PlotEnvelope.__qualname__ = "PlotEnvelope"
PlotFrame.__module__ = "dokime_sdk.client"
PlotFrame.__qualname__ = "PlotFrame"
PlotHistogramAnalysis.__module__ = "dokime_sdk.client"
PlotHistogramAnalysis.__qualname__ = "PlotHistogramAnalysis"
PlotHistogramBin.__module__ = "dokime_sdk.client"
PlotHistogramBin.__qualname__ = "PlotHistogramBin"
PlotHistogramSeries.__module__ = "dokime_sdk.client"
PlotHistogramSeries.__qualname__ = "PlotHistogramSeries"
PlotHistogramStatistics.__module__ = "dokime_sdk.client"
PlotHistogramStatistics.__qualname__ = "PlotHistogramStatistics"
PlotMarker.__module__ = "dokime_sdk.client"
PlotMarker.__qualname__ = "PlotMarker"
PlotPayload.__module__ = "dokime_sdk.client"
PlotPayload.__qualname__ = "PlotPayload"
PlotPayloadDiagnostics.__module__ = "dokime_sdk.client"
PlotPayloadDiagnostics.__qualname__ = "PlotPayloadDiagnostics"
PlotRequest.__module__ = "dokime_sdk.client"
PlotRequest.__qualname__ = "PlotRequest"
PlotSeries.__module__ = "dokime_sdk.client"
PlotSeries.__qualname__ = "PlotSeries"
PlotSeriesSummary.__module__ = "dokime_sdk.client"
PlotSeriesSummary.__qualname__ = "PlotSeriesSummary"
PlotSigmaBin.__module__ = "dokime_sdk.client"
PlotSigmaBin.__qualname__ = "PlotSigmaBin"
PlotSigmaIntervalAnalysis.__module__ = "dokime_sdk.client"
PlotSigmaIntervalAnalysis.__qualname__ = "PlotSigmaIntervalAnalysis"
PlotSigmaSeries.__module__ = "dokime_sdk.client"
PlotSigmaSeries.__qualname__ = "PlotSigmaSeries"
PlotStreamUpdate.__module__ = "dokime_sdk.client"
PlotStreamUpdate.__qualname__ = "PlotStreamUpdate"
PlotWindow.__module__ = "dokime_sdk.client"
PlotWindow.__qualname__ = "PlotWindow"
SnapshotsListResponse.__module__ = "dokime_sdk.client"
SnapshotsListResponse.__qualname__ = "SnapshotsListResponse"
SnapshotsWatchResponse.__module__ = "dokime_sdk.client"
SnapshotsWatchResponse.__qualname__ = "SnapshotsWatchResponse"
SecretRequirement.__module__ = "dokime_sdk.client"
SecretRequirement.__qualname__ = "SecretRequirement"
PromptDefinitionsGetResponse.__module__ = "dokime_sdk.client"
PromptDefinitionsGetResponse.__qualname__ = "PromptDefinitionsGetResponse"
PromptDefinitionsListResponse.__module__ = "dokime_sdk.client"
PromptDefinitionsListResponse.__qualname__ = "PromptDefinitionsListResponse"
AnswersCreateResponse.__module__ = "dokime_sdk.client"
AnswersCreateResponse.__qualname__ = "AnswersCreateResponse"
PromptInstance.__module__ = "dokime_sdk.client"
PromptInstance.__qualname__ = "PromptInstance"
PromptResponse.__module__ = "dokime_sdk.client"
PromptResponse.__qualname__ = "PromptResponse"
PromptView.__module__ = "dokime_sdk.client"
PromptView.__qualname__ = "PromptView"
PromptsGetResponse.__module__ = "dokime_sdk.client"
PromptsGetResponse.__qualname__ = "PromptsGetResponse"
PromptsListResponse.__module__ = "dokime_sdk.client"
PromptsListResponse.__qualname__ = "PromptsListResponse"
PromptsWatchResponse.__module__ = "dokime_sdk.client"
PromptsWatchResponse.__qualname__ = "PromptsWatchResponse"
AbsoluteTimeFormat.__module__ = "dokime_sdk.client"
AbsoluteTimeFormat.__qualname__ = "AbsoluteTimeFormat"
ElapsedFormat.__module__ = "dokime_sdk.client"
ElapsedFormat.__qualname__ = "ElapsedFormat"
EngineeringFormat.__module__ = "dokime_sdk.client"
EngineeringFormat.__qualname__ = "EngineeringFormat"
FixedFormat.__module__ = "dokime_sdk.client"
FixedFormat.__qualname__ = "FixedFormat"
MeasurementUnit.__module__ = "dokime_sdk.client"
MeasurementUnit.__qualname__ = "MeasurementUnit"
NormalFormat.__module__ = "dokime_sdk.client"
NormalFormat.__qualname__ = "NormalFormat"
RecordDisplay.__module__ = "dokime_sdk.client"
RecordDisplay.__qualname__ = "RecordDisplay"
RecordField.__module__ = "dokime_sdk.client"
RecordField.__qualname__ = "RecordField"
RecordSchema.__module__ = "dokime_sdk.client"
RecordSchema.__qualname__ = "RecordSchema"
RecordSchemasGetResponse.__module__ = "dokime_sdk.client"
RecordSchemasGetResponse.__qualname__ = "RecordSchemasGetResponse"
RecordSchemasListResponse.__module__ = "dokime_sdk.client"
RecordSchemasListResponse.__qualname__ = "RecordSchemasListResponse"
ScientificFormat.__module__ = "dokime_sdk.client"
ScientificFormat.__qualname__ = "ScientificFormat"
SiPrefixFormat.__module__ = "dokime_sdk.client"
SiPrefixFormat.__qualname__ = "SiPrefixFormat"
TelemetryColumn.__module__ = "dokime_sdk.client"
TelemetryColumn.__qualname__ = "TelemetryColumn"
TelemetrySchema.__module__ = "dokime_sdk.client"
TelemetrySchema.__qualname__ = "TelemetrySchema"
ValueFormat.__module__ = "dokime_sdk.client"
ValueFormat.__qualname__ = "ValueFormat"
ValueFormatKind.__module__ = "dokime_sdk.client"
ValueFormatKind.__qualname__ = "ValueFormatKind"
ReportCapture.__module__ = "dokime_sdk.client"
ReportCapture.__qualname__ = "ReportCapture"
ReportCase.__module__ = "dokime_sdk.client"
ReportCase.__qualname__ = "ReportCase"
ReportExecution.__module__ = "dokime_sdk.client"
ReportExecution.__qualname__ = "ReportExecution"
ReportFile.__module__ = "dokime_sdk.client"
ReportFile.__qualname__ = "ReportFile"
ReportMetadata.__module__ = "dokime_sdk.client"
ReportMetadata.__qualname__ = "ReportMetadata"
ReportSubject.__module__ = "dokime_sdk.client"
ReportSubject.__qualname__ = "ReportSubject"
ReportsCreateResponse.__module__ = "dokime_sdk.client"
ReportsCreateResponse.__qualname__ = "ReportsCreateResponse"
ReportsGetResponse.__module__ = "dokime_sdk.client"
ReportsGetResponse.__qualname__ = "ReportsGetResponse"
ReportsPublishResponse.__module__ = "dokime_sdk.client"
ReportsPublishResponse.__qualname__ = "ReportsPublishResponse"
SecretArchive.__module__ = "dokime_sdk.client"
SecretArchive.__qualname__ = "SecretArchive"
SecretImportChoice.__module__ = "dokime_sdk.client"
SecretImportChoice.__qualname__ = "SecretImportChoice"
SecretImportEntry.__module__ = "dokime_sdk.client"
SecretImportEntry.__qualname__ = "SecretImportEntry"
SecretImportPreview.__module__ = "dokime_sdk.client"
SecretImportPreview.__qualname__ = "SecretImportPreview"
SecretSnapshot.__module__ = "dokime_sdk.client"
SecretSnapshot.__qualname__ = "SecretSnapshot"
SecretSummary.__module__ = "dokime_sdk.client"
SecretSummary.__qualname__ = "SecretSummary"
SecretsDeleteResponse.__module__ = "dokime_sdk.client"
SecretsDeleteResponse.__qualname__ = "SecretsDeleteResponse"
SecretsExportResponse.__module__ = "dokime_sdk.client"
SecretsExportResponse.__qualname__ = "SecretsExportResponse"
SecretsImportResponse.__module__ = "dokime_sdk.client"
SecretsImportResponse.__qualname__ = "SecretsImportResponse"
SecretsSetResponse.__module__ = "dokime_sdk.client"
SecretsSetResponse.__qualname__ = "SecretsSetResponse"
SecretsWatchResponse.__module__ = "dokime_sdk.client"
SecretsWatchResponse.__qualname__ = "SecretsWatchResponse"
ApplicationAppearance.__module__ = "dokime_sdk.client"
ApplicationAppearance.__qualname__ = "ApplicationAppearance"
ApplicationSettings.__module__ = "dokime_sdk.client"
ApplicationSettings.__qualname__ = "ApplicationSettings"
BootstrapPaths.__module__ = "dokime_sdk.client"
BootstrapPaths.__qualname__ = "BootstrapPaths"
CachedRefresh.__module__ = "dokime_sdk.client"
CachedRefresh.__qualname__ = "CachedRefresh"
ClientPreferences.__module__ = "dokime_sdk.client"
ClientPreferences.__qualname__ = "ClientPreferences"
HostCardBlock.__module__ = "dokime_sdk.client"
HostCardBlock.__qualname__ = "HostCardBlock"
HostCardBlockKind.__module__ = "dokime_sdk.client"
HostCardBlockKind.__qualname__ = "HostCardBlockKind"
HostCardBody.__module__ = "dokime_sdk.client"
HostCardBody.__qualname__ = "HostCardBody"
HostCardKeyValue.__module__ = "dokime_sdk.client"
HostCardKeyValue.__qualname__ = "HostCardKeyValue"
HostCardKeyValueList.__module__ = "dokime_sdk.client"
HostCardKeyValueList.__qualname__ = "HostCardKeyValueList"
HostCardMetric.__module__ = "dokime_sdk.client"
HostCardMetric.__qualname__ = "HostCardMetric"
HostCardMetrics.__module__ = "dokime_sdk.client"
HostCardMetrics.__qualname__ = "HostCardMetrics"
HostCardStatus.__module__ = "dokime_sdk.client"
HostCardStatus.__qualname__ = "HostCardStatus"
ManualRefresh.__module__ = "dokime_sdk.client"
ManualRefresh.__qualname__ = "ManualRefresh"
OnDemandRefresh.__module__ = "dokime_sdk.client"
OnDemandRefresh.__qualname__ = "OnDemandRefresh"
RemoteListenerSettings.__module__ = "dokime_sdk.client"
RemoteListenerSettings.__qualname__ = "RemoteListenerSettings"
ReportDefaults.__module__ = "dokime_sdk.client"
ReportDefaults.__qualname__ = "ReportDefaults"
SettingsCard.__module__ = "dokime_sdk.client"
SettingsCard.__qualname__ = "SettingsCard"
SettingsCardRefreshPolicy.__module__ = "dokime_sdk.client"
SettingsCardRefreshPolicy.__qualname__ = "SettingsCardRefreshPolicy"
SettingsCardRefreshPolicyPolicy.__module__ = "dokime_sdk.client"
SettingsCardRefreshPolicyPolicy.__qualname__ = "SettingsCardRefreshPolicyPolicy"
SettingsFrame.__module__ = "dokime_sdk.client"
SettingsFrame.__qualname__ = "SettingsFrame"
SettingsSchemaResponse.__module__ = "dokime_sdk.client"
SettingsSchemaResponse.__qualname__ = "SettingsSchemaResponse"
SettingsSection.__module__ = "dokime_sdk.client"
SettingsSection.__qualname__ = "SettingsSection"
SettingsSectionOwner.__module__ = "dokime_sdk.client"
SettingsSectionOwner.__qualname__ = "SettingsSectionOwner"
TelemetryPreferences.__module__ = "dokime_sdk.client"
TelemetryPreferences.__qualname__ = "TelemetryPreferences"
HostBuild.__module__ = "dokime_sdk.client"
HostBuild.__qualname__ = "HostBuild"
SourcesGetResponse.__module__ = "dokime_sdk.client"
SourcesGetResponse.__qualname__ = "SourcesGetResponse"
SourcesSetResponse.__module__ = "dokime_sdk.client"
SourcesSetResponse.__qualname__ = "SourcesSetResponse"
StorageSource.__module__ = "dokime_sdk.client"
StorageSource.__qualname__ = "StorageSource"
StorageSources.__module__ = "dokime_sdk.client"
StorageSources.__qualname__ = "StorageSources"
StreamSchemasGetResponse.__module__ = "dokime_sdk.client"
StreamSchemasGetResponse.__qualname__ = "StreamSchemasGetResponse"
StreamSchemasListResponse.__module__ = "dokime_sdk.client"
StreamSchemasListResponse.__qualname__ = "StreamSchemasListResponse"
SuitesGetResponse.__module__ = "dokime_sdk.client"
SuitesGetResponse.__qualname__ = "SuitesGetResponse"
SuitesListResponse.__module__ = "dokime_sdk.client"
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
    "CASE_STATUS_NOT_RUN",
    "CASE_STATUS_PASSED",
    "CASE_STATUS_PLANNED",
    "CASE_STATUS_QUEUED",
    "CASE_STATUS_RUNNING",
    "CASE_STATUS_SKIPPED",
    "CASE_STATUS_STOPPED",
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
    "AggregateReply",
    "AggregatesGetResponse",
    "AnnotationOwner",
    "AnnotationOwnerOwner",
    "AnnotationSelection",
    "AnswersCreateResponse",
    "AppearanceAssetMetadata",
    "AppearanceAssetReference",
    "AppearanceFailure",
    "AppearanceFailureKind",
    "AppearanceLogo",
    "AppearanceSnapshot",
    "ApplicationAppearance",
    "ApplicationSettings",
    "ArchiveScope",
    "ArtifactsGetResponse",
    "ArtifactsListResponse",
    "ArtifactsWatchResponse",
    "BackendAppearanceResponse",
    "BootstrapPaths",
    "Bucket",
    "BucketRequest",
    "BucketValue",
    "CachedRefresh",
    "Capture",
    "CaptureKind",
    "CapturesCreateResponse",
    "CapturesGetResponse",
    "CapturesWatchResponse",
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
    "CertificatesCreateResponse",
    "ClientPreferences",
    "CommandsCreateResponse",
    "CommandsGetResponse",
    "ComponentState",
    "ContourGrid",
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
    "DiagnosticsGetResponse",
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
    "Event",
    "EvidenceArchiveResponse",
    "EvidenceCounts",
    "EvidenceSubject",
    "ExecutionBinding",
    "ExecutionCommandAction",
    "ExecutionCommandPhase",
    "ExecutionCommandReceipt",
    "ExecutionControls",
    "ExecutionDetail",
    "ExecutionFileKind",
    "ExecutionFileMetadata",
    "ExecutionFileState",
    "ExecutionFilesGetResponse",
    "ExecutionFilesListResponse",
    "ExecutionFilesWatchResponse",
    "ExecutionFilter",
    "ExecutionMode",
    "ExecutionNoteOwner",
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
    "ExecutionsCreateResponse",
    "ExecutionsDeleteResponse",
    "ExecutionsGetResponse",
    "ExecutionsListResponse",
    "ExecutionsParametersResponse",
    "ExecutionsSealResponse",
    "ExecutionsSetResponse",
    "ExecutionsStartResponse",
    "ExecutionsStepsResponse",
    "ExecutionsStopResponse",
    "ExecutionsWatchResponse",
    "FixedFormat",
    "FrozenSchemas",
    "GridCell",
    "GridFrame",
    "GridFrames",
    "HealthComponent",
    "HealthEventsListResponse",
    "HealthEventsWatchResponse",
    "HealthItem",
    "HostBuild",
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
    "ListenerGetResponse",
    "ListenerSetResponse",
    "ListenerValue",
    "LiveExecutionState",
    "LogoVariant",
    "LogsListResponse",
    "LogsWatchResponse",
    "ManualRefresh",
    "MaterializedParameter",
    "MeasurementUnit",
    "NormalFormat",
    "NoteWritable",
    "NotesCreateResponse",
    "NotesGetResponse",
    "NotesListResponse",
    "NotesSetResponse",
    "NotesWatchResponse",
    "NullableNumber",
    "OnDemandRefresh",
    "OperatorNote",
    "OrganizationIdentity",
    "OverviewGetResponse",
    "OverviewWatchResponse",
    "Pairing",
    "PairingsCreateResponse",
    "PairingsDeleteResponse",
    "PairingsListResponse",
    "ParameterSource",
    "PayloadEncoding",
    "PlannedCase",
    "PlannedCaseNoteOwner",
    "PlotAnalysis",
    "PlotAnalysisError",
    "PlotAnalysisResult",
    "PlotAxisKind",
    "PlotBoxSummary",
    "PlotBoxSummaryAnalysis",
    "PlotCartesianMode",
    "PlotColumn",
    "PlotDomain",
    "PlotEnvelope",
    "PlotFrame",
    "PlotHistogramAnalysis",
    "PlotHistogramBin",
    "PlotHistogramSeries",
    "PlotHistogramStatistics",
    "PlotMarker",
    "PlotPayload",
    "PlotPayloadDiagnostics",
    "PlotPresentation",
    "PlotReferenceBand",
    "PlotReferenceLine",
    "PlotReferenceValue",
    "PlotRequest",
    "PlotSeries",
    "PlotSeriesSummary",
    "PlotSettingsValue",
    "PlotSigmaBin",
    "PlotSigmaIntervalAnalysis",
    "PlotSigmaSeries",
    "PlotStreamKind",
    "PlotStreamUpdate",
    "PlotWindow",
    "ProgressWatchResponse",
    "PromptDefinitionsGetResponse",
    "PromptDefinitionsListResponse",
    "PromptInstance",
    "PromptKind",
    "PromptResponse",
    "PromptView",
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
    "RecordingAudio",
    "RecordingProgress",
    "RecordsWatchResponse",
    "RegisteredSelection",
    "RegisteredSelectionFilesEntry",
    "RegisteredSelectionParametersEntry",
    "RemoteListenerSettings",
    "ReportCapture",
    "ReportCase",
    "ReportDefaults",
    "ReportExecution",
    "ReportFile",
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
    "RuntimeShellSnapshot",
    "SampleRow",
    "SamplesListResponse",
    "SamplesWatchResponse",
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
    "SchemasGetResponse",
    "SchemasListResponse",
    "ScientificFormat",
    "SecretArchive",
    "SecretImportAction",
    "SecretImportChoice",
    "SecretImportEntry",
    "SecretImportPreview",
    "SecretRequirement",
    "SecretSnapshot",
    "SecretSummary",
    "SecretsDeleteResponse",
    "SecretsExportResponse",
    "SecretsImportResponse",
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
    "SettingsSchemaResponse",
    "SettingsSection",
    "SettingsSectionOwner",
    "SiPrefixFormat",
    "SnapshotsListResponse",
    "SnapshotsWatchResponse",
    "SourcesGetResponse",
    "SourcesSetResponse",
    "StatisticValue",
    "StepCriterion",
    "StepStatus",
    "StorageSource",
    "StorageSources",
    "StreamIdentity",
    "StreamSchemasGetResponse",
    "StreamSchemasListResponse",
    "StreamsListResponse",
    "SuitesGetResponse",
    "SuitesListResponse",
    "TelemetryColumn",
    "TelemetryPreferences",
    "TelemetryRole",
    "TelemetrySample",
    "TelemetrySchema",
    "ThemeCornerRadii",
    "ThemeFamily",
    "ThemeMode",
    "ThemeModePolicy",
    "ThemeVariant",
    "TraceEventsListResponse",
    "TraceEventsWatchResponse",
    "TraceabilityWatchResponse",
    "UnevaluatedOutcome",
    "UnitFamily",
    "ValidationMode",
    "ValueFormat",
    "ValueFormatKind",
    "harmos",
    "host",
]
