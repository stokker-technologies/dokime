// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Clients;
using Dokime.Clients.PlotsResources;
using Google.Protobuf;
using Grpc.Core;
using Harmos;
using Harmos.Clients;
using Stories.Support;
using Xunit;
using Client = Dokime.Client;
using Metadata = Grpc.Core.Metadata;

namespace Stories.Integration;

public sealed class ClientInspectsABackend
{
  [Fact]
  public async Task AuthenticateInspectAndReadFiniteLogs()
  {
    // Connect Through the SDK's Caller-Owned Invoker.
    var server = new FakeBackend();
    using var client = new Client(server);

    // Observe Authentication Failure Before Supplying a Token.
    var failure = await Assert.ThrowsAsync<RpcException>(() => client.Status.Get());
    Assert.Equal(StatusCode.Unauthenticated, failure.StatusCode);

    // Inspect the Backend with Explicit Per-Call Credentials and a Deadline Override.
    var headers = new Metadata { { "authorization", "Bearer test-token" } };
    var deadline = new DateTime(2031, 1, 1, 0, 0, 0, DateTimeKind.Utc);
    var options = new CallOptions(headers, deadline);
    var observed = await client.Status.Get(context: options);
    Assert.Equal("bench", observed.Status.Identity.HostId);
    Assert.True(observed.Status.Available);
    Assert.Equal(deadline, server.Options.Deadline);

    // Drain a Finite Stream Through the Same Resource Tree.
    List<LogsWatchResponse> frames = [];
    await foreach (var frame in client.Status.Logs.Watch(4096, 8, context: options))
      frames.Add(frame);
    Assert.Empty(frames[0].Events);
    Assert.Equal(new[] { "INFO", "ERROR" }, frames.SelectMany(frame => frame.Events).Select(item => item.Level));

    // Preserve the Product Read Cut and Server Refusal.
    var cut = new ReadCut { Positions = { new Position { History = new History { Store = "cases", Epoch = "epoch" }, Sequence = 7 } } };
    var planned = await client.Cases.Get("case-a", "cycle-a", at: cut, context: options);
    Assert.Equal("case-a", planned.Value.Summary.CaseId);
    Assert.Equal(CaseStatus.Planned, planned.Value.Summary.Status);
    Assert.Equal(cut, planned.ResolvedAt);
    var missing = await Assert.ThrowsAsync<RpcException>(() => client.Cases.Get("", "cycle-a", context: options));
    Assert.Equal(StatusCode.InvalidArgument, missing.StatusCode);

    // Create and List a Library Entry Beside Product Resources.
    var placement = new PluginPlacement { Remote = new PluginRemote { Url = "http://127.0.0.1:7001" } };
    var added = await client.Plugins.Create("example", placement, new PluginValue(), context: options);
    Assert.Equal(placement, added.Entry.Placement);
    Assert.Equal(PluginState.Connecting, added.Entry.State);
    var listed = await client.Plugins.List(context: options);
    Assert.Equal(added.Entry, Assert.Single(listed.Entries));

    // Retain Integer and Rational Evidence Separately from Plot Coordinates.
    var statistics = await client.Evidence.Statistics(new EvidenceScope { ExecutionId = "first" }, "readings", [],
        new Window(), [], "counter", "", context: options);
    Assert.Equal(18_446_744_073_709_551_615ul, statistics.Statistics.Sum.UnsignedInteger);
    Assert.Equal("18446744073709551615", statistics.Statistics.Mean.Exact.Numerator);
    Assert.Equal("2", statistics.Statistics.Mean.Exact.Denominator);
    var snapshots = await client.Plots.Snapshots.List([new PlotRequest()], context: options);
    var column = Assert.Single(Assert.Single(snapshots.Envelopes).Payload.Frame.Series);
    Assert.Equal(9_007_199_254_740_993ul, Assert.Single(column.Totals).UnsignedInteger);
    Assert.Equal(9_007_199_254_740_992d, Assert.Single(column.Y).Value);

    // Retain Host-Owned Document Binding, Summary State and Completion Order.
    var document = await client.Documents.Get(new DocumentScope { ExecutionId = "first" }, "report", false,
        [], "", 4096, 8, context: options);
    Assert.Equal(1ul, document.Frames[0].Frame.Binding.Result.Summary.GroupCount);
    Assert.Equal(DocumentSummaryState.Critical, document.Frames[1].Frame.SummaryTile.State);
    Assert.Equal(statistics.Statistics.Mean, document.Frames[1].Frame.SummaryTile.Value);
    Assert.Equal(document.Frames[1].Frame.SummaryTile.CardId, document.Frames[2].Frame.CardComplete);

    // Fetch Descriptor Pages Once, Then Reuse the Verified Digest Across Executions.
    var schemas = new FrozenSchemaCache(new PageRequest { PageSize = 8, MaxBytes = 4096 });
    var detail = new ExecutionDetail { Summary = new ExecutionSummary { ExecutionId = "first" }, Schemas = new FrozenSchemas { Digest = server.Digest } };
    var types = await schemas.GetAsync(client, detail, options);
    Assert.Equal(new[] { "google/protobuf/wrappers.proto", "google/protobuf/empty.proto" }, types.File.Select(file => file.Name));
    var calls = server.SchemaCalls;
    Assert.Equal(types.File.Count, calls);
    types.File.Clear();
    detail.Summary.ExecutionId = "second";
    var reused = await schemas.GetAsync(client, detail, options);
    Assert.Equal(calls, server.SchemaCalls);
    Assert.Equal(2, reused.File.Count);

    // Preserve the Library's Refusal Through the Composed Tree.
    var invalid = new PluginPlacement { Remote = new PluginRemote { Url = "invalid" } };
    var rejected = await Assert.ThrowsAsync<RpcException>(() => client.Plugins.Create("invalid", invalid, new PluginValue(), context: options));
    Assert.Equal(StatusCode.InvalidArgument, rejected.StatusCode);

    // Cancel an Owned Watch After Its Opening Frame, Without a Timed Wait.
    using var cancellation = new CancellationTokenSource();
    await using var watch = client.Status.Logs.Watch(4096, 8, context: options,
        cancellationToken: cancellation.Token).GetAsyncEnumerator();
    Assert.True(await watch.MoveNextAsync());
    await cancellation.CancelAsync();
    await Assert.ThrowsAnyAsync<OperationCanceledException>(async () => await watch.MoveNextAsync());

    // A Following Call Retains Its Own Default Context.
    await Assert.ThrowsAsync<RpcException>(() => client.Status.Get());
    Assert.Null(server.Options.Deadline);
    Assert.Null(server.Options.Headers);
  }
}
