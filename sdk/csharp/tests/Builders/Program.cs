/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using Dokime.Templates;
using Dokime;
using Google.Protobuf;

var input = Expressions.Field("scan", "temperature");
var surface = Dokime.Templates.Plots.Contour(input.Scale(2).Offset(-1))
    .WithX(Expressions.Field("scan", "position_x"))
    .WithY(Expressions.Field("scan", "position_y"))
    .WithLabel(new Metadata { Identifier = "temperature.map", Label = "Temperature Map" })
    .Reduce(Reduction.Mean).Render(ContourRender.Bands)
    .ReferenceBand(new ReferenceBand
    {
      Center = new DataExpression { Parameter = "target" },
      Tolerance = new DataExpression { Parameter = "tolerance" }
    });
var section = new Section { Metadata = new Metadata { Identifier = "map", Label = "Map" } }.Plot(surface);
var document = new Document { Metadata = new Metadata { Identifier = "scan", Label = "Scan" } }.Section(section);
input.Stream.Field = "changed";
surface.Y.Clear();
var decoded = Document.Parser.ParseFrom(document.ToByteArray());
if (decoded.Sections[0].Cards[0].Plot.Y.Count != 1 ||
    decoded.Sections[0].Cards[0].Plot.Z.Transform.Input.Transform.Input.Stream.Field != "temperature" ||
    decoded.Sections[0].Cards[0].Plot.Render != ContourRender.Bands ||
    decoded.Sections[0].Cards[0].Plot.ReferenceBands[0].Tolerance.Parameter != "tolerance")
  throw new InvalidOperationException("canonical contour operands were not copied");

var grid = Dokime.Templates.Plots.GridChart(Expressions.Field("scan", "value"))
    .Row(Expressions.Field("scan", "row")).Column(Expressions.Field("scan", "column"))
    .Frame(Expressions.Field("scan", "timestamp"))
    .Rows([new GridAxisEntry { Key = "sensor", Label = "Sensor" }])
    .Columns([new GridAxisEntry { Key = "site", Label = "Site" }]).Reduce(Reduction.Max);
if (grid.Grid.Frame.Stream.Field != "timestamp" || grid.Reduction != Reduction.Max)
  throw new InvalidOperationException("canonical grid bindings differ");
var voltage = Parameters.Number("voltage", "Voltage").WithUnit("V").WithDefault(230.0).Range(0.0, 300.0);
var count = Parameters.Count("samples", "Samples").WithDefault(ulong.MaxValue);
var flag = Parameters.Flag("enabled", "Enabled").WithDefault(true);
var sweep = Parameters.Sweep("load", "Load", new[] { 80.0, 100.0 });
var ladder = Parameters.Numbers("steps", "Steps", new[] { 10.0, 20.0 }).Items(1, 8);
var decodedParameter = Parameter.Parser.ParseFrom(voltage.ToByteArray());
if (decodedParameter.Min != 0.0 || decodedParameter.Max != 300.0 || decodedParameter.Expression.Literal.Number != 230.0 ||
    count.Expression.Literal.UnsignedInteger != ulong.MaxValue || !flag.Expression.Literal.Boolean ||
    !sweep.Expansion.Cartesian || sweep.Expression.Array.Values.Count != 2 ||
    ladder.Expression.Literal.List.Values[1].Number != 20.0 || ladder.MinItems != 1 || ladder.MaxItems != 8)
  throw new InvalidOperationException("Parameter Builders: Restore Types, Expansion and Bounds");
var line = Dokime.Templates.Plots.Line("voltage", "Voltage").WithUnit("V").WithY(Expressions.Field("samples", "voltage"));
var card = Cards.Plot(line).Columns(6);
line.Y.Clear();
if (card.Plot.Y.Count != 1 || card.Layout.Columns != 6 || Cards.Records("records").Evidence != EvidenceCard.Records)
  throw new InvalidOperationException("Card Builders: Preserve Their Owned Plot and Evidence Kind");
voltage.Range();
ladder.Items();
if (voltage.HasMin || voltage.HasMax || ladder.HasMinItems || ladder.HasMaxItems)
  throw new InvalidOperationException("Parameter Builders: Clear Unbounded Fields");
Console.WriteLine("4 builder tests passed");
