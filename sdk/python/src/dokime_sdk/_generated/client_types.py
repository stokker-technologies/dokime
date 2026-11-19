# @generated from protocol descriptors; do not edit.
from dokime_sdk._wire.host.v1.aggregates_pb2 import AggregateReply

AggregateReply.__module__ = "dokime_sdk.client"
AggregateReply.__qualname__ = "AggregateReply"
from dokime_sdk._wire.host.v1.aggregates_pb2 import AggregatesGetResponse

AggregatesGetResponse.__module__ = "dokime_sdk.client"
AggregatesGetResponse.__qualname__ = "AggregatesGetResponse"
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
    QueryAggregate,
)

QueryAggregate.__module__ = "dokime_sdk.client"
QueryAggregate.__qualname__ = "QueryAggregate"
from dokime_sdk._wire.host.v1.aggregates_pb2 import QueryFilter

QueryFilter.__module__ = "dokime_sdk.client"
QueryFilter.__qualname__ = "QueryFilter"
from dokime_sdk._wire.host.v1.aggregates_pb2 import QueryFilterOperator, QueryRow

QueryRow.__module__ = "dokime_sdk.client"
QueryRow.__qualname__ = "QueryRow"
from dokime_sdk._wire.host.v1.aggregates_pb2 import QuerySource
from dokime_sdk._wire.host.v1.backend.overview_pb2 import EvidenceCounts

EvidenceCounts.__module__ = "dokime_sdk.client"
EvidenceCounts.__qualname__ = "EvidenceCounts"
from dokime_sdk._wire.host.v1.backend.overview_pb2 import OverviewGetResponse

OverviewGetResponse.__module__ = "dokime_sdk.client"
OverviewGetResponse.__qualname__ = "OverviewGetResponse"
from dokime_sdk._wire.host.v1.backend.overview_pb2 import OverviewWatchResponse

OverviewWatchResponse.__module__ = "dokime_sdk.client"
OverviewWatchResponse.__qualname__ = "OverviewWatchResponse"
from dokime_sdk._wire.host.v1.backend.overview_pb2 import RuntimeShellSnapshot

RuntimeShellSnapshot.__module__ = "dokime_sdk.client"
RuntimeShellSnapshot.__qualname__ = "RuntimeShellSnapshot"
from dokime_sdk._wire.host.v1.backend_pb2 import (
    APPEARANCE_FAILURE_KIND_ASSET_CHANGED,
    APPEARANCE_FAILURE_KIND_ASSET_MISSING,
    APPEARANCE_FAILURE_KIND_ASSET_UNAVAILABLE,
    APPEARANCE_FAILURE_KIND_INVALID_REFERENCE,
    APPEARANCE_FAILURE_KIND_TOO_LARGE,
    APPEARANCE_FAILURE_KIND_UNSPECIFIED,
    AppearanceAssetMetadata,
)

AppearanceAssetMetadata.__module__ = "dokime_sdk.client"
AppearanceAssetMetadata.__qualname__ = "AppearanceAssetMetadata"
from dokime_sdk._wire.host.v1.backend_pb2 import AppearanceAssetReference

AppearanceAssetReference.__module__ = "dokime_sdk.client"
AppearanceAssetReference.__qualname__ = "AppearanceAssetReference"
from dokime_sdk._wire.host.v1.backend_pb2 import AppearanceFailure

AppearanceFailure.__module__ = "dokime_sdk.client"
AppearanceFailure.__qualname__ = "AppearanceFailure"
from dokime_sdk._wire.host.v1.backend_pb2 import AppearanceFailureKind, AppearanceLogo

AppearanceLogo.__module__ = "dokime_sdk.client"
AppearanceLogo.__qualname__ = "AppearanceLogo"
from dokime_sdk._wire.host.v1.backend_pb2 import AppearanceSnapshot

AppearanceSnapshot.__module__ = "dokime_sdk.client"
AppearanceSnapshot.__qualname__ = "AppearanceSnapshot"
from dokime_sdk._wire.host.v1.backend_pb2 import BackendAppearanceResponse

BackendAppearanceResponse.__module__ = "dokime_sdk.client"
BackendAppearanceResponse.__qualname__ = "BackendAppearanceResponse"
from dokime_sdk._wire.host.v1.backend_pb2 import (
    LOGO_VARIANT_CONTAINED_ON_DARK,
    LOGO_VARIANT_CONTAINED_ON_LIGHT,
    LOGO_VARIANT_TRANSPARENT_ON_DARK,
    LOGO_VARIANT_TRANSPARENT_ON_LIGHT,
    LOGO_VARIANT_UNSPECIFIED,
    LogoVariant,
    OrganizationIdentity,
)

OrganizationIdentity.__module__ = "dokime_sdk.client"
OrganizationIdentity.__qualname__ = "OrganizationIdentity"
from dokime_sdk._wire.host.v1.backend_pb2 import (
    THEME_MODE_POLICY_FORCE_DARK,
    THEME_MODE_POLICY_FORCE_LIGHT,
    THEME_MODE_POLICY_FORCE_SYSTEM,
    THEME_MODE_POLICY_UNSPECIFIED,
    THEME_MODE_POLICY_USER_OR_SYSTEM,
    ThemeCornerRadii,
)

ThemeCornerRadii.__module__ = "dokime_sdk.client"
ThemeCornerRadii.__qualname__ = "ThemeCornerRadii"
from dokime_sdk._wire.host.v1.backend_pb2 import ThemeFamily

ThemeFamily.__module__ = "dokime_sdk.client"
ThemeFamily.__qualname__ = "ThemeFamily"
from dokime_sdk._wire.host.v1.backend_pb2 import ThemeModePolicy, ThemeVariant

ThemeVariant.__module__ = "dokime_sdk.client"
ThemeVariant.__qualname__ = "ThemeVariant"
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
    CASE_STATUS_UNSPECIFIED,
    CaseActions,
)

CaseActions.__module__ = "dokime_sdk.client"
CaseActions.__qualname__ = "CaseActions"
from dokime_sdk._wire.host.v1.cases_pb2 import CaseDetail

CaseDetail.__module__ = "dokime_sdk.client"
CaseDetail.__qualname__ = "CaseDetail"
from dokime_sdk._wire.host.v1.cases_pb2 import CaseParameter

CaseParameter.__module__ = "dokime_sdk.client"
CaseParameter.__qualname__ = "CaseParameter"
from dokime_sdk._wire.host.v1.cases_pb2 import CaseStatus, CaseSummary

CaseSummary.__module__ = "dokime_sdk.client"
CaseSummary.__qualname__ = "CaseSummary"
from dokime_sdk._wire.host.v1.cases_pb2 import CasesCreateResponse

CasesCreateResponse.__module__ = "dokime_sdk.client"
CasesCreateResponse.__qualname__ = "CasesCreateResponse"
from dokime_sdk._wire.host.v1.cases_pb2 import CasesDeleteResponse

CasesDeleteResponse.__module__ = "dokime_sdk.client"
CasesDeleteResponse.__qualname__ = "CasesDeleteResponse"
from dokime_sdk._wire.host.v1.cases_pb2 import CasesGetResponse

CasesGetResponse.__module__ = "dokime_sdk.client"
CasesGetResponse.__qualname__ = "CasesGetResponse"
from dokime_sdk._wire.host.v1.cases_pb2 import CasesListResponse

CasesListResponse.__module__ = "dokime_sdk.client"
CasesListResponse.__qualname__ = "CasesListResponse"
from dokime_sdk._wire.host.v1.cases_pb2 import CasesParametersResponse

CasesParametersResponse.__module__ = "dokime_sdk.client"
CasesParametersResponse.__qualname__ = "CasesParametersResponse"
from dokime_sdk._wire.host.v1.cases_pb2 import CasesWatchResponse

CasesWatchResponse.__module__ = "dokime_sdk.client"
CasesWatchResponse.__qualname__ = "CasesWatchResponse"
from dokime_sdk._wire.host.v1.cases_pb2 import PlannedCase

PlannedCase.__module__ = "dokime_sdk.client"
PlannedCase.__qualname__ = "PlannedCase"
from dokime_sdk._wire.host.v1.cases_pb2 import RegisteredSelection

RegisteredSelection.__module__ = "dokime_sdk.client"
RegisteredSelection.__qualname__ = "RegisteredSelection"
from dokime_sdk._wire.host.v1.cases_pb2 import RegisteredSelectionFilesEntry

RegisteredSelectionFilesEntry.__module__ = "dokime_sdk.client"
RegisteredSelectionFilesEntry.__qualname__ = "RegisteredSelectionFilesEntry"
from dokime_sdk._wire.host.v1.cases_pb2 import RegisteredSelectionParametersEntry

RegisteredSelectionParametersEntry.__module__ = "dokime_sdk.client"
RegisteredSelectionParametersEntry.__qualname__ = "RegisteredSelectionParametersEntry"
from dokime_sdk._wire.host.v1.cases_pb2 import SelectedFile

SelectedFile.__module__ = "dokime_sdk.client"
SelectedFile.__qualname__ = "SelectedFile"
from dokime_sdk._wire.host.v1.cases_pb2 import SelectedFileKind

SelectedFileKind.__module__ = "dokime_sdk.client"
SelectedFileKind.__qualname__ = "SelectedFileKind"
from dokime_sdk._wire.host.v1.cases_pb2 import SelectionPlan

SelectionPlan.__module__ = "dokime_sdk.client"
SelectionPlan.__qualname__ = "SelectionPlan"
from dokime_sdk._wire.host.v1.cases_pb2 import SelectionValues

SelectionValues.__module__ = "dokime_sdk.client"
SelectionValues.__qualname__ = "SelectionValues"
from dokime_sdk._wire.host.v1.content.artifacts_pb2 import ArtifactsGetResponse

ArtifactsGetResponse.__module__ = "dokime_sdk.client"
ArtifactsGetResponse.__qualname__ = "ArtifactsGetResponse"
from dokime_sdk._wire.host.v1.content.artifacts_pb2 import ArtifactsListResponse

ArtifactsListResponse.__module__ = "dokime_sdk.client"
ArtifactsListResponse.__qualname__ = "ArtifactsListResponse"
from dokime_sdk._wire.host.v1.content.artifacts_pb2 import ArtifactsWatchResponse

ArtifactsWatchResponse.__module__ = "dokime_sdk.client"
ArtifactsWatchResponse.__qualname__ = "ArtifactsWatchResponse"
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
)

ExecutionFileMetadata.__module__ = "dokime_sdk.client"
ExecutionFileMetadata.__qualname__ = "ExecutionFileMetadata"
from dokime_sdk._wire.host.v1.content.execution_files_pb2 import ExecutionFilesGetResponse, ExecutionFileState

ExecutionFilesGetResponse.__module__ = "dokime_sdk.client"
ExecutionFilesGetResponse.__qualname__ = "ExecutionFilesGetResponse"
from dokime_sdk._wire.host.v1.content.execution_files_pb2 import ExecutionFilesListResponse

ExecutionFilesListResponse.__module__ = "dokime_sdk.client"
ExecutionFilesListResponse.__qualname__ = "ExecutionFilesListResponse"
from dokime_sdk._wire.host.v1.content.execution_files_pb2 import ExecutionFilesWatchResponse

ExecutionFilesWatchResponse.__module__ = "dokime_sdk.client"
ExecutionFilesWatchResponse.__qualname__ = "ExecutionFilesWatchResponse"
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
    ArchiveScope,
    CycleFilter,
)

CycleFilter.__module__ = "dokime_sdk.client"
CycleFilter.__qualname__ = "CycleFilter"
from dokime_sdk._wire.host.v1.cycles_pb2 import CycleOrder

CycleOrder.__module__ = "dokime_sdk.client"
CycleOrder.__qualname__ = "CycleOrder"
from dokime_sdk._wire.host.v1.cycles_pb2 import CycleOrderKey, CyclePhase, CycleSelection

CycleSelection.__module__ = "dokime_sdk.client"
CycleSelection.__qualname__ = "CycleSelection"
from dokime_sdk._wire.host.v1.cycles_pb2 import CycleStatus, CycleSummary

CycleSummary.__module__ = "dokime_sdk.client"
CycleSummary.__qualname__ = "CycleSummary"
from dokime_sdk._wire.host.v1.cycles_pb2 import CycleWritable

CycleWritable.__module__ = "dokime_sdk.client"
CycleWritable.__qualname__ = "CycleWritable"
from dokime_sdk._wire.host.v1.cycles_pb2 import CyclesCreateResponse

CyclesCreateResponse.__module__ = "dokime_sdk.client"
CyclesCreateResponse.__qualname__ = "CyclesCreateResponse"
from dokime_sdk._wire.host.v1.cycles_pb2 import CyclesGetResponse

CyclesGetResponse.__module__ = "dokime_sdk.client"
CyclesGetResponse.__qualname__ = "CyclesGetResponse"
from dokime_sdk._wire.host.v1.cycles_pb2 import CyclesListResponse

CyclesListResponse.__module__ = "dokime_sdk.client"
CyclesListResponse.__qualname__ = "CyclesListResponse"
from dokime_sdk._wire.host.v1.cycles_pb2 import CyclesSealResponse

CyclesSealResponse.__module__ = "dokime_sdk.client"
CyclesSealResponse.__qualname__ = "CyclesSealResponse"
from dokime_sdk._wire.host.v1.cycles_pb2 import CyclesSetResponse

CyclesSetResponse.__module__ = "dokime_sdk.client"
CyclesSetResponse.__qualname__ = "CyclesSetResponse"
from dokime_sdk._wire.host.v1.cycles_pb2 import CyclesWatchResponse

CyclesWatchResponse.__module__ = "dokime_sdk.client"
CyclesWatchResponse.__qualname__ = "CyclesWatchResponse"
from dokime_sdk._wire.host.v1.cycles_pb2 import (
    RESULT_STATUS_BLOCKED,
    RESULT_STATUS_CANCELLED,
    RESULT_STATUS_FAILED,
    RESULT_STATUS_PASSED,
    RESULT_STATUS_PENDING,
    RESULT_STATUS_UNSPECIFIED,
    RerunCaseNotice,
)

RerunCaseNotice.__module__ = "dokime_sdk.client"
RerunCaseNotice.__qualname__ = "RerunCaseNotice"
from dokime_sdk._wire.host.v1.cycles_pb2 import ResultStatus, ResultSummary

ResultSummary.__module__ = "dokime_sdk.client"
ResultSummary.__qualname__ = "ResultSummary"
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
    DocumentBinding,
)

DocumentBinding.__module__ = "dokime_sdk.client"
DocumentBinding.__qualname__ = "DocumentBinding"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentBindingResult

DocumentBindingResult.__module__ = "dokime_sdk.client"
DocumentBindingResult.__qualname__ = "DocumentBindingResult"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentCard

DocumentCard.__module__ = "dokime_sdk.client"
DocumentCard.__qualname__ = "DocumentCard"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentCardBody

DocumentCardBody.__module__ = "dokime_sdk.client"
DocumentCardBody.__qualname__ = "DocumentCardBody"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentCardBudget

DocumentCardBudget.__module__ = "dokime_sdk.client"
DocumentCardBudget.__qualname__ = "DocumentCardBudget"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentCardFailure

DocumentCardFailure.__module__ = "dokime_sdk.client"
DocumentCardFailure.__qualname__ = "DocumentCardFailure"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentCardKind, DocumentCompletion

DocumentCompletion.__module__ = "dokime_sdk.client"
DocumentCompletion.__qualname__ = "DocumentCompletion"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentDensity, DocumentEvidenceCut

DocumentEvidenceCut.__module__ = "dokime_sdk.client"
DocumentEvidenceCut.__qualname__ = "DocumentEvidenceCut"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentFrame

DocumentFrame.__module__ = "dokime_sdk.client"
DocumentFrame.__qualname__ = "DocumentFrame"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentFrameFrame

DocumentFrameFrame.__module__ = "dokime_sdk.client"
DocumentFrameFrame.__qualname__ = "DocumentFrameFrame"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentFrontiers

DocumentFrontiers.__module__ = "dokime_sdk.client"
DocumentFrontiers.__qualname__ = "DocumentFrontiers"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentHeader

DocumentHeader.__module__ = "dokime_sdk.client"
DocumentHeader.__qualname__ = "DocumentHeader"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentLayoutNode

DocumentLayoutNode.__module__ = "dokime_sdk.client"
DocumentLayoutNode.__qualname__ = "DocumentLayoutNode"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentLayoutNodeNode

DocumentLayoutNodeNode.__module__ = "dokime_sdk.client"
DocumentLayoutNodeNode.__qualname__ = "DocumentLayoutNodeNode"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentOwnerReference

DocumentOwnerReference.__module__ = "dokime_sdk.client"
DocumentOwnerReference.__qualname__ = "DocumentOwnerReference"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentPage

DocumentPage.__module__ = "dokime_sdk.client"
DocumentPage.__qualname__ = "DocumentPage"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentPlacement

DocumentPlacement.__module__ = "dokime_sdk.client"
DocumentPlacement.__qualname__ = "DocumentPlacement"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentPlotDefinition

DocumentPlotDefinition.__module__ = "dokime_sdk.client"
DocumentPlotDefinition.__qualname__ = "DocumentPlotDefinition"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentRail, DocumentRepeat

DocumentRepeat.__module__ = "dokime_sdk.client"
DocumentRepeat.__qualname__ = "DocumentRepeat"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentRepeatInstance

DocumentRepeatInstance.__module__ = "dokime_sdk.client"
DocumentRepeatInstance.__qualname__ = "DocumentRepeatInstance"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentRepeatSelection

DocumentRepeatSelection.__module__ = "dokime_sdk.client"
DocumentRepeatSelection.__qualname__ = "DocumentRepeatSelection"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentScope

DocumentScope.__module__ = "dokime_sdk.client"
DocumentScope.__qualname__ = "DocumentScope"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSection

DocumentSection.__module__ = "dokime_sdk.client"
DocumentSection.__qualname__ = "DocumentSection"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSectionIdentity

DocumentSectionIdentity.__module__ = "dokime_sdk.client"
DocumentSectionIdentity.__qualname__ = "DocumentSectionIdentity"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSpan

DocumentSpan.__module__ = "dokime_sdk.client"
DocumentSpan.__qualname__ = "DocumentSpan"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSummary

DocumentSummary.__module__ = "dokime_sdk.client"
DocumentSummary.__qualname__ = "DocumentSummary"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSummaryCondition

DocumentSummaryCondition.__module__ = "dokime_sdk.client"
DocumentSummaryCondition.__qualname__ = "DocumentSummaryCondition"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSummaryFormat, DocumentSummaryGroup

DocumentSummaryGroup.__module__ = "dokime_sdk.client"
DocumentSummaryGroup.__qualname__ = "DocumentSummaryGroup"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSummaryGroupDefinition

DocumentSummaryGroupDefinition.__module__ = "dokime_sdk.client"
DocumentSummaryGroupDefinition.__qualname__ = "DocumentSummaryGroupDefinition"
from dokime_sdk._wire.host.v1.documents_pb2 import (
    DocumentSummaryPresentation,
    DocumentSummaryStat,
    DocumentSummaryState,
    DocumentSummaryTile,
)

DocumentSummaryTile.__module__ = "dokime_sdk.client"
DocumentSummaryTile.__qualname__ = "DocumentSummaryTile"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentSummaryTileDefinition

DocumentSummaryTileDefinition.__module__ = "dokime_sdk.client"
DocumentSummaryTileDefinition.__qualname__ = "DocumentSummaryTileDefinition"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentTable

DocumentTable.__module__ = "dokime_sdk.client"
DocumentTable.__qualname__ = "DocumentTable"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentTableRow

DocumentTableRow.__module__ = "dokime_sdk.client"
DocumentTableRow.__qualname__ = "DocumentTableRow"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentTelemetryCut

DocumentTelemetryCut.__module__ = "dokime_sdk.client"
DocumentTelemetryCut.__qualname__ = "DocumentTelemetryCut"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentWarning

DocumentWarning.__module__ = "dokime_sdk.client"
DocumentWarning.__qualname__ = "DocumentWarning"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentsGetResponse

DocumentsGetResponse.__module__ = "dokime_sdk.client"
DocumentsGetResponse.__qualname__ = "DocumentsGetResponse"
from dokime_sdk._wire.host.v1.documents_pb2 import DocumentsWatchResponse

DocumentsWatchResponse.__module__ = "dokime_sdk.client"
DocumentsWatchResponse.__qualname__ = "DocumentsWatchResponse"
from dokime_sdk._wire.host.v1.documents_pb2 import (
    PLOT_AXIS_KIND_ELAPSED,
    PLOT_AXIS_KIND_UNSPECIFIED,
    PLOT_AXIS_KIND_WALL_TIME,
    PlotAxisKind,
    PlotPresentation,
)

PlotPresentation.__module__ = "dokime_sdk.client"
PlotPresentation.__qualname__ = "PlotPresentation"
from dokime_sdk._wire.host.v1.documents_pb2 import PlotReferenceBand

PlotReferenceBand.__module__ = "dokime_sdk.client"
PlotReferenceBand.__qualname__ = "PlotReferenceBand"
from dokime_sdk._wire.host.v1.documents_pb2 import PlotReferenceLine

PlotReferenceLine.__module__ = "dokime_sdk.client"
PlotReferenceLine.__qualname__ = "PlotReferenceLine"
from dokime_sdk._wire.host.v1.documents_pb2 import PlotReferenceValue

PlotReferenceValue.__module__ = "dokime_sdk.client"
PlotReferenceValue.__qualname__ = "PlotReferenceValue"
from dokime_sdk._wire.host.v1.evidence.health_events_pb2 import HealthEventsListResponse

HealthEventsListResponse.__module__ = "dokime_sdk.client"
HealthEventsListResponse.__qualname__ = "HealthEventsListResponse"
from dokime_sdk._wire.host.v1.evidence.health_events_pb2 import HealthEventsWatchResponse

HealthEventsWatchResponse.__module__ = "dokime_sdk.client"
HealthEventsWatchResponse.__qualname__ = "HealthEventsWatchResponse"
from dokime_sdk._wire.host.v1.evidence.health_events_pb2 import HealthItem

HealthItem.__module__ = "dokime_sdk.client"
HealthItem.__qualname__ = "HealthItem"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import AnnotationOwner

AnnotationOwner.__module__ = "dokime_sdk.client"
AnnotationOwner.__qualname__ = "AnnotationOwner"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import AnnotationOwnerOwner

AnnotationOwnerOwner.__module__ = "dokime_sdk.client"
AnnotationOwnerOwner.__qualname__ = "AnnotationOwnerOwner"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import AnnotationSelection

AnnotationSelection.__module__ = "dokime_sdk.client"
AnnotationSelection.__qualname__ = "AnnotationSelection"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import ExecutionNoteOwner

ExecutionNoteOwner.__module__ = "dokime_sdk.client"
ExecutionNoteOwner.__qualname__ = "ExecutionNoteOwner"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import NoteWritable

NoteWritable.__module__ = "dokime_sdk.client"
NoteWritable.__qualname__ = "NoteWritable"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import NotesCreateResponse

NotesCreateResponse.__module__ = "dokime_sdk.client"
NotesCreateResponse.__qualname__ = "NotesCreateResponse"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import NotesGetResponse

NotesGetResponse.__module__ = "dokime_sdk.client"
NotesGetResponse.__qualname__ = "NotesGetResponse"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import NotesListResponse

NotesListResponse.__module__ = "dokime_sdk.client"
NotesListResponse.__qualname__ = "NotesListResponse"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import NotesSetResponse

NotesSetResponse.__module__ = "dokime_sdk.client"
NotesSetResponse.__qualname__ = "NotesSetResponse"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import NotesWatchResponse

NotesWatchResponse.__module__ = "dokime_sdk.client"
NotesWatchResponse.__qualname__ = "NotesWatchResponse"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import OperatorNote

OperatorNote.__module__ = "dokime_sdk.client"
OperatorNote.__qualname__ = "OperatorNote"
from dokime_sdk._wire.host.v1.evidence.notes_pb2 import PlannedCaseNoteOwner

PlannedCaseNoteOwner.__module__ = "dokime_sdk.client"
PlannedCaseNoteOwner.__qualname__ = "PlannedCaseNoteOwner"
from dokime_sdk._wire.host.v1.evidence.records_pb2 import RecordsWatchResponse

RecordsWatchResponse.__module__ = "dokime_sdk.client"
RecordsWatchResponse.__qualname__ = "RecordsWatchResponse"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import Bucket

Bucket.__module__ = "dokime_sdk.client"
Bucket.__qualname__ = "Bucket"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import BucketRequest

BucketRequest.__module__ = "dokime_sdk.client"
BucketRequest.__qualname__ = "BucketRequest"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import BucketValue

BucketValue.__module__ = "dokime_sdk.client"
BucketValue.__qualname__ = "BucketValue"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import SampleRow

SampleRow.__module__ = "dokime_sdk.client"
SampleRow.__qualname__ = "SampleRow"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import SamplesListResponse

SamplesListResponse.__module__ = "dokime_sdk.client"
SamplesListResponse.__qualname__ = "SamplesListResponse"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import SamplesWatchResponse

SamplesWatchResponse.__module__ = "dokime_sdk.client"
SamplesWatchResponse.__qualname__ = "SamplesWatchResponse"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import StatisticValue

StatisticValue.__module__ = "dokime_sdk.client"
StatisticValue.__qualname__ = "StatisticValue"
from dokime_sdk._wire.host.v1.evidence.samples_pb2 import TelemetrySample

TelemetrySample.__module__ = "dokime_sdk.client"
TelemetrySample.__qualname__ = "TelemetrySample"
from dokime_sdk._wire.host.v1.evidence.traceability_pb2 import TraceabilityWatchResponse

TraceabilityWatchResponse.__module__ = "dokime_sdk.client"
TraceabilityWatchResponse.__qualname__ = "TraceabilityWatchResponse"
from dokime_sdk._wire.host.v1.evidence_pb2 import EvidenceArchiveResponse

EvidenceArchiveResponse.__module__ = "dokime_sdk.client"
EvidenceArchiveResponse.__qualname__ = "EvidenceArchiveResponse"
from dokime_sdk._wire.host.v1.evidence_pb2 import EvidenceSubject

EvidenceSubject.__module__ = "dokime_sdk.client"
EvidenceSubject.__qualname__ = "EvidenceSubject"
from dokime_sdk._wire.host.v1.executions.commands_pb2 import CommandsCreateResponse

CommandsCreateResponse.__module__ = "dokime_sdk.client"
CommandsCreateResponse.__qualname__ = "CommandsCreateResponse"
from dokime_sdk._wire.host.v1.executions.commands_pb2 import CommandsGetResponse

CommandsGetResponse.__module__ = "dokime_sdk.client"
CommandsGetResponse.__qualname__ = "CommandsGetResponse"
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
    ExecutionCommandAction,
    ExecutionCommandPhase,
    ExecutionCommandReceipt,
)

ExecutionCommandReceipt.__module__ = "dokime_sdk.client"
ExecutionCommandReceipt.__qualname__ = "ExecutionCommandReceipt"
from dokime_sdk._wire.host.v1.executions.progress_pb2 import ProgressWatchResponse

ProgressWatchResponse.__module__ = "dokime_sdk.client"
ProgressWatchResponse.__qualname__ = "ProgressWatchResponse"
from dokime_sdk._wire.host.v1.executions.schemas_pb2 import SchemasGetResponse

SchemasGetResponse.__module__ = "dokime_sdk.client"
SchemasGetResponse.__qualname__ = "SchemasGetResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import (
    CRITERION_STATUS_FAILED,
    CRITERION_STATUS_INDETERMINATE,
    CRITERION_STATUS_PASSED,
    CRITERION_STATUS_PENDING,
    CRITERION_STATUS_SKIPPED,
    CRITERION_STATUS_UNSPECIFIED,
    CriterionStatus,
    DeclaredOutcome,
)

DeclaredOutcome.__module__ = "dokime_sdk.client"
DeclaredOutcome.__qualname__ = "DeclaredOutcome"
from dokime_sdk._wire.host.v1.executions_pb2 import Diagnostic

Diagnostic.__module__ = "dokime_sdk.client"
Diagnostic.__qualname__ = "Diagnostic"
from dokime_sdk._wire.host.v1.executions_pb2 import (
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
    EvaluatedOutcome,
)

EvaluatedOutcome.__module__ = "dokime_sdk.client"
EvaluatedOutcome.__qualname__ = "EvaluatedOutcome"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionBinding

ExecutionBinding.__module__ = "dokime_sdk.client"
ExecutionBinding.__qualname__ = "ExecutionBinding"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionControls

ExecutionControls.__module__ = "dokime_sdk.client"
ExecutionControls.__qualname__ = "ExecutionControls"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionDetail

ExecutionDetail.__module__ = "dokime_sdk.client"
ExecutionDetail.__qualname__ = "ExecutionDetail"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionFilter

ExecutionFilter.__module__ = "dokime_sdk.client"
ExecutionFilter.__qualname__ = "ExecutionFilter"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionMode, ExecutionOrder

ExecutionOrder.__module__ = "dokime_sdk.client"
ExecutionOrder.__qualname__ = "ExecutionOrder"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionOrderKey, ExecutionOutcome

ExecutionOutcome.__module__ = "dokime_sdk.client"
ExecutionOutcome.__qualname__ = "ExecutionOutcome"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionOutcomeCounts

ExecutionOutcomeCounts.__module__ = "dokime_sdk.client"
ExecutionOutcomeCounts.__qualname__ = "ExecutionOutcomeCounts"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionParameterSet, ExecutionSelection

ExecutionSelection.__module__ = "dokime_sdk.client"
ExecutionSelection.__qualname__ = "ExecutionSelection"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionState, ExecutionStep

ExecutionStep.__module__ = "dokime_sdk.client"
ExecutionStep.__qualname__ = "ExecutionStep"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionSummary

ExecutionSummary.__module__ = "dokime_sdk.client"
ExecutionSummary.__qualname__ = "ExecutionSummary"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionWritable

ExecutionWritable.__module__ = "dokime_sdk.client"
ExecutionWritable.__qualname__ = "ExecutionWritable"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsCancelResponse

ExecutionsCancelResponse.__module__ = "dokime_sdk.client"
ExecutionsCancelResponse.__qualname__ = "ExecutionsCancelResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsCreateResponse

ExecutionsCreateResponse.__module__ = "dokime_sdk.client"
ExecutionsCreateResponse.__qualname__ = "ExecutionsCreateResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsDeleteResponse

ExecutionsDeleteResponse.__module__ = "dokime_sdk.client"
ExecutionsDeleteResponse.__qualname__ = "ExecutionsDeleteResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsGetResponse

ExecutionsGetResponse.__module__ = "dokime_sdk.client"
ExecutionsGetResponse.__qualname__ = "ExecutionsGetResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsListResponse

ExecutionsListResponse.__module__ = "dokime_sdk.client"
ExecutionsListResponse.__qualname__ = "ExecutionsListResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsParametersResponse

ExecutionsParametersResponse.__module__ = "dokime_sdk.client"
ExecutionsParametersResponse.__qualname__ = "ExecutionsParametersResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsSealResponse

ExecutionsSealResponse.__module__ = "dokime_sdk.client"
ExecutionsSealResponse.__qualname__ = "ExecutionsSealResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsSetResponse

ExecutionsSetResponse.__module__ = "dokime_sdk.client"
ExecutionsSetResponse.__qualname__ = "ExecutionsSetResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsStartResponse

ExecutionsStartResponse.__module__ = "dokime_sdk.client"
ExecutionsStartResponse.__qualname__ = "ExecutionsStartResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsStepsResponse

ExecutionsStepsResponse.__module__ = "dokime_sdk.client"
ExecutionsStepsResponse.__qualname__ = "ExecutionsStepsResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsStopResponse

ExecutionsStopResponse.__module__ = "dokime_sdk.client"
ExecutionsStopResponse.__qualname__ = "ExecutionsStopResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import ExecutionsWatchResponse

ExecutionsWatchResponse.__module__ = "dokime_sdk.client"
ExecutionsWatchResponse.__qualname__ = "ExecutionsWatchResponse"
from dokime_sdk._wire.host.v1.executions_pb2 import FrozenSchemas

FrozenSchemas.__module__ = "dokime_sdk.client"
FrozenSchemas.__qualname__ = "FrozenSchemas"
from dokime_sdk._wire.host.v1.executions_pb2 import (
    LIVE_EXECUTION_STATE_CANCELLED,
    LIVE_EXECUTION_STATE_CLEANING_UP,
    LIVE_EXECUTION_STATE_EVALUATING,
    LIVE_EXECUTION_STATE_FINISHED,
    LIVE_EXECUTION_STATE_INITIALIZING,
    LIVE_EXECUTION_STATE_INTERRUPTED,
    LIVE_EXECUTION_STATE_QUEUED,
    LIVE_EXECUTION_STATE_STEPPING,
    LIVE_EXECUTION_STATE_UNSPECIFIED,
    LiveExecutionState,
    MaterializedParameter,
)

MaterializedParameter.__module__ = "dokime_sdk.client"
MaterializedParameter.__qualname__ = "MaterializedParameter"
from dokime_sdk._wire.host.v1.executions_pb2 import (
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
    ParameterSource,
    ReportState,
    ReportStatus,
)

ReportStatus.__module__ = "dokime_sdk.client"
ReportStatus.__qualname__ = "ReportStatus"
from dokime_sdk._wire.host.v1.executions_pb2 import (
    STEP_STATUS_CANCELLED,
    STEP_STATUS_FAILED,
    STEP_STATUS_PASSED,
    STEP_STATUS_PENDING,
    STEP_STATUS_RUNNING,
    STEP_STATUS_SKIPPED,
    STEP_STATUS_UNSPECIFIED,
    StepCriterion,
)

StepCriterion.__module__ = "dokime_sdk.client"
StepCriterion.__qualname__ = "StepCriterion"
from dokime_sdk._wire.host.v1.executions_pb2 import StepStatus, UnevaluatedOutcome

UnevaluatedOutcome.__module__ = "dokime_sdk.client"
UnevaluatedOutcome.__qualname__ = "UnevaluatedOutcome"
from dokime_sdk._wire.host.v1.media.captures_pb2 import (
    CAPTURE_KIND_AUDIO,
    CAPTURE_KIND_CAMERA,
    CAPTURE_KIND_PHOTO,
    CAPTURE_KIND_SCREEN,
    CAPTURE_KIND_SCREENSHOT,
    CAPTURE_KIND_UNSPECIFIED,
    Capture,
)

Capture.__module__ = "dokime_sdk.client"
Capture.__qualname__ = "Capture"
from dokime_sdk._wire.host.v1.media.captures_pb2 import CaptureKind, CapturesCreateResponse

CapturesCreateResponse.__module__ = "dokime_sdk.client"
CapturesCreateResponse.__qualname__ = "CapturesCreateResponse"
from dokime_sdk._wire.host.v1.media.captures_pb2 import CapturesGetResponse

CapturesGetResponse.__module__ = "dokime_sdk.client"
CapturesGetResponse.__qualname__ = "CapturesGetResponse"
from dokime_sdk._wire.host.v1.media.captures_pb2 import CapturesWatchResponse

from ._exports.client_types import host

CapturesWatchResponse.__module__ = "dokime_sdk.client"
CapturesWatchResponse.__qualname__ = "CapturesWatchResponse"
from dokime_sdk._wire.host.v1.media.captures_pb2 import RecordingAudio

RecordingAudio.__module__ = "dokime_sdk.client"
RecordingAudio.__qualname__ = "RecordingAudio"
from dokime_sdk._wire.host.v1.media.captures_pb2 import RecordingProgress

RecordingProgress.__module__ = "dokime_sdk.client"
RecordingProgress.__qualname__ = "RecordingProgress"
from dokime_sdk._wire.host.v1.plots.settings_pb2 import PlotSettingsValue

PlotSettingsValue.__module__ = "dokime_sdk.client"
PlotSettingsValue.__qualname__ = "PlotSettingsValue"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import ContourGrid

ContourGrid.__module__ = "dokime_sdk.client"
ContourGrid.__qualname__ = "ContourGrid"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import GridCell

GridCell.__module__ = "dokime_sdk.client"
GridCell.__qualname__ = "GridCell"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import GridFrame

GridFrame.__module__ = "dokime_sdk.client"
GridFrame.__qualname__ = "GridFrame"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import GridFrames

GridFrames.__module__ = "dokime_sdk.client"
GridFrames.__qualname__ = "GridFrames"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import NullableNumber

NullableNumber.__module__ = "dokime_sdk.client"
NullableNumber.__qualname__ = "NullableNumber"
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
    PlotAnalysis,
)

PlotAnalysis.__module__ = "dokime_sdk.client"
PlotAnalysis.__qualname__ = "PlotAnalysis"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotAnalysisError

PlotAnalysisError.__module__ = "dokime_sdk.client"
PlotAnalysisError.__qualname__ = "PlotAnalysisError"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotAnalysisResult

PlotAnalysisResult.__module__ = "dokime_sdk.client"
PlotAnalysisResult.__qualname__ = "PlotAnalysisResult"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotBoxSummary

PlotBoxSummary.__module__ = "dokime_sdk.client"
PlotBoxSummary.__qualname__ = "PlotBoxSummary"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotBoxSummaryAnalysis

PlotBoxSummaryAnalysis.__module__ = "dokime_sdk.client"
PlotBoxSummaryAnalysis.__qualname__ = "PlotBoxSummaryAnalysis"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotCartesianMode, PlotColumn

PlotColumn.__module__ = "dokime_sdk.client"
PlotColumn.__qualname__ = "PlotColumn"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotDomain

PlotDomain.__module__ = "dokime_sdk.client"
PlotDomain.__qualname__ = "PlotDomain"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotEnvelope

PlotEnvelope.__module__ = "dokime_sdk.client"
PlotEnvelope.__qualname__ = "PlotEnvelope"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotFrame

PlotFrame.__module__ = "dokime_sdk.client"
PlotFrame.__qualname__ = "PlotFrame"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotHistogramAnalysis

PlotHistogramAnalysis.__module__ = "dokime_sdk.client"
PlotHistogramAnalysis.__qualname__ = "PlotHistogramAnalysis"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotHistogramBin

PlotHistogramBin.__module__ = "dokime_sdk.client"
PlotHistogramBin.__qualname__ = "PlotHistogramBin"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotHistogramSeries

PlotHistogramSeries.__module__ = "dokime_sdk.client"
PlotHistogramSeries.__qualname__ = "PlotHistogramSeries"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotHistogramStatistics

PlotHistogramStatistics.__module__ = "dokime_sdk.client"
PlotHistogramStatistics.__qualname__ = "PlotHistogramStatistics"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotMarker

PlotMarker.__module__ = "dokime_sdk.client"
PlotMarker.__qualname__ = "PlotMarker"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotPayload

PlotPayload.__module__ = "dokime_sdk.client"
PlotPayload.__qualname__ = "PlotPayload"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotPayloadDiagnostics

PlotPayloadDiagnostics.__module__ = "dokime_sdk.client"
PlotPayloadDiagnostics.__qualname__ = "PlotPayloadDiagnostics"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotRequest

PlotRequest.__module__ = "dokime_sdk.client"
PlotRequest.__qualname__ = "PlotRequest"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotSeries

PlotSeries.__module__ = "dokime_sdk.client"
PlotSeries.__qualname__ = "PlotSeries"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotSeriesSummary

PlotSeriesSummary.__module__ = "dokime_sdk.client"
PlotSeriesSummary.__qualname__ = "PlotSeriesSummary"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotSigmaBin

PlotSigmaBin.__module__ = "dokime_sdk.client"
PlotSigmaBin.__qualname__ = "PlotSigmaBin"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotSigmaIntervalAnalysis

PlotSigmaIntervalAnalysis.__module__ = "dokime_sdk.client"
PlotSigmaIntervalAnalysis.__qualname__ = "PlotSigmaIntervalAnalysis"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotSigmaSeries

PlotSigmaSeries.__module__ = "dokime_sdk.client"
PlotSigmaSeries.__qualname__ = "PlotSigmaSeries"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotStreamKind, PlotStreamUpdate

PlotStreamUpdate.__module__ = "dokime_sdk.client"
PlotStreamUpdate.__qualname__ = "PlotStreamUpdate"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import PlotWindow

PlotWindow.__module__ = "dokime_sdk.client"
PlotWindow.__qualname__ = "PlotWindow"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import SnapshotsListResponse

SnapshotsListResponse.__module__ = "dokime_sdk.client"
SnapshotsListResponse.__qualname__ = "SnapshotsListResponse"
from dokime_sdk._wire.host.v1.plots.snapshots_pb2 import SnapshotsWatchResponse

SnapshotsWatchResponse.__module__ = "dokime_sdk.client"
SnapshotsWatchResponse.__qualname__ = "SnapshotsWatchResponse"
from dokime_sdk._wire.host.v1.plugins.secrets_pb2 import SecretRequirement

SecretRequirement.__module__ = "dokime_sdk.client"
SecretRequirement.__qualname__ = "SecretRequirement"
from dokime_sdk._wire.host.v1.prompt_definitions_pb2 import PromptDefinitionsGetResponse

PromptDefinitionsGetResponse.__module__ = "dokime_sdk.client"
PromptDefinitionsGetResponse.__qualname__ = "PromptDefinitionsGetResponse"
from dokime_sdk._wire.host.v1.prompt_definitions_pb2 import PromptDefinitionsListResponse

PromptDefinitionsListResponse.__module__ = "dokime_sdk.client"
PromptDefinitionsListResponse.__qualname__ = "PromptDefinitionsListResponse"
from dokime_sdk._wire.host.v1.prompts.answers_pb2 import AnswersCreateResponse

AnswersCreateResponse.__module__ = "dokime_sdk.client"
AnswersCreateResponse.__qualname__ = "AnswersCreateResponse"
from dokime_sdk._wire.host.v1.prompts_pb2 import (
    PROMPT_KIND_CHOICE,
    PROMPT_KIND_CONFIRMATION,
    PROMPT_KIND_FORM,
    PROMPT_KIND_STATUS,
    PROMPT_KIND_UNSPECIFIED,
    PROMPT_KIND_VERIFICATION,
    PromptInstance,
)

PromptInstance.__module__ = "dokime_sdk.client"
PromptInstance.__qualname__ = "PromptInstance"
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptKind, PromptResponse

PromptResponse.__module__ = "dokime_sdk.client"
PromptResponse.__qualname__ = "PromptResponse"
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptView

PromptView.__module__ = "dokime_sdk.client"
PromptView.__qualname__ = "PromptView"
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptsGetResponse

PromptsGetResponse.__module__ = "dokime_sdk.client"
PromptsGetResponse.__qualname__ = "PromptsGetResponse"
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptsListResponse

PromptsListResponse.__module__ = "dokime_sdk.client"
PromptsListResponse.__qualname__ = "PromptsListResponse"
from dokime_sdk._wire.host.v1.prompts_pb2 import PromptsWatchResponse

PromptsWatchResponse.__module__ = "dokime_sdk.client"
PromptsWatchResponse.__qualname__ = "PromptsWatchResponse"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import AbsoluteTimeFormat

AbsoluteTimeFormat.__module__ = "dokime_sdk.client"
AbsoluteTimeFormat.__qualname__ = "AbsoluteTimeFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import (
    ELAPSED_UNIT_MILLISECONDS,
    ELAPSED_UNIT_MINUTES,
    ELAPSED_UNIT_SECONDS,
    ELAPSED_UNIT_UNSPECIFIED,
    ElapsedFormat,
)

ElapsedFormat.__module__ = "dokime_sdk.client"
ElapsedFormat.__qualname__ = "ElapsedFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import ElapsedUnit, EngineeringFormat

EngineeringFormat.__module__ = "dokime_sdk.client"
EngineeringFormat.__qualname__ = "EngineeringFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import FixedFormat

FixedFormat.__module__ = "dokime_sdk.client"
FixedFormat.__qualname__ = "FixedFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import MeasurementUnit

MeasurementUnit.__module__ = "dokime_sdk.client"
MeasurementUnit.__qualname__ = "MeasurementUnit"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import NormalFormat

NormalFormat.__module__ = "dokime_sdk.client"
NormalFormat.__qualname__ = "NormalFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import (
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
    RecordDisplay,
)

RecordDisplay.__module__ = "dokime_sdk.client"
RecordDisplay.__qualname__ = "RecordDisplay"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import RecordField

RecordField.__module__ = "dokime_sdk.client"
RecordField.__qualname__ = "RecordField"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import RecordIcon, RecordSchema

RecordSchema.__module__ = "dokime_sdk.client"
RecordSchema.__qualname__ = "RecordSchema"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import RecordSchemasGetResponse

RecordSchemasGetResponse.__module__ = "dokime_sdk.client"
RecordSchemasGetResponse.__qualname__ = "RecordSchemasGetResponse"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import RecordSchemasListResponse

RecordSchemasListResponse.__module__ = "dokime_sdk.client"
RecordSchemasListResponse.__qualname__ = "RecordSchemasListResponse"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import RecordTone, ScientificFormat

ScientificFormat.__module__ = "dokime_sdk.client"
ScientificFormat.__qualname__ = "ScientificFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import SiPrefixFormat

SiPrefixFormat.__module__ = "dokime_sdk.client"
SiPrefixFormat.__qualname__ = "SiPrefixFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import (
    TELEMETRY_ROLE_COUNTER,
    TELEMETRY_ROLE_DISTANCE,
    TELEMETRY_ROLE_LATENCY,
    TELEMETRY_ROLE_STATE,
    TELEMETRY_ROLE_THROUGHPUT,
    TELEMETRY_ROLE_UNSPECIFIED,
    TELEMETRY_ROLE_VALUE,
    TelemetryColumn,
)

TelemetryColumn.__module__ = "dokime_sdk.client"
TelemetryColumn.__qualname__ = "TelemetryColumn"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import TelemetryRole, TelemetrySchema

TelemetrySchema.__module__ = "dokime_sdk.client"
TelemetrySchema.__qualname__ = "TelemetrySchema"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import (
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
    UnitFamily,
    ValidationMode,
    ValueFormat,
)

ValueFormat.__module__ = "dokime_sdk.client"
ValueFormat.__qualname__ = "ValueFormat"
from dokime_sdk._wire.host.v1.record_schemas_pb2 import ValueFormatKind

ValueFormatKind.__module__ = "dokime_sdk.client"
ValueFormatKind.__qualname__ = "ValueFormatKind"
from dokime_sdk._wire.host.v1.reports_pb2 import REPORT_ORIGIN_CURRENT, REPORT_ORIGIN_SEALED, REPORT_ORIGIN_UNSPECIFIED, ReportCapture

ReportCapture.__module__ = "dokime_sdk.client"
ReportCapture.__qualname__ = "ReportCapture"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportCase

ReportCase.__module__ = "dokime_sdk.client"
ReportCase.__qualname__ = "ReportCase"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportExecution

ReportExecution.__module__ = "dokime_sdk.client"
ReportExecution.__qualname__ = "ReportExecution"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportFile

ReportFile.__module__ = "dokime_sdk.client"
ReportFile.__qualname__ = "ReportFile"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportMetadata

ReportMetadata.__module__ = "dokime_sdk.client"
ReportMetadata.__qualname__ = "ReportMetadata"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportOrigin, ReportSubject

ReportSubject.__module__ = "dokime_sdk.client"
ReportSubject.__qualname__ = "ReportSubject"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportsCreateResponse

ReportsCreateResponse.__module__ = "dokime_sdk.client"
ReportsCreateResponse.__qualname__ = "ReportsCreateResponse"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportsGetResponse

ReportsGetResponse.__module__ = "dokime_sdk.client"
ReportsGetResponse.__qualname__ = "ReportsGetResponse"
from dokime_sdk._wire.host.v1.reports_pb2 import ReportsPublishResponse

ReportsPublishResponse.__module__ = "dokime_sdk.client"
ReportsPublishResponse.__qualname__ = "ReportsPublishResponse"
from dokime_sdk._wire.host.v1.secrets_pb2 import (
    SECRET_IMPORT_ACTION_REPLACE,
    SECRET_IMPORT_ACTION_SKIP,
    SECRET_IMPORT_ACTION_UNSPECIFIED,
    SecretArchive,
)

SecretArchive.__module__ = "dokime_sdk.client"
SecretArchive.__qualname__ = "SecretArchive"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretImportAction, SecretImportChoice

SecretImportChoice.__module__ = "dokime_sdk.client"
SecretImportChoice.__qualname__ = "SecretImportChoice"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretImportEntry

SecretImportEntry.__module__ = "dokime_sdk.client"
SecretImportEntry.__qualname__ = "SecretImportEntry"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretImportPreview

SecretImportPreview.__module__ = "dokime_sdk.client"
SecretImportPreview.__qualname__ = "SecretImportPreview"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretSnapshot

SecretSnapshot.__module__ = "dokime_sdk.client"
SecretSnapshot.__qualname__ = "SecretSnapshot"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretSummary

SecretSummary.__module__ = "dokime_sdk.client"
SecretSummary.__qualname__ = "SecretSummary"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsDeleteResponse

SecretsDeleteResponse.__module__ = "dokime_sdk.client"
SecretsDeleteResponse.__qualname__ = "SecretsDeleteResponse"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsExportResponse

SecretsExportResponse.__module__ = "dokime_sdk.client"
SecretsExportResponse.__qualname__ = "SecretsExportResponse"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsImportResponse

SecretsImportResponse.__module__ = "dokime_sdk.client"
SecretsImportResponse.__qualname__ = "SecretsImportResponse"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsSetResponse

SecretsSetResponse.__module__ = "dokime_sdk.client"
SecretsSetResponse.__qualname__ = "SecretsSetResponse"
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsWatchResponse

SecretsWatchResponse.__module__ = "dokime_sdk.client"
SecretsWatchResponse.__qualname__ = "SecretsWatchResponse"
from dokime_sdk._wire.host.v1.settings_pb2 import ApplicationAppearance

ApplicationAppearance.__module__ = "dokime_sdk.client"
ApplicationAppearance.__qualname__ = "ApplicationAppearance"
from dokime_sdk._wire.host.v1.settings_pb2 import ApplicationSettings

ApplicationSettings.__module__ = "dokime_sdk.client"
ApplicationSettings.__qualname__ = "ApplicationSettings"
from dokime_sdk._wire.host.v1.settings_pb2 import BootstrapPaths

BootstrapPaths.__module__ = "dokime_sdk.client"
BootstrapPaths.__qualname__ = "BootstrapPaths"
from dokime_sdk._wire.host.v1.settings_pb2 import CachedRefresh

CachedRefresh.__module__ = "dokime_sdk.client"
CachedRefresh.__qualname__ = "CachedRefresh"
from dokime_sdk._wire.host.v1.settings_pb2 import ClientPreferences

ClientPreferences.__module__ = "dokime_sdk.client"
ClientPreferences.__qualname__ = "ClientPreferences"
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
    Density,
    HostCardBlock,
)

HostCardBlock.__module__ = "dokime_sdk.client"
HostCardBlock.__qualname__ = "HostCardBlock"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardBlockKind

HostCardBlockKind.__module__ = "dokime_sdk.client"
HostCardBlockKind.__qualname__ = "HostCardBlockKind"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardBody

HostCardBody.__module__ = "dokime_sdk.client"
HostCardBody.__qualname__ = "HostCardBody"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardKeyValue

HostCardKeyValue.__module__ = "dokime_sdk.client"
HostCardKeyValue.__qualname__ = "HostCardKeyValue"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardKeyValueList

HostCardKeyValueList.__module__ = "dokime_sdk.client"
HostCardKeyValueList.__qualname__ = "HostCardKeyValueList"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardMetric

HostCardMetric.__module__ = "dokime_sdk.client"
HostCardMetric.__qualname__ = "HostCardMetric"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardMetrics

HostCardMetrics.__module__ = "dokime_sdk.client"
HostCardMetrics.__qualname__ = "HostCardMetrics"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardStatus

HostCardStatus.__module__ = "dokime_sdk.client"
HostCardStatus.__qualname__ = "HostCardStatus"
from dokime_sdk._wire.host.v1.settings_pb2 import HostCardTone, ManualRefresh

ManualRefresh.__module__ = "dokime_sdk.client"
ManualRefresh.__qualname__ = "ManualRefresh"
from dokime_sdk._wire.host.v1.settings_pb2 import OnDemandRefresh

OnDemandRefresh.__module__ = "dokime_sdk.client"
OnDemandRefresh.__qualname__ = "OnDemandRefresh"
from dokime_sdk._wire.host.v1.settings_pb2 import RemoteListenerSettings

RemoteListenerSettings.__module__ = "dokime_sdk.client"
RemoteListenerSettings.__qualname__ = "RemoteListenerSettings"
from dokime_sdk._wire.host.v1.settings_pb2 import ReportDefaults

ReportDefaults.__module__ = "dokime_sdk.client"
ReportDefaults.__qualname__ = "ReportDefaults"
from dokime_sdk._wire.host.v1.settings_pb2 import (
    SETTINGS_CARD_STATE_DISABLED,
    SETTINGS_CARD_STATE_DISCONNECTED,
    SETTINGS_CARD_STATE_ERROR,
    SETTINGS_CARD_STATE_PERMISSION_DENIED,
    SETTINGS_CARD_STATE_READY,
    SETTINGS_CARD_STATE_STALE,
    SETTINGS_CARD_STATE_UNSPECIFIED,
    SettingsCard,
)

SettingsCard.__module__ = "dokime_sdk.client"
SettingsCard.__qualname__ = "SettingsCard"
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsCardRefreshPolicy

SettingsCardRefreshPolicy.__module__ = "dokime_sdk.client"
SettingsCardRefreshPolicy.__qualname__ = "SettingsCardRefreshPolicy"
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsCardRefreshPolicyPolicy

SettingsCardRefreshPolicyPolicy.__module__ = "dokime_sdk.client"
SettingsCardRefreshPolicyPolicy.__qualname__ = "SettingsCardRefreshPolicyPolicy"
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsCardState, SettingsFrame

SettingsFrame.__module__ = "dokime_sdk.client"
SettingsFrame.__qualname__ = "SettingsFrame"
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsSchemaResponse

SettingsSchemaResponse.__module__ = "dokime_sdk.client"
SettingsSchemaResponse.__qualname__ = "SettingsSchemaResponse"
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsSection

SettingsSection.__module__ = "dokime_sdk.client"
SettingsSection.__qualname__ = "SettingsSection"
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsSectionOwner

SettingsSectionOwner.__module__ = "dokime_sdk.client"
SettingsSectionOwner.__qualname__ = "SettingsSectionOwner"
from dokime_sdk._wire.host.v1.settings_pb2 import (
    THEME_MODE_DARK,
    THEME_MODE_LIGHT,
    THEME_MODE_SYSTEM,
    THEME_MODE_UNSPECIFIED,
    TelemetryPreferences,
)

TelemetryPreferences.__module__ = "dokime_sdk.client"
TelemetryPreferences.__qualname__ = "TelemetryPreferences"
from dokime_sdk._wire.host.v1.settings_pb2 import ThemeMode
from dokime_sdk._wire.host.v1.storage.sources_pb2 import SourcesGetResponse

SourcesGetResponse.__module__ = "dokime_sdk.client"
SourcesGetResponse.__qualname__ = "SourcesGetResponse"
from dokime_sdk._wire.host.v1.storage.sources_pb2 import SourcesSetResponse

SourcesSetResponse.__module__ = "dokime_sdk.client"
SourcesSetResponse.__qualname__ = "SourcesSetResponse"
from dokime_sdk._wire.host.v1.storage.sources_pb2 import StorageSource

StorageSource.__module__ = "dokime_sdk.client"
StorageSource.__qualname__ = "StorageSource"
from dokime_sdk._wire.host.v1.storage.sources_pb2 import StorageSources

StorageSources.__module__ = "dokime_sdk.client"
StorageSources.__qualname__ = "StorageSources"
from dokime_sdk._wire.host.v1.stream_schemas_pb2 import StreamSchemasGetResponse

StreamSchemasGetResponse.__module__ = "dokime_sdk.client"
StreamSchemasGetResponse.__qualname__ = "StreamSchemasGetResponse"
from dokime_sdk._wire.host.v1.stream_schemas_pb2 import StreamSchemasListResponse

StreamSchemasListResponse.__module__ = "dokime_sdk.client"
StreamSchemasListResponse.__qualname__ = "StreamSchemasListResponse"
from dokime_sdk._wire.host.v1.suites_pb2 import SuitesGetResponse

SuitesGetResponse.__module__ = "dokime_sdk.client"
SuitesGetResponse.__qualname__ = "SuitesGetResponse"
from dokime_sdk._wire.host.v1.suites_pb2 import SuitesListResponse

SuitesListResponse.__module__ = "dokime_sdk.client"
SuitesListResponse.__qualname__ = "SuitesListResponse"
from harmos._wire.host.v1.access.certificates_pb2 import CertificatesCreateResponse
from harmos._wire.host.v1.access.listener_pb2 import ListenerGetResponse, ListenerSetResponse, ListenerValue
from harmos._wire.host.v1.access.pairings_pb2 import Pairing, PairingsCreateResponse, PairingsDeleteResponse, PairingsListResponse
from harmos._wire.host.v1.invocations_pb2 import InvocationsCreateResponse
from harmos._wire.host.v1.plugins_pb2 import (
    PLUGIN_STATE_CONNECTING,
    PLUGIN_STATE_READY,
    PLUGIN_STATE_STOPPED,
    PLUGIN_STATE_STOPPING,
    PLUGIN_STATE_UNAVAILABLE,
    PLUGIN_STATE_UNSPECIFIED,
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
)
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
    StatusGetResponse,
    StatusWatchResponse,
)
from harmos._wire.host.v1.streams_pb2 import StreamIdentity, StreamsListResponse

from ._exports.client_types import harmos

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
    "PluginsSetResponse",
    "PluginsWatchResponse",
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
    "StatusGetResponse",
    "StatusWatchResponse",
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
