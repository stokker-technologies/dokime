// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Templates;
using Google.Protobuf;
using Google.Protobuf.Collections;
using Google.Protobuf.WellKnownTypes;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorSamplesPlainValues
{
  [Fact]
  public async Task ScalarObserverAnswersTypedColumns()
  {
    await using var server = await PluginServer.StartAsync(new ScalarPlugin());
    var columns = await server.Wire.WindowAsync("scalar.value.reading", "execution-a", new Timestamp { Seconds = 123 }, new Duration { Seconds = 1 }, 2);
    var values = new RepeatedField<double>();
    var input = new CodedInputStream(columns.ToByteArray());
    for (var tag = input.ReadTag(); tag != 0; tag = input.ReadTag())
    {
      if (WireFormat.GetTagFieldNumber(tag) == 1)
        values.AddEntriesFrom(input, FieldCodec.ForDouble(tag));
      else
        input.SkipLastField();
    }
    Assert.Equal([42.5, 42.5], values);
  }

  public sealed class ScalarPlugin : Dokime.Plugin
  {
    public override string Version => "1.0.0";
    public override IReadOnlyList<System.Type> Suites => [typeof(ScalarSuite)];
  }

  public sealed class ScalarSuite : Suite
  {
    public override IReadOnlyList<System.Type> Templates => [typeof(ValueTestTemplate)];
  }

  public sealed class ValueTestTemplate : TestTemplate
  {
    public static readonly Dokime.Templates.Stream<double> Reading = new();
    public static readonly Observer<double> Observer = new(Reading, TimeSpan.FromSeconds(1), context => Task.FromResult(42.5));
  }
}
