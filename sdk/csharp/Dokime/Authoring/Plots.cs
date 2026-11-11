/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using Dokime.V1;

namespace Dokime.Authoring;

// Each Convenience Produces A Canonical Message And Copies Its Operands.
public static class Plots
{
    public static Plot TimeSeries(DataExpression value) => new() { Kind = PlotKind.TimeSeries, Y = { value.Clone() } };
    public static Plot FrequencySeries(DataExpression value) => new() { Kind = PlotKind.FrequencySeries, Y = { value.Clone() } };
    public static Plot Xy(DataExpression value) => new() { Kind = PlotKind.Xy, Y = { value.Clone() } };
    public static Plot Trajectory(DataExpression value) => new() { Kind = PlotKind.Trajectory, Y = { value.Clone() } };
    public static Plot Contour(DataExpression value) => new() { Kind = PlotKind.Contour, Z = value.Clone() };
    public static Plot GridChart(DataExpression value) => new() { Kind = PlotKind.GridChart, Y = { value.Clone() } };
    public static Plot Bar(DataExpression value) => new() { Kind = PlotKind.Bar, Y = { value.Clone() } };
    public static Plot Pie(DataExpression value) => new() { Kind = PlotKind.Pie, Y = { value.Clone() } };
    public static Plot Histogram(DataExpression value) => new() { Kind = PlotKind.Histogram, X = value.Clone() };
    public static Plot BoxPlot(DataExpression value) => new() { Kind = PlotKind.BoxPlot, Y = { value.Clone() } };
}

public static class Expressions
{
    public static DataExpression Field(string stream, string field) => new() { Stream = new FieldReference { Identifier = stream, Field = field } };
    public static DataExpression Scale(this DataExpression input, double factor) => new() { Label = input.Label, Transform = new TransformExpression { Input = input.Clone(), Scale = factor } };
    public static DataExpression Offset(this DataExpression input, double offset) => new() { Label = input.Label, Transform = new TransformExpression { Input = input.Clone(), Offset = offset } };
    public static DataExpression Reduce(this DataExpression input, Aggregate kind) => new() { Label = input.Label, Aggregate = new AggregateExpression { Input = input.Clone(), Kind = kind } };
    public static DataExpression Mean(this DataExpression input) => input.Reduce(Aggregate.Mean);
    public static DataExpression Std(this DataExpression input) => input.Reduce(Aggregate.Stddev);
}

public static class PlotOptions
{
    public static Plot WithX(this Plot plot, DataExpression value) { plot.X = value.Clone(); return plot; }
    public static Plot WithY(this Plot plot, DataExpression value) { plot.Y.Add(value.Clone()); return plot; }
    public static Plot SeriesBy(this Plot plot, DataExpression value) { plot.SeriesBy = value.Clone(); return plot; }
    public static Plot WithLabel(this Plot plot, Metadata metadata) { plot.Metadata = metadata.Clone(); return plot; }
    public static Plot WithUnit(this Plot plot, string unit) { plot.Unit = unit; return plot; }
    public static Plot Reduce(this Plot plot, Reduction reduction) { plot.Reduction = reduction; return plot; }
    public static Plot Render(this Plot plot, ContourRender render) { plot.Render = render; return plot; }
    public static Plot Arrange(this Plot plot, BarArrangement arrangement) { plot.Arrangement = arrangement; return plot; }
    public static Plot WithInterval(this Plot plot, XyInterval interval) { plot.Interval = interval; return plot; }
    public static Plot WithPareto(this Plot plot, bool enabled) { plot.Pareto = enabled; return plot; }
    public static Plot WithHole(this Plot plot, double fraction) { plot.Hole = fraction; return plot; }
    public static Plot Row(this Plot plot, DataExpression value) { plot.Grid ??= new(); plot.Grid.Row = value.Clone(); return plot; }
    public static Plot Column(this Plot plot, DataExpression value) { plot.Grid ??= new(); plot.Grid.Column = value.Clone(); return plot; }
    public static Plot Frame(this Plot plot, DataExpression value) { plot.Grid ??= new(); plot.Grid.Frame = value.Clone(); return plot; }
    public static Plot Rows(this Plot plot, System.Collections.Generic.IEnumerable<GridAxisEntry> entries) { plot.Grid ??= new(); plot.Grid.Rows.Clear(); foreach (var entry in entries) plot.Grid.Rows.Add(entry.Clone()); return plot; }
    public static Plot Columns(this Plot plot, System.Collections.Generic.IEnumerable<GridAxisEntry> entries) { plot.Grid ??= new(); plot.Grid.Columns.Clear(); foreach (var entry in entries) plot.Grid.Columns.Add(entry.Clone()); return plot; }
    public static Plot Highlight(this Plot plot, GridHighlight value) { plot.Grid ??= new(); plot.Grid.Highlights.Add(value.Clone()); return plot; }
}

public static class Documents
{
    public static Document Section(this Document document, Section section) { document.Sections.Add(section.Clone()); return document; }
    public static Section Section(this Section section, Section child) { section.Sections.Add(child.Clone()); return section; }
    public static Section Plot(this Section section, Plot plot) { section.Cards.Add(new Card { Metadata = plot.Metadata?.Clone(), Plot = plot.Clone() }); return section; }
    public static Section Summary(this Section section, Metadata metadata, Summary summary) { section.Cards.Add(new Card { Metadata = metadata.Clone(), Summary = summary.Clone() }); return section; }
}

