# @generated from protocol descriptors; do not edit.
from dokime._wire.host.v1.aggregates_pb2 import AggregateReply

AggregateReply.__module__ = "dokime.client"
AggregateReply.__qualname__ = "AggregateReply"
from dokime._wire.host.v1.aggregates_pb2 import AggregatesGetResponse

AggregatesGetResponse.__module__ = "dokime.client"
AggregatesGetResponse.__qualname__ = "AggregatesGetResponse"
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
    QueryAggregate,
)

QueryAggregate.__module__ = "dokime.client"
QueryAggregate.__qualname__ = "QueryAggregate"
from dokime._wire.host.v1.aggregates_pb2 import QueryFilter

QueryFilter.__module__ = "dokime.client"
QueryFilter.__qualname__ = "QueryFilter"
from dokime._wire.host.v1.aggregates_pb2 import QueryFilterOperator, QueryRow

QueryRow.__module__ = "dokime.client"
QueryRow.__qualname__ = "QueryRow"
from dokime._wire.host.v1.aggregates_pb2 import QuerySource
from dokime._wire.host.v1.backend.overview_pb2 import EvidenceCounts

EvidenceCounts.__module__ = "dokime.client"
EvidenceCounts.__qualname__ = "EvidenceCounts"
from dokime._wire.host.v1.backend.overview_pb2 import OverviewGetResponse

OverviewGetResponse.__module__ = "dokime.client"
OverviewGetResponse.__qualname__ = "OverviewGetResponse"
from dokime._wire.host.v1.backend.overview_pb2 import OverviewWatchResponse

OverviewWatchResponse.__module__ = "dokime.client"
OverviewWatchResponse.__qualname__ = "OverviewWatchResponse"
from dokime._wire.host.v1.backend.overview_pb2 import RuntimeShellSnapshot

RuntimeShellSnapshot.__module__ = "dokime.client"
RuntimeShellSnapshot.__qualname__ = "RuntimeShellSnapshot"
from dokime._wire.host.v1.backend_pb2 import (
    APPEARANCE_FAILURE_KIND_ASSET_CHANGED,
    APPEARANCE_FAILURE_KIND_ASSET_MISSING,
    APPEARANCE_FAILURE_KIND_ASSET_UNAVAILABLE,
    APPEARANCE_FAILURE_KIND_INVALID_REFERENCE,
    APPEARANCE_FAILURE_KIND_TOO_LARGE,
    APPEARANCE_FAILURE_KIND_UNSPECIFIED,
    AppearanceAssetMetadata,
)

AppearanceAssetMetadata.__module__ = "dokime.client"
AppearanceAssetMetadata.__qualname__ = "AppearanceAssetMetadata"
from dokime._wire.host.v1.backend_pb2 import AppearanceAssetReference

AppearanceAssetReference.__module__ = "dokime.client"
AppearanceAssetReference.__qualname__ = "AppearanceAssetReference"
from dokime._wire.host.v1.backend_pb2 import AppearanceFailure

AppearanceFailure.__module__ = "dokime.client"
AppearanceFailure.__qualname__ = "AppearanceFailure"
from dokime._wire.host.v1.backend_pb2 import AppearanceFailureKind, AppearanceLogo

AppearanceLogo.__module__ = "dokime.client"
AppearanceLogo.__qualname__ = "AppearanceLogo"
from dokime._wire.host.v1.backend_pb2 import AppearanceSnapshot

AppearanceSnapshot.__module__ = "dokime.client"
AppearanceSnapshot.__qualname__ = "AppearanceSnapshot"
from dokime._wire.host.v1.backend_pb2 import BackendAppearanceResponse

BackendAppearanceResponse.__module__ = "dokime.client"
BackendAppearanceResponse.__qualname__ = "BackendAppearanceResponse"
from dokime._wire.host.v1.backend_pb2 import (
    LOGO_VARIANT_CONTAINED_ON_DARK,
    LOGO_VARIANT_CONTAINED_ON_LIGHT,
    LOGO_VARIANT_TRANSPARENT_ON_DARK,
    LOGO_VARIANT_TRANSPARENT_ON_LIGHT,
    LOGO_VARIANT_UNSPECIFIED,
    LogoVariant,
    OrganizationIdentity,
)

OrganizationIdentity.__module__ = "dokime.client"
OrganizationIdentity.__qualname__ = "OrganizationIdentity"
from dokime._wire.host.v1.backend_pb2 import (
    THEME_MODE_POLICY_FORCE_DARK,
    THEME_MODE_POLICY_FORCE_LIGHT,
    THEME_MODE_POLICY_FORCE_SYSTEM,
    THEME_MODE_POLICY_UNSPECIFIED,
    THEME_MODE_POLICY_USER_OR_SYSTEM,
    ThemeCornerRadii,
)

ThemeCornerRadii.__module__ = "dokime.client"
ThemeCornerRadii.__qualname__ = "ThemeCornerRadii"
from dokime._wire.host.v1.backend_pb2 import ThemeFamily

ThemeFamily.__module__ = "dokime.client"
ThemeFamily.__qualname__ = "ThemeFamily"
from dokime._wire.host.v1.backend_pb2 import ThemeModePolicy, ThemeVariant

ThemeVariant.__module__ = "dokime.client"
ThemeVariant.__qualname__ = "ThemeVariant"
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
)

CaseActions.__module__ = "dokime.client"
CaseActions.__qualname__ = "CaseActions"
from dokime._wire.host.v1.cases_pb2 import CaseDetail

CaseDetail.__module__ = "dokime.client"
CaseDetail.__qualname__ = "CaseDetail"
from dokime._wire.host.v1.cases_pb2 import CaseParameter

CaseParameter.__module__ = "dokime.client"
CaseParameter.__qualname__ = "CaseParameter"
from dokime._wire.host.v1.cases_pb2 import CaseStatus, CaseSummary

CaseSummary.__module__ = "dokime.client"
CaseSummary.__qualname__ = "CaseSummary"
from dokime._wire.host.v1.cases_pb2 import CasesCreateResponse

CasesCreateResponse.__module__ = "dokime.client"
CasesCreateResponse.__qualname__ = "CasesCreateResponse"
from dokime._wire.host.v1.cases_pb2 import CasesDeleteResponse

CasesDeleteResponse.__module__ = "dokime.client"
CasesDeleteResponse.__qualname__ = "CasesDeleteResponse"
from dokime._wire.host.v1.cases_pb2 import CasesGetResponse

CasesGetResponse.__module__ = "dokime.client"
CasesGetResponse.__qualname__ = "CasesGetResponse"
from dokime._wire.host.v1.cases_pb2 import CasesListResponse

CasesListResponse.__module__ = "dokime.client"
CasesListResponse.__qualname__ = "CasesListResponse"
from dokime._wire.host.v1.cases_pb2 import CasesParametersResponse

CasesParametersResponse.__module__ = "dokime.client"
CasesParametersResponse.__qualname__ = "CasesParametersResponse"
from dokime._wire.host.v1.cases_pb2 import CasesWatchResponse

CasesWatchResponse.__module__ = "dokime.client"
CasesWatchResponse.__qualname__ = "CasesWatchResponse"
from dokime._wire.host.v1.cases_pb2 import PlannedCase

PlannedCase.__module__ = "dokime.client"
PlannedCase.__qualname__ = "PlannedCase"
from dokime._wire.host.v1.cases_pb2 import RegisteredSelection

RegisteredSelection.__module__ = "dokime.client"
RegisteredSelection.__qualname__ = "RegisteredSelection"
from dokime._wire.host.v1.cases_pb2 import RegisteredSelectionFilesEntry

RegisteredSelectionFilesEntry.__module__ = "dokime.client"
RegisteredSelectionFilesEntry.__qualname__ = "RegisteredSelectionFilesEntry"
from dokime._wire.host.v1.cases_pb2 import RegisteredSelectionParametersEntry

RegisteredSelectionParametersEntry.__module__ = "dokime.client"
RegisteredSelectionParametersEntry.__qualname__ = "RegisteredSelectionParametersEntry"
from dokime._wire.host.v1.cases_pb2 import SelectedFile

SelectedFile.__module__ = "dokime.client"
SelectedFile.__qualname__ = "SelectedFile"
from dokime._wire.host.v1.cases_pb2 import SelectedFileKind

SelectedFileKind.__module__ = "dokime.client"
SelectedFileKind.__qualname__ = "SelectedFileKind"
from dokime._wire.host.v1.cases_pb2 import SelectionPlan

SelectionPlan.__module__ = "dokime.client"
SelectionPlan.__qualname__ = "SelectionPlan"
from dokime._wire.host.v1.cases_pb2 import SelectionValues

SelectionValues.__module__ = "dokime.client"
SelectionValues.__qualname__ = "SelectionValues"
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsGetResponse

ArtifactsGetResponse.__module__ = "dokime.client"
ArtifactsGetResponse.__qualname__ = "ArtifactsGetResponse"
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsListResponse

ArtifactsListResponse.__module__ = "dokime.client"
ArtifactsListResponse.__qualname__ = "ArtifactsListResponse"
from dokime._wire.host.v1.content.artifacts_pb2 import ArtifactsWatchResponse

ArtifactsWatchResponse.__module__ = "dokime.client"
ArtifactsWatchResponse.__qualname__ = "ArtifactsWatchResponse"
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
    ExecutionFileKind,
    ExecutionFileMetadata,
)

ExecutionFileMetadata.__module__ = "dokime.client"
ExecutionFileMetadata.__qualname__ = "ExecutionFileMetadata"
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesGetResponse, ExecutionFileState

ExecutionFilesGetResponse.__module__ = "dokime.client"
ExecutionFilesGetResponse.__qualname__ = "ExecutionFilesGetResponse"
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesListResponse

ExecutionFilesListResponse.__module__ = "dokime.client"
ExecutionFilesListResponse.__qualname__ = "ExecutionFilesListResponse"
from dokime._wire.host.v1.content.execution_files_pb2 import ExecutionFilesWatchResponse

ExecutionFilesWatchResponse.__module__ = "dokime.client"
ExecutionFilesWatchResponse.__qualname__ = "ExecutionFilesWatchResponse"
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
    ArchiveScope,
    CycleFilter,
)

CycleFilter.__module__ = "dokime.client"
CycleFilter.__qualname__ = "CycleFilter"
from dokime._wire.host.v1.cycles_pb2 import CycleOrder

CycleOrder.__module__ = "dokime.client"
CycleOrder.__qualname__ = "CycleOrder"
from dokime._wire.host.v1.cycles_pb2 import CycleOrderKey, CyclePhase, CycleSelection

CycleSelection.__module__ = "dokime.client"
CycleSelection.__qualname__ = "CycleSelection"
from dokime._wire.host.v1.cycles_pb2 import CycleStatus, CycleSummary

CycleSummary.__module__ = "dokime.client"
CycleSummary.__qualname__ = "CycleSummary"
from dokime._wire.host.v1.cycles_pb2 import CycleWritable

CycleWritable.__module__ = "dokime.client"
CycleWritable.__qualname__ = "CycleWritable"
from dokime._wire.host.v1.cycles_pb2 import CyclesCreateResponse

CyclesCreateResponse.__module__ = "dokime.client"
CyclesCreateResponse.__qualname__ = "CyclesCreateResponse"
from dokime._wire.host.v1.cycles_pb2 import CyclesGetResponse

CyclesGetResponse.__module__ = "dokime.client"
CyclesGetResponse.__qualname__ = "CyclesGetResponse"
from dokime._wire.host.v1.cycles_pb2 import CyclesListResponse

CyclesListResponse.__module__ = "dokime.client"
CyclesListResponse.__qualname__ = "CyclesListResponse"
from dokime._wire.host.v1.cycles_pb2 import CyclesSealResponse

CyclesSealResponse.__module__ = "dokime.client"
CyclesSealResponse.__qualname__ = "CyclesSealResponse"
from dokime._wire.host.v1.cycles_pb2 import CyclesSetResponse

CyclesSetResponse.__module__ = "dokime.client"
CyclesSetResponse.__qualname__ = "CyclesSetResponse"
from dokime._wire.host.v1.cycles_pb2 import CyclesWatchResponse

CyclesWatchResponse.__module__ = "dokime.client"
CyclesWatchResponse.__qualname__ = "CyclesWatchResponse"
from dokime._wire.host.v1.cycles_pb2 import (
    RESULT_STATUS_BLOCKED,
    RESULT_STATUS_CANCELLED,
    RESULT_STATUS_FAILED,
    RESULT_STATUS_PASSED,
    RESULT_STATUS_PENDING,
    RESULT_STATUS_UNSPECIFIED,
    RerunCaseNotice,
)

RerunCaseNotice.__module__ = "dokime.client"
RerunCaseNotice.__qualname__ = "RerunCaseNotice"
from dokime._wire.host.v1.cycles_pb2 import ResultStatus, ResultSummary

ResultSummary.__module__ = "dokime.client"
ResultSummary.__qualname__ = "ResultSummary"
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
    DocumentBinding,
)

DocumentBinding.__module__ = "dokime.client"
DocumentBinding.__qualname__ = "DocumentBinding"
from dokime._wire.host.v1.documents_pb2 import DocumentBindingResult

DocumentBindingResult.__module__ = "dokime.client"
DocumentBindingResult.__qualname__ = "DocumentBindingResult"
from dokime._wire.host.v1.documents_pb2 import DocumentCard

DocumentCard.__module__ = "dokime.client"
DocumentCard.__qualname__ = "DocumentCard"
from dokime._wire.host.v1.documents_pb2 import DocumentCardBody

DocumentCardBody.__module__ = "dokime.client"
DocumentCardBody.__qualname__ = "DocumentCardBody"
from dokime._wire.host.v1.documents_pb2 import DocumentCardBudget

DocumentCardBudget.__module__ = "dokime.client"
DocumentCardBudget.__qualname__ = "DocumentCardBudget"
from dokime._wire.host.v1.documents_pb2 import DocumentCardFailure

DocumentCardFailure.__module__ = "dokime.client"
DocumentCardFailure.__qualname__ = "DocumentCardFailure"
from dokime._wire.host.v1.documents_pb2 import DocumentCardKind, DocumentCompletion

DocumentCompletion.__module__ = "dokime.client"
DocumentCompletion.__qualname__ = "DocumentCompletion"
from dokime._wire.host.v1.documents_pb2 import DocumentDensity, DocumentEvidenceCut

DocumentEvidenceCut.__module__ = "dokime.client"
DocumentEvidenceCut.__qualname__ = "DocumentEvidenceCut"
from dokime._wire.host.v1.documents_pb2 import DocumentFrame

DocumentFrame.__module__ = "dokime.client"
DocumentFrame.__qualname__ = "DocumentFrame"
from dokime._wire.host.v1.documents_pb2 import DocumentFrameFrame

DocumentFrameFrame.__module__ = "dokime.client"
DocumentFrameFrame.__qualname__ = "DocumentFrameFrame"
from dokime._wire.host.v1.documents_pb2 import DocumentFrontiers

DocumentFrontiers.__module__ = "dokime.client"
DocumentFrontiers.__qualname__ = "DocumentFrontiers"
from dokime._wire.host.v1.documents_pb2 import DocumentHeader

DocumentHeader.__module__ = "dokime.client"
DocumentHeader.__qualname__ = "DocumentHeader"
from dokime._wire.host.v1.documents_pb2 import DocumentLayoutNode

DocumentLayoutNode.__module__ = "dokime.client"
DocumentLayoutNode.__qualname__ = "DocumentLayoutNode"
from dokime._wire.host.v1.documents_pb2 import DocumentLayoutNodeNode

DocumentLayoutNodeNode.__module__ = "dokime.client"
DocumentLayoutNodeNode.__qualname__ = "DocumentLayoutNodeNode"
from dokime._wire.host.v1.documents_pb2 import DocumentOwnerReference

DocumentOwnerReference.__module__ = "dokime.client"
DocumentOwnerReference.__qualname__ = "DocumentOwnerReference"
from dokime._wire.host.v1.documents_pb2 import DocumentPage

DocumentPage.__module__ = "dokime.client"
DocumentPage.__qualname__ = "DocumentPage"
from dokime._wire.host.v1.documents_pb2 import DocumentPlacement

DocumentPlacement.__module__ = "dokime.client"
DocumentPlacement.__qualname__ = "DocumentPlacement"
from dokime._wire.host.v1.documents_pb2 import DocumentPlotDefinition

DocumentPlotDefinition.__module__ = "dokime.client"
DocumentPlotDefinition.__qualname__ = "DocumentPlotDefinition"
from dokime._wire.host.v1.documents_pb2 import DocumentRail, DocumentRepeat

DocumentRepeat.__module__ = "dokime.client"
DocumentRepeat.__qualname__ = "DocumentRepeat"
from dokime._wire.host.v1.documents_pb2 import DocumentRepeatInstance

DocumentRepeatInstance.__module__ = "dokime.client"
DocumentRepeatInstance.__qualname__ = "DocumentRepeatInstance"
from dokime._wire.host.v1.documents_pb2 import DocumentRepeatSelection

DocumentRepeatSelection.__module__ = "dokime.client"
DocumentRepeatSelection.__qualname__ = "DocumentRepeatSelection"
from dokime._wire.host.v1.documents_pb2 import DocumentScope

DocumentScope.__module__ = "dokime.client"
DocumentScope.__qualname__ = "DocumentScope"
from dokime._wire.host.v1.documents_pb2 import DocumentSection

DocumentSection.__module__ = "dokime.client"
DocumentSection.__qualname__ = "DocumentSection"
from dokime._wire.host.v1.documents_pb2 import DocumentSectionIdentity

DocumentSectionIdentity.__module__ = "dokime.client"
DocumentSectionIdentity.__qualname__ = "DocumentSectionIdentity"
from dokime._wire.host.v1.documents_pb2 import DocumentSpan

DocumentSpan.__module__ = "dokime.client"
DocumentSpan.__qualname__ = "DocumentSpan"
from dokime._wire.host.v1.documents_pb2 import DocumentSummary

DocumentSummary.__module__ = "dokime.client"
DocumentSummary.__qualname__ = "DocumentSummary"
from dokime._wire.host.v1.documents_pb2 import DocumentSummaryCondition

DocumentSummaryCondition.__module__ = "dokime.client"
DocumentSummaryCondition.__qualname__ = "DocumentSummaryCondition"
from dokime._wire.host.v1.documents_pb2 import DocumentSummaryFormat, DocumentSummaryGroup

DocumentSummaryGroup.__module__ = "dokime.client"
DocumentSummaryGroup.__qualname__ = "DocumentSummaryGroup"
from dokime._wire.host.v1.documents_pb2 import DocumentSummaryGroupDefinition

DocumentSummaryGroupDefinition.__module__ = "dokime.client"
DocumentSummaryGroupDefinition.__qualname__ = "DocumentSummaryGroupDefinition"
from dokime._wire.host.v1.documents_pb2 import DocumentSummaryPresentation, DocumentSummaryStat, DocumentSummaryState, DocumentSummaryTile

DocumentSummaryTile.__module__ = "dokime.client"
DocumentSummaryTile.__qualname__ = "DocumentSummaryTile"
from dokime._wire.host.v1.documents_pb2 import DocumentSummaryTileDefinition

DocumentSummaryTileDefinition.__module__ = "dokime.client"
DocumentSummaryTileDefinition.__qualname__ = "DocumentSummaryTileDefinition"
from dokime._wire.host.v1.documents_pb2 import DocumentTable

DocumentTable.__module__ = "dokime.client"
DocumentTable.__qualname__ = "DocumentTable"
from dokime._wire.host.v1.documents_pb2 import DocumentTableRow

DocumentTableRow.__module__ = "dokime.client"
DocumentTableRow.__qualname__ = "DocumentTableRow"
from dokime._wire.host.v1.documents_pb2 import DocumentTelemetryCut

DocumentTelemetryCut.__module__ = "dokime.client"
DocumentTelemetryCut.__qualname__ = "DocumentTelemetryCut"
from dokime._wire.host.v1.documents_pb2 import DocumentWarning

DocumentWarning.__module__ = "dokime.client"
DocumentWarning.__qualname__ = "DocumentWarning"
from dokime._wire.host.v1.documents_pb2 import DocumentsGetResponse

DocumentsGetResponse.__module__ = "dokime.client"
DocumentsGetResponse.__qualname__ = "DocumentsGetResponse"
from dokime._wire.host.v1.documents_pb2 import DocumentsWatchResponse

DocumentsWatchResponse.__module__ = "dokime.client"
DocumentsWatchResponse.__qualname__ = "DocumentsWatchResponse"
from dokime._wire.host.v1.documents_pb2 import (
    PLOT_AXIS_KIND_ELAPSED,
    PLOT_AXIS_KIND_UNSPECIFIED,
    PLOT_AXIS_KIND_WALL_TIME,
    PlotAxisKind,
    PlotPresentation,
)

PlotPresentation.__module__ = "dokime.client"
PlotPresentation.__qualname__ = "PlotPresentation"
from dokime._wire.host.v1.documents_pb2 import PlotReferenceBand

PlotReferenceBand.__module__ = "dokime.client"
PlotReferenceBand.__qualname__ = "PlotReferenceBand"
from dokime._wire.host.v1.documents_pb2 import PlotReferenceLine

PlotReferenceLine.__module__ = "dokime.client"
PlotReferenceLine.__qualname__ = "PlotReferenceLine"
from dokime._wire.host.v1.documents_pb2 import PlotReferenceValue

PlotReferenceValue.__module__ = "dokime.client"
PlotReferenceValue.__qualname__ = "PlotReferenceValue"
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsListResponse

HealthEventsListResponse.__module__ = "dokime.client"
HealthEventsListResponse.__qualname__ = "HealthEventsListResponse"
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthEventsWatchResponse

HealthEventsWatchResponse.__module__ = "dokime.client"
HealthEventsWatchResponse.__qualname__ = "HealthEventsWatchResponse"
from dokime._wire.host.v1.evidence.health_events_pb2 import HealthItem

HealthItem.__module__ = "dokime.client"
HealthItem.__qualname__ = "HealthItem"
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationOwner

AnnotationOwner.__module__ = "dokime.client"
AnnotationOwner.__qualname__ = "AnnotationOwner"
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationOwnerOwner

AnnotationOwnerOwner.__module__ = "dokime.client"
AnnotationOwnerOwner.__qualname__ = "AnnotationOwnerOwner"
from dokime._wire.host.v1.evidence.notes_pb2 import AnnotationSelection

AnnotationSelection.__module__ = "dokime.client"
AnnotationSelection.__qualname__ = "AnnotationSelection"
from dokime._wire.host.v1.evidence.notes_pb2 import ExecutionNoteOwner

ExecutionNoteOwner.__module__ = "dokime.client"
ExecutionNoteOwner.__qualname__ = "ExecutionNoteOwner"
from dokime._wire.host.v1.evidence.notes_pb2 import NoteWritable

NoteWritable.__module__ = "dokime.client"
NoteWritable.__qualname__ = "NoteWritable"
from dokime._wire.host.v1.evidence.notes_pb2 import NotesCreateResponse

NotesCreateResponse.__module__ = "dokime.client"
NotesCreateResponse.__qualname__ = "NotesCreateResponse"
from dokime._wire.host.v1.evidence.notes_pb2 import NotesGetResponse

NotesGetResponse.__module__ = "dokime.client"
NotesGetResponse.__qualname__ = "NotesGetResponse"
from dokime._wire.host.v1.evidence.notes_pb2 import NotesListResponse

NotesListResponse.__module__ = "dokime.client"
NotesListResponse.__qualname__ = "NotesListResponse"
from dokime._wire.host.v1.evidence.notes_pb2 import NotesSetResponse

NotesSetResponse.__module__ = "dokime.client"
NotesSetResponse.__qualname__ = "NotesSetResponse"
from dokime._wire.host.v1.evidence.notes_pb2 import NotesWatchResponse

NotesWatchResponse.__module__ = "dokime.client"
NotesWatchResponse.__qualname__ = "NotesWatchResponse"
from dokime._wire.host.v1.evidence.notes_pb2 import OperatorNote

OperatorNote.__module__ = "dokime.client"
OperatorNote.__qualname__ = "OperatorNote"
from dokime._wire.host.v1.evidence.notes_pb2 import PlannedCaseNoteOwner

PlannedCaseNoteOwner.__module__ = "dokime.client"
PlannedCaseNoteOwner.__qualname__ = "PlannedCaseNoteOwner"
from dokime._wire.host.v1.evidence.records_pb2 import RecordsWatchResponse

RecordsWatchResponse.__module__ = "dokime.client"
RecordsWatchResponse.__qualname__ = "RecordsWatchResponse"
from dokime._wire.host.v1.evidence.samples_pb2 import Bucket

Bucket.__module__ = "dokime.client"
Bucket.__qualname__ = "Bucket"
from dokime._wire.host.v1.evidence.samples_pb2 import BucketRequest

BucketRequest.__module__ = "dokime.client"
BucketRequest.__qualname__ = "BucketRequest"
from dokime._wire.host.v1.evidence.samples_pb2 import BucketValue

BucketValue.__module__ = "dokime.client"
BucketValue.__qualname__ = "BucketValue"
from dokime._wire.host.v1.evidence.samples_pb2 import SampleRow

SampleRow.__module__ = "dokime.client"
SampleRow.__qualname__ = "SampleRow"
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesListResponse

SamplesListResponse.__module__ = "dokime.client"
SamplesListResponse.__qualname__ = "SamplesListResponse"
from dokime._wire.host.v1.evidence.samples_pb2 import SamplesWatchResponse

SamplesWatchResponse.__module__ = "dokime.client"
SamplesWatchResponse.__qualname__ = "SamplesWatchResponse"
from dokime._wire.host.v1.evidence.samples_pb2 import StatisticValue

StatisticValue.__module__ = "dokime.client"
StatisticValue.__qualname__ = "StatisticValue"
from dokime._wire.host.v1.evidence.samples_pb2 import TelemetrySample

TelemetrySample.__module__ = "dokime.client"
TelemetrySample.__qualname__ = "TelemetrySample"
from dokime._wire.host.v1.evidence.traceability_pb2 import TraceabilityWatchResponse

TraceabilityWatchResponse.__module__ = "dokime.client"
TraceabilityWatchResponse.__qualname__ = "TraceabilityWatchResponse"
from dokime._wire.host.v1.evidence_pb2 import EvidenceArchiveResponse

EvidenceArchiveResponse.__module__ = "dokime.client"
EvidenceArchiveResponse.__qualname__ = "EvidenceArchiveResponse"
from dokime._wire.host.v1.evidence_pb2 import EvidenceSubject

EvidenceSubject.__module__ = "dokime.client"
EvidenceSubject.__qualname__ = "EvidenceSubject"
from dokime._wire.host.v1.executions.commands_pb2 import CommandsCreateResponse

CommandsCreateResponse.__module__ = "dokime.client"
CommandsCreateResponse.__qualname__ = "CommandsCreateResponse"
from dokime._wire.host.v1.executions.commands_pb2 import CommandsGetResponse

CommandsGetResponse.__module__ = "dokime.client"
CommandsGetResponse.__qualname__ = "CommandsGetResponse"
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
    ExecutionCommandAction,
    ExecutionCommandPhase,
    ExecutionCommandReceipt,
)

ExecutionCommandReceipt.__module__ = "dokime.client"
ExecutionCommandReceipt.__qualname__ = "ExecutionCommandReceipt"
from dokime._wire.host.v1.executions.progress_pb2 import ProgressWatchResponse

ProgressWatchResponse.__module__ = "dokime.client"
ProgressWatchResponse.__qualname__ = "ProgressWatchResponse"
from dokime._wire.host.v1.executions.schemas_pb2 import SchemasGetResponse

SchemasGetResponse.__module__ = "dokime.client"
SchemasGetResponse.__qualname__ = "SchemasGetResponse"
from dokime._wire.host.v1.executions_pb2 import (
    CRITERION_STATUS_FAILED,
    CRITERION_STATUS_INDETERMINATE,
    CRITERION_STATUS_PASSED,
    CRITERION_STATUS_PENDING,
    CRITERION_STATUS_SKIPPED,
    CRITERION_STATUS_UNSPECIFIED,
    CriterionStatus,
    DeclaredOutcome,
)

DeclaredOutcome.__module__ = "dokime.client"
DeclaredOutcome.__qualname__ = "DeclaredOutcome"
from dokime._wire.host.v1.executions_pb2 import Diagnostic

Diagnostic.__module__ = "dokime.client"
Diagnostic.__qualname__ = "Diagnostic"
from dokime._wire.host.v1.executions_pb2 import (
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

EvaluatedOutcome.__module__ = "dokime.client"
EvaluatedOutcome.__qualname__ = "EvaluatedOutcome"
from dokime._wire.host.v1.executions_pb2 import ExecutionBinding

ExecutionBinding.__module__ = "dokime.client"
ExecutionBinding.__qualname__ = "ExecutionBinding"
from dokime._wire.host.v1.executions_pb2 import ExecutionControls

ExecutionControls.__module__ = "dokime.client"
ExecutionControls.__qualname__ = "ExecutionControls"
from dokime._wire.host.v1.executions_pb2 import ExecutionDetail

ExecutionDetail.__module__ = "dokime.client"
ExecutionDetail.__qualname__ = "ExecutionDetail"
from dokime._wire.host.v1.executions_pb2 import ExecutionFilter

ExecutionFilter.__module__ = "dokime.client"
ExecutionFilter.__qualname__ = "ExecutionFilter"
from dokime._wire.host.v1.executions_pb2 import ExecutionMode, ExecutionOrder

ExecutionOrder.__module__ = "dokime.client"
ExecutionOrder.__qualname__ = "ExecutionOrder"
from dokime._wire.host.v1.executions_pb2 import ExecutionOrderKey, ExecutionOutcome

ExecutionOutcome.__module__ = "dokime.client"
ExecutionOutcome.__qualname__ = "ExecutionOutcome"
from dokime._wire.host.v1.executions_pb2 import ExecutionOutcomeCounts

ExecutionOutcomeCounts.__module__ = "dokime.client"
ExecutionOutcomeCounts.__qualname__ = "ExecutionOutcomeCounts"
from dokime._wire.host.v1.executions_pb2 import ExecutionParameterSet, ExecutionSelection

ExecutionSelection.__module__ = "dokime.client"
ExecutionSelection.__qualname__ = "ExecutionSelection"
from dokime._wire.host.v1.executions_pb2 import ExecutionState, ExecutionStep

ExecutionStep.__module__ = "dokime.client"
ExecutionStep.__qualname__ = "ExecutionStep"
from dokime._wire.host.v1.executions_pb2 import ExecutionSummary

ExecutionSummary.__module__ = "dokime.client"
ExecutionSummary.__qualname__ = "ExecutionSummary"
from dokime._wire.host.v1.executions_pb2 import ExecutionWritable

ExecutionWritable.__module__ = "dokime.client"
ExecutionWritable.__qualname__ = "ExecutionWritable"
from dokime._wire.host.v1.executions_pb2 import ExecutionsCancelResponse

ExecutionsCancelResponse.__module__ = "dokime.client"
ExecutionsCancelResponse.__qualname__ = "ExecutionsCancelResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsCreateResponse

ExecutionsCreateResponse.__module__ = "dokime.client"
ExecutionsCreateResponse.__qualname__ = "ExecutionsCreateResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsDeleteResponse

ExecutionsDeleteResponse.__module__ = "dokime.client"
ExecutionsDeleteResponse.__qualname__ = "ExecutionsDeleteResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsGetResponse

ExecutionsGetResponse.__module__ = "dokime.client"
ExecutionsGetResponse.__qualname__ = "ExecutionsGetResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsListResponse

ExecutionsListResponse.__module__ = "dokime.client"
ExecutionsListResponse.__qualname__ = "ExecutionsListResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsParametersResponse

ExecutionsParametersResponse.__module__ = "dokime.client"
ExecutionsParametersResponse.__qualname__ = "ExecutionsParametersResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsSetResponse

ExecutionsSetResponse.__module__ = "dokime.client"
ExecutionsSetResponse.__qualname__ = "ExecutionsSetResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsStartResponse

ExecutionsStartResponse.__module__ = "dokime.client"
ExecutionsStartResponse.__qualname__ = "ExecutionsStartResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsStepsResponse

ExecutionsStepsResponse.__module__ = "dokime.client"
ExecutionsStepsResponse.__qualname__ = "ExecutionsStepsResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsStopResponse

ExecutionsStopResponse.__module__ = "dokime.client"
ExecutionsStopResponse.__qualname__ = "ExecutionsStopResponse"
from dokime._wire.host.v1.executions_pb2 import ExecutionsWatchResponse

ExecutionsWatchResponse.__module__ = "dokime.client"
ExecutionsWatchResponse.__qualname__ = "ExecutionsWatchResponse"
from dokime._wire.host.v1.executions_pb2 import FrozenSchemas

FrozenSchemas.__module__ = "dokime.client"
FrozenSchemas.__qualname__ = "FrozenSchemas"
from dokime._wire.host.v1.executions_pb2 import (
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

MaterializedParameter.__module__ = "dokime.client"
MaterializedParameter.__qualname__ = "MaterializedParameter"
from dokime._wire.host.v1.executions_pb2 import (
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

ReportStatus.__module__ = "dokime.client"
ReportStatus.__qualname__ = "ReportStatus"
from dokime._wire.host.v1.executions_pb2 import (
    STEP_STATUS_CANCELLED,
    STEP_STATUS_FAILED,
    STEP_STATUS_PASSED,
    STEP_STATUS_PENDING,
    STEP_STATUS_RUNNING,
    STEP_STATUS_SKIPPED,
    STEP_STATUS_UNSPECIFIED,
    StepCriterion,
)

StepCriterion.__module__ = "dokime.client"
StepCriterion.__qualname__ = "StepCriterion"
from dokime._wire.host.v1.executions_pb2 import StepStatus, UnevaluatedOutcome

UnevaluatedOutcome.__module__ = "dokime.client"
UnevaluatedOutcome.__qualname__ = "UnevaluatedOutcome"
from dokime._wire.host.v1.media.captures_pb2 import (
    CAPTURE_KIND_AUDIO,
    CAPTURE_KIND_CAMERA,
    CAPTURE_KIND_PHOTO,
    CAPTURE_KIND_SCREEN,
    CAPTURE_KIND_SCREENSHOT,
    CAPTURE_KIND_UNSPECIFIED,
    Capture,
)

Capture.__module__ = "dokime.client"
Capture.__qualname__ = "Capture"
from dokime._wire.host.v1.media.captures_pb2 import CaptureKind, CapturesCreateResponse

CapturesCreateResponse.__module__ = "dokime.client"
CapturesCreateResponse.__qualname__ = "CapturesCreateResponse"
from dokime._wire.host.v1.media.captures_pb2 import CapturesGetResponse

CapturesGetResponse.__module__ = "dokime.client"
CapturesGetResponse.__qualname__ = "CapturesGetResponse"
from dokime._wire.host.v1.media.captures_pb2 import CapturesWatchResponse

from ._exports.client_types import host

CapturesWatchResponse.__module__ = "dokime.client"
CapturesWatchResponse.__qualname__ = "CapturesWatchResponse"
from dokime._wire.host.v1.media.captures_pb2 import RecordingAudio

RecordingAudio.__module__ = "dokime.client"
RecordingAudio.__qualname__ = "RecordingAudio"
from dokime._wire.host.v1.media.captures_pb2 import RecordingProgress

RecordingProgress.__module__ = "dokime.client"
RecordingProgress.__qualname__ = "RecordingProgress"
from dokime._wire.host.v1.plots.settings_pb2 import PlotSettingsValue

PlotSettingsValue.__module__ = "dokime.client"
PlotSettingsValue.__qualname__ = "PlotSettingsValue"
from dokime._wire.host.v1.plots.snapshots_pb2 import ContourGrid

ContourGrid.__module__ = "dokime.client"
ContourGrid.__qualname__ = "ContourGrid"
from dokime._wire.host.v1.plots.snapshots_pb2 import GridCell

GridCell.__module__ = "dokime.client"
GridCell.__qualname__ = "GridCell"
from dokime._wire.host.v1.plots.snapshots_pb2 import GridFrame

GridFrame.__module__ = "dokime.client"
GridFrame.__qualname__ = "GridFrame"
from dokime._wire.host.v1.plots.snapshots_pb2 import GridFrames

GridFrames.__module__ = "dokime.client"
GridFrames.__qualname__ = "GridFrames"
from dokime._wire.host.v1.plots.snapshots_pb2 import NullableNumber

NullableNumber.__module__ = "dokime.client"
NullableNumber.__qualname__ = "NullableNumber"
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
    PlotAnalysis,
)

PlotAnalysis.__module__ = "dokime.client"
PlotAnalysis.__qualname__ = "PlotAnalysis"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotAnalysisError

PlotAnalysisError.__module__ = "dokime.client"
PlotAnalysisError.__qualname__ = "PlotAnalysisError"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotAnalysisResult

PlotAnalysisResult.__module__ = "dokime.client"
PlotAnalysisResult.__qualname__ = "PlotAnalysisResult"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotBoxSummary

PlotBoxSummary.__module__ = "dokime.client"
PlotBoxSummary.__qualname__ = "PlotBoxSummary"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotBoxSummaryAnalysis

PlotBoxSummaryAnalysis.__module__ = "dokime.client"
PlotBoxSummaryAnalysis.__qualname__ = "PlotBoxSummaryAnalysis"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotCartesianMode, PlotColumn

PlotColumn.__module__ = "dokime.client"
PlotColumn.__qualname__ = "PlotColumn"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotDomain

PlotDomain.__module__ = "dokime.client"
PlotDomain.__qualname__ = "PlotDomain"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotEnvelope

PlotEnvelope.__module__ = "dokime.client"
PlotEnvelope.__qualname__ = "PlotEnvelope"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotFrame

PlotFrame.__module__ = "dokime.client"
PlotFrame.__qualname__ = "PlotFrame"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramAnalysis

PlotHistogramAnalysis.__module__ = "dokime.client"
PlotHistogramAnalysis.__qualname__ = "PlotHistogramAnalysis"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramBin

PlotHistogramBin.__module__ = "dokime.client"
PlotHistogramBin.__qualname__ = "PlotHistogramBin"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramSeries

PlotHistogramSeries.__module__ = "dokime.client"
PlotHistogramSeries.__qualname__ = "PlotHistogramSeries"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotHistogramStatistics

PlotHistogramStatistics.__module__ = "dokime.client"
PlotHistogramStatistics.__qualname__ = "PlotHistogramStatistics"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotMarker

PlotMarker.__module__ = "dokime.client"
PlotMarker.__qualname__ = "PlotMarker"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotPayload

PlotPayload.__module__ = "dokime.client"
PlotPayload.__qualname__ = "PlotPayload"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotPayloadDiagnostics

PlotPayloadDiagnostics.__module__ = "dokime.client"
PlotPayloadDiagnostics.__qualname__ = "PlotPayloadDiagnostics"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotRequest

PlotRequest.__module__ = "dokime.client"
PlotRequest.__qualname__ = "PlotRequest"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSeries

PlotSeries.__module__ = "dokime.client"
PlotSeries.__qualname__ = "PlotSeries"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSeriesSummary

PlotSeriesSummary.__module__ = "dokime.client"
PlotSeriesSummary.__qualname__ = "PlotSeriesSummary"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSigmaBin

PlotSigmaBin.__module__ = "dokime.client"
PlotSigmaBin.__qualname__ = "PlotSigmaBin"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSigmaIntervalAnalysis

PlotSigmaIntervalAnalysis.__module__ = "dokime.client"
PlotSigmaIntervalAnalysis.__qualname__ = "PlotSigmaIntervalAnalysis"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotSigmaSeries

PlotSigmaSeries.__module__ = "dokime.client"
PlotSigmaSeries.__qualname__ = "PlotSigmaSeries"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotStreamKind, PlotStreamUpdate

PlotStreamUpdate.__module__ = "dokime.client"
PlotStreamUpdate.__qualname__ = "PlotStreamUpdate"
from dokime._wire.host.v1.plots.snapshots_pb2 import PlotWindow

PlotWindow.__module__ = "dokime.client"
PlotWindow.__qualname__ = "PlotWindow"
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsListResponse

SnapshotsListResponse.__module__ = "dokime.client"
SnapshotsListResponse.__qualname__ = "SnapshotsListResponse"
from dokime._wire.host.v1.plots.snapshots_pb2 import SnapshotsWatchResponse

SnapshotsWatchResponse.__module__ = "dokime.client"
SnapshotsWatchResponse.__qualname__ = "SnapshotsWatchResponse"
from dokime._wire.host.v1.plugins.secrets_pb2 import SecretRequirement

SecretRequirement.__module__ = "dokime.client"
SecretRequirement.__qualname__ = "SecretRequirement"
from dokime._wire.host.v1.prompt_definitions_pb2 import PromptDefinitionsGetResponse

PromptDefinitionsGetResponse.__module__ = "dokime.client"
PromptDefinitionsGetResponse.__qualname__ = "PromptDefinitionsGetResponse"
from dokime._wire.host.v1.prompt_definitions_pb2 import PromptDefinitionsListResponse

PromptDefinitionsListResponse.__module__ = "dokime.client"
PromptDefinitionsListResponse.__qualname__ = "PromptDefinitionsListResponse"
from dokime._wire.host.v1.prompts.answers_pb2 import AnswersCreateResponse

AnswersCreateResponse.__module__ = "dokime.client"
AnswersCreateResponse.__qualname__ = "AnswersCreateResponse"
from dokime._wire.host.v1.prompts_pb2 import (
    PROMPT_KIND_CHOICE,
    PROMPT_KIND_CONFIRMATION,
    PROMPT_KIND_FORM,
    PROMPT_KIND_STATUS,
    PROMPT_KIND_UNSPECIFIED,
    PROMPT_KIND_VERIFICATION,
    PromptInstance,
)

PromptInstance.__module__ = "dokime.client"
PromptInstance.__qualname__ = "PromptInstance"
from dokime._wire.host.v1.prompts_pb2 import PromptKind, PromptResponse

PromptResponse.__module__ = "dokime.client"
PromptResponse.__qualname__ = "PromptResponse"
from dokime._wire.host.v1.prompts_pb2 import PromptView

PromptView.__module__ = "dokime.client"
PromptView.__qualname__ = "PromptView"
from dokime._wire.host.v1.prompts_pb2 import PromptsGetResponse

PromptsGetResponse.__module__ = "dokime.client"
PromptsGetResponse.__qualname__ = "PromptsGetResponse"
from dokime._wire.host.v1.prompts_pb2 import PromptsListResponse

PromptsListResponse.__module__ = "dokime.client"
PromptsListResponse.__qualname__ = "PromptsListResponse"
from dokime._wire.host.v1.prompts_pb2 import PromptsWatchResponse

PromptsWatchResponse.__module__ = "dokime.client"
PromptsWatchResponse.__qualname__ = "PromptsWatchResponse"
from dokime._wire.host.v1.record_schemas_pb2 import AbsoluteTimeFormat

AbsoluteTimeFormat.__module__ = "dokime.client"
AbsoluteTimeFormat.__qualname__ = "AbsoluteTimeFormat"
from dokime._wire.host.v1.record_schemas_pb2 import (
    ELAPSED_UNIT_MILLISECONDS,
    ELAPSED_UNIT_MINUTES,
    ELAPSED_UNIT_SECONDS,
    ELAPSED_UNIT_UNSPECIFIED,
    ElapsedFormat,
)

ElapsedFormat.__module__ = "dokime.client"
ElapsedFormat.__qualname__ = "ElapsedFormat"
from dokime._wire.host.v1.record_schemas_pb2 import ElapsedUnit, EngineeringFormat

EngineeringFormat.__module__ = "dokime.client"
EngineeringFormat.__qualname__ = "EngineeringFormat"
from dokime._wire.host.v1.record_schemas_pb2 import FixedFormat

FixedFormat.__module__ = "dokime.client"
FixedFormat.__qualname__ = "FixedFormat"
from dokime._wire.host.v1.record_schemas_pb2 import MeasurementUnit

MeasurementUnit.__module__ = "dokime.client"
MeasurementUnit.__qualname__ = "MeasurementUnit"
from dokime._wire.host.v1.record_schemas_pb2 import NormalFormat

NormalFormat.__module__ = "dokime.client"
NormalFormat.__qualname__ = "NormalFormat"
from dokime._wire.host.v1.record_schemas_pb2 import (
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

RecordDisplay.__module__ = "dokime.client"
RecordDisplay.__qualname__ = "RecordDisplay"
from dokime._wire.host.v1.record_schemas_pb2 import RecordField

RecordField.__module__ = "dokime.client"
RecordField.__qualname__ = "RecordField"
from dokime._wire.host.v1.record_schemas_pb2 import RecordIcon, RecordSchema

RecordSchema.__module__ = "dokime.client"
RecordSchema.__qualname__ = "RecordSchema"
from dokime._wire.host.v1.record_schemas_pb2 import RecordSchemasGetResponse

RecordSchemasGetResponse.__module__ = "dokime.client"
RecordSchemasGetResponse.__qualname__ = "RecordSchemasGetResponse"
from dokime._wire.host.v1.record_schemas_pb2 import RecordSchemasListResponse

RecordSchemasListResponse.__module__ = "dokime.client"
RecordSchemasListResponse.__qualname__ = "RecordSchemasListResponse"
from dokime._wire.host.v1.record_schemas_pb2 import RecordTone, ScientificFormat

ScientificFormat.__module__ = "dokime.client"
ScientificFormat.__qualname__ = "ScientificFormat"
from dokime._wire.host.v1.record_schemas_pb2 import SiPrefixFormat

SiPrefixFormat.__module__ = "dokime.client"
SiPrefixFormat.__qualname__ = "SiPrefixFormat"
from dokime._wire.host.v1.record_schemas_pb2 import (
    TELEMETRY_ROLE_COUNTER,
    TELEMETRY_ROLE_DISTANCE,
    TELEMETRY_ROLE_LATENCY,
    TELEMETRY_ROLE_STATE,
    TELEMETRY_ROLE_THROUGHPUT,
    TELEMETRY_ROLE_UNSPECIFIED,
    TELEMETRY_ROLE_VALUE,
    TelemetryColumn,
)

TelemetryColumn.__module__ = "dokime.client"
TelemetryColumn.__qualname__ = "TelemetryColumn"
from dokime._wire.host.v1.record_schemas_pb2 import TelemetryRole, TelemetrySchema

TelemetrySchema.__module__ = "dokime.client"
TelemetrySchema.__qualname__ = "TelemetrySchema"
from dokime._wire.host.v1.record_schemas_pb2 import (
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

ValueFormat.__module__ = "dokime.client"
ValueFormat.__qualname__ = "ValueFormat"
from dokime._wire.host.v1.record_schemas_pb2 import ValueFormatKind

ValueFormatKind.__module__ = "dokime.client"
ValueFormatKind.__qualname__ = "ValueFormatKind"
from dokime._wire.host.v1.reports_pb2 import REPORT_ORIGIN_CURRENT, REPORT_ORIGIN_SEALED, REPORT_ORIGIN_UNSPECIFIED, ReportMetadata

ReportMetadata.__module__ = "dokime.client"
ReportMetadata.__qualname__ = "ReportMetadata"
from dokime._wire.host.v1.reports_pb2 import ReportOrigin, ReportSubject

ReportSubject.__module__ = "dokime.client"
ReportSubject.__qualname__ = "ReportSubject"
from dokime._wire.host.v1.reports_pb2 import ReportsCreateResponse

ReportsCreateResponse.__module__ = "dokime.client"
ReportsCreateResponse.__qualname__ = "ReportsCreateResponse"
from dokime._wire.host.v1.reports_pb2 import ReportsGetResponse

ReportsGetResponse.__module__ = "dokime.client"
ReportsGetResponse.__qualname__ = "ReportsGetResponse"
from dokime._wire.host.v1.reports_pb2 import ReportsPublishResponse

ReportsPublishResponse.__module__ = "dokime.client"
ReportsPublishResponse.__qualname__ = "ReportsPublishResponse"
from dokime._wire.host.v1.secrets_pb2 import (
    SECRET_IMPORT_ACTION_REPLACE,
    SECRET_IMPORT_ACTION_SKIP,
    SECRET_IMPORT_ACTION_UNSPECIFIED,
    SecretArchive,
)

SecretArchive.__module__ = "dokime.client"
SecretArchive.__qualname__ = "SecretArchive"
from dokime._wire.host.v1.secrets_pb2 import SecretImportAction, SecretImportChoice

SecretImportChoice.__module__ = "dokime.client"
SecretImportChoice.__qualname__ = "SecretImportChoice"
from dokime._wire.host.v1.secrets_pb2 import SecretImportEntry

SecretImportEntry.__module__ = "dokime.client"
SecretImportEntry.__qualname__ = "SecretImportEntry"
from dokime._wire.host.v1.secrets_pb2 import SecretImportPreview

SecretImportPreview.__module__ = "dokime.client"
SecretImportPreview.__qualname__ = "SecretImportPreview"
from dokime._wire.host.v1.secrets_pb2 import SecretSnapshot

SecretSnapshot.__module__ = "dokime.client"
SecretSnapshot.__qualname__ = "SecretSnapshot"
from dokime._wire.host.v1.secrets_pb2 import SecretSummary

SecretSummary.__module__ = "dokime.client"
SecretSummary.__qualname__ = "SecretSummary"
from dokime._wire.host.v1.secrets_pb2 import SecretsDeleteResponse

SecretsDeleteResponse.__module__ = "dokime.client"
SecretsDeleteResponse.__qualname__ = "SecretsDeleteResponse"
from dokime._wire.host.v1.secrets_pb2 import SecretsExportResponse

SecretsExportResponse.__module__ = "dokime.client"
SecretsExportResponse.__qualname__ = "SecretsExportResponse"
from dokime._wire.host.v1.secrets_pb2 import SecretsImportResponse

SecretsImportResponse.__module__ = "dokime.client"
SecretsImportResponse.__qualname__ = "SecretsImportResponse"
from dokime._wire.host.v1.secrets_pb2 import SecretsSetResponse

SecretsSetResponse.__module__ = "dokime.client"
SecretsSetResponse.__qualname__ = "SecretsSetResponse"
from dokime._wire.host.v1.secrets_pb2 import SecretsWatchResponse

SecretsWatchResponse.__module__ = "dokime.client"
SecretsWatchResponse.__qualname__ = "SecretsWatchResponse"
from dokime._wire.host.v1.settings_pb2 import ApplicationAppearance

ApplicationAppearance.__module__ = "dokime.client"
ApplicationAppearance.__qualname__ = "ApplicationAppearance"
from dokime._wire.host.v1.settings_pb2 import ApplicationSettings

ApplicationSettings.__module__ = "dokime.client"
ApplicationSettings.__qualname__ = "ApplicationSettings"
from dokime._wire.host.v1.settings_pb2 import BootstrapPaths

BootstrapPaths.__module__ = "dokime.client"
BootstrapPaths.__qualname__ = "BootstrapPaths"
from dokime._wire.host.v1.settings_pb2 import CachedRefresh

CachedRefresh.__module__ = "dokime.client"
CachedRefresh.__qualname__ = "CachedRefresh"
from dokime._wire.host.v1.settings_pb2 import ClientPreferences

ClientPreferences.__module__ = "dokime.client"
ClientPreferences.__qualname__ = "ClientPreferences"
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
    Density,
    HostCardBlock,
)

HostCardBlock.__module__ = "dokime.client"
HostCardBlock.__qualname__ = "HostCardBlock"
from dokime._wire.host.v1.settings_pb2 import HostCardBlockKind

HostCardBlockKind.__module__ = "dokime.client"
HostCardBlockKind.__qualname__ = "HostCardBlockKind"
from dokime._wire.host.v1.settings_pb2 import HostCardBody

HostCardBody.__module__ = "dokime.client"
HostCardBody.__qualname__ = "HostCardBody"
from dokime._wire.host.v1.settings_pb2 import HostCardKeyValue

HostCardKeyValue.__module__ = "dokime.client"
HostCardKeyValue.__qualname__ = "HostCardKeyValue"
from dokime._wire.host.v1.settings_pb2 import HostCardKeyValueList

HostCardKeyValueList.__module__ = "dokime.client"
HostCardKeyValueList.__qualname__ = "HostCardKeyValueList"
from dokime._wire.host.v1.settings_pb2 import HostCardMetric

HostCardMetric.__module__ = "dokime.client"
HostCardMetric.__qualname__ = "HostCardMetric"
from dokime._wire.host.v1.settings_pb2 import HostCardMetrics

HostCardMetrics.__module__ = "dokime.client"
HostCardMetrics.__qualname__ = "HostCardMetrics"
from dokime._wire.host.v1.settings_pb2 import HostCardStatus

HostCardStatus.__module__ = "dokime.client"
HostCardStatus.__qualname__ = "HostCardStatus"
from dokime._wire.host.v1.settings_pb2 import HostCardTone, ManualRefresh

ManualRefresh.__module__ = "dokime.client"
ManualRefresh.__qualname__ = "ManualRefresh"
from dokime._wire.host.v1.settings_pb2 import OnDemandRefresh

OnDemandRefresh.__module__ = "dokime.client"
OnDemandRefresh.__qualname__ = "OnDemandRefresh"
from dokime._wire.host.v1.settings_pb2 import RemoteListenerSettings

RemoteListenerSettings.__module__ = "dokime.client"
RemoteListenerSettings.__qualname__ = "RemoteListenerSettings"
from dokime._wire.host.v1.settings_pb2 import ReportDefaults

ReportDefaults.__module__ = "dokime.client"
ReportDefaults.__qualname__ = "ReportDefaults"
from dokime._wire.host.v1.settings_pb2 import (
    SETTINGS_CARD_STATE_DISABLED,
    SETTINGS_CARD_STATE_DISCONNECTED,
    SETTINGS_CARD_STATE_ERROR,
    SETTINGS_CARD_STATE_PERMISSION_DENIED,
    SETTINGS_CARD_STATE_READY,
    SETTINGS_CARD_STATE_STALE,
    SETTINGS_CARD_STATE_UNSPECIFIED,
    SettingsCard,
)

SettingsCard.__module__ = "dokime.client"
SettingsCard.__qualname__ = "SettingsCard"
from dokime._wire.host.v1.settings_pb2 import SettingsCardRefreshPolicy

SettingsCardRefreshPolicy.__module__ = "dokime.client"
SettingsCardRefreshPolicy.__qualname__ = "SettingsCardRefreshPolicy"
from dokime._wire.host.v1.settings_pb2 import SettingsCardRefreshPolicyPolicy

SettingsCardRefreshPolicyPolicy.__module__ = "dokime.client"
SettingsCardRefreshPolicyPolicy.__qualname__ = "SettingsCardRefreshPolicyPolicy"
from dokime._wire.host.v1.settings_pb2 import SettingsCardState, SettingsFrame

SettingsFrame.__module__ = "dokime.client"
SettingsFrame.__qualname__ = "SettingsFrame"
from dokime._wire.host.v1.settings_pb2 import SettingsSchemaResponse

SettingsSchemaResponse.__module__ = "dokime.client"
SettingsSchemaResponse.__qualname__ = "SettingsSchemaResponse"
from dokime._wire.host.v1.settings_pb2 import SettingsSection

SettingsSection.__module__ = "dokime.client"
SettingsSection.__qualname__ = "SettingsSection"
from dokime._wire.host.v1.settings_pb2 import SettingsSectionOwner

SettingsSectionOwner.__module__ = "dokime.client"
SettingsSectionOwner.__qualname__ = "SettingsSectionOwner"
from dokime._wire.host.v1.settings_pb2 import (
    THEME_MODE_DARK,
    THEME_MODE_LIGHT,
    THEME_MODE_SYSTEM,
    THEME_MODE_UNSPECIFIED,
    TelemetryPreferences,
)

TelemetryPreferences.__module__ = "dokime.client"
TelemetryPreferences.__qualname__ = "TelemetryPreferences"
from dokime._wire.host.v1.settings_pb2 import ThemeMode
from dokime._wire.host.v1.storage.sources_pb2 import SourcesGetResponse

SourcesGetResponse.__module__ = "dokime.client"
SourcesGetResponse.__qualname__ = "SourcesGetResponse"
from dokime._wire.host.v1.storage.sources_pb2 import SourcesSetResponse

SourcesSetResponse.__module__ = "dokime.client"
SourcesSetResponse.__qualname__ = "SourcesSetResponse"
from dokime._wire.host.v1.storage.sources_pb2 import StorageSource

StorageSource.__module__ = "dokime.client"
StorageSource.__qualname__ = "StorageSource"
from dokime._wire.host.v1.storage.sources_pb2 import StorageSources

StorageSources.__module__ = "dokime.client"
StorageSources.__qualname__ = "StorageSources"
from dokime._wire.host.v1.stream_schemas_pb2 import StreamSchemasGetResponse

StreamSchemasGetResponse.__module__ = "dokime.client"
StreamSchemasGetResponse.__qualname__ = "StreamSchemasGetResponse"
from dokime._wire.host.v1.stream_schemas_pb2 import StreamSchemasListResponse

StreamSchemasListResponse.__module__ = "dokime.client"
StreamSchemasListResponse.__qualname__ = "StreamSchemasListResponse"
from dokime._wire.host.v1.suites_pb2 import SuitesGetResponse

SuitesGetResponse.__module__ = "dokime.client"
SuitesGetResponse.__qualname__ = "SuitesGetResponse"
from dokime._wire.host.v1.suites_pb2 import SuitesListResponse

SuitesListResponse.__module__ = "dokime.client"
SuitesListResponse.__qualname__ = "SuitesListResponse"
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
    CertificatesCreateResponse,
    ComponentState,
    DiagnosticsGetResponse,
    Event,
    HealthComponent,
    HostIdentity,
    HostStatus,
    InvocationsCreateResponse,
    ListenerGetResponse,
    ListenerSetResponse,
    ListenerValue,
    LogsListResponse,
    LogsWatchResponse,
    Pairing,
    PairingsCreateResponse,
    PairingsDeleteResponse,
    PairingsListResponse,
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
    TraceEventsListResponse,
    TraceEventsWatchResponse,
)

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
