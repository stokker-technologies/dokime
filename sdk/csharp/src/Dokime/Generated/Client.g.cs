// @generated from protocol descriptors; do not edit.
#nullable enable
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Grpc.Net.Client;
using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
namespace Dokime.Transport;

internal sealed class Client : IDisposable
{
  private GrpcChannel? ownedChannel;
  public AccessResource Access { get; }
  public AggregatesResource Aggregates { get; }
  public BackendResource Backend { get; }
  public CasesResource Cases { get; }
  public ContentResource Content { get; }
  public CyclesResource Cycles { get; }
  public DocumentsResource Documents { get; }
  public EvidenceResource Evidence { get; }
  public ExecutionsResource Executions { get; }
  public FilesResource Files { get; }
  public InvocationsResource Invocations { get; }
  public MediaResource Media { get; }
  public OperationsResource Operations { get; }
  public PlotsResource Plots { get; }
  public PluginsResource Plugins { get; }
  public PromptDefinitionsResource PromptDefinitions { get; }
  public PromptsResource Prompts { get; }
  public RecordSchemasResource RecordSchemas { get; }
  public ReportsResource Reports { get; }
  public SecretsResource Secrets { get; }
  public SettingsResource Settings { get; }
  public StatusResource Status { get; }
  public StorageResource Storage { get; }
  public StreamSchemasResource StreamSchemas { get; }
  public SuitesResource Suites { get; }
  public Client(CallInvoker invoker)
  {
    Access = new AccessResource(invoker);
    Aggregates = new AggregatesResource(invoker);
    Backend = new BackendResource(invoker);
    Cases = new CasesResource(invoker);
    Content = new ContentResource(invoker);
    Cycles = new CyclesResource(invoker);
    Documents = new DocumentsResource(invoker);
    Evidence = new EvidenceResource(invoker);
    Executions = new ExecutionsResource(invoker);
    Files = new FilesResource(invoker);
    Invocations = new InvocationsResource(invoker);
    Media = new MediaResource(invoker);
    Operations = new OperationsResource(invoker);
    Plots = new PlotsResource(invoker);
    Plugins = new PluginsResource(invoker);
    PromptDefinitions = new PromptDefinitionsResource(invoker);
    Prompts = new PromptsResource(invoker);
    RecordSchemas = new RecordSchemasResource(invoker);
    Reports = new ReportsResource(invoker);
    Secrets = new SecretsResource(invoker);
    Settings = new SettingsResource(invoker);
    Status = new StatusResource(invoker);
    Storage = new StorageResource(invoker);
    StreamSchemas = new StreamSchemasResource(invoker);
    Suites = new SuitesResource(invoker);
  }
  public static Client Connect(string address, global::Harmos.StorageTransferSettings? transfer = null, int? messageBytes = null)
  {
    var maximum = global::Harmos.Transfer.MessageBytes(transfer, messageBytes);
    var channel = GrpcChannel.ForAddress(address, new() { MaxSendMessageSize = maximum, MaxReceiveMessageSize = maximum });
    return new Client(channel.CreateCallInvoker()) { ownedChannel = channel };
  }
  public void Dispose() => ownedChannel?.Dispose();
}
internal sealed class AccessResource
{
  public AccessCertificatesResource Certificates { get; }
  public AccessListenerResource Listener { get; }
  public AccessPairingsResource Pairings { get; }
  public AccessResource(CallInvoker invoker)
  {
    Certificates = new AccessCertificatesResource(invoker);
    Listener = new AccessListenerResource(invoker);
    Pairings = new AccessPairingsResource(invoker);
  }
}
internal sealed class AccessCertificatesResource
{
  private readonly CallInvoker invoker;
  public AccessCertificatesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<string> Create(CallOptions context = default)
  {
    var request = new global::Harmos.Clients.CertificatesCreateRequest
    {
    };
    var method = new Method<global::Harmos.Clients.CertificatesCreateRequest, global::Harmos.Clients.CertificatesCreateResponse>(MethodType.Unary, "dokime.host.v1.access.Certificates", "Create",
        Marshallers.Create<global::Harmos.Clients.CertificatesCreateRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.CertificatesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.CertificatesCreateResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.CertificatesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.CertificatesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call access.certificates.create: " + error.Status.Detail), error.Trailers);
    }
    return response.CertificatePem;
  }
}
internal sealed class AccessListenerResource
{
  private readonly CallInvoker invoker;
  public AccessListenerResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Harmos.Clients.ListenerGetResponse> Get(CallOptions context = default)
  {
    var request = new global::Harmos.Clients.ListenerGetRequest
    {
    };
    var method = new Method<global::Harmos.Clients.ListenerGetRequest, global::Harmos.Clients.ListenerGetResponse>(MethodType.Unary, "dokime.host.v1.access.Listener", "Get",
        Marshallers.Create<global::Harmos.Clients.ListenerGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.ListenerGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.ListenerGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.ListenerGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.ListenerGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call access.listener.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Harmos.Clients.ListenerSetResponse> Set(global::Harmos.Clients.ListenerValue value, ByteString expectedRevision, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.ListenerSetRequest
    {
      Value = value,
      ExpectedRevision = expectedRevision,
    };
    var method = new Method<global::Harmos.Clients.ListenerSetRequest, global::Harmos.Clients.ListenerSetResponse>(MethodType.Unary, "dokime.host.v1.access.Listener", "Set",
        Marshallers.Create<global::Harmos.Clients.ListenerSetRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.ListenerSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.ListenerSetResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.ListenerSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.ListenerSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call access.listener.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class AccessPairingsResource
{
  private readonly CallInvoker invoker;
  public AccessPairingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Harmos.Clients.PairingsCreateResponse> Create(string label, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.PairingsCreateRequest
    {
      Label = label,
    };
    var method = new Method<global::Harmos.Clients.PairingsCreateRequest, global::Harmos.Clients.PairingsCreateResponse>(MethodType.Unary, "dokime.host.v1.access.Pairings", "Create",
        Marshallers.Create<global::Harmos.Clients.PairingsCreateRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.PairingsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.PairingsCreateResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.PairingsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.PairingsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call access.pairings.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Harmos.Clients.PairingsListResponse> List(global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.PairingsListRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Harmos.Clients.PairingsListRequest, global::Harmos.Clients.PairingsListResponse>(MethodType.Unary, "dokime.host.v1.access.Pairings", "List",
        Marshallers.Create<global::Harmos.Clients.PairingsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.PairingsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.PairingsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.PairingsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.PairingsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call access.pairings.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Harmos.Clients.PairingsDeleteResponse> Delete(string name, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.PairingsDeleteRequest
    {
      Name = name,
    };
    var method = new Method<global::Harmos.Clients.PairingsDeleteRequest, global::Harmos.Clients.PairingsDeleteResponse>(MethodType.Unary, "dokime.host.v1.access.Pairings", "Delete",
        Marshallers.Create<global::Harmos.Clients.PairingsDeleteRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.PairingsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.PairingsDeleteResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.PairingsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.PairingsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call access.pairings.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class AggregatesResource
{
  private readonly CallInvoker invoker;
  public AggregatesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.AggregatesGetResponse> Get(global::Dokime.Clients.QuerySource source, IReadOnlyList<global::Dokime.Clients.QueryFilter> filters, IReadOnlyList<string> groupBy, IReadOnlyList<global::Dokime.Clients.QueryAggregate> aggregates, uint limit, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.AggregatesGetRequest
    {
      Source = source,
      Limit = limit,
    };
    request.Filters.Add(filters);
    request.GroupBy.Add(groupBy);
    request.Aggregates.Add(aggregates);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.AggregatesGetRequest, global::Dokime.Clients.AggregatesGetResponse>(MethodType.Unary, "dokime.host.v1.Aggregates", "Get",
        Marshallers.Create<global::Dokime.Clients.AggregatesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.AggregatesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.AggregatesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.AggregatesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.AggregatesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call aggregates.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class BackendResource
{
  private readonly CallInvoker invoker;
  public BackendOverviewResource Overview { get; }
  public BackendResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Overview = new BackendOverviewResource(invoker);
  }
  public async Task<global::Dokime.Clients.BackendAppearanceResponse> Appearance(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.BackendAppearanceRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.BackendAppearanceRequest, global::Dokime.Clients.BackendAppearanceResponse>(MethodType.Unary, "dokime.host.v1.Backend", "Appearance",
        Marshallers.Create<global::Dokime.Clients.BackendAppearanceRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.BackendAppearanceRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.BackendAppearanceResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.BackendAppearanceResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.BackendAppearanceResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call backend.appearance: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class BackendOverviewResource
{
  private readonly CallInvoker invoker;
  public BackendOverviewResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.BackendResources.OverviewGetResponse> Get(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.BackendResources.OverviewGetRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.BackendResources.OverviewGetRequest, global::Dokime.Clients.BackendResources.OverviewGetResponse>(MethodType.Unary, "dokime.host.v1.backend.Overview", "Get",
        Marshallers.Create<global::Dokime.Clients.BackendResources.OverviewGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.BackendResources.OverviewGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.BackendResources.OverviewGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.BackendResources.OverviewGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.BackendResources.OverviewGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call backend.overview.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.BackendResources.OverviewWatchResponse> Watch(global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.BackendResources.OverviewWatchRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.BackendResources.OverviewWatchRequest, global::Dokime.Clients.BackendResources.OverviewWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.backend.Overview", "Watch",
        Marshallers.Create<global::Dokime.Clients.BackendResources.OverviewWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.BackendResources.OverviewWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.BackendResources.OverviewWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.BackendResources.OverviewWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call backend.overview.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class CasesResource
{
  private readonly CallInvoker invoker;
  public CasesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.CasesCreateResponse> Create(global::Dokime.Clients.SelectionPlan plan, bool validateOnly, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CasesCreateRequest
    {
      Plan = plan,
      ValidateOnly = validateOnly,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.CasesCreateRequest, global::Dokime.Clients.CasesCreateResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Create",
        Marshallers.Create<global::Dokime.Clients.CasesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CasesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CasesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.CasesGetResponse> Get(string name, string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CasesGetRequest
    {
      Name = name,
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.CasesGetRequest, global::Dokime.Clients.CasesGetResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Get",
        Marshallers.Create<global::Dokime.Clients.CasesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CasesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CasesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.CasesListResponse> List(string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CasesListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.CasesListRequest, global::Dokime.Clients.CasesListResponse>(MethodType.Unary, "dokime.host.v1.Cases", "List",
        Marshallers.Create<global::Dokime.Clients.CasesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CasesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CasesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.CasesWatchResponse> Watch(string parent, global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.CasesWatchRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.CasesWatchRequest, global::Dokime.Clients.CasesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Cases", "Watch",
        Marshallers.Create<global::Dokime.Clients.CasesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CasesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call cases.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Harmos.Receipt> Delete(string name, string parent, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CasesDeleteRequest
    {
      Name = name,
      Parent = parent,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.CasesDeleteRequest, global::Dokime.Clients.CasesDeleteResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Delete",
        Marshallers.Create<global::Dokime.Clients.CasesDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CasesDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CasesDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.delete: " + error.Status.Detail), error.Trailers);
    }
    return response.Receipt ?? throw new InvalidOperationException("Read Cases.Delete: Missing Payload");
  }
  public async Task<global::Dokime.Clients.CasesParametersResponse> Parameters(string name, string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CasesParametersRequest
    {
      Name = name,
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.CasesParametersRequest, global::Dokime.Clients.CasesParametersResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Parameters",
        Marshallers.Create<global::Dokime.Clients.CasesParametersRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CasesParametersResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CasesParametersResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CasesParametersResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.parameters: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class ContentResource
{
  public ContentArtifactsResource Artifacts { get; }
  public ContentExecutionFilesResource ExecutionFiles { get; }
  public ContentResource(CallInvoker invoker)
  {
    Artifacts = new ContentArtifactsResource(invoker);
    ExecutionFiles = new ContentExecutionFilesResource(invoker);
  }
}
internal sealed class ContentArtifactsResource
{
  private readonly CallInvoker invoker;
  public ContentArtifactsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.ContentResources.ArtifactsGetResponse> Get(string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ArtifactsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ArtifactsGetRequest, global::Dokime.Clients.ContentResources.ArtifactsGetResponse>(MethodType.Unary, "dokime.host.v1.content.Artifacts", "Get",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ContentResources.ArtifactsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ContentResources.ArtifactsCreateResponse> Create(global::Harmos.FileReference file, string filename, string contentType, global::Dokime.ArtifactScope? scope = null, string? requestId = null, global::Dokime.Metadata? metadata = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ArtifactsCreateRequest
    {
      File = file,
      Filename = filename,
      ContentType = contentType,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (metadata is not null)
    {
      request.Metadata = metadata;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ArtifactsCreateRequest, global::Dokime.Clients.ContentResources.ArtifactsCreateResponse>(MethodType.Unary, "dokime.host.v1.content.Artifacts", "Create",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ContentResources.ArtifactsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ContentResources.ArtifactsListResponse> List(global::Dokime.ArtifactScope scope, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ArtifactsListRequest
    {
      Scope = scope,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ArtifactsListRequest, global::Dokime.Clients.ContentResources.ArtifactsListResponse>(MethodType.Unary, "dokime.host.v1.content.Artifacts", "List",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ContentResources.ArtifactsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.ContentResources.ArtifactsWatchResponse> Watch(global::Dokime.ArtifactScope scope, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ArtifactsWatchRequest
    {
      Scope = scope,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ArtifactsWatchRequest, global::Dokime.Clients.ContentResources.ArtifactsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.content.Artifacts", "Watch",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ArtifactsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ArtifactsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class ContentExecutionFilesResource
{
  private readonly CallInvoker invoker;
  public ContentExecutionFilesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.ContentResources.ExecutionFilesGetResponse> Get(string parent, string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ExecutionFilesGetRequest
    {
      Parent = parent,
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ExecutionFilesGetRequest, global::Dokime.Clients.ContentResources.ExecutionFilesGetResponse>(MethodType.Unary, "dokime.host.v1.content.ExecutionFiles", "Get",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ExecutionFilesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ExecutionFilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ExecutionFilesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ExecutionFilesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ContentResources.ExecutionFilesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.execution_files.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ContentResources.ExecutionFilesListResponse> List(string parent, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ExecutionFilesListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ExecutionFilesListRequest, global::Dokime.Clients.ContentResources.ExecutionFilesListResponse>(MethodType.Unary, "dokime.host.v1.content.ExecutionFiles", "List",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ExecutionFilesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ExecutionFilesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ExecutionFilesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ExecutionFilesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ContentResources.ExecutionFilesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.execution_files.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.ContentResources.ExecutionFilesWatchResponse> Watch(string parent, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ExecutionFilesWatchRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ContentResources.ExecutionFilesWatchRequest, global::Dokime.Clients.ContentResources.ExecutionFilesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.content.ExecutionFiles", "Watch",
        Marshallers.Create<global::Dokime.Clients.ContentResources.ExecutionFilesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ExecutionFilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ContentResources.ExecutionFilesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ContentResources.ExecutionFilesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call content.execution_files.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class CyclesResource
{
  private readonly CallInvoker invoker;
  public CyclesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.CyclesGetResponse> Get(string name, global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CyclesGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.CyclesGetRequest, global::Dokime.Clients.CyclesGetResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Get",
        Marshallers.Create<global::Dokime.Clients.CyclesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CyclesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CyclesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.CyclesSetResponse> Set(string name, global::Dokime.Clients.CycleWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CyclesSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.CyclesSetRequest, global::Dokime.Clients.CyclesSetResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Set",
        Marshallers.Create<global::Dokime.Clients.CyclesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CyclesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CyclesSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.CyclesListResponse> List(global::Dokime.Clients.CycleSelection selection, global::Dokime.Clients.CycleFilter filter, global::Dokime.Clients.CycleOrder order, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CyclesListRequest
    {
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.CyclesListRequest, global::Dokime.Clients.CyclesListResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "List",
        Marshallers.Create<global::Dokime.Clients.CyclesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CyclesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CyclesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.CyclesWatchResponse> Watch(global::Dokime.Clients.CycleSelection selection, global::Dokime.Clients.CycleFilter filter, global::Dokime.Clients.CycleOrder order, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.CyclesWatchRequest
    {
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.CyclesWatchRequest, global::Dokime.Clients.CyclesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Cycles", "Watch",
        Marshallers.Create<global::Dokime.Clients.CyclesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CyclesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call cycles.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Dokime.Clients.CyclesCreateResponse> Create(IReadOnlyList<global::Dokime.Origin> origins, global::Dokime.Metadata metadata, bool continueAutomatically, string sourceCycle, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CyclesCreateRequest
    {
      Metadata = metadata,
      ContinueAutomatically = continueAutomatically,
      SourceCycle = sourceCycle,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Origins.Add(origins);
    var method = new Method<global::Dokime.Clients.CyclesCreateRequest, global::Dokime.Clients.CyclesCreateResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Create",
        Marshallers.Create<global::Dokime.Clients.CyclesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CyclesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CyclesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.CyclesSealResponse> Seal(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.CyclesSealRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.CyclesSealRequest, global::Dokime.Clients.CyclesSealResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Seal",
        Marshallers.Create<global::Dokime.Clients.CyclesSealRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesSealRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.CyclesSealResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.CyclesSealResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.CyclesSealResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.seal: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class DocumentsResource
{
  private readonly CallInvoker invoker;
  public DocumentsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.DocumentsGetResponse> Get(global::Dokime.Clients.DocumentScope scope, string documentId, bool structureOnly, IReadOnlyList<string> cardIds, string pageToken, ulong maxBytes, uint maxFrames, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.DocumentsGetRequest
    {
      Scope = scope,
      DocumentId = documentId,
      StructureOnly = structureOnly,
      PageToken = pageToken,
      MaxBytes = maxBytes,
      MaxFrames = maxFrames,
    };
    request.CardIds.Add(cardIds);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.DocumentsGetRequest, global::Dokime.Clients.DocumentsGetResponse>(MethodType.Unary, "dokime.host.v1.Documents", "Get",
        Marshallers.Create<global::Dokime.Clients.DocumentsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.DocumentsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.DocumentsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.DocumentsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.DocumentsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call documents.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.DocumentsWatchResponse> Watch(global::Dokime.Clients.DocumentScope scope, string documentId, bool structureOnly, IReadOnlyList<string> cardIds, string pageToken, ulong maxBytes, uint maxFrames, global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.DocumentsWatchRequest
    {
      Scope = scope,
      DocumentId = documentId,
      StructureOnly = structureOnly,
      PageToken = pageToken,
      MaxBytes = maxBytes,
      MaxFrames = maxFrames,
    };
    request.CardIds.Add(cardIds);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.DocumentsWatchRequest, global::Dokime.Clients.DocumentsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Documents", "Watch",
        Marshallers.Create<global::Dokime.Clients.DocumentsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.DocumentsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.DocumentsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.DocumentsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call documents.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class EvidenceResource
{
  private readonly CallInvoker invoker;
  public EvidenceHealthEventsResource HealthEvents { get; }
  public EvidenceNotesResource Notes { get; }
  public EvidenceRecordsResource Records { get; }
  public EvidenceSamplesResource Samples { get; }
  public EvidenceTraceabilityResource Traceability { get; }
  public EvidenceResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    HealthEvents = new EvidenceHealthEventsResource(invoker);
    Notes = new EvidenceNotesResource(invoker);
    Records = new EvidenceRecordsResource(invoker);
    Samples = new EvidenceSamplesResource(invoker);
    Traceability = new EvidenceTraceabilityResource(invoker);
  }
  public async Task<global::Dokime.Clients.EvidenceStatisticsResponse> Statistics(string stream, IReadOnlyList<uint> versions, global::Dokime.Window between, IReadOnlyList<global::Dokime.Clients.Output> outputs, string field, string step, global::Dokime.EvidenceScope? scope = null, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceStatisticsRequest
    {
      Stream = stream,
      Between = between,
      Field = field,
      Step = step,
    };
    request.Versions.Add(versions);
    request.Outputs.Add(outputs);
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.EvidenceStatisticsRequest, global::Dokime.Clients.EvidenceStatisticsResponse>(MethodType.Unary, "dokime.host.v1.Evidence", "Statistics",
        Marshallers.Create<global::Dokime.Clients.EvidenceStatisticsRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceStatisticsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceStatisticsResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceStatisticsResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceStatisticsResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.statistics: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.EvidenceArchiveResponse> Archive(global::Dokime.Clients.EvidenceSubject subject, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceArchiveRequest
    {
      Subject = subject,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.EvidenceArchiveRequest, global::Dokime.Clients.EvidenceArchiveResponse>(MethodType.Unary, "dokime.host.v1.Evidence", "Archive",
        Marshallers.Create<global::Dokime.Clients.EvidenceArchiveRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceArchiveRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceArchiveResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceArchiveResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceArchiveResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.archive: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class EvidenceHealthEventsResource
{
  private readonly CallInvoker invoker;
  public EvidenceHealthEventsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.HealthEventsListResponse> List(global::Dokime.Clients.EvidenceResources.AnnotationSelection selection, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.HealthEventsListRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.HealthEventsListRequest, global::Dokime.Clients.EvidenceResources.HealthEventsListResponse>(MethodType.Unary, "dokime.host.v1.evidence.HealthEvents", "List",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.HealthEventsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.HealthEventsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.HealthEventsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.HealthEventsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.HealthEventsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.health_events.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.EvidenceResources.HealthEventsWatchResponse> Watch(global::Dokime.Clients.EvidenceResources.AnnotationSelection selection, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.HealthEventsWatchRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.HealthEventsWatchRequest, global::Dokime.Clients.EvidenceResources.HealthEventsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.HealthEvents", "Watch",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.HealthEventsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.HealthEventsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.HealthEventsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.HealthEventsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.health_events.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class EvidenceNotesResource
{
  private readonly CallInvoker invoker;
  public EvidenceNotesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.NotesGetResponse> Get(string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.NotesGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.NotesGetRequest, global::Dokime.Clients.EvidenceResources.NotesGetResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "Get",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.NotesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.NotesSetResponse> Set(string name, global::Dokime.Clients.EvidenceResources.NoteWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.NotesSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.EvidenceResources.NotesSetRequest, global::Dokime.Clients.EvidenceResources.NotesSetResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "Set",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.NotesSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.NotesListResponse> List(global::Dokime.Clients.EvidenceResources.AnnotationSelection selection, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.NotesListRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.NotesListRequest, global::Dokime.Clients.EvidenceResources.NotesListResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "List",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.NotesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.EvidenceResources.NotesWatchResponse> Watch(global::Dokime.Clients.EvidenceResources.AnnotationSelection selection, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.NotesWatchRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.NotesWatchRequest, global::Dokime.Clients.EvidenceResources.NotesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Notes", "Watch",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Dokime.Clients.EvidenceResources.NotesCreateResponse> Create(global::Dokime.Clients.EvidenceResources.AnnotationOwner owner, global::Dokime.Clients.EvidenceResources.NoteWritable value, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.NotesCreateRequest
    {
      Owner = owner,
      Value = value,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.EvidenceResources.NotesCreateRequest, global::Dokime.Clients.EvidenceResources.NotesCreateResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "Create",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.NotesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.NotesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.NotesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class EvidenceRecordsResource
{
  private readonly CallInvoker invoker;
  public EvidenceRecordsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.RecordsListResponse> List(string recordType, IReadOnlyList<uint> versions, global::Dokime.Window between, string step, global::Dokime.Clients.EvidenceResources.CapturedRecordSelection selection, global::Dokime.Clients.EvidenceResources.RecordScope? scope = null, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.RecordsListRequest
    {
      RecordType = recordType,
      Between = between,
      Step = step,
      Selection = selection,
    };
    request.Versions.Add(versions);
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.RecordsListRequest, global::Dokime.Clients.EvidenceResources.RecordsListResponse>(MethodType.Unary, "dokime.host.v1.evidence.Records", "List",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.RecordsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.RecordsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.RecordsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.RecordsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.RecordsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.records.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.EvidenceResources.RecordsWatchResponse> Watch(global::Dokime.Clients.EvidenceResources.RecordScope scope, string recordType, IReadOnlyList<uint> versions, global::Dokime.Window between, string step, global::Dokime.Clients.EvidenceResources.CapturedRecordSelection selection, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.RecordsWatchRequest
    {
      Scope = scope,
      RecordType = recordType,
      Between = between,
      Step = step,
      Selection = selection,
    };
    request.Versions.Add(versions);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.RecordsWatchRequest, global::Dokime.Clients.EvidenceResources.RecordsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Records", "Watch",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.RecordsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.RecordsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.RecordsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.RecordsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.records.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class EvidenceSamplesResource
{
  private readonly CallInvoker invoker;
  public EvidenceSamplesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.SamplesListResponse> List(global::Dokime.EvidenceScope scope, IReadOnlyList<string> streams, IReadOnlyList<uint> versions, IReadOnlyList<string> fields, IReadOnlyList<string> sources, IReadOnlyList<string> signals, global::Dokime.Window between, global::Dokime.Clients.EvidenceResources.BucketRequest bucket, bool latest, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.SamplesListRequest
    {
      Scope = scope,
      Between = between,
      Bucket = bucket,
      Latest = latest,
    };
    request.Streams.Add(streams);
    request.Versions.Add(versions);
    request.Fields.Add(fields);
    request.Sources.Add(sources);
    request.Signals.Add(signals);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.SamplesListRequest, global::Dokime.Clients.EvidenceResources.SamplesListResponse>(MethodType.Unary, "dokime.host.v1.evidence.Samples", "List",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.SamplesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.SamplesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.SamplesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.SamplesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.SamplesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.samples.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.EvidenceResources.SamplesWatchResponse> Watch(global::Dokime.EvidenceScope scope, IReadOnlyList<string> streams, IReadOnlyList<uint> versions, IReadOnlyList<string> fields, IReadOnlyList<string> sources, IReadOnlyList<string> signals, global::Dokime.Window between, global::Dokime.Clients.EvidenceResources.BucketRequest bucket, bool latest, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.SamplesWatchRequest
    {
      Scope = scope,
      Between = between,
      Bucket = bucket,
      Latest = latest,
    };
    request.Streams.Add(streams);
    request.Versions.Add(versions);
    request.Fields.Add(fields);
    request.Sources.Add(sources);
    request.Signals.Add(signals);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.SamplesWatchRequest, global::Dokime.Clients.EvidenceResources.SamplesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Samples", "Watch",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.SamplesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.SamplesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.SamplesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.SamplesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.samples.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class EvidenceTraceabilityResource
{
  private readonly CallInvoker invoker;
  public EvidenceTraceabilityResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse> Get(global::Dokime.EvidenceScope? scope = null, global::Harmos.PageRequest? page = null, global::Harmos.ReadCut? at = null, string? entry = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest
    {
    };
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (entry is not null)
    {
      request.Entry = entry;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest, global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse>(MethodType.Unary, "dokime.host.v1.evidence.Traceability", "Get",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.traceability.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.EvidenceResources.TraceabilityWatchResponse> Watch(global::Dokime.EvidenceScope scope, global::Harmos.PageRequest? page = null, global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.TraceabilityWatchRequest
    {
      Scope = scope,
    };
    if (page is not null)
    {
      request.Page = page;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.EvidenceResources.TraceabilityWatchRequest, global::Dokime.Clients.EvidenceResources.TraceabilityWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Traceability", "Watch",
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.TraceabilityWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.TraceabilityWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.EvidenceResources.TraceabilityWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.EvidenceResources.TraceabilityWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.traceability.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class ExecutionsResource
{
  private readonly CallInvoker invoker;
  public ExecutionsCommandsResource Commands { get; }
  public ExecutionsProgressResource Progress { get; }
  public ExecutionsSchemasResource Schemas { get; }
  public ExecutionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Commands = new ExecutionsCommandsResource(invoker);
    Progress = new ExecutionsProgressResource(invoker);
    Schemas = new ExecutionsSchemasResource(invoker);
  }
  public async Task<global::Dokime.Clients.ExecutionsGetResponse> Get(string name, global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsGetRequest, global::Dokime.Clients.ExecutionsGetResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Get",
        Marshallers.Create<global::Dokime.Clients.ExecutionsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsSetResponse> Set(string name, global::Dokime.Clients.ExecutionWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ExecutionsSetRequest, global::Dokime.Clients.ExecutionsSetResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Set",
        Marshallers.Create<global::Dokime.Clients.ExecutionsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsListResponse> List(string parent, global::Dokime.Clients.ExecutionSelection selection, global::Dokime.Clients.ExecutionFilter filter, global::Dokime.Clients.ExecutionOrder order, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsListRequest
    {
      Parent = parent,
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsListRequest, global::Dokime.Clients.ExecutionsListResponse>(MethodType.Unary, "dokime.host.v1.Executions", "List",
        Marshallers.Create<global::Dokime.Clients.ExecutionsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.ExecutionsWatchResponse> Watch(string parent, global::Dokime.Clients.ExecutionSelection selection, global::Dokime.Clients.ExecutionFilter filter, global::Dokime.Clients.ExecutionOrder order, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ExecutionsWatchRequest
    {
      Parent = parent,
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsWatchRequest, global::Dokime.Clients.ExecutionsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Executions", "Watch",
        Marshallers.Create<global::Dokime.Clients.ExecutionsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call executions.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Dokime.Clients.ExecutionsCreateResponse> Create(string parent, IReadOnlyList<global::Dokime.Clients.ExecutionBinding> executions, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsCreateRequest
    {
      Parent = parent,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Executions.Add(executions);
    var method = new Method<global::Dokime.Clients.ExecutionsCreateRequest, global::Dokime.Clients.ExecutionsCreateResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Create",
        Marshallers.Create<global::Dokime.Clients.ExecutionsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsStartResponse> Start(string name, global::Dokime.Clients.ExecutionMode mode, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsStartRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
      Mode = mode,
    };
    var method = new Method<global::Dokime.Clients.ExecutionsStartRequest, global::Dokime.Clients.ExecutionsStartResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Start",
        Marshallers.Create<global::Dokime.Clients.ExecutionsStartRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsStartRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsStartResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsStartResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsStartResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.start: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsStopResponse> Stop(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsStopRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ExecutionsStopRequest, global::Dokime.Clients.ExecutionsStopResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Stop",
        Marshallers.Create<global::Dokime.Clients.ExecutionsStopRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsStopResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsStopResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsStopResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.stop: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsCancelResponse> Cancel(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsCancelRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ExecutionsCancelRequest, global::Dokime.Clients.ExecutionsCancelResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Cancel",
        Marshallers.Create<global::Dokime.Clients.ExecutionsCancelRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsCancelRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsCancelResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsCancelResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsCancelResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.cancel: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsSealResponse> Seal(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsSealRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ExecutionsSealRequest, global::Dokime.Clients.ExecutionsSealResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Seal",
        Marshallers.Create<global::Dokime.Clients.ExecutionsSealRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsSealRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsSealResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsSealResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsSealResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.seal: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsDeleteResponse> Delete(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsDeleteRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ExecutionsDeleteRequest, global::Dokime.Clients.ExecutionsDeleteResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Delete",
        Marshallers.Create<global::Dokime.Clients.ExecutionsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsParametersResponse> Parameters(string name, global::Dokime.Clients.ExecutionParameterSet set, global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsParametersRequest
    {
      Name = name,
      Set = set,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsParametersRequest, global::Dokime.Clients.ExecutionsParametersResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Parameters",
        Marshallers.Create<global::Dokime.Clients.ExecutionsParametersRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsParametersResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsParametersResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsParametersResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.parameters: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsStepsResponse> Steps(string name, global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsStepsRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsStepsRequest, global::Dokime.Clients.ExecutionsStepsResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Steps",
        Marshallers.Create<global::Dokime.Clients.ExecutionsStepsRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsStepsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsStepsResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsStepsResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsStepsResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.steps: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class ExecutionsCommandsResource
{
  private readonly CallInvoker invoker;
  public ExecutionsCommandsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.ExecutionsResources.CommandsGetResponse> Get(string name, string parent, global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.CommandsGetRequest
    {
      Name = name,
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsResources.CommandsGetRequest, global::Dokime.Clients.ExecutionsResources.CommandsGetResponse>(MethodType.Unary, "dokime.host.v1.executions.Commands", "Get",
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.CommandsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.CommandsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.CommandsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.CommandsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsResources.CommandsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.commands.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsResources.CommandsCreateResponse> Create(string parent, string commandId, global::Dokime.Clients.ExecutionsResources.ExecutionCommandAction action, string? requestId = null, string? stepId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.CommandsCreateRequest
    {
      Parent = parent,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
      CommandId = commandId,
      Action = action,
    };
    if (stepId is not null)
    {
      request.StepId = stepId;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsResources.CommandsCreateRequest, global::Dokime.Clients.ExecutionsResources.CommandsCreateResponse>(MethodType.Unary, "dokime.host.v1.executions.Commands", "Create",
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.CommandsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.CommandsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.CommandsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.CommandsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsResources.CommandsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.commands.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class ExecutionsProgressResource
{
  private readonly CallInvoker invoker;
  public ExecutionsProgressResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.ExecutionsResources.ProgressGetResponse> Get(string? name = null, global::Harmos.Position? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.ProgressGetRequest
    {
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsResources.ProgressGetRequest, global::Dokime.Clients.ExecutionsResources.ProgressGetResponse>(MethodType.Unary, "dokime.host.v1.executions.Progress", "Get",
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.ProgressGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.ProgressGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.ProgressGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.ProgressGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsResources.ProgressGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.progress.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.ExecutionsResources.ProgressWatchResponse> Watch(string name, global::Harmos.Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.ProgressWatchRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsResources.ProgressWatchRequest, global::Dokime.Clients.ExecutionsResources.ProgressWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.executions.Progress", "Watch",
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.ProgressWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.ProgressWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.ProgressWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.ProgressWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call executions.progress.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class ExecutionsSchemasResource
{
  private readonly CallInvoker invoker;
  public ExecutionsSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.ExecutionsResources.SchemasGetResponse> Get(string name, global::Harmos.Position? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.SchemasGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.ExecutionsResources.SchemasGetRequest, global::Dokime.Clients.ExecutionsResources.SchemasGetResponse>(MethodType.Unary, "dokime.host.v1.executions.Schemas", "Get",
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.SchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.SchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ExecutionsResources.SchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ExecutionsResources.SchemasGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ExecutionsResources.SchemasGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.schemas.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class FilesResource
{
  private readonly CallInvoker invoker;
  public FilesUploadsResource Uploads { get; }
  public FilesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Uploads = new FilesUploadsResource(invoker);
  }
  public async Task<global::Dokime.Clients.FilesGetResponse> Get(global::Harmos.FileReference file, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesGetRequest
    {
      File = file,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.FilesGetRequest, global::Dokime.Clients.FilesGetResponse>(MethodType.Unary, "dokime.host.v1.Files", "Get",
        Marshallers.Create<global::Dokime.Clients.FilesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.FilesReadResponse> Read(global::Harmos.FileReference file, ulong offset, ulong maxBytes, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesReadRequest
    {
      File = file,
      Offset = offset,
      MaxBytes = maxBytes,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.FilesReadRequest, global::Dokime.Clients.FilesReadResponse>(MethodType.Unary, "dokime.host.v1.Files", "Read",
        Marshallers.Create<global::Dokime.Clients.FilesReadRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesReadRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesReadResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesReadResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesReadResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.read: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.FilesWatchResponse> Watch(ulong offset, global::Harmos.FileReference? file = null, global::Harmos.Upload? upload = null, global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesWatchRequest
    {
      Offset = offset,
    };
    if (file is not null)
    {
      request.File = file;
    }
    if (upload is not null)
    {
      request.Upload = upload;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.FilesWatchRequest, global::Dokime.Clients.FilesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Files", "Watch",
        Marshallers.Create<global::Dokime.Clients.FilesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call files.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Harmos.Receipt> Delete(global::Harmos.FileReference file, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesDeleteRequest
    {
      File = file,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.FilesDeleteRequest, global::Dokime.Clients.FilesDeleteResponse>(MethodType.Unary, "dokime.host.v1.Files", "Delete",
        Marshallers.Create<global::Dokime.Clients.FilesDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.delete: " + error.Status.Detail), error.Trailers);
    }
    return response.Receipt ?? throw new InvalidOperationException("Read Files.Delete: Missing Payload");
  }
}
internal sealed class FilesUploadsResource
{
  private readonly CallInvoker invoker;
  public FilesUploadsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsCreateResponse> Create(global::Harmos.FileMetadata metadata, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsCreateRequest
    {
      Metadata = metadata,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.FilesResources.UploadsCreateRequest, global::Dokime.Clients.FilesResources.UploadsCreateResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Create",
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesResources.UploadsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsGetResponse> Get(global::Harmos.Upload upload, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsGetRequest
    {
      Upload = upload,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.FilesResources.UploadsGetRequest, global::Dokime.Clients.FilesResources.UploadsGetResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Get",
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesResources.UploadsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsResumeResponse> Resume(global::Harmos.Upload upload, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsResumeRequest
    {
      Upload = upload,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.FilesResources.UploadsResumeRequest, global::Dokime.Clients.FilesResources.UploadsResumeResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Resume",
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsResumeRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsResumeRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsResumeResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsResumeResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesResources.UploadsResumeResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.resume: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsAppendResponse> Append(global::Harmos.Upload upload, ulong offset, ByteString data, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsAppendRequest
    {
      Upload = upload,
      Offset = offset,
      Data = data,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.FilesResources.UploadsAppendRequest, global::Dokime.Clients.FilesResources.UploadsAppendResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Append",
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsAppendRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsAppendRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsAppendResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsAppendResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesResources.UploadsAppendResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.append: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsPublishResponse> Publish(global::Harmos.Upload upload, ulong length, ByteString sha256, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsPublishRequest
    {
      Upload = upload,
      Length = length,
      Sha256 = sha256,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.FilesResources.UploadsPublishRequest, global::Dokime.Clients.FilesResources.UploadsPublishResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Publish",
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsPublishRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsPublishResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsPublishResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesResources.UploadsPublishResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.publish: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsDeleteResponse> Delete(global::Harmos.Upload upload, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsDeleteRequest
    {
      Upload = upload,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.FilesResources.UploadsDeleteRequest, global::Dokime.Clients.FilesResources.UploadsDeleteResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Delete",
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.FilesResources.UploadsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.FilesResources.UploadsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.FilesResources.UploadsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class InvocationsResource
{
  private readonly CallInvoker invoker;
  public InvocationsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ByteString> Create(string operation, ByteString fingerprint, ByteString payload, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.InvocationsCreateRequest
    {
      Operation = operation,
      Fingerprint = fingerprint,
      Payload = payload,
    };
    var method = new Method<global::Harmos.Clients.InvocationsCreateRequest, global::Harmos.Clients.InvocationsCreateResponse>(MethodType.Unary, "dokime.host.v1.Invocations", "Create",
        Marshallers.Create<global::Harmos.Clients.InvocationsCreateRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.InvocationsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.InvocationsCreateResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.InvocationsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.InvocationsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call invocations.create: " + error.Status.Detail), error.Trailers);
    }
    return response.Payload;
  }
}
internal sealed class MediaResource
{
  public MediaCapturesResource Captures { get; }
  public MediaResource(CallInvoker invoker)
  {
    Captures = new MediaCapturesResource(invoker);
  }
}
internal sealed class MediaCapturesResource
{
  private readonly CallInvoker invoker;
  public MediaCapturesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.MediaResources.CapturesCreateResponse> Create(global::Dokime.ArtifactScope scope, global::Dokime.Clients.MediaResources.CaptureKind kind, global::Dokime.Clients.MediaResources.RecordingAudio audio, uint? input = null, uint? camera = null, uint? display = null, uint? maximumSeconds = null, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.MediaResources.CapturesCreateRequest
    {
      Scope = scope,
      Kind = kind,
      Audio = audio,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    if (input is not null)
    {
      request.Input = input.Value;
    }
    if (camera is not null)
    {
      request.Camera = camera.Value;
    }
    if (display is not null)
    {
      request.Display = display.Value;
    }
    if (maximumSeconds is not null)
    {
      request.MaximumSeconds = maximumSeconds.Value;
    }
    var method = new Method<global::Dokime.Clients.MediaResources.CapturesCreateRequest, global::Dokime.Clients.MediaResources.CapturesCreateResponse>(MethodType.Unary, "dokime.host.v1.media.Captures", "Create",
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.MediaResources.CapturesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call media.captures.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.MediaResources.CapturesGetResponse> Get(string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.MediaResources.CapturesGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.MediaResources.CapturesGetRequest, global::Dokime.Clients.MediaResources.CapturesGetResponse>(MethodType.Unary, "dokime.host.v1.media.Captures", "Get",
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.MediaResources.CapturesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call media.captures.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.MediaResources.CapturesWatchResponse> Watch(string name, global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.MediaResources.CapturesWatchRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.MediaResources.CapturesWatchRequest, global::Dokime.Clients.MediaResources.CapturesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.media.Captures", "Watch",
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call media.captures.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Dokime.Clients.MediaResources.CapturesStopResponse> Stop(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.MediaResources.CapturesStopRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.MediaResources.CapturesStopRequest, global::Dokime.Clients.MediaResources.CapturesStopResponse>(MethodType.Unary, "dokime.host.v1.media.Captures", "Stop",
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesStopRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.MediaResources.CapturesStopResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.MediaResources.CapturesStopResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.MediaResources.CapturesStopResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call media.captures.stop: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class OperationsResource
{
  private readonly CallInvoker invoker;
  public OperationsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<IReadOnlyList<global::Harmos.DiscoveredOperation>> List(CallOptions context = default)
  {
    var request = new global::Harmos.Clients.OperationsListRequest
    {
    };
    var method = new Method<global::Harmos.Clients.OperationsListRequest, global::Harmos.Clients.OperationsListResponse>(MethodType.Unary, "dokime.host.v1.Operations", "List",
        Marshallers.Create<global::Harmos.Clients.OperationsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.OperationsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.OperationsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.OperationsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.OperationsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call operations.list: " + error.Status.Detail), error.Trailers);
    }
    return response.Operations;
  }
  public async Task<global::Harmos.DiscoveredOperation> Get(string id, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.OperationsGetRequest
    {
      Id = id,
    };
    var method = new Method<global::Harmos.Clients.OperationsGetRequest, global::Harmos.Clients.OperationsGetResponse>(MethodType.Unary, "dokime.host.v1.Operations", "Get",
        Marshallers.Create<global::Harmos.Clients.OperationsGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.OperationsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.OperationsGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.OperationsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.OperationsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call operations.get: " + error.Status.Detail), error.Trailers);
    }
    return response.Operation ?? throw new InvalidOperationException("Read Operations.Get: Missing Payload");
  }
}
internal sealed class PlotsResource
{
  public PlotsSettingsResource Settings { get; }
  public PlotsSnapshotsResource Snapshots { get; }
  public PlotsResource(CallInvoker invoker)
  {
    Settings = new PlotsSettingsResource(invoker);
    Snapshots = new PlotsSnapshotsResource(invoker);
  }
}
internal sealed class PlotsSettingsResource
{
  private readonly CallInvoker invoker;
  public PlotsSettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.PlotsResources.SettingsGetResponse> Get(global::Dokime.Clients.DocumentScope scope, string documentId, string plotId, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PlotsResources.SettingsGetRequest
    {
      Scope = scope,
      DocumentId = documentId,
      PlotId = plotId,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PlotsResources.SettingsGetRequest, global::Dokime.Clients.PlotsResources.SettingsGetResponse>(MethodType.Unary, "dokime.host.v1.plots.Settings", "Get",
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SettingsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PlotsResources.SettingsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plots.settings.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.PlotsResources.SettingsSetResponse> Set(global::Dokime.Clients.DocumentScope scope, string documentId, string plotId, global::Dokime.Clients.PlotsResources.PlotSettingsValue value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PlotsResources.SettingsSetRequest
    {
      Scope = scope,
      DocumentId = documentId,
      PlotId = plotId,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.PlotsResources.SettingsSetRequest, global::Dokime.Clients.PlotsResources.SettingsSetResponse>(MethodType.Unary, "dokime.host.v1.plots.Settings", "Set",
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SettingsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PlotsResources.SettingsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plots.settings.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class PlotsSnapshotsResource
{
  private readonly CallInvoker invoker;
  public PlotsSnapshotsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.PlotsResources.SnapshotsListResponse> List(IReadOnlyList<global::Dokime.Clients.PlotsResources.PlotRequest> requests, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PlotsResources.SnapshotsListRequest
    {
    };
    request.Requests.Add(requests);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.PlotsResources.SnapshotsListRequest, global::Dokime.Clients.PlotsResources.SnapshotsListResponse>(MethodType.Unary, "dokime.host.v1.plots.Snapshots", "List",
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SnapshotsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SnapshotsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SnapshotsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SnapshotsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PlotsResources.SnapshotsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plots.snapshots.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.PlotsResources.SnapshotsWatchResponse> Watch(IReadOnlyList<global::Dokime.Clients.PlotsResources.PlotRequest> requests, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.PlotsResources.SnapshotsWatchRequest
    {
    };
    request.Requests.Add(requests);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.PlotsResources.SnapshotsWatchRequest, global::Dokime.Clients.PlotsResources.SnapshotsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.plots.Snapshots", "Watch",
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SnapshotsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SnapshotsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PlotsResources.SnapshotsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PlotsResources.SnapshotsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call plots.snapshots.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class PluginsResource
{
  private readonly CallInvoker invoker;
  public PluginsSecretsResource Secrets { get; }
  public PluginsSettingsResource Settings { get; }
  public PluginsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Secrets = new PluginsSecretsResource(invoker);
    Settings = new PluginsSettingsResource(invoker);
  }
  public async Task<global::Dokime.Clients.PluginsGetResponse> Get(string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PluginsGetRequest, global::Dokime.Clients.PluginsGetResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "Get",
        Marshallers.Create<global::Dokime.Clients.PluginsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.PluginsListResponse> List(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsListRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PluginsListRequest, global::Dokime.Clients.PluginsListResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "List",
        Marshallers.Create<global::Dokime.Clients.PluginsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.PluginsWatchResponse> Watch(string? name = null, global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.PluginsWatchRequest
    {
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PluginsWatchRequest, global::Dokime.Clients.PluginsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Plugins", "Watch",
        Marshallers.Create<global::Dokime.Clients.PluginsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call plugins.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Dokime.Clients.PluginsCreateResponse> Create(string name, global::Dokime.Clients.PluginPlacement placement, global::Dokime.Clients.PluginValue value, string? requestId = null, string? origin = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsCreateRequest
    {
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
      Name = name,
      Placement = placement,
      Value = value,
    };
    if (origin is not null)
    {
      request.Origin = origin;
    }
    var method = new Method<global::Dokime.Clients.PluginsCreateRequest, global::Dokime.Clients.PluginsCreateResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "Create",
        Marshallers.Create<global::Dokime.Clients.PluginsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.PluginsSetResponse> Set(string name, global::Dokime.Clients.PluginValue value, ByteString expectedRevision, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
    };
    var method = new Method<global::Dokime.Clients.PluginsSetRequest, global::Dokime.Clients.PluginsSetResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "Set",
        Marshallers.Create<global::Dokime.Clients.PluginsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Harmos.Receipt> Delete(string name, ByteString expectedRevision, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsDeleteRequest
    {
      Name = name,
      ExpectedRevision = expectedRevision,
    };
    var method = new Method<global::Dokime.Clients.PluginsDeleteRequest, global::Dokime.Clients.PluginsDeleteResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "Delete",
        Marshallers.Create<global::Dokime.Clients.PluginsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.delete: " + error.Status.Detail), error.Trailers);
    }
    return response.Receipt ?? throw new InvalidOperationException("Read Plugins.Delete: Missing Payload");
  }
  public async Task<global::Dokime.Clients.PluginEntry> Restart(string name, ByteString expectedRevision, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsRestartRequest
    {
      Name = name,
      ExpectedRevision = expectedRevision,
    };
    var method = new Method<global::Dokime.Clients.PluginsRestartRequest, global::Dokime.Clients.PluginsRestartResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "Restart",
        Marshallers.Create<global::Dokime.Clients.PluginsRestartRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsRestartRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsRestartResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsRestartResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsRestartResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.restart: " + error.Status.Detail), error.Trailers);
    }
    return response.Entry ?? throw new InvalidOperationException("Read Plugins.Restart: Missing Payload");
  }
  public async Task<global::Dokime.Clients.PluginsReplaceResponse> Replace(string name, global::Dokime.Clients.PluginPlacement placement, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsReplaceRequest
    {
      Name = name,
      Placement = placement,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.PluginsReplaceRequest, global::Dokime.Clients.PluginsReplaceResponse>(MethodType.Unary, "dokime.host.v1.Plugins", "Replace",
        Marshallers.Create<global::Dokime.Clients.PluginsReplaceRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsReplaceRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsReplaceResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsReplaceResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsReplaceResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.replace: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class PluginsSecretsResource
{
  private readonly CallInvoker invoker;
  public PluginsSecretsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.PluginsResources.SecretsListResponse> List(string? name = null, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsResources.SecretsListRequest
    {
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PluginsResources.SecretsListRequest, global::Dokime.Clients.PluginsResources.SecretsListResponse>(MethodType.Unary, "dokime.host.v1.plugins.Secrets", "List",
        Marshallers.Create<global::Dokime.Clients.PluginsResources.SecretsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsResources.SecretsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsResources.SecretsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsResources.SecretsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsResources.SecretsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.secrets.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class PluginsSettingsResource
{
  private readonly CallInvoker invoker;
  public PluginsSettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.PluginsResources.SettingsListResponse> List(string? name = null, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsResources.SettingsListRequest
    {
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PluginsResources.SettingsListRequest, global::Dokime.Clients.PluginsResources.SettingsListResponse>(MethodType.Unary, "dokime.host.v1.plugins.Settings", "List",
        Marshallers.Create<global::Dokime.Clients.PluginsResources.SettingsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsResources.SettingsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsResources.SettingsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsResources.SettingsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsResources.SettingsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.settings.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.PluginsResources.SettingsSetResponse> Set(string name, global::Harmos.SettingValues value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PluginsResources.SettingsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.PluginsResources.SettingsSetRequest, global::Dokime.Clients.PluginsResources.SettingsSetResponse>(MethodType.Unary, "dokime.host.v1.plugins.Settings", "Set",
        Marshallers.Create<global::Dokime.Clients.PluginsResources.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsResources.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PluginsResources.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PluginsResources.SettingsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PluginsResources.SettingsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.settings.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class PromptDefinitionsResource
{
  private readonly CallInvoker invoker;
  public PromptDefinitionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.PromptDefinitionsGetResponse> Get(string name, string parent, uint schemaVersion, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PromptDefinitionsGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PromptDefinitionsGetRequest, global::Dokime.Clients.PromptDefinitionsGetResponse>(MethodType.Unary, "dokime.host.v1.PromptDefinitions", "Get",
        Marshallers.Create<global::Dokime.Clients.PromptDefinitionsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptDefinitionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PromptDefinitionsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptDefinitionsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PromptDefinitionsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompt_definitions.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.PromptDefinitionsListResponse> List(string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PromptDefinitionsListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PromptDefinitionsListRequest, global::Dokime.Clients.PromptDefinitionsListResponse>(MethodType.Unary, "dokime.host.v1.PromptDefinitions", "List",
        Marshallers.Create<global::Dokime.Clients.PromptDefinitionsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptDefinitionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PromptDefinitionsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptDefinitionsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PromptDefinitionsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompt_definitions.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class PromptsResource
{
  private readonly CallInvoker invoker;
  public PromptsAnswersResource Answers { get; }
  public PromptsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Answers = new PromptsAnswersResource(invoker);
  }
  public async Task<global::Dokime.Clients.PromptsGetResponse> Get(string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PromptsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.PromptsGetRequest, global::Dokime.Clients.PromptsGetResponse>(MethodType.Unary, "dokime.host.v1.Prompts", "Get",
        Marshallers.Create<global::Dokime.Clients.PromptsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PromptsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PromptsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompts.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.PromptsListResponse> List(string parent, bool pendingOnly, string? cycleId = null, string? executionId = null, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PromptsListRequest
    {
      Parent = parent,
      PendingOnly = pendingOnly,
    };
    if (cycleId is not null)
    {
      request.CycleId = cycleId;
    }
    if (executionId is not null)
    {
      request.ExecutionId = executionId;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.PromptsListRequest, global::Dokime.Clients.PromptsListResponse>(MethodType.Unary, "dokime.host.v1.Prompts", "List",
        Marshallers.Create<global::Dokime.Clients.PromptsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PromptsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PromptsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompts.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.PromptsWatchResponse> Watch(string parent, bool pendingOnly, string? cycleId = null, string? executionId = null, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.PromptsWatchRequest
    {
      Parent = parent,
      PendingOnly = pendingOnly,
    };
    if (cycleId is not null)
    {
      request.CycleId = cycleId;
    }
    if (executionId is not null)
    {
      request.ExecutionId = executionId;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Dokime.Clients.PromptsWatchRequest, global::Dokime.Clients.PromptsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Prompts", "Watch",
        Marshallers.Create<global::Dokime.Clients.PromptsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PromptsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call prompts.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class PromptsAnswersResource
{
  private readonly CallInvoker invoker;
  public PromptsAnswersResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.PromptsResources.AnswersCreateResponse> Create(string parent, string action, global::Dokime.MapValue values, bool accepted, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.PromptsResources.AnswersCreateRequest
    {
      Parent = parent,
      Action = action,
      Values = values,
      Accepted = accepted,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.PromptsResources.AnswersCreateRequest, global::Dokime.Clients.PromptsResources.AnswersCreateResponse>(MethodType.Unary, "dokime.host.v1.prompts.Answers", "Create",
        Marshallers.Create<global::Dokime.Clients.PromptsResources.AnswersCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsResources.AnswersCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.PromptsResources.AnswersCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.PromptsResources.AnswersCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.PromptsResources.AnswersCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompts.answers.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class RecordSchemasResource
{
  private readonly CallInvoker invoker;
  public RecordSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.RecordSchemasGetResponse> Get(string name, string parent, uint schemaVersion, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.RecordSchemasGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.RecordSchemasGetRequest, global::Dokime.Clients.RecordSchemasGetResponse>(MethodType.Unary, "dokime.host.v1.RecordSchemas", "Get",
        Marshallers.Create<global::Dokime.Clients.RecordSchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.RecordSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.RecordSchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.RecordSchemasGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.RecordSchemasGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call record_schemas.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.RecordSchemasListResponse> List(string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.RecordSchemasListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.RecordSchemasListRequest, global::Dokime.Clients.RecordSchemasListResponse>(MethodType.Unary, "dokime.host.v1.RecordSchemas", "List",
        Marshallers.Create<global::Dokime.Clients.RecordSchemasListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.RecordSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.RecordSchemasListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.RecordSchemasListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.RecordSchemasListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call record_schemas.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class ReportsResource
{
  private readonly CallInvoker invoker;
  public ReportsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.ReportsGetResponse> Get(global::Dokime.Clients.ReportSubject subject, string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ReportsGetRequest
    {
      Subject = subject,
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.ReportsGetRequest, global::Dokime.Clients.ReportsGetResponse>(MethodType.Unary, "dokime.host.v1.Reports", "Get",
        Marshallers.Create<global::Dokime.Clients.ReportsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ReportsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ReportsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ReportsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ReportsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call reports.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ReportsCreateResponse> Create(global::Dokime.Clients.ReportSubject subject, string documentId, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ReportsCreateRequest
    {
      Subject = subject,
      DocumentId = documentId,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ReportsCreateRequest, global::Dokime.Clients.ReportsCreateResponse>(MethodType.Unary, "dokime.host.v1.Reports", "Create",
        Marshallers.Create<global::Dokime.Clients.ReportsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ReportsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ReportsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ReportsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ReportsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call reports.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.ReportsPublishResponse> Publish(string name, global::Dokime.Clients.ReportSubject subject, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.ReportsPublishRequest
    {
      Name = name,
      Subject = subject,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.ReportsPublishRequest, global::Dokime.Clients.ReportsPublishResponse>(MethodType.Unary, "dokime.host.v1.Reports", "Publish",
        Marshallers.Create<global::Dokime.Clients.ReportsPublishRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ReportsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.ReportsPublishResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.ReportsPublishResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.ReportsPublishResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call reports.publish: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class SecretsResource
{
  private readonly CallInvoker invoker;
  public SecretsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.SecretsGetResponse> Get(string name, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SecretsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SecretsGetRequest, global::Dokime.Clients.SecretsGetResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Get",
        Marshallers.Create<global::Dokime.Clients.SecretsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SecretsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SecretsSetResponse> Set(string name, global::Dokime.Clients.SecretWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SecretsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.SecretsSetRequest, global::Dokime.Clients.SecretsSetResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Set",
        Marshallers.Create<global::Dokime.Clients.SecretsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SecretsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SecretsListResponse> List(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SecretsListRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SecretsListRequest, global::Dokime.Clients.SecretsListResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "List",
        Marshallers.Create<global::Dokime.Clients.SecretsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SecretsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.SecretsWatchResponse> Watch(global::Harmos.ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.SecretsWatchRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SecretsWatchRequest, global::Dokime.Clients.SecretsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Secrets", "Watch",
        Marshallers.Create<global::Dokime.Clients.SecretsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call secrets.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<global::Dokime.Clients.SecretsDeleteResponse> Delete(IReadOnlyList<string> names, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SecretsDeleteRequest
    {
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Names.Add(names);
    var method = new Method<global::Dokime.Clients.SecretsDeleteRequest, global::Dokime.Clients.SecretsDeleteResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Delete",
        Marshallers.Create<global::Dokime.Clients.SecretsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SecretsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SecretsExportResponse> Export(IReadOnlyList<string> names, ulong generation, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SecretsExportRequest
    {
      Generation = generation,
    };
    request.Names.Add(names);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SecretsExportRequest, global::Dokime.Clients.SecretsExportResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Export",
        Marshallers.Create<global::Dokime.Clients.SecretsExportRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsExportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsExportResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsExportResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SecretsExportResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.export: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SecretsImportResponse> Import(ByteString data, IReadOnlyList<global::Dokime.Clients.SecretImportChoice> choices, ulong generation, bool validateOnly, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SecretsImportRequest
    {
      Data = data,
      Generation = generation,
      ValidateOnly = validateOnly,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Choices.Add(choices);
    var method = new Method<global::Dokime.Clients.SecretsImportRequest, global::Dokime.Clients.SecretsImportResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Import",
        Marshallers.Create<global::Dokime.Clients.SecretsImportRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsImportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SecretsImportResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SecretsImportResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SecretsImportResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.import: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class SettingsResource
{
  private readonly CallInvoker invoker;
  public SettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.SettingsGetResponse> Get(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SettingsGetRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SettingsGetRequest, global::Dokime.Clients.SettingsGetResponse>(MethodType.Unary, "dokime.host.v1.Settings", "Get",
        Marshallers.Create<global::Dokime.Clients.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SettingsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SettingsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call settings.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SettingsSetResponse> Set(global::Dokime.Clients.ApplicationSettings value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SettingsSetRequest
    {
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.SettingsSetRequest, global::Dokime.Clients.SettingsSetResponse>(MethodType.Unary, "dokime.host.v1.Settings", "Set",
        Marshallers.Create<global::Dokime.Clients.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SettingsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SettingsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call settings.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SettingsSchemaResponse> Schema(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SettingsSchemaRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SettingsSchemaRequest, global::Dokime.Clients.SettingsSchemaResponse>(MethodType.Unary, "dokime.host.v1.Settings", "Schema",
        Marshallers.Create<global::Dokime.Clients.SettingsSchemaRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SettingsSchemaRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SettingsSchemaResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SettingsSchemaResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SettingsSchemaResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call settings.schema: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class StatusResource
{
  private readonly CallInvoker invoker;
  public StatusDiagnosticsResource Diagnostics { get; }
  public StatusLogsResource Logs { get; }
  public StatusTraceEventsResource TraceEvents { get; }
  public StatusResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Diagnostics = new StatusDiagnosticsResource(invoker);
    Logs = new StatusLogsResource(invoker);
    TraceEvents = new StatusTraceEventsResource(invoker);
  }
  public async Task<global::Harmos.Clients.StatusGetResponse> Get(CallOptions context = default)
  {
    var request = new global::Harmos.Clients.StatusGetRequest
    {
    };
    var method = new Method<global::Harmos.Clients.StatusGetRequest, global::Harmos.Clients.StatusGetResponse>(MethodType.Unary, "dokime.host.v1.Status", "Get",
        Marshallers.Create<global::Harmos.Clients.StatusGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.StatusGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.StatusGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.StatusGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.StatusGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call status.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Harmos.Clients.StatusWatchResponse> Watch(global::Harmos.Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Harmos.Clients.StatusWatchRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Harmos.Clients.StatusWatchRequest, global::Harmos.Clients.StatusWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Status", "Watch",
        Marshallers.Create<global::Harmos.Clients.StatusWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.StatusWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.StatusWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.StatusWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call status.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class StatusDiagnosticsResource
{
  private readonly CallInvoker invoker;
  public StatusDiagnosticsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Harmos.Clients.DiagnosticsGetResponse> Get(string name, ulong offset, ulong maxBytes, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.DiagnosticsGetRequest
    {
      Name = name,
      Offset = offset,
      MaxBytes = maxBytes,
    };
    var method = new Method<global::Harmos.Clients.DiagnosticsGetRequest, global::Harmos.Clients.DiagnosticsGetResponse>(MethodType.Unary, "dokime.host.v1.status.Diagnostics", "Get",
        Marshallers.Create<global::Harmos.Clients.DiagnosticsGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.DiagnosticsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.DiagnosticsGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.DiagnosticsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.DiagnosticsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call status.diagnostics.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class StatusLogsResource
{
  private readonly CallInvoker invoker;
  public StatusLogsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async IAsyncEnumerable<global::Harmos.Clients.LogsWatchResponse> Watch(ulong maxBytes, uint maxEvents, global::Harmos.Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Harmos.Clients.LogsWatchRequest
    {
      MaxBytes = maxBytes,
      MaxEvents = maxEvents,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Harmos.Clients.LogsWatchRequest, global::Harmos.Clients.LogsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.status.Logs", "Watch",
        Marshallers.Create<global::Harmos.Clients.LogsWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.LogsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.LogsWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.LogsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call status.logs.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class StatusTraceEventsResource
{
  private readonly CallInvoker invoker;
  public StatusTraceEventsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Harmos.Clients.TraceEventsListResponse> List(global::Harmos.Position? at = null, global::Harmos.PageRequest? page = null, CallOptions context = default)
  {
    var request = new global::Harmos.Clients.TraceEventsListRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<global::Harmos.Clients.TraceEventsListRequest, global::Harmos.Clients.TraceEventsListResponse>(MethodType.Unary, "dokime.host.v1.status.TraceEvents", "List",
        Marshallers.Create<global::Harmos.Clients.TraceEventsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.TraceEventsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.TraceEventsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.TraceEventsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Harmos.Clients.TraceEventsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call status.trace_events.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<global::Harmos.Clients.TraceEventsWatchResponse> Watch(ulong maxBytes, uint maxEvents, global::Harmos.Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Harmos.Clients.TraceEventsWatchRequest
    {
      MaxBytes = maxBytes,
      MaxEvents = maxEvents,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Harmos.Clients.TraceEventsWatchRequest, global::Harmos.Clients.TraceEventsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.status.TraceEvents", "Watch",
        Marshallers.Create<global::Harmos.Clients.TraceEventsWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.Clients.TraceEventsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.Clients.TraceEventsWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.Clients.TraceEventsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call status.trace_events.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
internal sealed class StorageResource
{
  public StorageSourcesResource Sources { get; }
  public StorageResource(CallInvoker invoker)
  {
    Sources = new StorageSourcesResource(invoker);
  }
}
internal sealed class StorageSourcesResource
{
  private readonly CallInvoker invoker;
  public StorageSourcesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.StorageResources.SourcesGetResponse> Get(global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.StorageResources.SourcesGetRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.StorageResources.SourcesGetRequest, global::Dokime.Clients.StorageResources.SourcesGetResponse>(MethodType.Unary, "dokime.host.v1.storage.Sources", "Get",
        Marshallers.Create<global::Dokime.Clients.StorageResources.SourcesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StorageResources.SourcesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.StorageResources.SourcesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StorageResources.SourcesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.StorageResources.SourcesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call storage.sources.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.StorageResources.SourcesSetResponse> Set(global::Dokime.Clients.StorageResources.StorageSources value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.StorageResources.SourcesSetRequest
    {
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<global::Dokime.Clients.StorageResources.SourcesSetRequest, global::Dokime.Clients.StorageResources.SourcesSetResponse>(MethodType.Unary, "dokime.host.v1.storage.Sources", "Set",
        Marshallers.Create<global::Dokime.Clients.StorageResources.SourcesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StorageResources.SourcesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.StorageResources.SourcesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StorageResources.SourcesSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.StorageResources.SourcesSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call storage.sources.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class StreamSchemasResource
{
  private readonly CallInvoker invoker;
  public StreamSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.StreamSchemasGetResponse> Get(string name, string parent, uint schemaVersion, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.StreamSchemasGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.StreamSchemasGetRequest, global::Dokime.Clients.StreamSchemasGetResponse>(MethodType.Unary, "dokime.host.v1.StreamSchemas", "Get",
        Marshallers.Create<global::Dokime.Clients.StreamSchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StreamSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.StreamSchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StreamSchemasGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.StreamSchemasGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call stream_schemas.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.StreamSchemasListResponse> List(string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.StreamSchemasListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.StreamSchemasListRequest, global::Dokime.Clients.StreamSchemasListResponse>(MethodType.Unary, "dokime.host.v1.StreamSchemas", "List",
        Marshallers.Create<global::Dokime.Clients.StreamSchemasListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StreamSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.StreamSchemasListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.StreamSchemasListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.StreamSchemasListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call stream_schemas.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
internal sealed class SuitesResource
{
  private readonly CallInvoker invoker;
  public SuitesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<global::Dokime.Clients.SuitesGetResponse> Get(string name, string parent, uint schemaVersion, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SuitesGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SuitesGetRequest, global::Dokime.Clients.SuitesGetResponse>(MethodType.Unary, "dokime.host.v1.Suites", "Get",
        Marshallers.Create<global::Dokime.Clients.SuitesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SuitesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SuitesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SuitesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SuitesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call suites.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<global::Dokime.Clients.SuitesListResponse> List(string parent, global::Harmos.ReadCut? at = null, CallOptions context = default)
  {
    var request = new global::Dokime.Clients.SuitesListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<global::Dokime.Clients.SuitesListRequest, global::Dokime.Clients.SuitesListResponse>(MethodType.Unary, "dokime.host.v1.Suites", "List",
        Marshallers.Create<global::Dokime.Clients.SuitesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SuitesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Clients.SuitesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Clients.SuitesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    global::Dokime.Clients.SuitesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call suites.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
