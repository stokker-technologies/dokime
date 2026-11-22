# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Template Declarations, Execution Contexts and Document Values."""

# === Imports === #

# Standard Library Imports

# Third Party Imports

# Local Imports
from harmos import plugin

from .. import _authoring, _evidence, _values
from .._generated import shared
from . import _captures, _context, _documents, _parameters, _plots

# Type Checking Imports


# === Public Exports === #

Window = plugin.Window
Materialization = _authoring.Materialization
PromptDeclaration = _authoring.PromptDeclaration
Record = _authoring.Record
Stream = _authoring.Stream
TestTemplate = _authoring.TestTemplate
criterion = _authoring.criterion
materialize = _authoring.materialize
observer = _authoring.observer
prompt = _authoring.prompt
record = _authoring.record
step = _authoring.step
stream = _authoring.stream
window_observer = _authoring.window_observer
Card = shared.Card
DataExpression = shared.DataExpression
FileParameter = shared.FileParameter
GridAxisEntry = shared.GridAxisEntry
GridHighlight = shared.GridHighlight
Prompt = shared.Prompt
ReferenceBand = shared.ReferenceBand
ReferenceLine = shared.ReferenceLine
CapturedArtifact = _captures.CapturedArtifact
CaptureGap = _captures.CaptureGap
CaptureMode = _captures.CaptureMode
CaptureResult = _captures.CaptureResult
Environment = _captures.Environment
FileLocation = _captures.FileLocation
FileRule = _captures.FileRule
FileRuleAction = _captures.FileRuleAction
FileSelection = _captures.FileSelection
FileSource = _captures.FileSource
FtpLocation = _captures.FtpLocation
LineField = _captures.LineField
LineMarker = _captures.LineMarker
LineMatcher = _captures.LineMatcher
MirrorCapture = _captures.MirrorCapture
Severity = _captures.Severity
StreamCapture = _captures.StreamCapture
Termination = _values.Termination
Context = _context.Context
RecordsQuery = _evidence.RecordsQuery
SamplesQuery = _evidence.SamplesQuery
RecordRow = _values.RecordRow
SampleRow = _values.SampleRow
Statistics = _values.Statistics
Document = _documents.Document
Section = _documents.Section
Summary = _documents.Summary
plot_card = _documents.plot_card
records_card = _documents.records_card
Parameter = _parameters.Parameter
Aggregate = _plots.Aggregate
AxisScale = _plots.AxisScale
BarArrangement = _plots.BarArrangement
ContourRender = _plots.ContourRender
Expression = _plots.Expression
FilterOperator = _plots.FilterOperator
Plot = _plots.Plot
PlotStyle = _plots.PlotStyle
Reduction = _plots.Reduction
XyInterval = _plots.XyInterval
bar = _plots.bar
box_plot = _plots.box_plot
contour = _plots.contour
field = _plots.field
frequency_series = _plots.frequency_series
grid_chart = _plots.grid_chart
heat_strip = _plots.heat_strip
histogram = _plots.histogram
pie = _plots.pie
time_series = _plots.time_series
trajectory = _plots.trajectory
xy = _plots.xy

# === Constants === #

# === Types === #

ParameterDefinition = shared.Parameter

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
