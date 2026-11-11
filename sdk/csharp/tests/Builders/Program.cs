/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using Dokime.Authoring;
using Dokime.V1;
using Google.Protobuf;

var input = Expressions.Field("scan", "temperature");
var surface = Plots.Contour(input.Scale(2).Offset(-1))
    .WithX(Expressions.Field("scan", "position_x"))
    .WithY(Expressions.Field("scan", "position_y"))
    .WithLabel(new Metadata { Identifier = "temperature.map", Label = "Temperature Map" })
    .Reduce(Reduction.Mean).Render(ContourRender.Bands)
    .ReferenceBand(new ReferenceBand {
        Center = new DataExpression { Parameter = "target" },
        Tolerance = new DataExpression { Parameter = "tolerance" }
    });
var section = new Section { Metadata = new Metadata { Identifier = "map", Label = "Map" } }.Plot(surface);
var document = new Document { Metadata = new Metadata { Identifier = "scan", Label = "Scan" } }.Section(section);
var decoded = Document.Parser.ParseFrom(document.ToByteArray());
input.Stream.Field = "changed";
surface.Y.Clear();
if (decoded.Sections[0].Cards[0].Plot.Y.Count != 1 ||
    decoded.Sections[0].Cards[0].Plot.Z.Transform.Input.Transform.Input.Stream.Field != "temperature" ||
    decoded.Sections[0].Cards[0].Plot.Render != ContourRender.Bands ||
    decoded.Sections[0].Cards[0].Plot.ReferenceBands[0].Tolerance.Parameter != "tolerance")
    throw new InvalidOperationException("canonical contour operands were not copied");

var grid = Plots.GridChart(Expressions.Field("scan", "value"))
    .Row(Expressions.Field("scan", "row")).Column(Expressions.Field("scan", "column"))
    .Frame(Expressions.Field("scan", "timestamp"))
    .Rows([new GridAxisEntry { Key = "sensor", Label = "Sensor" }])
    .Columns([new GridAxisEntry { Key = "site", Label = "Site" }]).Reduce(Reduction.Max);
if (grid.Grid.Frame.Stream.Field != "timestamp" || grid.Reduction != Reduction.Max)
    throw new InvalidOperationException("canonical grid bindings differ");
Console.WriteLine("2 document builder tests passed");
