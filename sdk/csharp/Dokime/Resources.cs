// @generated from the protocol descriptor table; do not edit.
using Grpc.Core;
using Google.Protobuf;
namespace Dokime.Sdk;

public sealed class Client
{
  public AccessResource Access { get; }
  public AggregatesResource Aggregates { get; }
  public AnalysesResource Analyses { get; }
  public BackendResource Backend { get; }
  public CasesResource Cases { get; }
  public CatalogEntriesResource CatalogEntries { get; }
  public ContentResource Content { get; }
  public CyclesResource Cycles { get; }
  public DocumentsResource Documents { get; }
  public EvidenceResource Evidence { get; }
  public ExecutionsResource Executions { get; }
  public FilesResource Files { get; }
  public InvocationsResource Invocations { get; }
  public MediaResource Media { get; }
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
  public TimeResource Time { get; }
  public Client(CallInvoker invoker)
  {
    Access = new AccessResource(invoker);
    Aggregates = new AggregatesResource(invoker);
    Analyses = new AnalysesResource(invoker);
    Backend = new BackendResource(invoker);
    Cases = new CasesResource(invoker);
    CatalogEntries = new CatalogEntriesResource(invoker);
    Content = new ContentResource(invoker);
    Cycles = new CyclesResource(invoker);
    Documents = new DocumentsResource(invoker);
    Evidence = new EvidenceResource(invoker);
    Executions = new ExecutionsResource(invoker);
    Files = new FilesResource(invoker);
    Invocations = new InvocationsResource(invoker);
    Media = new MediaResource(invoker);
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.AggregatesGetResponse> Get(global::Dokime.Protocol.Client.V1.AggregatesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.AggregatesGetRequest, global::Dokime.Protocol.Client.V1.AggregatesGetResponse>(MethodType.Unary, "dokime.client.v1.Aggregates", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.AggregatesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.AggregatesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.AggregatesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.AggregatesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class AnalysesResource
{
  private readonly CallInvoker invoker;
  public AnalysesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.AnalysesListResponse> List(global::Dokime.Protocol.Client.V1.AnalysesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.AnalysesListRequest, global::Dokime.Protocol.Client.V1.AnalysesListResponse>(MethodType.Unary, "dokime.client.v1.Analyses", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.AnalysesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.AnalysesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.AnalysesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.AnalysesListResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.BackendAppearanceResponse> Appearance(global::Dokime.Protocol.Client.V1.BackendAppearanceRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.BackendAppearanceRequest, global::Dokime.Protocol.Client.V1.BackendAppearanceResponse>(MethodType.Unary, "dokime.client.v1.Backend", "Appearance",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.BackendAppearanceRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.BackendAppearanceRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.BackendAppearanceResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.BackendAppearanceResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetResponse> Get(global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetRequest, global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetResponse>(MethodType.Unary, "dokime.client.v1.backend.Overview", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.BackendResources.OverviewGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchResponse> Watch(global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchRequest, global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.backend.Overview", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.BackendResources.OverviewWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CasesCreateResponse> Create(global::Dokime.Protocol.Client.V1.CasesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CasesCreateRequest, global::Dokime.Protocol.Client.V1.CasesCreateResponse>(MethodType.Unary, "dokime.client.v1.Cases", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CasesGetResponse> Get(global::Dokime.Protocol.Client.V1.CasesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CasesGetRequest, global::Dokime.Protocol.Client.V1.CasesGetResponse>(MethodType.Unary, "dokime.client.v1.Cases", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CasesListResponse> List(global::Dokime.Protocol.Client.V1.CasesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CasesListRequest, global::Dokime.Protocol.Client.V1.CasesListResponse>(MethodType.Unary, "dokime.client.v1.Cases", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.CasesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.CasesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CasesWatchRequest, global::Dokime.Protocol.Client.V1.CasesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Cases", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CasesDeleteResponse> Delete(global::Dokime.Protocol.Client.V1.CasesDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CasesDeleteRequest, global::Dokime.Protocol.Client.V1.CasesDeleteResponse>(MethodType.Unary, "dokime.client.v1.Cases", "Delete",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CasesParametersResponse> Parameters(global::Dokime.Protocol.Client.V1.CasesParametersRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CasesParametersRequest, global::Dokime.Protocol.Client.V1.CasesParametersResponse>(MethodType.Unary, "dokime.client.v1.Cases", "Parameters",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesParametersRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CasesParametersResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CasesParametersResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetResponse> Get(global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetRequest, global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetResponse>(MethodType.Unary, "dokime.client.v1.content.Artifacts", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateResponse> Create(global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateRequest, global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateResponse>(MethodType.Unary, "dokime.client.v1.content.Artifacts", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListResponse> List(global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListRequest, global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListResponse>(MethodType.Unary, "dokime.client.v1.content.Artifacts", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchRequest, global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.content.Artifacts", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ArtifactsWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetResponse> Get(global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetRequest, global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetResponse>(MethodType.Unary, "dokime.client.v1.content.ExecutionFiles", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListResponse> List(global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListRequest, global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListResponse>(MethodType.Unary, "dokime.client.v1.content.ExecutionFiles", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchRequest, global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.content.ExecutionFiles", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ContentResources.ExecutionFilesWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CyclesGetResponse> Get(global::Dokime.Protocol.Client.V1.CyclesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CyclesGetRequest, global::Dokime.Protocol.Client.V1.CyclesGetResponse>(MethodType.Unary, "dokime.client.v1.Cycles", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CyclesSetResponse> Set(global::Dokime.Protocol.Client.V1.CyclesSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CyclesSetRequest, global::Dokime.Protocol.Client.V1.CyclesSetResponse>(MethodType.Unary, "dokime.client.v1.Cycles", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CyclesListResponse> List(global::Dokime.Protocol.Client.V1.CyclesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CyclesListRequest, global::Dokime.Protocol.Client.V1.CyclesListResponse>(MethodType.Unary, "dokime.client.v1.Cycles", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.CyclesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.CyclesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CyclesWatchRequest, global::Dokime.Protocol.Client.V1.CyclesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Cycles", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CyclesCreateResponse> Create(global::Dokime.Protocol.Client.V1.CyclesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CyclesCreateRequest, global::Dokime.Protocol.Client.V1.CyclesCreateResponse>(MethodType.Unary, "dokime.client.v1.Cycles", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.CyclesSealResponse> Seal(global::Dokime.Protocol.Client.V1.CyclesSealRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.CyclesSealRequest, global::Dokime.Protocol.Client.V1.CyclesSealResponse>(MethodType.Unary, "dokime.client.v1.Cycles", "Seal",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesSealRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesSealRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.CyclesSealResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.CyclesSealResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.DocumentsGetResponse> Get(global::Dokime.Protocol.Client.V1.DocumentsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.DocumentsGetRequest, global::Dokime.Protocol.Client.V1.DocumentsGetResponse>(MethodType.Unary, "dokime.client.v1.Documents", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.DocumentsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.DocumentsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.DocumentsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.DocumentsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.DocumentsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.DocumentsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.DocumentsWatchRequest, global::Dokime.Protocol.Client.V1.DocumentsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Documents", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.DocumentsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.DocumentsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.DocumentsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.DocumentsWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceStatisticsResponse> Statistics(global::Dokime.Protocol.Client.V1.EvidenceStatisticsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceStatisticsRequest, global::Dokime.Protocol.Client.V1.EvidenceStatisticsResponse>(MethodType.Unary, "dokime.client.v1.Evidence", "Statistics",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceStatisticsRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceStatisticsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceStatisticsResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceStatisticsResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceArchiveResponse> Archive(global::Dokime.Protocol.Client.V1.EvidenceArchiveRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceArchiveRequest, global::Dokime.Protocol.Client.V1.EvidenceArchiveResponse>(MethodType.Unary, "dokime.client.v1.Evidence", "Archive",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceArchiveRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceArchiveRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceArchiveResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceArchiveResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListResponse> List(global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListResponse>(MethodType.Unary, "dokime.client.v1.evidence.HealthEvents", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.evidence.HealthEvents", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.HealthEventsWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetResponse> Get(global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetResponse>(MethodType.Unary, "dokime.client.v1.evidence.Notes", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetResponse> Set(global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetResponse>(MethodType.Unary, "dokime.client.v1.evidence.Notes", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListResponse> List(global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListResponse>(MethodType.Unary, "dokime.client.v1.evidence.Notes", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.evidence.Notes", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateResponse> Create(global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateResponse>(MethodType.Unary, "dokime.client.v1.evidence.Notes", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.NotesCreateResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListResponse> List(global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListResponse>(MethodType.Unary, "dokime.client.v1.evidence.Records", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.evidence.Records", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.RecordsWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListResponse> List(global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListResponse>(MethodType.Unary, "dokime.client.v1.evidence.Samples", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.evidence.Samples", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.SamplesWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetResponse> Get(global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetResponse>(MethodType.Unary, "dokime.client.v1.evidence.Traceability", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchResponse> Watch(global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchRequest, global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.evidence.Traceability", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.EvidenceResources.TraceabilityWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class ExecutionsResource
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsGetResponse> Get(global::Dokime.Protocol.Client.V1.ExecutionsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsGetRequest, global::Dokime.Protocol.Client.V1.ExecutionsGetResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsSetResponse> Set(global::Dokime.Protocol.Client.V1.ExecutionsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsSetRequest, global::Dokime.Protocol.Client.V1.ExecutionsSetResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsListResponse> List(global::Dokime.Protocol.Client.V1.ExecutionsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsListRequest, global::Dokime.Protocol.Client.V1.ExecutionsListResponse>(MethodType.Unary, "dokime.client.v1.Executions", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.ExecutionsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.ExecutionsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsWatchRequest, global::Dokime.Protocol.Client.V1.ExecutionsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Executions", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsCreateResponse> Create(global::Dokime.Protocol.Client.V1.ExecutionsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsCreateRequest, global::Dokime.Protocol.Client.V1.ExecutionsCreateResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsStartResponse> Start(global::Dokime.Protocol.Client.V1.ExecutionsStartRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsStartRequest, global::Dokime.Protocol.Client.V1.ExecutionsStartResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Start",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsStartRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsStartRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsStartResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsStartResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsStopResponse> Stop(global::Dokime.Protocol.Client.V1.ExecutionsStopRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsStopRequest, global::Dokime.Protocol.Client.V1.ExecutionsStopResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Stop",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsStopRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsStopResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsStopResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsCancelResponse> Cancel(global::Dokime.Protocol.Client.V1.ExecutionsCancelRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsCancelRequest, global::Dokime.Protocol.Client.V1.ExecutionsCancelResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Cancel",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsCancelRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsCancelRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsCancelResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsCancelResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsDeleteResponse> Delete(global::Dokime.Protocol.Client.V1.ExecutionsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsDeleteRequest, global::Dokime.Protocol.Client.V1.ExecutionsDeleteResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Delete",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsParametersResponse> Parameters(global::Dokime.Protocol.Client.V1.ExecutionsParametersRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsParametersRequest, global::Dokime.Protocol.Client.V1.ExecutionsParametersResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Parameters",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsParametersRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsParametersResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsParametersResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsStepsResponse> Steps(global::Dokime.Protocol.Client.V1.ExecutionsStepsRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsStepsRequest, global::Dokime.Protocol.Client.V1.ExecutionsStepsResponse>(MethodType.Unary, "dokime.client.v1.Executions", "Steps",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsStepsRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsStepsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsStepsResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsStepsResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetResponse> Get(global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetRequest, global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetResponse>(MethodType.Unary, "dokime.client.v1.executions.Commands", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateResponse> Create(global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateRequest, global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateResponse>(MethodType.Unary, "dokime.client.v1.executions.Commands", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.CommandsCreateResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetResponse> Get(global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetRequest, global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetResponse>(MethodType.Unary, "dokime.client.v1.executions.Progress", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchResponse> Watch(global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchRequest, global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.executions.Progress", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.ProgressWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
}
public sealed class ExecutionsSchemasResource
{
  private readonly CallInvoker invoker;
  public ExecutionsSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetResponse> Get(global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetRequest, global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetResponse>(MethodType.Unary, "dokime.client.v1.executions.Schemas", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ExecutionsResources.SchemasGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class FilesResource
{
  private readonly CallInvoker invoker;
  public FilesUploadsResource Uploads { get; }
  public FilesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Uploads = new FilesUploadsResource(invoker);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesGetResponse> Get(global::Dokime.Protocol.Client.V1.FilesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesGetRequest, global::Dokime.Protocol.Client.V1.FilesGetResponse>(MethodType.Unary, "dokime.client.v1.Files", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesReadResponse> Read(global::Dokime.Protocol.Client.V1.FilesReadRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesReadRequest, global::Dokime.Protocol.Client.V1.FilesReadResponse>(MethodType.Unary, "dokime.client.v1.Files", "Read",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesReadRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesReadRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesReadResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesReadResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.FilesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.FilesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesWatchRequest, global::Dokime.Protocol.Client.V1.FilesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Files", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesDeleteResponse> Delete(global::Dokime.Protocol.Client.V1.FilesDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesDeleteRequest, global::Dokime.Protocol.Client.V1.FilesDeleteResponse>(MethodType.Unary, "dokime.client.v1.Files", "Delete",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class FilesUploadsResource
{
  private readonly CallInvoker invoker;
  public FilesUploadsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateResponse> Create(global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateRequest, global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateResponse>(MethodType.Unary, "dokime.client.v1.files.Uploads", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetResponse> Get(global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetRequest, global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetResponse>(MethodType.Unary, "dokime.client.v1.files.Uploads", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeResponse> Resume(global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeRequest, global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeResponse>(MethodType.Unary, "dokime.client.v1.files.Uploads", "Resume",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsResumeResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendResponse> Append(global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendRequest, global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendResponse>(MethodType.Unary, "dokime.client.v1.files.Uploads", "Append",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsAppendResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishResponse> Publish(global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishRequest, global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishResponse>(MethodType.Unary, "dokime.client.v1.files.Uploads", "Publish",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsPublishResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteResponse> Delete(global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteRequest, global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteResponse>(MethodType.Unary, "dokime.client.v1.files.Uploads", "Delete",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.FilesResources.UploadsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateResponse> Create(global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateRequest, global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateResponse>(MethodType.Unary, "dokime.client.v1.media.Captures", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetResponse> Get(global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetRequest, global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetResponse>(MethodType.Unary, "dokime.client.v1.media.Captures", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchResponse> Watch(global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchRequest, global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.media.Captures", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopResponse> Stop(global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopRequest, global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopResponse>(MethodType.Unary, "dokime.client.v1.media.Captures", "Stop",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.MediaResources.CapturesStopResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetResponse> Get(global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetRequest, global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetResponse>(MethodType.Unary, "dokime.client.v1.plots.Settings", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SettingsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetResponse> Set(global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetRequest, global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetResponse>(MethodType.Unary, "dokime.client.v1.plots.Settings", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SettingsSetResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListResponse> List(global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListRequest, global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListResponse>(MethodType.Unary, "dokime.client.v1.plots.Snapshots", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchRequest, global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.plots.Snapshots", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PlotsResources.SnapshotsWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetResponse> Get(global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetRequest, global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetResponse>(MethodType.Unary, "dokime.client.v1.plugins.Settings", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PluginsResources.SettingsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetResponse> Set(global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetRequest, global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetResponse>(MethodType.Unary, "dokime.client.v1.plugins.Settings", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PluginsResources.SettingsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class PromptDefinitionsResource
{
  private readonly CallInvoker invoker;
  public PromptDefinitionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PromptDefinitionsGetResponse> Get(global::Dokime.Protocol.Client.V1.PromptDefinitionsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PromptDefinitionsGetRequest, global::Dokime.Protocol.Client.V1.PromptDefinitionsGetResponse>(MethodType.Unary, "dokime.client.v1.PromptDefinitions", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptDefinitionsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptDefinitionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptDefinitionsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptDefinitionsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PromptDefinitionsListResponse> List(global::Dokime.Protocol.Client.V1.PromptDefinitionsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PromptDefinitionsListRequest, global::Dokime.Protocol.Client.V1.PromptDefinitionsListResponse>(MethodType.Unary, "dokime.client.v1.PromptDefinitions", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptDefinitionsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptDefinitionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptDefinitionsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptDefinitionsListResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PromptsGetResponse> Get(global::Dokime.Protocol.Client.V1.PromptsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PromptsGetRequest, global::Dokime.Protocol.Client.V1.PromptsGetResponse>(MethodType.Unary, "dokime.client.v1.Prompts", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PromptsListResponse> List(global::Dokime.Protocol.Client.V1.PromptsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PromptsListRequest, global::Dokime.Protocol.Client.V1.PromptsListResponse>(MethodType.Unary, "dokime.client.v1.Prompts", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.PromptsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.PromptsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PromptsWatchRequest, global::Dokime.Protocol.Client.V1.PromptsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Prompts", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsWatchResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateResponse> Create(global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateRequest, global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateResponse>(MethodType.Unary, "dokime.client.v1.prompts.Answers", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.PromptsResources.AnswersCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class RecordSchemasResource
{
  private readonly CallInvoker invoker;
  public RecordSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.RecordSchemasGetResponse> Get(global::Dokime.Protocol.Client.V1.RecordSchemasGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.RecordSchemasGetRequest, global::Dokime.Protocol.Client.V1.RecordSchemasGetResponse>(MethodType.Unary, "dokime.client.v1.RecordSchemas", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.RecordSchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.RecordSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.RecordSchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.RecordSchemasGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.RecordSchemasListResponse> List(global::Dokime.Protocol.Client.V1.RecordSchemasListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.RecordSchemasListRequest, global::Dokime.Protocol.Client.V1.RecordSchemasListResponse>(MethodType.Unary, "dokime.client.v1.RecordSchemas", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.RecordSchemasListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.RecordSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.RecordSchemasListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.RecordSchemasListResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ReportsGetResponse> Get(global::Dokime.Protocol.Client.V1.ReportsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ReportsGetRequest, global::Dokime.Protocol.Client.V1.ReportsGetResponse>(MethodType.Unary, "dokime.client.v1.Reports", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ReportsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ReportsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ReportsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ReportsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ReportsCreateResponse> Create(global::Dokime.Protocol.Client.V1.ReportsCreateRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ReportsCreateRequest, global::Dokime.Protocol.Client.V1.ReportsCreateResponse>(MethodType.Unary, "dokime.client.v1.Reports", "Create",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ReportsCreateRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ReportsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ReportsCreateResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ReportsCreateResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.ReportsPublishResponse> Publish(global::Dokime.Protocol.Client.V1.ReportsPublishRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.ReportsPublishRequest, global::Dokime.Protocol.Client.V1.ReportsPublishResponse>(MethodType.Unary, "dokime.client.v1.Reports", "Publish",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ReportsPublishRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ReportsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.ReportsPublishResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.ReportsPublishResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SecretsGetResponse> Get(global::Dokime.Protocol.Client.V1.SecretsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsGetRequest, global::Dokime.Protocol.Client.V1.SecretsGetResponse>(MethodType.Unary, "dokime.client.v1.Secrets", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SecretsSetResponse> Set(global::Dokime.Protocol.Client.V1.SecretsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsSetRequest, global::Dokime.Protocol.Client.V1.SecretsSetResponse>(MethodType.Unary, "dokime.client.v1.Secrets", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SecretsListResponse> List(global::Dokime.Protocol.Client.V1.SecretsListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsListRequest, global::Dokime.Protocol.Client.V1.SecretsListResponse>(MethodType.Unary, "dokime.client.v1.Secrets", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncServerStreamingCall<global::Dokime.Protocol.Client.V1.SecretsWatchResponse> Watch(global::Dokime.Protocol.Client.V1.SecretsWatchRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsWatchRequest, global::Dokime.Protocol.Client.V1.SecretsWatchResponse>(MethodType.ServerStreaming, "dokime.client.v1.Secrets", "Watch",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsWatchRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsWatchResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsWatchResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncServerStreamingCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SecretsDeleteResponse> Delete(global::Dokime.Protocol.Client.V1.SecretsDeleteRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsDeleteRequest, global::Dokime.Protocol.Client.V1.SecretsDeleteResponse>(MethodType.Unary, "dokime.client.v1.Secrets", "Delete",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsDeleteRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsDeleteResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsDeleteResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SecretsExportResponse> Export(global::Dokime.Protocol.Client.V1.SecretsExportRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsExportRequest, global::Dokime.Protocol.Client.V1.SecretsExportResponse>(MethodType.Unary, "dokime.client.v1.Secrets", "Export",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsExportRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsExportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsExportResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsExportResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SecretsImportResponse> Import(global::Dokime.Protocol.Client.V1.SecretsImportRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SecretsImportRequest, global::Dokime.Protocol.Client.V1.SecretsImportResponse>(MethodType.Unary, "dokime.client.v1.Secrets", "Import",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsImportRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsImportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SecretsImportResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SecretsImportResponse.Parser.ParseFrom(bytes)));
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
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SettingsGetResponse> Get(global::Dokime.Protocol.Client.V1.SettingsGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SettingsGetRequest, global::Dokime.Protocol.Client.V1.SettingsGetResponse>(MethodType.Unary, "dokime.client.v1.Settings", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SettingsGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SettingsGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SettingsGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SettingsSetResponse> Set(global::Dokime.Protocol.Client.V1.SettingsSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SettingsSetRequest, global::Dokime.Protocol.Client.V1.SettingsSetResponse>(MethodType.Unary, "dokime.client.v1.Settings", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SettingsSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SettingsSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SettingsSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SettingsSchemaResponse> Schema(global::Dokime.Protocol.Client.V1.SettingsSchemaRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SettingsSchemaRequest, global::Dokime.Protocol.Client.V1.SettingsSchemaResponse>(MethodType.Unary, "dokime.client.v1.Settings", "Schema",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SettingsSchemaRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SettingsSchemaRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SettingsSchemaResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SettingsSchemaResponse.Parser.ParseFrom(bytes)));
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
  public StorageSourcesResource Sources { get; }
  public StorageResource(CallInvoker invoker)
  {
    Sources = new StorageSourcesResource(invoker);
  }
}
public sealed class StorageSourcesResource
{
  private readonly CallInvoker invoker;
  public StorageSourcesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetResponse> Get(global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetRequest, global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetResponse>(MethodType.Unary, "dokime.client.v1.storage.Sources", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StorageResources.SourcesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetResponse> Set(global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetRequest, global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetResponse>(MethodType.Unary, "dokime.client.v1.storage.Sources", "Set",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StorageResources.SourcesSetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class StreamSchemasResource
{
  private readonly CallInvoker invoker;
  public StreamSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.StreamSchemasGetResponse> Get(global::Dokime.Protocol.Client.V1.StreamSchemasGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.StreamSchemasGetRequest, global::Dokime.Protocol.Client.V1.StreamSchemasGetResponse>(MethodType.Unary, "dokime.client.v1.StreamSchemas", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StreamSchemasGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StreamSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StreamSchemasGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StreamSchemasGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.StreamSchemasListResponse> List(global::Dokime.Protocol.Client.V1.StreamSchemasListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.StreamSchemasListRequest, global::Dokime.Protocol.Client.V1.StreamSchemasListResponse>(MethodType.Unary, "dokime.client.v1.StreamSchemas", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StreamSchemasListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StreamSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.StreamSchemasListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.StreamSchemasListResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
}
public sealed class SuitesResource
{
  private readonly CallInvoker invoker;
  public SuitesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SuitesGetResponse> Get(global::Dokime.Protocol.Client.V1.SuitesGetRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SuitesGetRequest, global::Dokime.Protocol.Client.V1.SuitesGetResponse>(MethodType.Unary, "dokime.client.v1.Suites", "Get",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SuitesGetRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SuitesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SuitesGetResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SuitesGetResponse.Parser.ParseFrom(bytes)));
    return invoker.AsyncUnaryCall(method, null, options, request);
  }
  public AsyncUnaryCall<global::Dokime.Protocol.Client.V1.SuitesListResponse> List(global::Dokime.Protocol.Client.V1.SuitesListRequest request, CallOptions options = default)
  {
    var method = new Method<global::Dokime.Protocol.Client.V1.SuitesListRequest, global::Dokime.Protocol.Client.V1.SuitesListResponse>(MethodType.Unary, "dokime.client.v1.Suites", "List",
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SuitesListRequest>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SuitesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<global::Dokime.Protocol.Client.V1.SuitesListResponse>(value => value.ToByteArray(), bytes => global::Dokime.Protocol.Client.V1.SuitesListResponse.Parser.ParseFrom(bytes)));
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
