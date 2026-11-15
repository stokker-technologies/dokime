// SPDX-License-Identifier: Apache-2.0

using System.Security.Cryptography;
using Dokime;
using Dokime.Clients;
using Dokime.Clients.ExecutionsResources;
using Dokime.Clients.PlotsResources;
using ExactNumber = Dokime.ExactNumber;
using Value = Dokime.Value;
using Google.Protobuf;
using Google.Protobuf.Reflection;
using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Harmos;
using Harmos.Clients;
using Metadata = Grpc.Core.Metadata;
using Status = Grpc.Core.Status;

namespace Stories.Support;

internal sealed class FakeBackend : CallInvoker
{
  private readonly Dictionary<string, PluginEntry> entries = [];
  internal FileDescriptorSet Types
  {
    get;
  } = new()
  {
    File = { BoolValue.Descriptor.File.ToProto(), Empty.Descriptor.File.ToProto() }
  };
  internal int SchemaCalls
  {
    get;
    private set;
  }
  internal ByteString Digest => ByteString.CopyFrom(SHA256.HashData(Types.ToByteArray()));
  internal CallOptions Options
  {
    get;
    private set;
  }

  private static T Field<T>(IMessage request, string name)
      => (T)request.Descriptor.FindFieldByName(name).Accessor.GetValue(request);

  private IMessage Reply(string method, IMessage request, CallOptions options)
  {
    Options = options;
    if (options.Headers?.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, $"Call {method}: Supply Credentials"));
    if (method == "/harmos.host.v1.Status/Get")
      return new StatusGetResponse
      {
        Status = new HostStatus { Identity = new HostIdentity { HostId = "bench", ApplicationVersion = "1.0.0" }, Available = true }
      };
    if (method == "/dokime.host.v1.Cases/Get")
    {
      var name = Field<string>(request, "name");
      if (string.IsNullOrWhiteSpace(name))
        throw new RpcException(new Status(StatusCode.InvalidArgument, "Get Case: Supply a Name"));
      return new CasesGetResponse
      {
        Value = new CaseDetail { Summary = new CaseSummary { CaseId = name, Status = CaseStatus.Planned } },
        ResolvedAt = Field<ReadCut?>(request, "at")?.Clone()
      };
    }
    if (method == "/harmos.host.v1.Plugins/Create")
    {
      var placement = Field<PluginPlacement>(request, "placement");
      if (!Uri.TryCreate(placement.Remote?.Url, UriKind.Absolute, out var url) || url.Scheme != "http")
        throw new RpcException(new Status(StatusCode.InvalidArgument, "Create Plugin: Supply a Valid URL"));
      var name = Field<string>(request, "name");
      var entry = new PluginEntry { Name = name, Placement = placement.Clone(), State = PluginState.Connecting };
      entries.Add(name, entry);
      return new PluginsCreateResponse { Entry = entry.Clone(), Receipt = new Receipt() };
    }
    if (method == "/harmos.host.v1.Plugins/List")
    {
      var response = new PluginsListResponse();
      response.Entries.AddRange(entries.Values.OrderBy(entry => entry.Name).Select(entry => entry.Clone()));
      return response;
    }
    if (method == "/dokime.host.v1.Evidence/Statistics")
    {
      var scope = Field<EvidenceScope>(request, "scope");
      if (scope.ExecutionId != "first" || Field<string>(request, "field") != "counter")
        throw new RpcException(new Status(StatusCode.InvalidArgument, "Read Statistics: Select the Counter Execution"));
      return new EvidenceStatisticsResponse
      {
        Statistics = new Statistics
        {
          Count = 2,
          Sum = new Value { UnsignedInteger = 18_446_744_073_709_551_615 },
          Mean = new Value { Exact = new ExactNumber { Numerator = "18446744073709551615", Denominator = "2" } }
        }
      };
    }
    if (method == "/dokime.host.v1.plots.Snapshots/List")
      return new SnapshotsListResponse
      {
        Envelopes = { new PlotEnvelope { Payload = new PlotPayload { Frame = new PlotFrame
        {
          Series = { new PlotColumn
          {
            Y = { new NullableNumber { Value = 9_007_199_254_740_992 } },
            Totals = { new Value { UnsignedInteger = 9_007_199_254_740_993 } }
          } }
        } } } }
      };
    if (method == "/dokime.host.v1.Documents/Get")
      return new DocumentsGetResponse
      {
        Frames =
        {
          new DocumentFrame { Frame = new DocumentFrameFrame { Binding = new DocumentBinding
          {
            SourceId = "counter",
            Result = new DocumentBindingResult { Summary = new DocumentSummary { GroupCount = 1 } }
          } } },
          new DocumentFrame { Frame = new DocumentFrameFrame { SummaryTile = new DocumentSummaryTile
          {
            CardId = "counter",
            Value = new Value { Exact = new ExactNumber { Numerator = "18446744073709551615", Denominator = "2" } },
            State = DocumentSummaryState.Critical
          } } },
          new DocumentFrame { Frame = new DocumentFrameFrame { CardComplete = "counter" } }
        }
      };
    if (method == "/dokime.host.v1.executions.Schemas/Get")
    {
      SchemaCalls++;
      var cut = new Position { History = new History { Store = "schemas", Epoch = Field<string>(request, "name") }, Sequence = 1 };
      var page = Field<PageRequest?>(request, "page");
      var first = page?.PageToken.Length == 0;
      if (!first && (!cut.Equals(Field<Position?>(request, "at")) || page?.PageToken != "second"))
        throw new RpcException(new Status(StatusCode.InvalidArgument, "Get Schemas: Continue the Same Read Cut"));
      return new SchemasGetResponse
      {
        Digest = Digest,
        Types_ = new FileDescriptorSet { File = { Types.File[first ? 0 : 1].Clone() } },
        ResolvedAt = cut,
        NextPageToken = first ? "second" : ""
      };
    }
    throw new RpcException(new Status(StatusCode.Unimplemented, $"Call {method}: Unsupported Fixture Operation"));
  }

  public override AsyncUnaryCall<TResponse> AsyncUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method,
      string? host, CallOptions options, TRequest request)
  {
    Task<TResponse> response;
    try
    {
      var reply = Reply(method.FullName, (IMessage)request, options);
      response = Task.FromResult(method.ResponseMarshaller.Deserializer(reply.ToByteArray()));
    }
    catch (RpcException error)
    {
      response = Task.FromException<TResponse>(error);
    }
    return new(response, Task.FromResult(new Metadata()), () => Status.DefaultSuccess, () => new Metadata(), () => { });
  }

  public override AsyncServerStreamingCall<TResponse> AsyncServerStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method,
      string? host, CallOptions options, TRequest request)
  {
    Options = options;
    var opening = new LogsWatchResponse();
    var batch = new LogsWatchResponse
    {
      Events = { new Event { Level = "INFO", Message = "ready" }, new Event { Level = "ERROR", Message = "fixture failure" } }
    };
    var reader = new Reader<TResponse>([(TResponse)(object)opening, (TResponse)(object)batch]);
    return new(reader, Task.FromResult(new Metadata()), () => Status.DefaultSuccess, () => new Metadata(), () => { });
  }

  private sealed class Reader<T>(IReadOnlyList<T> values) : IAsyncStreamReader<T>
  {
    private int index = -1;
    public T Current => values[index];
    public Task<bool> MoveNext(CancellationToken cancellationToken)
    {
      cancellationToken.ThrowIfCancellationRequested();
      index++;
      return Task.FromResult(index < values.Count);
    }
  }

  public override TResponse BlockingUnaryCall<TRequest, TResponse>(Method<TRequest, TResponse> method,
      string? host, CallOptions options, TRequest request) => throw new NotSupportedException();
  public override AsyncClientStreamingCall<TRequest, TResponse> AsyncClientStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method,
      string? host, CallOptions options) => throw new NotSupportedException();
  public override AsyncDuplexStreamingCall<TRequest, TResponse> AsyncDuplexStreamingCall<TRequest, TResponse>(Method<TRequest, TResponse> method,
      string? host, CallOptions options) => throw new NotSupportedException();
}
