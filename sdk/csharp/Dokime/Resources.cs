// @generated from the protocol descriptor table; do not edit.
using Grpc.Core;
using Google.Protobuf;
namespace Dokime.Sdk;

public sealed class Client
{
  public AccessResource Access { get; }
  public AggregatesResource Aggregates { get; }
  public BackendResource Backend { get; }
  public CasesResource Cases { get; }
  public CatalogResource Catalog { get; }
  public CatalogEntriesResource CatalogEntries { get; }
  public ContentResource Content { get; }
  public CyclesResource Cycles { get; }
  public DocumentsResource Documents { get; }
  public EvidenceResource Evidence { get; }
  public ExecutionsResource Executions { get; }
  public InvocationsResource Invocations { get; }
  public MediaResource Media { get; }
  public PlotsResource Plots { get; }
  public PluginsResource Plugins { get; }
  public PromptsResource Prompts { get; }
  public ReportsResource Reports { get; }
  public SecretsResource Secrets { get; }
  public SettingsResource Settings { get; }
  public StatusResource Status { get; }
  public StorageResource Storage { get; }
  public TimeResource Time { get; }
  public Client(CallInvoker invoker)
  {
    Access = new AccessResource(invoker);
    Aggregates = new AggregatesResource(invoker);
    Backend = new BackendResource(invoker);
    Cases = new CasesResource(invoker);
    Catalog = new CatalogResource(invoker);
    CatalogEntries = new CatalogEntriesResource(invoker);
    Content = new ContentResource(invoker);
    Cycles = new CyclesResource(invoker);
    Documents = new DocumentsResource(invoker);
    Evidence = new EvidenceResource(invoker);
    Executions = new ExecutionsResource(invoker);
    Invocations = new InvocationsResource(invoker);
    Media = new MediaResource(invoker);
    Plots = new PlotsResource(invoker);
    Plugins = new PluginsResource(invoker);
    Prompts = new PromptsResource(invoker);
    Reports = new ReportsResource(invoker);
    Secrets = new SecretsResource(invoker);
    Settings = new SettingsResource(invoker);
    Status = new StatusResource(invoker);
    Storage = new StorageResource(invoker);
    Time = new TimeResource(invoker);
  }
}
public sealed class AccessResource
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
public sealed class AccessCertificatesResource
{
  private readonly CallInvoker invoker;
  public AccessCertificatesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.Access.CertificatesCreateResponse> Create(global::Harmos.V1.Access.CertificatesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.Access.CertificatesCreateRequest, global::Harmos.V1.Access.CertificatesCreateResponse>(MethodType.Unary, "harmos.v1.access.Certificates", "Create",
        Marshallers.Create<global::Harmos.V1.Access.CertificatesCreateRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.CertificatesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.Access.CertificatesCreateResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.CertificatesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class AccessListenerResource
{
  private readonly CallInvoker invoker;
  public AccessListenerResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.Access.ListenerGetResponse> Get(global::Harmos.V1.Access.ListenerGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.Access.ListenerGetRequest, global::Harmos.V1.Access.ListenerGetResponse>(MethodType.Unary, "harmos.v1.access.Listener", "Get",
        Marshallers.Create<global::Harmos.V1.Access.ListenerGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.ListenerGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.Access.ListenerGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.ListenerGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.Access.ListenerSetResponse> Set(global::Harmos.V1.Access.ListenerSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.Access.ListenerSetRequest, global::Harmos.V1.Access.ListenerSetResponse>(MethodType.Unary, "harmos.v1.access.Listener", "Set",
        Marshallers.Create<global::Harmos.V1.Access.ListenerSetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.ListenerSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.Access.ListenerSetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.ListenerSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class AccessPairingsResource
{
  private readonly CallInvoker invoker;
  public AccessPairingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.Access.PairingsCreateResponse> Create(global::Harmos.V1.Access.PairingsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.Access.PairingsCreateRequest, global::Harmos.V1.Access.PairingsCreateResponse>(MethodType.Unary, "harmos.v1.access.Pairings", "Create",
        Marshallers.Create<global::Harmos.V1.Access.PairingsCreateRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.PairingsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.Access.PairingsCreateResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.PairingsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.Access.PairingsListResponse> List(global::Harmos.V1.Access.PairingsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.Access.PairingsListRequest, global::Harmos.V1.Access.PairingsListResponse>(MethodType.Unary, "harmos.v1.access.Pairings", "List",
        Marshallers.Create<global::Harmos.V1.Access.PairingsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.PairingsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.Access.PairingsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.PairingsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.Access.PairingsDeleteResponse> Delete(global::Harmos.V1.Access.PairingsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.Access.PairingsDeleteRequest, global::Harmos.V1.Access.PairingsDeleteResponse>(MethodType.Unary, "harmos.v1.access.Pairings", "Delete",
        Marshallers.Create<global::Harmos.V1.Access.PairingsDeleteRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.PairingsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.Access.PairingsDeleteResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.Access.PairingsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class AggregatesResource
{
  private readonly CallInvoker invoker;
  public AggregatesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.AggregatesGetResponse> Get(global::Dokime.Protocol.V1.AggregatesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.AggregatesGetRequest, global::Dokime.Protocol.V1.AggregatesGetResponse>(MethodType.Unary, "dokime.v1.Aggregates", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.AggregatesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.AggregatesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.AggregatesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.AggregatesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class BackendResource
{
  private readonly CallInvoker invoker;
  public BackendOverviewResource Overview { get; }
  public BackendResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Overview = new BackendOverviewResource(invoker);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.BackendAppearanceResponse> Appearance(global::Dokime.Protocol.V1.BackendAppearanceRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.BackendAppearanceRequest, global::Dokime.Protocol.V1.BackendAppearanceResponse>(MethodType.Unary, "dokime.v1.Backend", "Appearance",
        Marshallers.Create<global::Dokime.Protocol.V1.BackendAppearanceRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.BackendAppearanceRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.BackendAppearanceResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.BackendAppearanceResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class BackendOverviewResource
{
  private readonly CallInvoker invoker;
  public BackendOverviewResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Backend.OverviewGetResponse> Get(global::Dokime.Protocol.V1.Backend.OverviewGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Backend.OverviewGetRequest, global::Dokime.Protocol.V1.Backend.OverviewGetResponse>(MethodType.Unary, "dokime.v1.backend.Overview", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Backend.OverviewGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Backend.OverviewGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Backend.OverviewGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Backend.OverviewGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Backend.OverviewWatchResponse> Watch(global::Dokime.Protocol.V1.Backend.OverviewWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Backend.OverviewWatchRequest, global::Dokime.Protocol.V1.Backend.OverviewWatchResponse>(MethodType.ServerStreaming, "dokime.v1.backend.Overview", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Backend.OverviewWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Backend.OverviewWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Backend.OverviewWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Backend.OverviewWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class CasesResource
{
  private readonly CallInvoker invoker;
  public CasesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CasesCreateResponse> Create(global::Dokime.Protocol.V1.CasesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CasesCreateRequest, global::Dokime.Protocol.V1.CasesCreateResponse>(MethodType.Unary, "dokime.v1.Cases", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.CasesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CasesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CasesGetResponse> Get(global::Dokime.Protocol.V1.CasesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CasesGetRequest, global::Dokime.Protocol.V1.CasesGetResponse>(MethodType.Unary, "dokime.v1.Cases", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.CasesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CasesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CasesListResponse> List(global::Dokime.Protocol.V1.CasesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CasesListRequest, global::Dokime.Protocol.V1.CasesListResponse>(MethodType.Unary, "dokime.v1.Cases", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.CasesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CasesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.CasesWatchResponse> Watch(global::Dokime.Protocol.V1.CasesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CasesWatchRequest, global::Dokime.Protocol.V1.CasesWatchResponse>(MethodType.ServerStreaming, "dokime.v1.Cases", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.CasesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CasesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CasesDeleteResponse> Delete(global::Dokime.Protocol.V1.CasesDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CasesDeleteRequest, global::Dokime.Protocol.V1.CasesDeleteResponse>(MethodType.Unary, "dokime.v1.Cases", "Delete",
        Marshallers.Create<global::Dokime.Protocol.V1.CasesDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CasesDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CasesParametersResponse> Parameters(global::Dokime.Protocol.V1.CasesParametersRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CasesParametersRequest, global::Dokime.Protocol.V1.CasesParametersResponse>(MethodType.Unary, "dokime.v1.Cases", "Parameters",
        Marshallers.Create<global::Dokime.Protocol.V1.CasesParametersRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CasesParametersResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CasesParametersResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class CatalogResource
{
  private readonly CallInvoker invoker;
  public CatalogPromptDefinitionsResource PromptDefinitions { get; }
  public CatalogRecordSchemasResource RecordSchemas { get; }
  public CatalogStreamSchemasResource StreamSchemas { get; }
  public CatalogSuitesResource Suites { get; }
  public CatalogResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    PromptDefinitions = new CatalogPromptDefinitionsResource(invoker);
    RecordSchemas = new CatalogRecordSchemasResource(invoker);
    StreamSchemas = new CatalogStreamSchemasResource(invoker);
    Suites = new CatalogSuitesResource(invoker);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CatalogAnalysesResponse> Analyses(global::Dokime.Protocol.V1.CatalogAnalysesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CatalogAnalysesRequest, global::Dokime.Protocol.V1.CatalogAnalysesResponse>(MethodType.Unary, "dokime.v1.Catalog", "Analyses",
        Marshallers.Create<global::Dokime.Protocol.V1.CatalogAnalysesRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CatalogAnalysesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CatalogAnalysesResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CatalogAnalysesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class CatalogPromptDefinitionsResource
{
  private readonly CallInvoker invoker;
  public CatalogPromptDefinitionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetResponse> Get(global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetRequest, global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetResponse>(MethodType.Unary, "dokime.v1.catalog.PromptDefinitions", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.PromptDefinitionsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListResponse> List(global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListRequest, global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListResponse>(MethodType.Unary, "dokime.v1.catalog.PromptDefinitions", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.PromptDefinitionsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class CatalogRecordSchemasResource
{
  private readonly CallInvoker invoker;
  public CatalogRecordSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.RecordSchemasGetResponse> Get(global::Dokime.Protocol.V1.Catalog.RecordSchemasGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.RecordSchemasGetRequest, global::Dokime.Protocol.V1.Catalog.RecordSchemasGetResponse>(MethodType.Unary, "dokime.v1.catalog.RecordSchemas", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.RecordSchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.RecordSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.RecordSchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.RecordSchemasGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.RecordSchemasListResponse> List(global::Dokime.Protocol.V1.Catalog.RecordSchemasListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.RecordSchemasListRequest, global::Dokime.Protocol.V1.Catalog.RecordSchemasListResponse>(MethodType.Unary, "dokime.v1.catalog.RecordSchemas", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.RecordSchemasListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.RecordSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.RecordSchemasListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.RecordSchemasListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class CatalogStreamSchemasResource
{
  private readonly CallInvoker invoker;
  public CatalogStreamSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.StreamSchemasGetResponse> Get(global::Dokime.Protocol.V1.Catalog.StreamSchemasGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.StreamSchemasGetRequest, global::Dokime.Protocol.V1.Catalog.StreamSchemasGetResponse>(MethodType.Unary, "dokime.v1.catalog.StreamSchemas", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.StreamSchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.StreamSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.StreamSchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.StreamSchemasGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.StreamSchemasListResponse> List(global::Dokime.Protocol.V1.Catalog.StreamSchemasListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.StreamSchemasListRequest, global::Dokime.Protocol.V1.Catalog.StreamSchemasListResponse>(MethodType.Unary, "dokime.v1.catalog.StreamSchemas", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.StreamSchemasListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.StreamSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.StreamSchemasListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.StreamSchemasListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class CatalogSuitesResource
{
  private readonly CallInvoker invoker;
  public CatalogSuitesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.SuitesGetResponse> Get(global::Dokime.Protocol.V1.Catalog.SuitesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.SuitesGetRequest, global::Dokime.Protocol.V1.Catalog.SuitesGetResponse>(MethodType.Unary, "dokime.v1.catalog.Suites", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.SuitesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.SuitesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.SuitesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.SuitesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Catalog.SuitesListResponse> List(global::Dokime.Protocol.V1.Catalog.SuitesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Catalog.SuitesListRequest, global::Dokime.Protocol.V1.Catalog.SuitesListResponse>(MethodType.Unary, "dokime.v1.catalog.Suites", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.SuitesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.SuitesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Catalog.SuitesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Catalog.SuitesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class CatalogEntriesResource
{
  private readonly CallInvoker invoker;
  public CatalogEntriesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.CatalogEntriesListResponse> List(global::Harmos.V1.CatalogEntriesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.CatalogEntriesListRequest, global::Harmos.V1.CatalogEntriesListResponse>(MethodType.Unary, "harmos.v1.CatalogEntries", "List",
        Marshallers.Create<global::Harmos.V1.CatalogEntriesListRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.CatalogEntriesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.CatalogEntriesListResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.CatalogEntriesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class ContentResource
{
  public ContentArtifactsResource Artifacts { get; }
  public ContentExecutionFilesResource ExecutionFiles { get; }
  public ContentResource(CallInvoker invoker)
  {
    Artifacts = new ContentArtifactsResource(invoker);
    ExecutionFiles = new ContentExecutionFilesResource(invoker);
  }
}
public sealed class ContentArtifactsResource
{
  private readonly CallInvoker invoker;
  public ContentArtifactsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Content.ArtifactsGetResponse> Get(global::Dokime.Protocol.V1.Content.ArtifactsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ArtifactsGetRequest, global::Dokime.Protocol.V1.Content.ArtifactsGetResponse>(MethodType.Unary, "dokime.v1.content.Artifacts", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Content.ArtifactsCreateResponse> Create(global::Dokime.Protocol.V1.Content.ArtifactsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ArtifactsCreateRequest, global::Dokime.Protocol.V1.Content.ArtifactsCreateResponse>(MethodType.Unary, "dokime.v1.content.Artifacts", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Content.ArtifactsListResponse> List(global::Dokime.Protocol.V1.Content.ArtifactsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ArtifactsListRequest, global::Dokime.Protocol.V1.Content.ArtifactsListResponse>(MethodType.Unary, "dokime.v1.content.Artifacts", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Content.ArtifactsWatchResponse> Watch(global::Dokime.Protocol.V1.Content.ArtifactsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ArtifactsWatchRequest, global::Dokime.Protocol.V1.Content.ArtifactsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.content.Artifacts", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ArtifactsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ArtifactsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class ContentExecutionFilesResource
{
  private readonly CallInvoker invoker;
  public ContentExecutionFilesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Content.ExecutionFilesGetResponse> Get(global::Dokime.Protocol.V1.Content.ExecutionFilesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ExecutionFilesGetRequest, global::Dokime.Protocol.V1.Content.ExecutionFilesGetResponse>(MethodType.Unary, "dokime.v1.content.ExecutionFiles", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ExecutionFilesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ExecutionFilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ExecutionFilesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ExecutionFilesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Content.ExecutionFilesListResponse> List(global::Dokime.Protocol.V1.Content.ExecutionFilesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ExecutionFilesListRequest, global::Dokime.Protocol.V1.Content.ExecutionFilesListResponse>(MethodType.Unary, "dokime.v1.content.ExecutionFiles", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ExecutionFilesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ExecutionFilesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ExecutionFilesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ExecutionFilesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Content.ExecutionFilesWatchResponse> Watch(global::Dokime.Protocol.V1.Content.ExecutionFilesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Content.ExecutionFilesWatchRequest, global::Dokime.Protocol.V1.Content.ExecutionFilesWatchResponse>(MethodType.ServerStreaming, "dokime.v1.content.ExecutionFiles", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ExecutionFilesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ExecutionFilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Content.ExecutionFilesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Content.ExecutionFilesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class CyclesResource
{
  private readonly CallInvoker invoker;
  public CyclesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CyclesGetResponse> Get(global::Dokime.Protocol.V1.CyclesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CyclesGetRequest, global::Dokime.Protocol.V1.CyclesGetResponse>(MethodType.Unary, "dokime.v1.Cycles", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CyclesSetResponse> Set(global::Dokime.Protocol.V1.CyclesSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CyclesSetRequest, global::Dokime.Protocol.V1.CyclesSetResponse>(MethodType.Unary, "dokime.v1.Cycles", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CyclesListResponse> List(global::Dokime.Protocol.V1.CyclesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CyclesListRequest, global::Dokime.Protocol.V1.CyclesListResponse>(MethodType.Unary, "dokime.v1.Cycles", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.CyclesWatchResponse> Watch(global::Dokime.Protocol.V1.CyclesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CyclesWatchRequest, global::Dokime.Protocol.V1.CyclesWatchResponse>(MethodType.ServerStreaming, "dokime.v1.Cycles", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CyclesCreateResponse> Create(global::Dokime.Protocol.V1.CyclesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CyclesCreateRequest, global::Dokime.Protocol.V1.CyclesCreateResponse>(MethodType.Unary, "dokime.v1.Cycles", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.CyclesSealResponse> Seal(global::Dokime.Protocol.V1.CyclesSealRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.CyclesSealRequest, global::Dokime.Protocol.V1.CyclesSealResponse>(MethodType.Unary, "dokime.v1.Cycles", "Seal",
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesSealRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesSealRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.CyclesSealResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.CyclesSealResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class DocumentsResource
{
  private readonly CallInvoker invoker;
  public DocumentsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.DocumentsGetResponse> Get(global::Dokime.Protocol.V1.DocumentsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.DocumentsGetRequest, global::Dokime.Protocol.V1.DocumentsGetResponse>(MethodType.Unary, "dokime.v1.Documents", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.DocumentsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.DocumentsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.DocumentsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.DocumentsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.DocumentsWatchResponse> Watch(global::Dokime.Protocol.V1.DocumentsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.DocumentsWatchRequest, global::Dokime.Protocol.V1.DocumentsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.Documents", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.DocumentsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.DocumentsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.DocumentsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.DocumentsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class EvidenceResource
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
  public AsyncUnaryCall<global::Dokime.Protocol.V1.EvidenceStatisticsResponse> Statistics(global::Dokime.Protocol.V1.EvidenceStatisticsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.EvidenceStatisticsRequest, global::Dokime.Protocol.V1.EvidenceStatisticsResponse>(MethodType.Unary, "dokime.v1.Evidence", "Statistics",
        Marshallers.Create<global::Dokime.Protocol.V1.EvidenceStatisticsRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.EvidenceStatisticsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.EvidenceStatisticsResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.EvidenceStatisticsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.EvidenceArchiveResponse> Archive(global::Dokime.Protocol.V1.EvidenceArchiveRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.EvidenceArchiveRequest, global::Dokime.Protocol.V1.EvidenceArchiveResponse>(MethodType.Unary, "dokime.v1.Evidence", "Archive",
        Marshallers.Create<global::Dokime.Protocol.V1.EvidenceArchiveRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.EvidenceArchiveRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.EvidenceArchiveResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.EvidenceArchiveResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class EvidenceHealthEventsResource
{
  private readonly CallInvoker invoker;
  public EvidenceHealthEventsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.HealthEventsListResponse> List(global::Dokime.Protocol.V1.Evidence.HealthEventsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.HealthEventsListRequest, global::Dokime.Protocol.V1.Evidence.HealthEventsListResponse>(MethodType.Unary, "dokime.v1.evidence.HealthEvents", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.HealthEventsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.HealthEventsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.HealthEventsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.HealthEventsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Evidence.HealthEventsWatchResponse> Watch(global::Dokime.Protocol.V1.Evidence.HealthEventsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.HealthEventsWatchRequest, global::Dokime.Protocol.V1.Evidence.HealthEventsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.evidence.HealthEvents", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.HealthEventsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.HealthEventsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.HealthEventsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.HealthEventsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class EvidenceNotesResource
{
  private readonly CallInvoker invoker;
  public EvidenceNotesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.NotesGetResponse> Get(global::Dokime.Protocol.V1.Evidence.NotesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.NotesGetRequest, global::Dokime.Protocol.V1.Evidence.NotesGetResponse>(MethodType.Unary, "dokime.v1.evidence.Notes", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.NotesSetResponse> Set(global::Dokime.Protocol.V1.Evidence.NotesSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.NotesSetRequest, global::Dokime.Protocol.V1.Evidence.NotesSetResponse>(MethodType.Unary, "dokime.v1.evidence.Notes", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.NotesListResponse> List(global::Dokime.Protocol.V1.Evidence.NotesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.NotesListRequest, global::Dokime.Protocol.V1.Evidence.NotesListResponse>(MethodType.Unary, "dokime.v1.evidence.Notes", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Evidence.NotesWatchResponse> Watch(global::Dokime.Protocol.V1.Evidence.NotesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.NotesWatchRequest, global::Dokime.Protocol.V1.Evidence.NotesWatchResponse>(MethodType.ServerStreaming, "dokime.v1.evidence.Notes", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.NotesCreateResponse> Create(global::Dokime.Protocol.V1.Evidence.NotesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.NotesCreateRequest, global::Dokime.Protocol.V1.Evidence.NotesCreateResponse>(MethodType.Unary, "dokime.v1.evidence.Notes", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.NotesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.NotesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class EvidenceRecordsResource
{
  private readonly CallInvoker invoker;
  public EvidenceRecordsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.RecordsListResponse> List(global::Dokime.Protocol.V1.Evidence.RecordsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.RecordsListRequest, global::Dokime.Protocol.V1.Evidence.RecordsListResponse>(MethodType.Unary, "dokime.v1.evidence.Records", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.RecordsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.RecordsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.RecordsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.RecordsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Evidence.RecordsWatchResponse> Watch(global::Dokime.Protocol.V1.Evidence.RecordsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.RecordsWatchRequest, global::Dokime.Protocol.V1.Evidence.RecordsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.evidence.Records", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.RecordsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.RecordsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.RecordsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.RecordsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class EvidenceSamplesResource
{
  private readonly CallInvoker invoker;
  public EvidenceSamplesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.SamplesListResponse> List(global::Dokime.Protocol.V1.Evidence.SamplesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.SamplesListRequest, global::Dokime.Protocol.V1.Evidence.SamplesListResponse>(MethodType.Unary, "dokime.v1.evidence.Samples", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.SamplesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.SamplesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.SamplesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.SamplesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Evidence.SamplesWatchResponse> Watch(global::Dokime.Protocol.V1.Evidence.SamplesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.SamplesWatchRequest, global::Dokime.Protocol.V1.Evidence.SamplesWatchResponse>(MethodType.ServerStreaming, "dokime.v1.evidence.Samples", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.SamplesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.SamplesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.SamplesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.SamplesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class EvidenceTraceabilityResource
{
  private readonly CallInvoker invoker;
  public EvidenceTraceabilityResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Evidence.TraceabilityGetResponse> Get(global::Dokime.Protocol.V1.Evidence.TraceabilityGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.TraceabilityGetRequest, global::Dokime.Protocol.V1.Evidence.TraceabilityGetResponse>(MethodType.Unary, "dokime.v1.evidence.Traceability", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.TraceabilityGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.TraceabilityGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.TraceabilityGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.TraceabilityGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Evidence.TraceabilityWatchResponse> Watch(global::Dokime.Protocol.V1.Evidence.TraceabilityWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Evidence.TraceabilityWatchRequest, global::Dokime.Protocol.V1.Evidence.TraceabilityWatchResponse>(MethodType.ServerStreaming, "dokime.v1.evidence.Traceability", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.TraceabilityWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.TraceabilityWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Evidence.TraceabilityWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Evidence.TraceabilityWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class ExecutionsResource
{
  private readonly CallInvoker invoker;
  public ExecutionsCommandsResource Commands { get; }
  public ExecutionsProgressResource Progress { get; }
  public ExecutionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Commands = new ExecutionsCommandsResource(invoker);
    Progress = new ExecutionsProgressResource(invoker);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsGetResponse> Get(global::Dokime.Protocol.V1.ExecutionsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsGetRequest, global::Dokime.Protocol.V1.ExecutionsGetResponse>(MethodType.Unary, "dokime.v1.Executions", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsSetResponse> Set(global::Dokime.Protocol.V1.ExecutionsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsSetRequest, global::Dokime.Protocol.V1.ExecutionsSetResponse>(MethodType.Unary, "dokime.v1.Executions", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsListResponse> List(global::Dokime.Protocol.V1.ExecutionsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsListRequest, global::Dokime.Protocol.V1.ExecutionsListResponse>(MethodType.Unary, "dokime.v1.Executions", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.ExecutionsWatchResponse> Watch(global::Dokime.Protocol.V1.ExecutionsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsWatchRequest, global::Dokime.Protocol.V1.ExecutionsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.Executions", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsCreateResponse> Create(global::Dokime.Protocol.V1.ExecutionsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsCreateRequest, global::Dokime.Protocol.V1.ExecutionsCreateResponse>(MethodType.Unary, "dokime.v1.Executions", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsStartResponse> Start(global::Dokime.Protocol.V1.ExecutionsStartRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsStartRequest, global::Dokime.Protocol.V1.ExecutionsStartResponse>(MethodType.Unary, "dokime.v1.Executions", "Start",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsStartRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsStartRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsStartResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsStartResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsStopResponse> Stop(global::Dokime.Protocol.V1.ExecutionsStopRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsStopRequest, global::Dokime.Protocol.V1.ExecutionsStopResponse>(MethodType.Unary, "dokime.v1.Executions", "Stop",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsStopRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsStopResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsStopResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsCancelResponse> Cancel(global::Dokime.Protocol.V1.ExecutionsCancelRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsCancelRequest, global::Dokime.Protocol.V1.ExecutionsCancelResponse>(MethodType.Unary, "dokime.v1.Executions", "Cancel",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsCancelRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsCancelRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsCancelResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsCancelResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsDeleteResponse> Delete(global::Dokime.Protocol.V1.ExecutionsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsDeleteRequest, global::Dokime.Protocol.V1.ExecutionsDeleteResponse>(MethodType.Unary, "dokime.v1.Executions", "Delete",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsParametersResponse> Parameters(global::Dokime.Protocol.V1.ExecutionsParametersRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsParametersRequest, global::Dokime.Protocol.V1.ExecutionsParametersResponse>(MethodType.Unary, "dokime.v1.Executions", "Parameters",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsParametersRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsParametersResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsParametersResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ExecutionsStepsResponse> Steps(global::Dokime.Protocol.V1.ExecutionsStepsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ExecutionsStepsRequest, global::Dokime.Protocol.V1.ExecutionsStepsResponse>(MethodType.Unary, "dokime.v1.Executions", "Steps",
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsStepsRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsStepsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ExecutionsStepsResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ExecutionsStepsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class ExecutionsCommandsResource
{
  private readonly CallInvoker invoker;
  public ExecutionsCommandsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Executions.CommandsGetResponse> Get(global::Dokime.Protocol.V1.Executions.CommandsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Executions.CommandsGetRequest, global::Dokime.Protocol.V1.Executions.CommandsGetResponse>(MethodType.Unary, "dokime.v1.executions.Commands", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.CommandsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.CommandsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.CommandsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.CommandsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Executions.CommandsCreateResponse> Create(global::Dokime.Protocol.V1.Executions.CommandsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Executions.CommandsCreateRequest, global::Dokime.Protocol.V1.Executions.CommandsCreateResponse>(MethodType.Unary, "dokime.v1.executions.Commands", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.CommandsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.CommandsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.CommandsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.CommandsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class ExecutionsProgressResource
{
  private readonly CallInvoker invoker;
  public ExecutionsProgressResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Executions.ProgressGetResponse> Get(global::Dokime.Protocol.V1.Executions.ProgressGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Executions.ProgressGetRequest, global::Dokime.Protocol.V1.Executions.ProgressGetResponse>(MethodType.Unary, "dokime.v1.executions.Progress", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.ProgressGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.ProgressGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.ProgressGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.ProgressGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Executions.ProgressWatchResponse> Watch(global::Dokime.Protocol.V1.Executions.ProgressWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Executions.ProgressWatchRequest, global::Dokime.Protocol.V1.Executions.ProgressWatchResponse>(MethodType.ServerStreaming, "dokime.v1.executions.Progress", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.ProgressWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.ProgressWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Executions.ProgressWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Executions.ProgressWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class InvocationsResource
{
  private readonly CallInvoker invoker;
  public InvocationsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.InvocationsInvokeResponse> Invoke(global::Harmos.V1.InvocationsInvokeRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.InvocationsInvokeRequest, global::Harmos.V1.InvocationsInvokeResponse>(MethodType.Unary, "harmos.v1.Invocations", "Invoke",
        Marshallers.Create<global::Harmos.V1.InvocationsInvokeRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsInvokeRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.InvocationsInvokeResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsInvokeResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.InvocationsGetResponse> Get(global::Harmos.V1.InvocationsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.InvocationsGetRequest, global::Harmos.V1.InvocationsGetResponse>(MethodType.Unary, "harmos.v1.Invocations", "Get",
        Marshallers.Create<global::Harmos.V1.InvocationsGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.InvocationsGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.InvocationsCancelResponse> Cancel(global::Harmos.V1.InvocationsCancelRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.InvocationsCancelRequest, global::Harmos.V1.InvocationsCancelResponse>(MethodType.Unary, "harmos.v1.Invocations", "Cancel",
        Marshallers.Create<global::Harmos.V1.InvocationsCancelRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsCancelRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.InvocationsCancelResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsCancelResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.InvocationsWatchResponse> Watch(global::Harmos.V1.InvocationsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.InvocationsWatchRequest, global::Harmos.V1.InvocationsWatchResponse>(MethodType.ServerStreaming, "harmos.v1.Invocations", "Watch",
        Marshallers.Create<global::Harmos.V1.InvocationsWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.InvocationsWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.InvocationsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class MediaResource
{
  public MediaCapturesResource Captures { get; }
  public MediaResource(CallInvoker invoker)
  {
    Captures = new MediaCapturesResource(invoker);
  }
}
public sealed class MediaCapturesResource
{
  private readonly CallInvoker invoker;
  public MediaCapturesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Media.CapturesCreateResponse> Create(global::Dokime.Protocol.V1.Media.CapturesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Media.CapturesCreateRequest, global::Dokime.Protocol.V1.Media.CapturesCreateResponse>(MethodType.Unary, "dokime.v1.media.Captures", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Media.CapturesGetResponse> Get(global::Dokime.Protocol.V1.Media.CapturesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Media.CapturesGetRequest, global::Dokime.Protocol.V1.Media.CapturesGetResponse>(MethodType.Unary, "dokime.v1.media.Captures", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Media.CapturesWatchResponse> Watch(global::Dokime.Protocol.V1.Media.CapturesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Media.CapturesWatchRequest, global::Dokime.Protocol.V1.Media.CapturesWatchResponse>(MethodType.ServerStreaming, "dokime.v1.media.Captures", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Media.CapturesStopResponse> Stop(global::Dokime.Protocol.V1.Media.CapturesStopRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Media.CapturesStopRequest, global::Dokime.Protocol.V1.Media.CapturesStopResponse>(MethodType.Unary, "dokime.v1.media.Captures", "Stop",
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesStopRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Media.CapturesStopResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Media.CapturesStopResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class PlotsResource
{
  public PlotsSettingsResource Settings { get; }
  public PlotsSnapshotsResource Snapshots { get; }
  public PlotsResource(CallInvoker invoker)
  {
    Settings = new PlotsSettingsResource(invoker);
    Snapshots = new PlotsSnapshotsResource(invoker);
  }
}
public sealed class PlotsSettingsResource
{
  private readonly CallInvoker invoker;
  public PlotsSettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Plots.SettingsGetResponse> Get(global::Dokime.Protocol.V1.Plots.SettingsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Plots.SettingsGetRequest, global::Dokime.Protocol.V1.Plots.SettingsGetResponse>(MethodType.Unary, "dokime.v1.plots.Settings", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SettingsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Plots.SettingsSetResponse> Set(global::Dokime.Protocol.V1.Plots.SettingsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Plots.SettingsSetRequest, global::Dokime.Protocol.V1.Plots.SettingsSetResponse>(MethodType.Unary, "dokime.v1.plots.Settings", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SettingsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class PlotsSnapshotsResource
{
  private readonly CallInvoker invoker;
  public PlotsSnapshotsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Plots.SnapshotsListResponse> List(global::Dokime.Protocol.V1.Plots.SnapshotsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Plots.SnapshotsListRequest, global::Dokime.Protocol.V1.Plots.SnapshotsListResponse>(MethodType.Unary, "dokime.v1.plots.Snapshots", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SnapshotsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SnapshotsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SnapshotsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SnapshotsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.Plots.SnapshotsWatchResponse> Watch(global::Dokime.Protocol.V1.Plots.SnapshotsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Plots.SnapshotsWatchRequest, global::Dokime.Protocol.V1.Plots.SnapshotsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.plots.Snapshots", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SnapshotsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SnapshotsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Plots.SnapshotsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plots.SnapshotsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class PluginsResource
{
  private readonly CallInvoker invoker;
  public PluginsSettingsResource Settings { get; }
  public PluginsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Settings = new PluginsSettingsResource(invoker);
  }
  public AsyncUnaryCall<global::Harmos.V1.PluginsGetResponse> Get(global::Harmos.V1.PluginsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsGetRequest, global::Harmos.V1.PluginsGetResponse>(MethodType.Unary, "harmos.v1.Plugins", "Get",
        Marshallers.Create<global::Harmos.V1.PluginsGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.PluginsListResponse> List(global::Harmos.V1.PluginsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsListRequest, global::Harmos.V1.PluginsListResponse>(MethodType.Unary, "harmos.v1.Plugins", "List",
        Marshallers.Create<global::Harmos.V1.PluginsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.PluginsWatchResponse> Watch(global::Harmos.V1.PluginsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsWatchRequest, global::Harmos.V1.PluginsWatchResponse>(MethodType.ServerStreaming, "harmos.v1.Plugins", "Watch",
        Marshallers.Create<global::Harmos.V1.PluginsWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.PluginsCreateResponse> Create(global::Harmos.V1.PluginsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsCreateRequest, global::Harmos.V1.PluginsCreateResponse>(MethodType.Unary, "harmos.v1.Plugins", "Create",
        Marshallers.Create<global::Harmos.V1.PluginsCreateRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsCreateResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.PluginsSetResponse> Set(global::Harmos.V1.PluginsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsSetRequest, global::Harmos.V1.PluginsSetResponse>(MethodType.Unary, "harmos.v1.Plugins", "Set",
        Marshallers.Create<global::Harmos.V1.PluginsSetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsSetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.PluginsDeleteResponse> Delete(global::Harmos.V1.PluginsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsDeleteRequest, global::Harmos.V1.PluginsDeleteResponse>(MethodType.Unary, "harmos.v1.Plugins", "Delete",
        Marshallers.Create<global::Harmos.V1.PluginsDeleteRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsDeleteResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.PluginsRestartResponse> Restart(global::Harmos.V1.PluginsRestartRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.PluginsRestartRequest, global::Harmos.V1.PluginsRestartResponse>(MethodType.Unary, "harmos.v1.Plugins", "Restart",
        Marshallers.Create<global::Harmos.V1.PluginsRestartRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsRestartRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.PluginsRestartResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.PluginsRestartResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class PluginsSettingsResource
{
  private readonly CallInvoker invoker;
  public PluginsSettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Plugins.SettingsGetResponse> Get(global::Dokime.Protocol.V1.Plugins.SettingsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Plugins.SettingsGetRequest, global::Dokime.Protocol.V1.Plugins.SettingsGetResponse>(MethodType.Unary, "dokime.v1.plugins.Settings", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Plugins.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plugins.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Plugins.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plugins.SettingsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Plugins.SettingsSetResponse> Set(global::Dokime.Protocol.V1.Plugins.SettingsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Plugins.SettingsSetRequest, global::Dokime.Protocol.V1.Plugins.SettingsSetResponse>(MethodType.Unary, "dokime.v1.plugins.Settings", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.Plugins.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plugins.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Plugins.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Plugins.SettingsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class PromptsResource
{
  private readonly CallInvoker invoker;
  public PromptsAnswersResource Answers { get; }
  public PromptsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Answers = new PromptsAnswersResource(invoker);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.PromptsGetResponse> Get(global::Dokime.Protocol.V1.PromptsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.PromptsGetRequest, global::Dokime.Protocol.V1.PromptsGetResponse>(MethodType.Unary, "dokime.v1.Prompts", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.PromptsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.PromptsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.PromptsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.PromptsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.PromptsListResponse> List(global::Dokime.Protocol.V1.PromptsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.PromptsListRequest, global::Dokime.Protocol.V1.PromptsListResponse>(MethodType.Unary, "dokime.v1.Prompts", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.PromptsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.PromptsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.PromptsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.PromptsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.PromptsWatchResponse> Watch(global::Dokime.Protocol.V1.PromptsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.PromptsWatchRequest, global::Dokime.Protocol.V1.PromptsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.Prompts", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.PromptsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.PromptsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.PromptsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.PromptsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class PromptsAnswersResource
{
  private readonly CallInvoker invoker;
  public PromptsAnswersResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Prompts.AnswersCreateResponse> Create(global::Dokime.Protocol.V1.Prompts.AnswersCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Prompts.AnswersCreateRequest, global::Dokime.Protocol.V1.Prompts.AnswersCreateResponse>(MethodType.Unary, "dokime.v1.prompts.Answers", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.Prompts.AnswersCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Prompts.AnswersCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Prompts.AnswersCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Prompts.AnswersCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class ReportsResource
{
  private readonly CallInvoker invoker;
  public ReportsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ReportsGetResponse> Get(global::Dokime.Protocol.V1.ReportsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ReportsGetRequest, global::Dokime.Protocol.V1.ReportsGetResponse>(MethodType.Unary, "dokime.v1.Reports", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.ReportsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ReportsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ReportsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ReportsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ReportsCreateResponse> Create(global::Dokime.Protocol.V1.ReportsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ReportsCreateRequest, global::Dokime.Protocol.V1.ReportsCreateResponse>(MethodType.Unary, "dokime.v1.Reports", "Create",
        Marshallers.Create<global::Dokime.Protocol.V1.ReportsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ReportsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ReportsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ReportsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.ReportsPublishResponse> Publish(global::Dokime.Protocol.V1.ReportsPublishRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.ReportsPublishRequest, global::Dokime.Protocol.V1.ReportsPublishResponse>(MethodType.Unary, "dokime.v1.Reports", "Publish",
        Marshallers.Create<global::Dokime.Protocol.V1.ReportsPublishRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ReportsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.ReportsPublishResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.ReportsPublishResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class SecretsResource
{
  private readonly CallInvoker invoker;
  public SecretsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SecretsGetResponse> Get(global::Dokime.Protocol.V1.SecretsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsGetRequest, global::Dokime.Protocol.V1.SecretsGetResponse>(MethodType.Unary, "dokime.v1.Secrets", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SecretsSetResponse> Set(global::Dokime.Protocol.V1.SecretsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsSetRequest, global::Dokime.Protocol.V1.SecretsSetResponse>(MethodType.Unary, "dokime.v1.Secrets", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SecretsListResponse> List(global::Dokime.Protocol.V1.SecretsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsListRequest, global::Dokime.Protocol.V1.SecretsListResponse>(MethodType.Unary, "dokime.v1.Secrets", "List",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.V1.SecretsWatchResponse> Watch(global::Dokime.Protocol.V1.SecretsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsWatchRequest, global::Dokime.Protocol.V1.SecretsWatchResponse>(MethodType.ServerStreaming, "dokime.v1.Secrets", "Watch",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SecretsDeleteResponse> Delete(global::Dokime.Protocol.V1.SecretsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsDeleteRequest, global::Dokime.Protocol.V1.SecretsDeleteResponse>(MethodType.Unary, "dokime.v1.Secrets", "Delete",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SecretsExportResponse> Export(global::Dokime.Protocol.V1.SecretsExportRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsExportRequest, global::Dokime.Protocol.V1.SecretsExportResponse>(MethodType.Unary, "dokime.v1.Secrets", "Export",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsExportRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsExportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsExportResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsExportResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SecretsImportResponse> Import(global::Dokime.Protocol.V1.SecretsImportRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SecretsImportRequest, global::Dokime.Protocol.V1.SecretsImportResponse>(MethodType.Unary, "dokime.v1.Secrets", "Import",
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsImportRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsImportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SecretsImportResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SecretsImportResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class SettingsResource
{
  private readonly CallInvoker invoker;
  public SettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SettingsGetResponse> Get(global::Dokime.Protocol.V1.SettingsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SettingsGetRequest, global::Dokime.Protocol.V1.SettingsGetResponse>(MethodType.Unary, "dokime.v1.Settings", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SettingsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SettingsSetResponse> Set(global::Dokime.Protocol.V1.SettingsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SettingsSetRequest, global::Dokime.Protocol.V1.SettingsSetResponse>(MethodType.Unary, "dokime.v1.Settings", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SettingsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.SettingsSchemaResponse> Schema(global::Dokime.Protocol.V1.SettingsSchemaRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.SettingsSchemaRequest, global::Dokime.Protocol.V1.SettingsSchemaResponse>(MethodType.Unary, "dokime.v1.Settings", "Schema",
        Marshallers.Create<global::Dokime.Protocol.V1.SettingsSchemaRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SettingsSchemaRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.SettingsSchemaResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.SettingsSchemaResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class StatusResource
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
  public AsyncUnaryCall<global::Harmos.V1.StatusGetResponse> Get(global::Harmos.V1.StatusGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusGetRequest, global::Harmos.V1.StatusGetResponse>(MethodType.Unary, "harmos.v1.Status", "Get",
        Marshallers.Create<global::Harmos.V1.StatusGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StatusWatchResponse> Watch(global::Harmos.V1.StatusWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusWatchRequest, global::Harmos.V1.StatusWatchResponse>(MethodType.ServerStreaming, "harmos.v1.Status", "Watch",
        Marshallers.Create<global::Harmos.V1.StatusWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class StatusDiagnosticsResource
{
  private readonly CallInvoker invoker;
  public StatusDiagnosticsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.StatusResources.DiagnosticsGetResponse> Get(global::Harmos.V1.StatusResources.DiagnosticsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusResources.DiagnosticsGetRequest, global::Harmos.V1.StatusResources.DiagnosticsGetResponse>(MethodType.Unary, "harmos.v1.status.Diagnostics", "Get",
        Marshallers.Create<global::Harmos.V1.StatusResources.DiagnosticsGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.DiagnosticsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusResources.DiagnosticsGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.DiagnosticsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class StatusLogsResource
{
  private readonly CallInvoker invoker;
  public StatusLogsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.StatusResources.LogsListResponse> List(global::Harmos.V1.StatusResources.LogsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusResources.LogsListRequest, global::Harmos.V1.StatusResources.LogsListResponse>(MethodType.Unary, "harmos.v1.status.Logs", "List",
        Marshallers.Create<global::Harmos.V1.StatusResources.LogsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.LogsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusResources.LogsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.LogsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StatusResources.LogsWatchResponse> Watch(global::Harmos.V1.StatusResources.LogsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusResources.LogsWatchRequest, global::Harmos.V1.StatusResources.LogsWatchResponse>(MethodType.ServerStreaming, "harmos.v1.status.Logs", "Watch",
        Marshallers.Create<global::Harmos.V1.StatusResources.LogsWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.LogsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusResources.LogsWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.LogsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class StatusTraceEventsResource
{
  private readonly CallInvoker invoker;
  public StatusTraceEventsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.StatusResources.TraceEventsListResponse> List(global::Harmos.V1.StatusResources.TraceEventsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusResources.TraceEventsListRequest, global::Harmos.V1.StatusResources.TraceEventsListResponse>(MethodType.Unary, "harmos.v1.status.TraceEvents", "List",
        Marshallers.Create<global::Harmos.V1.StatusResources.TraceEventsListRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.TraceEventsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusResources.TraceEventsListResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.TraceEventsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StatusResources.TraceEventsWatchResponse> Watch(global::Harmos.V1.StatusResources.TraceEventsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StatusResources.TraceEventsWatchRequest, global::Harmos.V1.StatusResources.TraceEventsWatchResponse>(MethodType.ServerStreaming, "harmos.v1.status.TraceEvents", "Watch",
        Marshallers.Create<global::Harmos.V1.StatusResources.TraceEventsWatchRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.TraceEventsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StatusResources.TraceEventsWatchResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StatusResources.TraceEventsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class StorageResource
{
  private readonly CallInvoker invoker;
  public StorageSourcesResource Sources { get; }
  public StorageResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Sources = new StorageSourcesResource(invoker);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageCreateFileProductionResponse> CreateFileProduction(global::Harmos.V1.StorageCreateFileProductionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageCreateFileProductionRequest, global::Harmos.V1.StorageCreateFileProductionResponse>(MethodType.Unary, "harmos.v1.Storage", "CreateFileProduction",
        Marshallers.Create<global::Harmos.V1.StorageCreateFileProductionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCreateFileProductionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageCreateFileProductionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCreateFileProductionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetFileProductionResponse> GetFileProduction(global::Harmos.V1.StorageGetFileProductionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetFileProductionRequest, global::Harmos.V1.StorageGetFileProductionResponse>(MethodType.Unary, "harmos.v1.Storage", "GetFileProduction",
        Marshallers.Create<global::Harmos.V1.StorageGetFileProductionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetFileProductionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetFileProductionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetFileProductionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageAppendFileBytesResponse> AppendFileBytes(global::Harmos.V1.StorageAppendFileBytesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageAppendFileBytesRequest, global::Harmos.V1.StorageAppendFileBytesResponse>(MethodType.Unary, "harmos.v1.Storage", "AppendFileBytes",
        Marshallers.Create<global::Harmos.V1.StorageAppendFileBytesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendFileBytesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageAppendFileBytesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendFileBytesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageCreateFileWriterResponse> CreateFileWriter(global::Harmos.V1.StorageCreateFileWriterRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageCreateFileWriterRequest, global::Harmos.V1.StorageCreateFileWriterResponse>(MethodType.Unary, "harmos.v1.Storage", "CreateFileWriter",
        Marshallers.Create<global::Harmos.V1.StorageCreateFileWriterRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCreateFileWriterRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageCreateFileWriterResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCreateFileWriterResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StoragePublishFileProductionResponse> PublishFileProduction(global::Harmos.V1.StoragePublishFileProductionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StoragePublishFileProductionRequest, global::Harmos.V1.StoragePublishFileProductionResponse>(MethodType.Unary, "harmos.v1.Storage", "PublishFileProduction",
        Marshallers.Create<global::Harmos.V1.StoragePublishFileProductionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StoragePublishFileProductionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StoragePublishFileProductionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StoragePublishFileProductionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageDeleteFileProductionResponse> DeleteFileProduction(global::Harmos.V1.StorageDeleteFileProductionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageDeleteFileProductionRequest, global::Harmos.V1.StorageDeleteFileProductionResponse>(MethodType.Unary, "harmos.v1.Storage", "DeleteFileProduction",
        Marshallers.Create<global::Harmos.V1.StorageDeleteFileProductionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageDeleteFileProductionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageDeleteFileProductionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageDeleteFileProductionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetFileBytesResponse> GetFileBytes(global::Harmos.V1.StorageGetFileBytesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetFileBytesRequest, global::Harmos.V1.StorageGetFileBytesResponse>(MethodType.Unary, "harmos.v1.Storage", "GetFileBytes",
        Marshallers.Create<global::Harmos.V1.StorageGetFileBytesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetFileBytesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetFileBytesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetFileBytesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageCapabilitiesResponse> Capabilities(global::Harmos.V1.StorageCapabilitiesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageCapabilitiesRequest, global::Harmos.V1.StorageCapabilitiesResponse>(MethodType.Unary, "harmos.v1.Storage", "Capabilities",
        Marshallers.Create<global::Harmos.V1.StorageCapabilitiesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCapabilitiesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageCapabilitiesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCapabilitiesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageCreateSnapshotResponse> CreateSnapshot(global::Harmos.V1.StorageCreateSnapshotRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageCreateSnapshotRequest, global::Harmos.V1.StorageCreateSnapshotResponse>(MethodType.Unary, "harmos.v1.Storage", "CreateSnapshot",
        Marshallers.Create<global::Harmos.V1.StorageCreateSnapshotRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCreateSnapshotRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageCreateSnapshotResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCreateSnapshotResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageDeleteSnapshotResponse> DeleteSnapshot(global::Harmos.V1.StorageDeleteSnapshotRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageDeleteSnapshotRequest, global::Harmos.V1.StorageDeleteSnapshotResponse>(MethodType.Unary, "harmos.v1.Storage", "DeleteSnapshot",
        Marshallers.Create<global::Harmos.V1.StorageDeleteSnapshotRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageDeleteSnapshotRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageDeleteSnapshotResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageDeleteSnapshotResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageListColumnRowsResponse> ListColumnRows(global::Harmos.V1.StorageListColumnRowsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageListColumnRowsRequest, global::Harmos.V1.StorageListColumnRowsResponse>(MethodType.Unary, "harmos.v1.Storage", "ListColumnRows",
        Marshallers.Create<global::Harmos.V1.StorageListColumnRowsRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListColumnRowsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageListColumnRowsResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListColumnRowsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageListIndexEntriesResponse> ListIndexEntries(global::Harmos.V1.StorageListIndexEntriesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageListIndexEntriesRequest, global::Harmos.V1.StorageListIndexEntriesResponse>(MethodType.Unary, "harmos.v1.Storage", "ListIndexEntries",
        Marshallers.Create<global::Harmos.V1.StorageListIndexEntriesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListIndexEntriesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageListIndexEntriesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListIndexEntriesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageSetIndexDefinitionResponse> SetIndexDefinition(global::Harmos.V1.StorageSetIndexDefinitionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageSetIndexDefinitionRequest, global::Harmos.V1.StorageSetIndexDefinitionResponse>(MethodType.Unary, "harmos.v1.Storage", "SetIndexDefinition",
        Marshallers.Create<global::Harmos.V1.StorageSetIndexDefinitionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageSetIndexDefinitionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageSetIndexDefinitionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageSetIndexDefinitionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageSetProjectionDefinitionResponse> SetProjectionDefinition(global::Harmos.V1.StorageSetProjectionDefinitionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageSetProjectionDefinitionRequest, global::Harmos.V1.StorageSetProjectionDefinitionResponse>(MethodType.Unary, "harmos.v1.Storage", "SetProjectionDefinition",
        Marshallers.Create<global::Harmos.V1.StorageSetProjectionDefinitionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageSetProjectionDefinitionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageSetProjectionDefinitionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageSetProjectionDefinitionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageAdvanceProjectionResponse> AdvanceProjection(global::Harmos.V1.StorageAdvanceProjectionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageAdvanceProjectionRequest, global::Harmos.V1.StorageAdvanceProjectionResponse>(MethodType.Unary, "harmos.v1.Storage", "AdvanceProjection",
        Marshallers.Create<global::Harmos.V1.StorageAdvanceProjectionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAdvanceProjectionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageAdvanceProjectionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAdvanceProjectionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageListProjectionRowsResponse> ListProjectionRows(global::Harmos.V1.StorageListProjectionRowsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageListProjectionRowsRequest, global::Harmos.V1.StorageListProjectionRowsResponse>(MethodType.Unary, "harmos.v1.Storage", "ListProjectionRows",
        Marshallers.Create<global::Harmos.V1.StorageListProjectionRowsRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListProjectionRowsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageListProjectionRowsResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListProjectionRowsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetProjectionStatusResponse> GetProjectionStatus(global::Harmos.V1.StorageGetProjectionStatusRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetProjectionStatusRequest, global::Harmos.V1.StorageGetProjectionStatusResponse>(MethodType.Unary, "harmos.v1.Storage", "GetProjectionStatus",
        Marshallers.Create<global::Harmos.V1.StorageGetProjectionStatusRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetProjectionStatusRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetProjectionStatusResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetProjectionStatusResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageSetObjectResponse> SetObject(global::Harmos.V1.StorageSetObjectRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageSetObjectRequest, global::Harmos.V1.StorageSetObjectResponse>(MethodType.Unary, "harmos.v1.Storage", "SetObject",
        Marshallers.Create<global::Harmos.V1.StorageSetObjectRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageSetObjectRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageSetObjectResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageSetObjectResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageListRowsResponse> ListRows(global::Harmos.V1.StorageListRowsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageListRowsRequest, global::Harmos.V1.StorageListRowsResponse>(MethodType.Unary, "harmos.v1.Storage", "ListRows",
        Marshallers.Create<global::Harmos.V1.StorageListRowsRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListRowsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageListRowsResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListRowsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StorageWatchRowsResponse> WatchRows(global::Harmos.V1.StorageWatchRowsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageWatchRowsRequest, global::Harmos.V1.StorageWatchRowsResponse>(MethodType.ServerStreaming, "harmos.v1.Storage", "WatchRows",
        Marshallers.Create<global::Harmos.V1.StorageWatchRowsRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchRowsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageWatchRowsResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchRowsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageAppendRecordsResponse> AppendRecords(global::Harmos.V1.StorageAppendRecordsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageAppendRecordsRequest, global::Harmos.V1.StorageAppendRecordsResponse>(MethodType.Unary, "harmos.v1.Storage", "AppendRecords",
        Marshallers.Create<global::Harmos.V1.StorageAppendRecordsRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendRecordsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageAppendRecordsResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendRecordsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageAppendSamplesResponse> AppendSamples(global::Harmos.V1.StorageAppendSamplesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageAppendSamplesRequest, global::Harmos.V1.StorageAppendSamplesResponse>(MethodType.Unary, "harmos.v1.Storage", "AppendSamples",
        Marshallers.Create<global::Harmos.V1.StorageAppendSamplesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendSamplesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageAppendSamplesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendSamplesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageAppendObjectRevisionsResponse> AppendObjectRevisions(global::Harmos.V1.StorageAppendObjectRevisionsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageAppendObjectRevisionsRequest, global::Harmos.V1.StorageAppendObjectRevisionsResponse>(MethodType.Unary, "harmos.v1.Storage", "AppendObjectRevisions",
        Marshallers.Create<global::Harmos.V1.StorageAppendObjectRevisionsRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendObjectRevisionsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageAppendObjectRevisionsResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageAppendObjectRevisionsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetFileHashResponse> GetFileHash(global::Harmos.V1.StorageGetFileHashRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetFileHashRequest, global::Harmos.V1.StorageGetFileHashResponse>(MethodType.Unary, "harmos.v1.Storage", "GetFileHash",
        Marshallers.Create<global::Harmos.V1.StorageGetFileHashRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetFileHashRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetFileHashResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetFileHashResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageCaptureFileResponse> CaptureFile(global::Harmos.V1.StorageCaptureFileRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageCaptureFileRequest, global::Harmos.V1.StorageCaptureFileResponse>(MethodType.Unary, "harmos.v1.Storage", "CaptureFile",
        Marshallers.Create<global::Harmos.V1.StorageCaptureFileRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCaptureFileRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageCaptureFileResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageCaptureFileResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StorageWatchFileBytesResponse> WatchFileBytes(global::Harmos.V1.StorageWatchFileBytesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageWatchFileBytesRequest, global::Harmos.V1.StorageWatchFileBytesResponse>(MethodType.ServerStreaming, "harmos.v1.Storage", "WatchFileBytes",
        Marshallers.Create<global::Harmos.V1.StorageWatchFileBytesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchFileBytesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageWatchFileBytesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchFileBytesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StorageWatchFileProductionResponse> WatchFileProduction(global::Harmos.V1.StorageWatchFileProductionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageWatchFileProductionRequest, global::Harmos.V1.StorageWatchFileProductionResponse>(MethodType.ServerStreaming, "harmos.v1.Storage", "WatchFileProduction",
        Marshallers.Create<global::Harmos.V1.StorageWatchFileProductionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchFileProductionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageWatchFileProductionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchFileProductionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageListDirectoryEntriesResponse> ListDirectoryEntries(global::Harmos.V1.StorageListDirectoryEntriesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageListDirectoryEntriesRequest, global::Harmos.V1.StorageListDirectoryEntriesResponse>(MethodType.Unary, "harmos.v1.Storage", "ListDirectoryEntries",
        Marshallers.Create<global::Harmos.V1.StorageListDirectoryEntriesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListDirectoryEntriesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageListDirectoryEntriesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageListDirectoryEntriesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Harmos.V1.StorageWatchDirectoryEntriesResponse> WatchDirectoryEntries(global::Harmos.V1.StorageWatchDirectoryEntriesRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageWatchDirectoryEntriesRequest, global::Harmos.V1.StorageWatchDirectoryEntriesResponse>(MethodType.ServerStreaming, "harmos.v1.Storage", "WatchDirectoryEntries",
        Marshallers.Create<global::Harmos.V1.StorageWatchDirectoryEntriesRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchDirectoryEntriesRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageWatchDirectoryEntriesResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageWatchDirectoryEntriesResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetIndexDefinitionResponse> GetIndexDefinition(global::Harmos.V1.StorageGetIndexDefinitionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetIndexDefinitionRequest, global::Harmos.V1.StorageGetIndexDefinitionResponse>(MethodType.Unary, "harmos.v1.Storage", "GetIndexDefinition",
        Marshallers.Create<global::Harmos.V1.StorageGetIndexDefinitionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetIndexDefinitionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetIndexDefinitionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetIndexDefinitionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetProjectionDefinitionResponse> GetProjectionDefinition(global::Harmos.V1.StorageGetProjectionDefinitionRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetProjectionDefinitionRequest, global::Harmos.V1.StorageGetProjectionDefinitionResponse>(MethodType.Unary, "harmos.v1.Storage", "GetProjectionDefinition",
        Marshallers.Create<global::Harmos.V1.StorageGetProjectionDefinitionRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetProjectionDefinitionRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetProjectionDefinitionResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetProjectionDefinitionResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.StorageGetObjectResponse> GetObject(global::Harmos.V1.StorageGetObjectRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.StorageGetObjectRequest, global::Harmos.V1.StorageGetObjectResponse>(MethodType.Unary, "harmos.v1.Storage", "GetObject",
        Marshallers.Create<global::Harmos.V1.StorageGetObjectRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetObjectRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.StorageGetObjectResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.StorageGetObjectResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class StorageSourcesResource
{
  private readonly CallInvoker invoker;
  public StorageSourcesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Storage.SourcesGetResponse> Get(global::Dokime.Protocol.V1.Storage.SourcesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Storage.SourcesGetRequest, global::Dokime.Protocol.V1.Storage.SourcesGetResponse>(MethodType.Unary, "dokime.v1.storage.Sources", "Get",
        Marshallers.Create<global::Dokime.Protocol.V1.Storage.SourcesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Storage.SourcesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Storage.SourcesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Storage.SourcesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.V1.Storage.SourcesSetResponse> Set(global::Dokime.Protocol.V1.Storage.SourcesSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.V1.Storage.SourcesSetRequest, global::Dokime.Protocol.V1.Storage.SourcesSetResponse>(MethodType.Unary, "dokime.v1.storage.Sources", "Set",
        Marshallers.Create<global::Dokime.Protocol.V1.Storage.SourcesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Storage.SourcesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.V1.Storage.SourcesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.V1.Storage.SourcesSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class TimeResource
{
  private readonly CallInvoker invoker;
  public TimeResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Harmos.V1.TimeGetResponse> Get(global::Harmos.V1.TimeGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.TimeGetRequest, global::Harmos.V1.TimeGetResponse>(MethodType.Unary, "harmos.v1.Time", "Get",
        Marshallers.Create<global::Harmos.V1.TimeGetRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.TimeGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.TimeGetResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.TimeGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Harmos.V1.TimeSleepResponse> Sleep(global::Harmos.V1.TimeSleepRequest request, CallOptions options = default)
  {
    var method = new Method<global::Harmos.V1.TimeSleepRequest, global::Harmos.V1.TimeSleepResponse>(MethodType.Unary, "harmos.v1.Time", "Sleep",
        Marshallers.Create<global::Harmos.V1.TimeSleepRequest>(value => value.ToByteArray(), bytes => global::Harmos.V1.TimeSleepRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Harmos.V1.TimeSleepResponse>(value => value.ToByteArray(), bytes => global::Harmos.V1.TimeSleepResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
