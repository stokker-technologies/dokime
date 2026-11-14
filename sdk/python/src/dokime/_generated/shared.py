# @generated from protocol descriptors; do not edit.
from harmos import ALL as ALL
from harmos import CALLERS_UNSPECIFIED as CALLERS_UNSPECIFIED
from harmos import CLASSIFICATION_CANCELLED as CLASSIFICATION_CANCELLED
from harmos import CLASSIFICATION_PERMANENT as CLASSIFICATION_PERMANENT
from harmos import CLASSIFICATION_REFUSED as CLASSIFICATION_REFUSED
from harmos import CLASSIFICATION_TIMED_OUT as CLASSIFICATION_TIMED_OUT
from harmos import CLASSIFICATION_TRANSIENT as CLASSIFICATION_TRANSIENT
from harmos import CLASSIFICATION_UNSPECIFIED as CLASSIFICATION_UNSPECIFIED
from harmos import CLIENTS as CLIENTS
from harmos import PLUGINS as PLUGINS
from harmos import TRANSFER_FRAMING_BYTES as TRANSFER_FRAMING_BYTES
from harmos import TRANSFER_FRAMING_UNSPECIFIED as TRANSFER_FRAMING_UNSPECIFIED
from harmos import UPLOAD_STATE_ABANDONED as UPLOAD_STATE_ABANDONED
from harmos import UPLOAD_STATE_FINISHED as UPLOAD_STATE_FINISHED
from harmos import UPLOAD_STATE_PRODUCING as UPLOAD_STATE_PRODUCING
from harmos import UPLOAD_STATE_UNSPECIFIED as UPLOAD_STATE_UNSPECIFIED
from harmos import VISIBILITY_PRIVATE as VISIBILITY_PRIVATE
from harmos import VISIBILITY_PROTECTED as VISIBILITY_PROTECTED
from harmos import VISIBILITY_PUBLIC as VISIBILITY_PUBLIC
from harmos import VISIBILITY_UNSPECIFIED as VISIBILITY_UNSPECIFIED
from harmos import Callers as Callers
from harmos import Classification as Classification
from harmos import Declarations as Declarations
from harmos import DiscoveredContract as DiscoveredContract
from harmos import DiscoveredOperation as DiscoveredOperation
from harmos import FailureValue as FailureValue
from harmos import File as File
from harmos import FileMetadata as FileMetadata
from harmos import FileReference as FileReference
from harmos import Grant as Grant
from harmos import Grpc as Grpc
from harmos import History as History
from harmos import InstanceState as InstanceState
from harmos import MessageSettings as MessageSettings
from harmos import OperationContract as OperationContract
from harmos import OperationPolicy as OperationPolicy
from harmos import PageRequest as PageRequest
from harmos import Position as Position
from harmos import Protobuf as Protobuf
from harmos import ReadCut as ReadCut
from harmos import Receipt as Receipt
from harmos import Requirement as Requirement
from harmos import ResourceRevision as ResourceRevision
from harmos import RetryPolicy as RetryPolicy
from harmos import Section as HarmosSection
from harmos import SettingDeclaration as SettingDeclaration
from harmos import SettingsDeclarations as SettingsDeclarations
from harmos import SettingValue as SettingValue
from harmos import SettingValues as SettingValues
from harmos import StorageTransferSettings as StorageTransferSettings
from harmos import StreamDeclaration as StreamDeclaration
from harmos import TransferFraming as TransferFraming
from harmos import Upload as Upload
from harmos import UploadState as UploadState
from harmos import Visibility as Visibility

from dokime._wire.model.v1.analyses_pb2 import Analyses as Analyses
from dokime._wire.model.v1.analyses_pb2 import Analysis as Analysis
from dokime._wire.model.v1.analyses_pb2 import AnalysisOutput as AnalysisOutput
from dokime._wire.model.v1.analyses_pb2 import AnalysisSelection as AnalysisSelection
from dokime._wire.model.v1.analyses_pb2 import AnalysisValues as AnalysisValues
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_ARTIFACTS as EVIDENCE_CARD_ARTIFACTS
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_CRITERIA as EVIDENCE_CARD_CRITERIA
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_HEALTH as EVIDENCE_CARD_HEALTH
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_MEDIA as EVIDENCE_CARD_MEDIA
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_NOTES as EVIDENCE_CARD_NOTES
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_RECORDS as EVIDENCE_CARD_RECORDS
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_STEPS as EVIDENCE_CARD_STEPS
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_STREAM_CONTEXT as EVIDENCE_CARD_STREAM_CONTEXT
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_TRACEABILITY as EVIDENCE_CARD_TRACEABILITY
from dokime._wire.model.v1.documents_pb2 import EVIDENCE_CARD_UNSPECIFIED as EVIDENCE_CARD_UNSPECIFIED
from dokime._wire.model.v1.documents_pb2 import Card as Card
from dokime._wire.model.v1.documents_pb2 import CardLayout as CardLayout
from dokime._wire.model.v1.documents_pb2 import Document as Document
from dokime._wire.model.v1.documents_pb2 import EvidenceCard as EvidenceCard
from dokime._wire.model.v1.documents_pb2 import Section as Section
from dokime._wire.model.v1.documents_pb2 import SectionRepeat as SectionRepeat
from dokime._wire.model.v1.documents_pb2 import Summary as Summary
from dokime._wire.model.v1.documents_pb2 import SummaryTile as SummaryTile
from dokime._wire.model.v1.documents_pb2 import Table as Table
from dokime._wire.model.v1.documents_pb2 import TableRow as TableRow
from dokime._wire.model.v1.evidence_pb2 import ARTIFACT_STATE_CANCELLED as ARTIFACT_STATE_CANCELLED
from dokime._wire.model.v1.evidence_pb2 import ARTIFACT_STATE_COMPLETE as ARTIFACT_STATE_COMPLETE
from dokime._wire.model.v1.evidence_pb2 import ARTIFACT_STATE_FAILED as ARTIFACT_STATE_FAILED
from dokime._wire.model.v1.evidence_pb2 import ARTIFACT_STATE_REJECTED as ARTIFACT_STATE_REJECTED
from dokime._wire.model.v1.evidence_pb2 import ARTIFACT_STATE_UNSPECIFIED as ARTIFACT_STATE_UNSPECIFIED
from dokime._wire.model.v1.evidence_pb2 import SEVERITY_DEBUG as SEVERITY_DEBUG
from dokime._wire.model.v1.evidence_pb2 import SEVERITY_ERROR as SEVERITY_ERROR
from dokime._wire.model.v1.evidence_pb2 import SEVERITY_INFO as SEVERITY_INFO
from dokime._wire.model.v1.evidence_pb2 import SEVERITY_UNSPECIFIED as SEVERITY_UNSPECIFIED
from dokime._wire.model.v1.evidence_pb2 import SEVERITY_WARNING as SEVERITY_WARNING
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_EQ as STREAM_MATCH_OPERATOR_EQ
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_EXISTS as STREAM_MATCH_OPERATOR_EXISTS
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_GE as STREAM_MATCH_OPERATOR_GE
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_GT as STREAM_MATCH_OPERATOR_GT
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_LE as STREAM_MATCH_OPERATOR_LE
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_LT as STREAM_MATCH_OPERATOR_LT
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_UNSPECIFIED as STREAM_MATCH_OPERATOR_UNSPECIFIED
from dokime._wire.model.v1.evidence_pb2 import STREAM_MATCH_OPERATOR_WITHIN as STREAM_MATCH_OPERATOR_WITHIN
from dokime._wire.model.v1.evidence_pb2 import ArtifactMetadata as ArtifactMetadata
from dokime._wire.model.v1.evidence_pb2 import ArtifactScope as ArtifactScope
from dokime._wire.model.v1.evidence_pb2 import ArtifactState as ArtifactState
from dokime._wire.model.v1.evidence_pb2 import EvidenceCondition as EvidenceCondition
from dokime._wire.model.v1.evidence_pb2 import EvidenceConditions as EvidenceConditions
from dokime._wire.model.v1.evidence_pb2 import EvidenceOrigin as EvidenceOrigin
from dokime._wire.model.v1.evidence_pb2 import EvidenceOriginKind as EvidenceOriginKind
from dokime._wire.model.v1.evidence_pb2 import EvidenceScope as EvidenceScope
from dokime._wire.model.v1.evidence_pb2 import RecordMatch as RecordMatch
from dokime._wire.model.v1.evidence_pb2 import Severity as Severity
from dokime._wire.model.v1.evidence_pb2 import StreamMatch as StreamMatch
from dokime._wire.model.v1.evidence_pb2 import StreamMatchOperator as StreamMatchOperator
from dokime._wire.model.v1.evidence_pb2 import Window as Window
from dokime._wire.model.v1.execution_pb2 import TERMINATION_CANCELLED as TERMINATION_CANCELLED
from dokime._wire.model.v1.execution_pb2 import TERMINATION_COMPLETED as TERMINATION_COMPLETED
from dokime._wire.model.v1.execution_pb2 import TERMINATION_FAILED as TERMINATION_FAILED
from dokime._wire.model.v1.execution_pb2 import TERMINATION_INTERRUPTED as TERMINATION_INTERRUPTED
from dokime._wire.model.v1.execution_pb2 import TERMINATION_UNSPECIFIED as TERMINATION_UNSPECIFIED
from dokime._wire.model.v1.execution_pb2 import Criterion as Criterion
from dokime._wire.model.v1.execution_pb2 import ExecutionProgressValue as ExecutionProgressValue
from dokime._wire.model.v1.execution_pb2 import Import as Import
from dokime._wire.model.v1.execution_pb2 import ImportSource as ImportSource
from dokime._wire.model.v1.execution_pb2 import Origin as Origin
from dokime._wire.model.v1.execution_pb2 import PluginRun as PluginRun
from dokime._wire.model.v1.execution_pb2 import Step as Step
from dokime._wire.model.v1.execution_pb2 import Termination as Termination
from dokime._wire.model.v1.execution_pb2 import TestCase as TestCase
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_COUNT as AGGREGATE_COUNT
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_FIRST as AGGREGATE_FIRST
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_LAST as AGGREGATE_LAST
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_MAX as AGGREGATE_MAX
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_MEAN as AGGREGATE_MEAN
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_MEDIAN as AGGREGATE_MEDIAN
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_MIN as AGGREGATE_MIN
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_PERCENTILE as AGGREGATE_PERCENTILE
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_RMS as AGGREGATE_RMS
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_STDDEV as AGGREGATE_STDDEV
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_SUM as AGGREGATE_SUM
from dokime._wire.model.v1.expressions_pb2 import AGGREGATE_UNSPECIFIED as AGGREGATE_UNSPECIFIED
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_CONTAINS as FILTER_OPERATOR_CONTAINS
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_EQ as FILTER_OPERATOR_EQ
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_EXISTS as FILTER_OPERATOR_EXISTS
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_GE as FILTER_OPERATOR_GE
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_GT as FILTER_OPERATOR_GT
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_LE as FILTER_OPERATOR_LE
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_LT as FILTER_OPERATOR_LT
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_NE as FILTER_OPERATOR_NE
from dokime._wire.model.v1.expressions_pb2 import FILTER_OPERATOR_UNSPECIFIED as FILTER_OPERATOR_UNSPECIFIED
from dokime._wire.model.v1.expressions_pb2 import Aggregate as Aggregate
from dokime._wire.model.v1.expressions_pb2 import AggregateExpression as AggregateExpression
from dokime._wire.model.v1.expressions_pb2 import DataExpression as DataExpression
from dokime._wire.model.v1.expressions_pb2 import DocumentFilter as DocumentFilter
from dokime._wire.model.v1.expressions_pb2 import FieldReference as FieldReference
from dokime._wire.model.v1.expressions_pb2 import FilterOperator as FilterOperator
from dokime._wire.model.v1.expressions_pb2 import TransformExpression as TransformExpression
from dokime._wire.model.v1.files_pb2 import CAPTURE_GAP_REASON_REPLACED as CAPTURE_GAP_REASON_REPLACED
from dokime._wire.model.v1.files_pb2 import CAPTURE_GAP_REASON_RESUME_UNSUPPORTED as CAPTURE_GAP_REASON_RESUME_UNSUPPORTED
from dokime._wire.model.v1.files_pb2 import CAPTURE_GAP_REASON_TRUNCATED as CAPTURE_GAP_REASON_TRUNCATED
from dokime._wire.model.v1.files_pb2 import CAPTURE_GAP_REASON_UNSPECIFIED as CAPTURE_GAP_REASON_UNSPECIFIED
from dokime._wire.model.v1.files_pb2 import CAPTURE_GAP_REASON_VANISHED as CAPTURE_GAP_REASON_VANISHED
from dokime._wire.model.v1.files_pb2 import FILE_RULE_ACTION_EXCLUDE as FILE_RULE_ACTION_EXCLUDE
from dokime._wire.model.v1.files_pb2 import FILE_RULE_ACTION_INCLUDE as FILE_RULE_ACTION_INCLUDE
from dokime._wire.model.v1.files_pb2 import FILE_RULE_ACTION_UNSPECIFIED as FILE_RULE_ACTION_UNSPECIFIED
from dokime._wire.model.v1.files_pb2 import CapturedArtifact as CapturedArtifact
from dokime._wire.model.v1.files_pb2 import CaptureGap as CaptureGap
from dokime._wire.model.v1.files_pb2 import CaptureGapReason as CaptureGapReason
from dokime._wire.model.v1.files_pb2 import FileLocation as FileLocation
from dokime._wire.model.v1.files_pb2 import FileRule as FileRule
from dokime._wire.model.v1.files_pb2 import FileRuleAction as FileRuleAction
from dokime._wire.model.v1.files_pb2 import FileSelection as FileSelection
from dokime._wire.model.v1.files_pb2 import FileSource as FileSource
from dokime._wire.model.v1.files_pb2 import FtpLocation as FtpLocation
from dokime._wire.model.v1.metadata_pb2 import Metadata as Metadata
from dokime._wire.model.v1.parameters_pb2 import DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL as DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL
from dokime._wire.model.v1.parameters_pb2 import DYNAMIC_DEFAULT_KEY_OPERATOR_NAME as DYNAMIC_DEFAULT_KEY_OPERATOR_NAME
from dokime._wire.model.v1.parameters_pb2 import DYNAMIC_DEFAULT_KEY_UNSPECIFIED as DYNAMIC_DEFAULT_KEY_UNSPECIFIED
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_BINARY_OPERATOR_ADD as PARAMETER_BINARY_OPERATOR_ADD
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_BINARY_OPERATOR_DIVIDE as PARAMETER_BINARY_OPERATOR_DIVIDE
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_BINARY_OPERATOR_MULTIPLY as PARAMETER_BINARY_OPERATOR_MULTIPLY
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_BINARY_OPERATOR_SUBTRACT as PARAMETER_BINARY_OPERATOR_SUBTRACT
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_BINARY_OPERATOR_UNSPECIFIED as PARAMETER_BINARY_OPERATOR_UNSPECIFIED
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_UNARY_OPERATOR_NEGATE as PARAMETER_UNARY_OPERATOR_NEGATE
from dokime._wire.model.v1.parameters_pb2 import PARAMETER_UNARY_OPERATOR_UNSPECIFIED as PARAMETER_UNARY_OPERATOR_UNSPECIFIED
from dokime._wire.model.v1.parameters_pb2 import DynamicDefault as DynamicDefault
from dokime._wire.model.v1.parameters_pb2 import DynamicDefaultKey as DynamicDefaultKey
from dokime._wire.model.v1.parameters_pb2 import FileParameter as FileParameter
from dokime._wire.model.v1.parameters_pb2 import IntegerRange as IntegerRange
from dokime._wire.model.v1.parameters_pb2 import LinearSpace as LinearSpace
from dokime._wire.model.v1.parameters_pb2 import NumberRange as NumberRange
from dokime._wire.model.v1.parameters_pb2 import Parameter as Parameter
from dokime._wire.model.v1.parameters_pb2 import ParameterBinary as ParameterBinary
from dokime._wire.model.v1.parameters_pb2 import ParameterBinaryOperator as ParameterBinaryOperator
from dokime._wire.model.v1.parameters_pb2 import ParameterExpansion as ParameterExpansion
from dokime._wire.model.v1.parameters_pb2 import ParameterExpression as ParameterExpression
from dokime._wire.model.v1.parameters_pb2 import ParameterExpressions as ParameterExpressions
from dokime._wire.model.v1.parameters_pb2 import ParameterUnary as ParameterUnary
from dokime._wire.model.v1.parameters_pb2 import ParameterUnaryOperator as ParameterUnaryOperator
from dokime._wire.model.v1.plots_pb2 import AXIS_SCALE_LINEAR as AXIS_SCALE_LINEAR
from dokime._wire.model.v1.plots_pb2 import AXIS_SCALE_LOG as AXIS_SCALE_LOG
from dokime._wire.model.v1.plots_pb2 import AXIS_SCALE_UNSPECIFIED as AXIS_SCALE_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import BAR_ARRANGEMENT_GROUPED as BAR_ARRANGEMENT_GROUPED
from dokime._wire.model.v1.plots_pb2 import BAR_ARRANGEMENT_STACKED as BAR_ARRANGEMENT_STACKED
from dokime._wire.model.v1.plots_pb2 import BAR_ARRANGEMENT_UNSPECIFIED as BAR_ARRANGEMENT_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import CONTOUR_RENDER_BANDS as CONTOUR_RENDER_BANDS
from dokime._wire.model.v1.plots_pb2 import CONTOUR_RENDER_CELLS as CONTOUR_RENDER_CELLS
from dokime._wire.model.v1.plots_pb2 import CONTOUR_RENDER_UNSPECIFIED as CONTOUR_RENDER_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import DOWNSAMPLE_AUTO as DOWNSAMPLE_AUTO
from dokime._wire.model.v1.plots_pb2 import DOWNSAMPLE_LTTB as DOWNSAMPLE_LTTB
from dokime._wire.model.v1.plots_pb2 import DOWNSAMPLE_MIN_MAX as DOWNSAMPLE_MIN_MAX
from dokime._wire.model.v1.plots_pb2 import DOWNSAMPLE_RAW as DOWNSAMPLE_RAW
from dokime._wire.model.v1.plots_pb2 import DOWNSAMPLE_UNSPECIFIED as DOWNSAMPLE_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import FFT_DETREND_MEAN as FFT_DETREND_MEAN
from dokime._wire.model.v1.plots_pb2 import FFT_DETREND_NONE as FFT_DETREND_NONE
from dokime._wire.model.v1.plots_pb2 import FFT_DETREND_UNSPECIFIED as FFT_DETREND_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import FFT_WINDOW_HANN as FFT_WINDOW_HANN
from dokime._wire.model.v1.plots_pb2 import FFT_WINDOW_RECTANGULAR as FFT_WINDOW_RECTANGULAR
from dokime._wire.model.v1.plots_pb2 import FFT_WINDOW_UNSPECIFIED as FFT_WINDOW_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import FREQUENCY_LEVEL_UNIT_DB as FREQUENCY_LEVEL_UNIT_DB
from dokime._wire.model.v1.plots_pb2 import FREQUENCY_LEVEL_UNIT_DBM as FREQUENCY_LEVEL_UNIT_DBM
from dokime._wire.model.v1.plots_pb2 import FREQUENCY_LEVEL_UNIT_DBU as FREQUENCY_LEVEL_UNIT_DBU
from dokime._wire.model.v1.plots_pb2 import FREQUENCY_LEVEL_UNIT_LINEAR as FREQUENCY_LEVEL_UNIT_LINEAR
from dokime._wire.model.v1.plots_pb2 import FREQUENCY_LEVEL_UNIT_UNSPECIFIED as FREQUENCY_LEVEL_UNIT_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import LINE_STYLE_LINE as LINE_STYLE_LINE
from dokime._wire.model.v1.plots_pb2 import LINE_STYLE_LINE_POINTS as LINE_STYLE_LINE_POINTS
from dokime._wire.model.v1.plots_pb2 import LINE_STYLE_POINTS as LINE_STYLE_POINTS
from dokime._wire.model.v1.plots_pb2 import LINE_STYLE_STEP as LINE_STYLE_STEP
from dokime._wire.model.v1.plots_pb2 import LINE_STYLE_UNSPECIFIED as LINE_STYLE_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_BAR as PLOT_KIND_BAR
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_BOX_PLOT as PLOT_KIND_BOX_PLOT
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_CONTOUR as PLOT_KIND_CONTOUR
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_FREQUENCY_SERIES as PLOT_KIND_FREQUENCY_SERIES
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_GRID_CHART as PLOT_KIND_GRID_CHART
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_HISTOGRAM as PLOT_KIND_HISTOGRAM
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_PIE as PLOT_KIND_PIE
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_TIME_SERIES as PLOT_KIND_TIME_SERIES
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_TRAJECTORY as PLOT_KIND_TRAJECTORY
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_UNSPECIFIED as PLOT_KIND_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import PLOT_KIND_XY as PLOT_KIND_XY
from dokime._wire.model.v1.plots_pb2 import PLOT_STYLE_LINE as PLOT_STYLE_LINE
from dokime._wire.model.v1.plots_pb2 import PLOT_STYLE_LINE_POINTS as PLOT_STYLE_LINE_POINTS
from dokime._wire.model.v1.plots_pb2 import PLOT_STYLE_POINTS as PLOT_STYLE_POINTS
from dokime._wire.model.v1.plots_pb2 import PLOT_STYLE_STEP as PLOT_STYLE_STEP
from dokime._wire.model.v1.plots_pb2 import PLOT_STYLE_UNSPECIFIED as PLOT_STYLE_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import REDUCTION_MAX as REDUCTION_MAX
from dokime._wire.model.v1.plots_pb2 import REDUCTION_MEAN as REDUCTION_MEAN
from dokime._wire.model.v1.plots_pb2 import REDUCTION_MIN as REDUCTION_MIN
from dokime._wire.model.v1.plots_pb2 import REDUCTION_UNSPECIFIED as REDUCTION_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import XY_INTERVAL_SIGMA as XY_INTERVAL_SIGMA
from dokime._wire.model.v1.plots_pb2 import XY_INTERVAL_UNSPECIFIED as XY_INTERVAL_UNSPECIFIED
from dokime._wire.model.v1.plots_pb2 import AxisScale as AxisScale
from dokime._wire.model.v1.plots_pb2 import BarArrangement as BarArrangement
from dokime._wire.model.v1.plots_pb2 import ContourRender as ContourRender
from dokime._wire.model.v1.plots_pb2 import Downsample as Downsample
from dokime._wire.model.v1.plots_pb2 import FftDetrend as FftDetrend
from dokime._wire.model.v1.plots_pb2 import FftWindow as FftWindow
from dokime._wire.model.v1.plots_pb2 import FrequencyLevelUnit as FrequencyLevelUnit
from dokime._wire.model.v1.plots_pb2 import GridAxisEntry as GridAxisEntry
from dokime._wire.model.v1.plots_pb2 import GridChart as GridChart
from dokime._wire.model.v1.plots_pb2 import GridDefinition as GridDefinition
from dokime._wire.model.v1.plots_pb2 import GridHighlight as GridHighlight
from dokime._wire.model.v1.plots_pb2 import LineStyle as LineStyle
from dokime._wire.model.v1.plots_pb2 import Plot as Plot
from dokime._wire.model.v1.plots_pb2 import PlotFftOptions as PlotFftOptions
from dokime._wire.model.v1.plots_pb2 import PlotFilter as PlotFilter
from dokime._wire.model.v1.plots_pb2 import PlotHistogramOptions as PlotHistogramOptions
from dokime._wire.model.v1.plots_pb2 import PlotIntervalOptions as PlotIntervalOptions
from dokime._wire.model.v1.plots_pb2 import PlotKind as PlotKind
from dokime._wire.model.v1.plots_pb2 import PlotStyle as PlotStyle
from dokime._wire.model.v1.plots_pb2 import Reduction as Reduction
from dokime._wire.model.v1.plots_pb2 import ReferenceBand as ReferenceBand
from dokime._wire.model.v1.plots_pb2 import ReferenceLine as ReferenceLine
from dokime._wire.model.v1.plots_pb2 import XyInterval as XyInterval
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_AND as ANSWER_OPERATOR_AND
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_CONTAINS as ANSWER_OPERATOR_CONTAINS
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_ENDS_WITH as ANSWER_OPERATOR_ENDS_WITH
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_EQ as ANSWER_OPERATOR_EQ
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_GE as ANSWER_OPERATOR_GE
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_GT as ANSWER_OPERATOR_GT
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_LE as ANSWER_OPERATOR_LE
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_LT as ANSWER_OPERATOR_LT
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_NE as ANSWER_OPERATOR_NE
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_OR as ANSWER_OPERATOR_OR
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_STARTS_WITH as ANSWER_OPERATOR_STARTS_WITH
from dokime._wire.model.v1.prompts_pb2 import ANSWER_OPERATOR_UNSPECIFIED as ANSWER_OPERATOR_UNSPECIFIED
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ACTION_EMPHASIS_DANGER as PROMPT_ACTION_EMPHASIS_DANGER
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ACTION_EMPHASIS_PRIMARY as PROMPT_ACTION_EMPHASIS_PRIMARY
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ACTION_EMPHASIS_SECONDARY as PROMPT_ACTION_EMPHASIS_SECONDARY
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ACTION_EMPHASIS_UNSPECIFIED as PROMPT_ACTION_EMPHASIS_UNSPECIFIED
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ARCHETYPE_CHOICE as PROMPT_ARCHETYPE_CHOICE
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ARCHETYPE_CONFIRMATION as PROMPT_ARCHETYPE_CONFIRMATION
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ARCHETYPE_FORM as PROMPT_ARCHETYPE_FORM
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ARCHETYPE_STATUS as PROMPT_ARCHETYPE_STATUS
from dokime._wire.model.v1.prompts_pb2 import PROMPT_ARCHETYPE_UNSPECIFIED as PROMPT_ARCHETYPE_UNSPECIFIED
from dokime._wire.model.v1.prompts_pb2 import PROMPT_FIELD_KIND_BOOLEAN as PROMPT_FIELD_KIND_BOOLEAN
from dokime._wire.model.v1.prompts_pb2 import PROMPT_FIELD_KIND_CHOICE as PROMPT_FIELD_KIND_CHOICE
from dokime._wire.model.v1.prompts_pb2 import PROMPT_FIELD_KIND_FLOAT as PROMPT_FIELD_KIND_FLOAT
from dokime._wire.model.v1.prompts_pb2 import PROMPT_FIELD_KIND_INTEGER as PROMPT_FIELD_KIND_INTEGER
from dokime._wire.model.v1.prompts_pb2 import PROMPT_FIELD_KIND_TEXT as PROMPT_FIELD_KIND_TEXT
from dokime._wire.model.v1.prompts_pb2 import PROMPT_FIELD_KIND_UNSPECIFIED as PROMPT_FIELD_KIND_UNSPECIFIED
from dokime._wire.model.v1.prompts_pb2 import AnswerBinaryPredicate as AnswerBinaryPredicate
from dokime._wire.model.v1.prompts_pb2 import AnswerOperator as AnswerOperator
from dokime._wire.model.v1.prompts_pb2 import AnswerPredicate as AnswerPredicate
from dokime._wire.model.v1.prompts_pb2 import AnswerRule as AnswerRule
from dokime._wire.model.v1.prompts_pb2 import Prompt as Prompt
from dokime._wire.model.v1.prompts_pb2 import PromptAction as PromptAction
from dokime._wire.model.v1.prompts_pb2 import PromptActionEmphasis as PromptActionEmphasis
from dokime._wire.model.v1.prompts_pb2 import PromptArchetype as PromptArchetype
from dokime._wire.model.v1.prompts_pb2 import PromptDefinitions as PromptDefinitions
from dokime._wire.model.v1.prompts_pb2 import PromptField as PromptField
from dokime._wire.model.v1.prompts_pb2 import PromptFieldKind as PromptFieldKind
from dokime._wire.model.v1.prompts_pb2 import PromptFieldValidation as PromptFieldValidation
from dokime._wire.model.v1.prompts_pb2 import PromptValues as PromptValues
from dokime._wire.model.v1.record_contracts_pb2 import Listeners as Listeners
from dokime._wire.model.v1.record_contracts_pb2 import RecordContract as RecordContract
from dokime._wire.model.v1.record_contracts_pb2 import RecordContracts as RecordContracts
from dokime._wire.model.v1.record_contracts_pb2 import RecordListener as RecordListener
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_BOOLEAN as VALUE_TYPE_BOOLEAN
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_BYTES as VALUE_TYPE_BYTES
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_DATETIME as VALUE_TYPE_DATETIME
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_INTEGER as VALUE_TYPE_INTEGER
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_LIST as VALUE_TYPE_LIST
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_NUMBER as VALUE_TYPE_NUMBER
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_TEXT as VALUE_TYPE_TEXT
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_UNSIGNED_INTEGER as VALUE_TYPE_UNSIGNED_INTEGER
from dokime._wire.model.v1.scalar_pb2 import VALUE_TYPE_UNSPECIFIED as VALUE_TYPE_UNSPECIFIED
from dokime._wire.model.v1.scalar_pb2 import ExactNumber as ExactNumber
from dokime._wire.model.v1.scalar_pb2 import MapValue as MapValue
from dokime._wire.model.v1.scalar_pb2 import Null as Null
from dokime._wire.model.v1.scalar_pb2 import Value as Value
from dokime._wire.model.v1.scalar_pb2 import ValueList as ValueList
from dokime._wire.model.v1.scalar_pb2 import ValueType as ValueType
from dokime._wire.model.v1.secrets_pb2 import SecretDeclaration as SecretDeclaration
from dokime._wire.model.v1.secrets_pb2 import SecretsDeclarations as SecretsDeclarations
from dokime._wire.model.v1.suites_pb2 import Suite as Suite
from dokime._wire.model.v1.suites_pb2 import Suites as Suites
from dokime._wire.model.v1.suites_pb2 import TestCaseTemplate as TestCaseTemplate
from dokime._wire.model.v1.traceability_pb2 import TraceabilityEntry as TraceabilityEntry

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
    "Aggregate",
    "AggregateExpression",
    "Analyses",
    "Analysis",
    "AnalysisOutput",
    "AnalysisSelection",
    "AnalysisValues",
    "AnswerBinaryPredicate",
    "AnswerOperator",
    "AnswerPredicate",
    "AnswerRule",
    "ArtifactMetadata",
    "ArtifactScope",
    "ArtifactState",
    "AxisScale",
    "BAR_ARRANGEMENT_GROUPED",
    "BAR_ARRANGEMENT_STACKED",
    "BAR_ARRANGEMENT_UNSPECIFIED",
    "BarArrangement",
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
    "Callers",
    "CaptureGap",
    "CaptureGapReason",
    "CapturedArtifact",
    "Card",
    "CardLayout",
    "Classification",
    "ContourRender",
    "Criterion",
    "DOWNSAMPLE_AUTO",
    "DOWNSAMPLE_LTTB",
    "DOWNSAMPLE_MIN_MAX",
    "DOWNSAMPLE_RAW",
    "DOWNSAMPLE_UNSPECIFIED",
    "DYNAMIC_DEFAULT_KEY_OPERATOR_EMAIL",
    "DYNAMIC_DEFAULT_KEY_OPERATOR_NAME",
    "DYNAMIC_DEFAULT_KEY_UNSPECIFIED",
    "DataExpression",
    "Declarations",
    "DiscoveredContract",
    "DiscoveredOperation",
    "Document",
    "DocumentFilter",
    "Downsample",
    "DynamicDefault",
    "DynamicDefaultKey",
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
    "EvidenceCard",
    "EvidenceCondition",
    "EvidenceConditions",
    "EvidenceOrigin",
    "EvidenceOriginKind",
    "EvidenceScope",
    "ExactNumber",
    "ExecutionProgressValue",
    "FFT_DETREND_MEAN",
    "FFT_DETREND_NONE",
    "FFT_DETREND_UNSPECIFIED",
    "FFT_WINDOW_HANN",
    "FFT_WINDOW_RECTANGULAR",
    "FFT_WINDOW_UNSPECIFIED",
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
    "FailureValue",
    "FftDetrend",
    "FftWindow",
    "FieldReference",
    "File",
    "FileLocation",
    "FileMetadata",
    "FileParameter",
    "FileReference",
    "FileRule",
    "FileRuleAction",
    "FileSelection",
    "FileSource",
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
    "LINE_STYLE_LINE",
    "LINE_STYLE_LINE_POINTS",
    "LINE_STYLE_POINTS",
    "LINE_STYLE_STEP",
    "LINE_STYLE_UNSPECIFIED",
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
    "Origin",
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
    "REDUCTION_MAX",
    "REDUCTION_MEAN",
    "REDUCTION_MIN",
    "REDUCTION_UNSPECIFIED",
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
    "SEVERITY_DEBUG",
    "SEVERITY_ERROR",
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
    "SecretDeclaration",
    "SecretsDeclarations",
    "Section",
    "SectionRepeat",
    "SettingDeclaration",
    "SettingValue",
    "SettingValues",
    "SettingsDeclarations",
    "Severity",
    "Step",
    "StorageTransferSettings",
    "StreamDeclaration",
    "StreamMatch",
    "StreamMatchOperator",
    "Suite",
    "Suites",
    "Summary",
    "SummaryTile",
    "TERMINATION_CANCELLED",
    "TERMINATION_COMPLETED",
    "TERMINATION_FAILED",
    "TERMINATION_INTERRUPTED",
    "TERMINATION_UNSPECIFIED",
    "TRANSFER_FRAMING_BYTES",
    "TRANSFER_FRAMING_UNSPECIFIED",
    "Table",
    "TableRow",
    "Termination",
    "TestCase",
    "TestCaseTemplate",
    "TraceabilityEntry",
    "TransferFraming",
    "TransformExpression",
    "UPLOAD_STATE_ABANDONED",
    "UPLOAD_STATE_FINISHED",
    "UPLOAD_STATE_PRODUCING",
    "UPLOAD_STATE_UNSPECIFIED",
    "Upload",
    "UploadState",
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
    "Value",
    "ValueList",
    "ValueType",
    "Visibility",
    "Window",
    "XY_INTERVAL_SIGMA",
    "XY_INTERVAL_UNSPECIFIED",
    "XyInterval",
]
