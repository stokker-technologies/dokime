import datetime

from dokime.client._generated.dokime.v1 import appearance_pb2 as _appearance_pb2
from dokime.client._generated.dokime.v1 import catalog_pb2 as _catalog_pb2
from dokime.client._generated.dokime.v1 import checkpoint_pb2 as _checkpoint_pb2
from dokime.client._generated.dokime.v1 import common_pb2 as _common_pb2
from dokime.client._generated.dokime.v1 import content_pb2 as _content_pb2
from dokime.client._generated.dokime.v1 import documents_pb2 as _documents_pb2
from dokime.client._generated.dokime.v1 import evidence_pb2 as _evidence_pb2
from dokime.client._generated.dokime.v1 import execution_pb2 as _execution_pb2
from dokime.client._generated.dokime.v1 import notes_pb2 as _notes_pb2
from dokime.client._generated.dokime.v1 import prompts_pb2 as _prompts_pb2
from dokime.client._generated.dokime.v1 import plots_pb2 as _plots_pb2
from dokime.client._generated.dokime.v1 import schema_pb2 as _schema_pb2
from dokime.client._generated.dokime.v1 import telemetry_pb2 as _telemetry_pb2
from dokime.client._generated.dokime.v1 import value_pb2 as _value_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ExecutionOrderKey(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_ORDER_KEY_UNSPECIFIED: _ClassVar[ExecutionOrderKey]
    EXECUTION_ORDER_KEY_STARTED_AT: _ClassVar[ExecutionOrderKey]
    EXECUTION_ORDER_KEY_FINISHED_AT: _ClassVar[ExecutionOrderKey]
    EXECUTION_ORDER_KEY_TITLE: _ClassVar[ExecutionOrderKey]

class ExecutionAvailability(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EXECUTION_AVAILABILITY_UNSPECIFIED: _ClassVar[ExecutionAvailability]
    EXECUTION_AVAILABILITY_LIVE: _ClassVar[ExecutionAvailability]
    EXECUTION_AVAILABILITY_STORED: _ClassVar[ExecutionAvailability]
    EXECUTION_AVAILABILITY_INTERRUPTED: _ClassVar[ExecutionAvailability]
    EXECUTION_AVAILABILITY_UNCHECKPOINTED: _ClassVar[ExecutionAvailability]

class StepTermination(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    STEP_TERMINATION_UNSPECIFIED: _ClassVar[StepTermination]
    STEP_TERMINATION_COMPLETED: _ClassVar[StepTermination]
    STEP_TERMINATION_FAILED: _ClassVar[StepTermination]
    STEP_TERMINATION_CANCELLED: _ClassVar[StepTermination]
    STEP_TERMINATION_INTERRUPTED: _ClassVar[StepTermination]

class EvidenceSection(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EVIDENCE_SECTION_UNSPECIFIED: _ClassVar[EvidenceSection]
    EVIDENCE_SECTION_OUTPUTS: _ClassVar[EvidenceSection]
    EVIDENCE_SECTION_ARTIFACTS: _ClassVar[EvidenceSection]
    EVIDENCE_SECTION_RECORD_SCHEMAS: _ClassVar[EvidenceSection]
    EVIDENCE_SECTION_TELEMETRY_TABLES: _ClassVar[EvidenceSection]
EXECUTION_ORDER_KEY_UNSPECIFIED: ExecutionOrderKey
EXECUTION_ORDER_KEY_STARTED_AT: ExecutionOrderKey
EXECUTION_ORDER_KEY_FINISHED_AT: ExecutionOrderKey
EXECUTION_ORDER_KEY_TITLE: ExecutionOrderKey
EXECUTION_AVAILABILITY_UNSPECIFIED: ExecutionAvailability
EXECUTION_AVAILABILITY_LIVE: ExecutionAvailability
EXECUTION_AVAILABILITY_STORED: ExecutionAvailability
EXECUTION_AVAILABILITY_INTERRUPTED: ExecutionAvailability
EXECUTION_AVAILABILITY_UNCHECKPOINTED: ExecutionAvailability
STEP_TERMINATION_UNSPECIFIED: StepTermination
STEP_TERMINATION_COMPLETED: StepTermination
STEP_TERMINATION_FAILED: StepTermination
STEP_TERMINATION_CANCELLED: StepTermination
STEP_TERMINATION_INTERRUPTED: StepTermination
EVIDENCE_SECTION_UNSPECIFIED: EvidenceSection
EVIDENCE_SECTION_OUTPUTS: EvidenceSection
EVIDENCE_SECTION_ARTIFACTS: EvidenceSection
EVIDENCE_SECTION_RECORD_SCHEMAS: EvidenceSection
EVIDENCE_SECTION_TELEMETRY_TABLES: EvidenceSection

class GetExecutionCheckpointRequest(_message.Message):
    __slots__ = ("execution_identifier",)
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    execution_identifier: str
    def __init__(self, execution_identifier: _Optional[str] = ...) -> None: ...

class DeleteExecutionRequest(_message.Message):
    __slots__ = ("checkpoint", "request_identifier")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    request_identifier: str
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., request_identifier: _Optional[str] = ...) -> None: ...

class DeleteExecutionResponse(_message.Message):
    __slots__ = ("request_identifier", "execution_identifier", "local_complete")
    REQUEST_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_IDENTIFIER_FIELD_NUMBER: _ClassVar[int]
    LOCAL_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    request_identifier: str
    execution_identifier: str
    local_complete: bool
    def __init__(self, request_identifier: _Optional[str] = ..., execution_identifier: _Optional[str] = ..., local_complete: _Optional[bool] = ...) -> None: ...

class ExecutionFilter(_message.Message):
    __slots__ = ("text", "statuses", "template_ids", "cycle_ids", "started_after", "started_before")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    STATUSES_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_IDS_FIELD_NUMBER: _ClassVar[int]
    CYCLE_IDS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AFTER_FIELD_NUMBER: _ClassVar[int]
    STARTED_BEFORE_FIELD_NUMBER: _ClassVar[int]
    text: str
    statuses: _containers.RepeatedScalarFieldContainer[_execution_pb2.ExecutionStatus]
    template_ids: _containers.RepeatedScalarFieldContainer[str]
    cycle_ids: _containers.RepeatedScalarFieldContainer[str]
    started_after: _timestamp_pb2.Timestamp
    started_before: _timestamp_pb2.Timestamp
    def __init__(self, text: _Optional[str] = ..., statuses: _Optional[_Iterable[_Union[_execution_pb2.ExecutionStatus, str]]] = ..., template_ids: _Optional[_Iterable[str]] = ..., cycle_ids: _Optional[_Iterable[str]] = ..., started_after: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., started_before: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class ExecutionOrder(_message.Message):
    __slots__ = ("key", "descending")
    KEY_FIELD_NUMBER: _ClassVar[int]
    DESCENDING_FIELD_NUMBER: _ClassVar[int]
    key: ExecutionOrderKey
    descending: bool
    def __init__(self, key: _Optional[_Union[ExecutionOrderKey, str]] = ..., descending: _Optional[bool] = ...) -> None: ...

class BrowseExecutionsRequest(_message.Message):
    __slots__ = ("limit", "after_execution_id", "filter", "order")
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    ORDER_FIELD_NUMBER: _ClassVar[int]
    limit: int
    after_execution_id: str
    filter: ExecutionFilter
    order: ExecutionOrder
    def __init__(self, limit: _Optional[int] = ..., after_execution_id: _Optional[str] = ..., filter: _Optional[_Union[ExecutionFilter, _Mapping]] = ..., order: _Optional[_Union[ExecutionOrder, _Mapping]] = ...) -> None: ...

class ExecutionBrowsePage(_message.Message):
    __slots__ = ("entries", "next_after_execution_id", "discovery_complete", "conflicts", "omitted_conflicts", "issues", "omitted_issues")
    ENTRIES_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    DISCOVERY_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    CONFLICTS_FIELD_NUMBER: _ClassVar[int]
    OMITTED_CONFLICTS_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    OMITTED_ISSUES_FIELD_NUMBER: _ClassVar[int]
    entries: _containers.RepeatedCompositeFieldContainer[ExecutionBrowseEntry]
    next_after_execution_id: str
    discovery_complete: bool
    conflicts: _containers.RepeatedScalarFieldContainer[str]
    omitted_conflicts: int
    issues: _containers.RepeatedCompositeFieldContainer[HistoryIssue]
    omitted_issues: int
    def __init__(self, entries: _Optional[_Iterable[_Union[ExecutionBrowseEntry, _Mapping]]] = ..., next_after_execution_id: _Optional[str] = ..., discovery_complete: _Optional[bool] = ..., conflicts: _Optional[_Iterable[str]] = ..., omitted_conflicts: _Optional[int] = ..., issues: _Optional[_Iterable[_Union[HistoryIssue, _Mapping]]] = ..., omitted_issues: _Optional[int] = ...) -> None: ...

class ExecutionBrowseEntry(_message.Message):
    __slots__ = ("metadata", "checkpoint", "availability")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    AVAILABILITY_FIELD_NUMBER: _ClassVar[int]
    metadata: HistoricalExecutionMetadata
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    availability: ExecutionAvailability
    def __init__(self, metadata: _Optional[_Union[HistoricalExecutionMetadata, _Mapping]] = ..., checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., availability: _Optional[_Union[ExecutionAvailability, str]] = ...) -> None: ...

class HistoricalExecutionMetadata(_message.Message):
    __slots__ = ("execution_id", "cycle_id", "case_id", "title", "status", "started_at", "finished_at", "result", "template_id", "suite_label")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_LABEL_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    cycle_id: str
    case_id: str
    title: str
    status: _execution_pb2.ExecutionStatus
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    result: _execution_pb2.ResultSummary
    template_id: str
    suite_label: str
    def __init__(self, execution_id: _Optional[str] = ..., cycle_id: _Optional[str] = ..., case_id: _Optional[str] = ..., title: _Optional[str] = ..., status: _Optional[_Union[_execution_pb2.ExecutionStatus, str]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., result: _Optional[_Union[_execution_pb2.ResultSummary, _Mapping]] = ..., template_id: _Optional[str] = ..., suite_label: _Optional[str] = ...) -> None: ...

class HistoryIssue(_message.Message):
    __slots__ = ("source", "dataset", "message")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    DATASET_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    source: str
    dataset: str
    message: str
    def __init__(self, source: _Optional[str] = ..., dataset: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...

class GetCheckpointDocumentRequest(_message.Message):
    __slots__ = ("checkpoint",)
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ...) -> None: ...

class CheckpointDocument(_message.Message):
    __slots__ = ("checkpoint", "document", "acquisition_finished")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_FIELD_NUMBER: _ClassVar[int]
    ACQUISITION_FINISHED_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    document: HistoricalExecutionDocument
    acquisition_finished: bool
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., document: _Optional[_Union[HistoricalExecutionDocument, _Mapping]] = ..., acquisition_finished: _Optional[bool] = ...) -> None: ...

class HistoricalExecutionDocument(_message.Message):
    __slots__ = ("cycle", "execution")
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_FIELD_NUMBER: _ClassVar[int]
    cycle: HistoricalExecutionCycle
    execution: HistoricalTestExecution
    def __init__(self, cycle: _Optional[_Union[HistoricalExecutionCycle, _Mapping]] = ..., execution: _Optional[_Union[HistoricalTestExecution, _Mapping]] = ...) -> None: ...

class HistoricalExecutionCycle(_message.Message):
    __slots__ = ("cycle_id", "metadata")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: _common_pb2.Metadata
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ...) -> None: ...

class HistoricalTestExecution(_message.Message):
    __slots__ = ("execution_id", "case_id", "suite", "status", "scope", "snapshot", "output_parameters", "documents", "health", "notes", "started_at", "finished_at", "steps", "result", "invalidation", "step_attempts", "active_step_attempt", "mode")
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    SUITE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    SCOPE_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    DOCUMENTS_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    STEPS_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    INVALIDATION_FIELD_NUMBER: _ClassVar[int]
    STEP_ATTEMPTS_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_STEP_ATTEMPT_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    execution_id: str
    case_id: str
    suite: _common_pb2.Metadata
    status: _execution_pb2.ExecutionStatus
    scope: _common_pb2.EvidenceScope
    snapshot: HistoricalExecutionSnapshot
    output_parameters: _containers.RepeatedCompositeFieldContainer[_execution_pb2.MaterializedParameter]
    documents: _containers.RepeatedCompositeFieldContainer[HistoricalDocument]
    health: _containers.RepeatedCompositeFieldContainer[_notes_pb2.HealthItem]
    notes: _containers.RepeatedCompositeFieldContainer[_notes_pb2.OperatorNote]
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    steps: _containers.RepeatedCompositeFieldContainer[HistoricalStep]
    result: _execution_pb2.ResultSummary
    invalidation: _execution_pb2.ExecutionInvalidation
    step_attempts: _containers.RepeatedCompositeFieldContainer[HistoricalStepAttempt]
    active_step_attempt: HistoricalStepAttemptStart
    mode: _execution_pb2.ExecutionMode
    def __init__(self, execution_id: _Optional[str] = ..., case_id: _Optional[str] = ..., suite: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[_execution_pb2.ExecutionStatus, str]] = ..., scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., snapshot: _Optional[_Union[HistoricalExecutionSnapshot, _Mapping]] = ..., output_parameters: _Optional[_Iterable[_Union[_execution_pb2.MaterializedParameter, _Mapping]]] = ..., documents: _Optional[_Iterable[_Union[HistoricalDocument, _Mapping]]] = ..., health: _Optional[_Iterable[_Union[_notes_pb2.HealthItem, _Mapping]]] = ..., notes: _Optional[_Iterable[_Union[_notes_pb2.OperatorNote, _Mapping]]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., steps: _Optional[_Iterable[_Union[HistoricalStep, _Mapping]]] = ..., result: _Optional[_Union[_execution_pb2.ResultSummary, _Mapping]] = ..., invalidation: _Optional[_Union[_execution_pb2.ExecutionInvalidation, _Mapping]] = ..., step_attempts: _Optional[_Iterable[_Union[HistoricalStepAttempt, _Mapping]]] = ..., active_step_attempt: _Optional[_Union[HistoricalStepAttemptStart, _Mapping]] = ..., mode: _Optional[_Union[_execution_pb2.ExecutionMode, str]] = ...) -> None: ...

class HistoricalStepAttemptStart(_message.Message):
    __slots__ = ("attempt_id", "step_id", "started_at")
    ATTEMPT_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    attempt_id: str
    step_id: str
    started_at: _timestamp_pb2.Timestamp
    def __init__(self, attempt_id: _Optional[str] = ..., step_id: _Optional[str] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class HistoricalStepAttempt(_message.Message):
    __slots__ = ("attempt_id", "step_id", "started_at", "finished_at", "termination", "detail")
    ATTEMPT_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    TERMINATION_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    attempt_id: str
    step_id: str
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    termination: StepTermination
    detail: _common_pb2.Diagnostic
    def __init__(self, attempt_id: _Optional[str] = ..., step_id: _Optional[str] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., termination: _Optional[_Union[StepTermination, str]] = ..., detail: _Optional[_Union[_common_pb2.Diagnostic, _Mapping]] = ...) -> None: ...

class HistoricalExecutionSnapshot(_message.Message):
    __slots__ = ("suite", "template_id", "template", "plugin", "parameter_definitions", "parameters", "documents", "case_definition", "host_version")
    SUITE_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_FIELD_NUMBER: _ClassVar[int]
    PLUGIN_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_DEFINITIONS_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    DOCUMENTS_FIELD_NUMBER: _ClassVar[int]
    CASE_DEFINITION_FIELD_NUMBER: _ClassVar[int]
    HOST_VERSION_FIELD_NUMBER: _ClassVar[int]
    suite: _common_pb2.Metadata
    template_id: str
    template: _common_pb2.Metadata
    plugin: _execution_pb2.ExecutionPluginIdentity
    parameter_definitions: _containers.RepeatedCompositeFieldContainer[_catalog_pb2.ParameterSchema]
    parameters: _containers.RepeatedCompositeFieldContainer[_execution_pb2.MaterializedParameter]
    documents: _containers.RepeatedCompositeFieldContainer[HistoricalDocument]
    case_definition: HistoricalCaseDefinition
    host_version: str
    def __init__(self, suite: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., template_id: _Optional[str] = ..., template: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., plugin: _Optional[_Union[_execution_pb2.ExecutionPluginIdentity, _Mapping]] = ..., parameter_definitions: _Optional[_Iterable[_Union[_catalog_pb2.ParameterSchema, _Mapping]]] = ..., parameters: _Optional[_Iterable[_Union[_execution_pb2.MaterializedParameter, _Mapping]]] = ..., documents: _Optional[_Iterable[_Union[HistoricalDocument, _Mapping]]] = ..., case_definition: _Optional[_Union[HistoricalCaseDefinition, _Mapping]] = ..., host_version: _Optional[str] = ...) -> None: ...

class HistoricalCaseDefinition(_message.Message):
    __slots__ = ("case_id", "template_id", "metadata", "suite", "parameters", "generation_index", "suite_id")
    CASE_ID_FIELD_NUMBER: _ClassVar[int]
    TEMPLATE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SUITE_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    GENERATION_INDEX_FIELD_NUMBER: _ClassVar[int]
    SUITE_ID_FIELD_NUMBER: _ClassVar[int]
    case_id: str
    template_id: str
    metadata: _common_pb2.Metadata
    suite: _common_pb2.Metadata
    parameters: _containers.RepeatedCompositeFieldContainer[_execution_pb2.CaseParameter]
    generation_index: int
    suite_id: str
    def __init__(self, case_id: _Optional[str] = ..., template_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., suite: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., parameters: _Optional[_Iterable[_Union[_execution_pb2.CaseParameter, _Mapping]]] = ..., generation_index: _Optional[int] = ..., suite_id: _Optional[str] = ...) -> None: ...

class HistoricalStep(_message.Message):
    __slots__ = ("step_id", "metadata", "status", "started_at", "finished_at", "measurements", "criteria")
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    MEASUREMENTS_FIELD_NUMBER: _ClassVar[int]
    CRITERIA_FIELD_NUMBER: _ClassVar[int]
    step_id: str
    metadata: _common_pb2.Metadata
    status: _execution_pb2.StepStatus
    started_at: _timestamp_pb2.Timestamp
    finished_at: _timestamp_pb2.Timestamp
    measurements: _containers.RepeatedCompositeFieldContainer[_execution_pb2.StepMeasurement]
    criteria: _containers.RepeatedCompositeFieldContainer[_execution_pb2.StepCriterion]
    def __init__(self, step_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., status: _Optional[_Union[_execution_pb2.StepStatus, str]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., finished_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., measurements: _Optional[_Iterable[_Union[_execution_pb2.StepMeasurement, _Mapping]]] = ..., criteria: _Optional[_Iterable[_Union[_execution_pb2.StepCriterion, _Mapping]]] = ...) -> None: ...

class HistoricalDocument(_message.Message):
    __slots__ = ("presentation", "layout_nodes", "cards", "table_rows")
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    LAYOUT_NODES_FIELD_NUMBER: _ClassVar[int]
    CARDS_FIELD_NUMBER: _ClassVar[int]
    TABLE_ROWS_FIELD_NUMBER: _ClassVar[int]
    presentation: _documents_pb2.DocumentHeader
    layout_nodes: _containers.RepeatedCompositeFieldContainer[_documents_pb2.DocumentLayoutNode]
    cards: _containers.RepeatedCompositeFieldContainer[HistoricalDocumentCard]
    table_rows: _containers.RepeatedCompositeFieldContainer[_documents_pb2.DocumentTableRow]
    def __init__(self, presentation: _Optional[_Union[_documents_pb2.DocumentHeader, _Mapping]] = ..., layout_nodes: _Optional[_Iterable[_Union[_documents_pb2.DocumentLayoutNode, _Mapping]]] = ..., cards: _Optional[_Iterable[_Union[HistoricalDocumentCard, _Mapping]]] = ..., table_rows: _Optional[_Iterable[_Union[_documents_pb2.DocumentTableRow, _Mapping]]] = ...) -> None: ...

class HistoricalDocumentCard(_message.Message):
    __slots__ = ("metadata", "filters", "markdown", "table", "summary", "plot", "owner")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    MARKDOWN_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    PLOT_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    filters: _containers.RepeatedCompositeFieldContainer[HistoricalDocumentFilter]
    markdown: str
    table: _documents_pb2.DocumentTable
    summary: HistoricalTelemetrySummary
    plot: _documents_pb2.DocumentPlotDefinition
    owner: _documents_pb2.DocumentOwnerReference
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[HistoricalDocumentFilter, _Mapping]]] = ..., markdown: _Optional[str] = ..., table: _Optional[_Union[_documents_pb2.DocumentTable, _Mapping]] = ..., summary: _Optional[_Union[HistoricalTelemetrySummary, _Mapping]] = ..., plot: _Optional[_Union[_documents_pb2.DocumentPlotDefinition, _Mapping]] = ..., owner: _Optional[_Union[_documents_pb2.DocumentOwnerReference, _Mapping]] = ...) -> None: ...

class HistoricalTelemetrySummary(_message.Message):
    __slots__ = ("groups",)
    GROUPS_FIELD_NUMBER: _ClassVar[int]
    groups: _containers.RepeatedCompositeFieldContainer[HistoricalSummaryGroup]
    def __init__(self, groups: _Optional[_Iterable[_Union[HistoricalSummaryGroup, _Mapping]]] = ...) -> None: ...

class HistoricalSummaryGroup(_message.Message):
    __slots__ = ("metadata", "tiles")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TILES_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    tiles: _containers.RepeatedCompositeFieldContainer[HistoricalSummaryTile]
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., tiles: _Optional[_Iterable[_Union[HistoricalSummaryTile, _Mapping]]] = ...) -> None: ...

class HistoricalSummaryTile(_message.Message):
    __slots__ = ("metadata", "value", "filters", "stat", "format", "presentation", "unit", "conditions")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    STAT_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    PRESENTATION_FIELD_NUMBER: _ClassVar[int]
    UNIT_FIELD_NUMBER: _ClassVar[int]
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    metadata: _common_pb2.Metadata
    value: _plots_pb2.DataExpression
    filters: _containers.RepeatedCompositeFieldContainer[HistoricalDocumentFilter]
    stat: _documents_pb2.DocumentSummaryStat
    format: _documents_pb2.DocumentSummaryFormat
    presentation: _documents_pb2.DocumentSummaryPresentation
    unit: str
    conditions: _containers.RepeatedCompositeFieldContainer[HistoricalSummaryCondition]
    def __init__(self, metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., value: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., filters: _Optional[_Iterable[_Union[HistoricalDocumentFilter, _Mapping]]] = ..., stat: _Optional[_Union[_documents_pb2.DocumentSummaryStat, str]] = ..., format: _Optional[_Union[_documents_pb2.DocumentSummaryFormat, str]] = ..., presentation: _Optional[_Union[_documents_pb2.DocumentSummaryPresentation, str]] = ..., unit: _Optional[str] = ..., conditions: _Optional[_Iterable[_Union[HistoricalSummaryCondition, _Mapping]]] = ...) -> None: ...

class HistoricalSummaryCondition(_message.Message):
    __slots__ = ("operator", "value", "state")
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    operator: _plots_pb2.FilterOperator
    value: _value_pb2.Value
    state: _documents_pb2.DocumentSummaryState
    def __init__(self, operator: _Optional[_Union[_plots_pb2.FilterOperator, str]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ..., state: _Optional[_Union[_documents_pb2.DocumentSummaryState, str]] = ...) -> None: ...

class HistoricalDocumentFilter(_message.Message):
    __slots__ = ("expression", "operator", "value")
    EXPRESSION_FIELD_NUMBER: _ClassVar[int]
    OPERATOR_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    expression: _plots_pb2.DataExpression
    operator: _plots_pb2.FilterOperator
    value: _value_pb2.Value
    def __init__(self, expression: _Optional[_Union[_plots_pb2.DataExpression, _Mapping]] = ..., operator: _Optional[_Union[_plots_pb2.FilterOperator, str]] = ..., value: _Optional[_Union[_value_pb2.Value, _Mapping]] = ...) -> None: ...

class GetCheckpointEvidenceRequest(_message.Message):
    __slots__ = ("checkpoint", "sections")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    SECTIONS_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    sections: _containers.RepeatedScalarFieldContainer[EvidenceSection]
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., sections: _Optional[_Iterable[_Union[EvidenceSection, str]]] = ...) -> None: ...

class CheckpointTelemetryTable(_message.Message):
    __slots__ = ("table", "schema")
    TABLE_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    table: str
    schema: _schema_pb2.TelemetrySchema
    def __init__(self, table: _Optional[str] = ..., schema: _Optional[_Union[_schema_pb2.TelemetrySchema, _Mapping]] = ...) -> None: ...

class CheckpointEvidence(_message.Message):
    __slots__ = ("checkpoint", "outputs", "artifacts", "record_schemas", "telemetry_tables")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    RECORD_SCHEMAS_FIELD_NUMBER: _ClassVar[int]
    TELEMETRY_TABLES_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    outputs: _containers.RepeatedCompositeFieldContainer[_content_pb2.ExecutionFileMetadata]
    artifacts: _containers.RepeatedCompositeFieldContainer[_content_pb2.ArtifactMetadata]
    record_schemas: _containers.RepeatedCompositeFieldContainer[_schema_pb2.RecordSchema]
    telemetry_tables: _containers.RepeatedCompositeFieldContainer[CheckpointTelemetryTable]
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., outputs: _Optional[_Iterable[_Union[_content_pb2.ExecutionFileMetadata, _Mapping]]] = ..., artifacts: _Optional[_Iterable[_Union[_content_pb2.ArtifactMetadata, _Mapping]]] = ..., record_schemas: _Optional[_Iterable[_Union[_schema_pb2.RecordSchema, _Mapping]]] = ..., telemetry_tables: _Optional[_Iterable[_Union[CheckpointTelemetryTable, _Mapping]]] = ...) -> None: ...

class ReadCheckpointOutputRequest(_message.Message):
    __slots__ = ("checkpoint", "relative_path", "offset", "max_bytes")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    MAX_BYTES_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    relative_path: str
    offset: int
    max_bytes: int
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., relative_path: _Optional[str] = ..., offset: _Optional[int] = ..., max_bytes: _Optional[int] = ...) -> None: ...

class CheckpointFileWindow(_message.Message):
    __slots__ = ("checkpoint", "content")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    CONTENT_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    content: _content_pb2.ExecutionFileWindow
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., content: _Optional[_Union[_content_pb2.ExecutionFileWindow, _Mapping]] = ...) -> None: ...

class DownloadCheckpointArtifactRequest(_message.Message):
    __slots__ = ("checkpoint", "artifact_id")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    artifact_id: str
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., artifact_id: _Optional[str] = ...) -> None: ...

class ListCheckpointRecordsRequest(_message.Message):
    __slots__ = ("checkpoint", "limit", "after")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    limit: int
    after: str
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., limit: _Optional[int] = ..., after: _Optional[str] = ...) -> None: ...

class CheckpointRecordPage(_message.Message):
    __slots__ = ("checkpoint", "records", "next_after", "issues", "issues_truncated")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    RECORDS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    ISSUES_TRUNCATED_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    records: _containers.RepeatedCompositeFieldContainer[SavedRecord]
    next_after: str
    issues: _containers.RepeatedCompositeFieldContainer[HistoryIssue]
    issues_truncated: bool
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., records: _Optional[_Iterable[_Union[SavedRecord, _Mapping]]] = ..., next_after: _Optional[str] = ..., issues: _Optional[_Iterable[_Union[HistoryIssue, _Mapping]]] = ..., issues_truncated: _Optional[bool] = ...) -> None: ...

class SavedRecord(_message.Message):
    __slots__ = ("epoch", "record")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    RECORD_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    record: _evidence_pb2.CapturedRecord
    def __init__(self, epoch: _Optional[str] = ..., record: _Optional[_Union[_evidence_pb2.CapturedRecord, _Mapping]] = ...) -> None: ...

class ListCheckpointTelemetryRequest(_message.Message):
    __slots__ = ("checkpoint", "table", "limit", "after")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    AFTER_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    table: str
    limit: int
    after: str
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., table: _Optional[str] = ..., limit: _Optional[int] = ..., after: _Optional[str] = ...) -> None: ...

class CheckpointTelemetryPage(_message.Message):
    __slots__ = ("checkpoint", "table", "rows", "next_after", "issues", "issues_truncated")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    TABLE_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    NEXT_AFTER_FIELD_NUMBER: _ClassVar[int]
    ISSUES_FIELD_NUMBER: _ClassVar[int]
    ISSUES_TRUNCATED_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    table: str
    rows: _containers.RepeatedCompositeFieldContainer[SavedTelemetrySample]
    next_after: str
    issues: _containers.RepeatedCompositeFieldContainer[HistoryIssue]
    issues_truncated: bool
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., table: _Optional[str] = ..., rows: _Optional[_Iterable[_Union[SavedTelemetrySample, _Mapping]]] = ..., next_after: _Optional[str] = ..., issues: _Optional[_Iterable[_Union[HistoryIssue, _Mapping]]] = ..., issues_truncated: _Optional[bool] = ...) -> None: ...

class SavedTelemetrySample(_message.Message):
    __slots__ = ("epoch", "ordinal", "sample")
    EPOCH_FIELD_NUMBER: _ClassVar[int]
    ORDINAL_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_FIELD_NUMBER: _ClassVar[int]
    epoch: str
    ordinal: int
    sample: _telemetry_pb2.TelemetrySample
    def __init__(self, epoch: _Optional[str] = ..., ordinal: _Optional[int] = ..., sample: _Optional[_Union[_telemetry_pb2.TelemetrySample, _Mapping]] = ...) -> None: ...

class GetCheckpointContextRequest(_message.Message):
    __slots__ = ("checkpoint",)
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ...) -> None: ...

class CheckpointContext(_message.Message):
    __slots__ = ("checkpoint", "session_id", "execution_id", "execution_scope", "cycle", "execution_documents", "prompts", "health", "notes", "traceability", "appearance")
    CHECKPOINT_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_SCOPE_FIELD_NUMBER: _ClassVar[int]
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_DOCUMENTS_FIELD_NUMBER: _ClassVar[int]
    PROMPTS_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    NOTES_FIELD_NUMBER: _ClassVar[int]
    TRACEABILITY_FIELD_NUMBER: _ClassVar[int]
    APPEARANCE_FIELD_NUMBER: _ClassVar[int]
    checkpoint: _checkpoint_pb2.ExecutionCheckpoint
    session_id: str
    execution_id: str
    execution_scope: _common_pb2.EvidenceScope
    cycle: HistoricalCycleContext
    execution_documents: _containers.RepeatedCompositeFieldContainer[HistoricalDocument]
    prompts: _containers.RepeatedCompositeFieldContainer[_prompts_pb2.PromptInstance]
    health: _containers.RepeatedCompositeFieldContainer[_notes_pb2.HealthItem]
    notes: _containers.RepeatedCompositeFieldContainer[_notes_pb2.OperatorNote]
    traceability: _containers.RepeatedCompositeFieldContainer[_common_pb2.TraceabilitySnapshot]
    appearance: _appearance_pb2.AppearanceSnapshot
    def __init__(self, checkpoint: _Optional[_Union[_checkpoint_pb2.ExecutionCheckpoint, _Mapping]] = ..., session_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., execution_scope: _Optional[_Union[_common_pb2.EvidenceScope, _Mapping]] = ..., cycle: _Optional[_Union[HistoricalCycleContext, _Mapping]] = ..., execution_documents: _Optional[_Iterable[_Union[HistoricalDocument, _Mapping]]] = ..., prompts: _Optional[_Iterable[_Union[_prompts_pb2.PromptInstance, _Mapping]]] = ..., health: _Optional[_Iterable[_Union[_notes_pb2.HealthItem, _Mapping]]] = ..., notes: _Optional[_Iterable[_Union[_notes_pb2.OperatorNote, _Mapping]]] = ..., traceability: _Optional[_Iterable[_Union[_common_pb2.TraceabilitySnapshot, _Mapping]]] = ..., appearance: _Optional[_Union[_appearance_pb2.AppearanceSnapshot, _Mapping]] = ...) -> None: ...

class HistoricalCycleContext(_message.Message):
    __slots__ = ("cycle", "execution_order")
    CYCLE_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ORDER_FIELD_NUMBER: _ClassVar[int]
    cycle: HistoricalCycle
    execution_order: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, cycle: _Optional[_Union[HistoricalCycle, _Mapping]] = ..., execution_order: _Optional[_Iterable[str]] = ...) -> None: ...

class HistoricalCycle(_message.Message):
    __slots__ = ("cycle_id", "metadata", "lifecycle", "created_at", "started_at", "updated_at", "cases", "result")
    CYCLE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    LIFECYCLE_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    UPDATED_AT_FIELD_NUMBER: _ClassVar[int]
    CASES_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    cycle_id: str
    metadata: _common_pb2.Metadata
    lifecycle: _execution_pb2.CycleStatus
    created_at: _timestamp_pb2.Timestamp
    started_at: _timestamp_pb2.Timestamp
    updated_at: _timestamp_pb2.Timestamp
    cases: _containers.RepeatedCompositeFieldContainer[_execution_pb2.CaseSummary]
    result: _execution_pb2.ResultSummary
    def __init__(self, cycle_id: _Optional[str] = ..., metadata: _Optional[_Union[_common_pb2.Metadata, _Mapping]] = ..., lifecycle: _Optional[_Union[_execution_pb2.CycleStatus, str]] = ..., created_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., started_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., updated_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., cases: _Optional[_Iterable[_Union[_execution_pb2.CaseSummary, _Mapping]]] = ..., result: _Optional[_Union[_execution_pb2.ResultSummary, _Mapping]] = ...) -> None: ...
