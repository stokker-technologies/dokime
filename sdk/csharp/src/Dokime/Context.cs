// SPDX-License-Identifier: Apache-2.0

using Dokime.Protocol.Plugin.Host.V1;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.Sdk;
using Harmos.V1;
using Dokime.Protocol.V1;

namespace Dokime;

/// <summary>Typed public host operations. Scope is inherited from the parent invocation.</summary>
public static class Host
{
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.PromptsCreateRequest, global::Dokime.Protocol.Plugin.Host.V1.PromptsCreateResponse> Prompt { get; } = new("dokime.plugin.host.v1.Prompts/Create", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendRequest, global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendResponse> Record { get; } = new("dokime.plugin.host.v1.evidence.Records/Append", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsListRequest, global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsListResponse> Records { get; } = new("dokime.plugin.host.v1.evidence.Records/List", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsRequest, global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse> Statistics { get; } = new("dokime.plugin.host.v1.Evidence/Statistics", Visibility.Public);
  public static Operation<ReadSettings, SavedSettings> Settings { get; } = new("dokime.settings.read", Visibility.Public);
  public static Operation<ReadSecret, Secret> Secret { get; } = new("dokime.secrets.read", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressGetRequest, global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressGetResponse> ProgressGet { get; } = new("dokime.plugin.host.v1.executions.Progress/Get", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressSetRequest, global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressSetResponse> ProgressSet { get; } = new("dokime.plugin.host.v1.executions.Progress/Set", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.LogsAppendRequest, global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.LogsAppendResponse> Log { get; } = new("dokime.plugin.host.v1.evidence.Logs/Append", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.TraceabilityGetRequest, global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.TraceabilityGetResponse> TraceabilityGet { get; } = new("dokime.plugin.host.v1.evidence.Traceability/Get", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.TraceabilitySetRequest, global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.TraceabilitySetResponse> TraceabilitySet { get; } = new("dokime.plugin.host.v1.evidence.Traceability/Set", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.DocumentsAppendPartsRequest, global::Dokime.Protocol.Plugin.Host.V1.DocumentsAppendPartsResponse> Document { get; } = new("dokime.plugin.host.v1.Documents/AppendParts", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.ContentResources.ArtifactsCreateRequest, global::Dokime.Protocol.Plugin.Host.V1.ContentResources.ArtifactsCreateResponse> Artifact { get; } = new("dokime.plugin.host.v1.content.Artifacts/Create", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.FilesCaptureRequest, global::Dokime.Protocol.Plugin.Host.V1.FilesCaptureResponse> CopyFiles { get; } = new("dokime.plugin.host.v1.Files/Capture", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.FilesResources.CapturesStartRequest, global::Dokime.Protocol.Plugin.Host.V1.FilesResources.CapturesStartResponse> StartCapture { get; } = new("dokime.plugin.host.v1.files.Captures/Start", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.FilesResources.CapturesStopRequest, global::Dokime.Protocol.Plugin.Host.V1.FilesResources.CapturesStopResponse> StopCapture { get; } = new("dokime.plugin.host.v1.files.Captures/Stop", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.BackendEnvironmentRequest, global::Dokime.Protocol.Plugin.Host.V1.BackendEnvironmentResponse> Environment { get; } = new("dokime.plugin.host.v1.Backend/Environment", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EffectsCreateRequest, global::Dokime.Protocol.Plugin.Host.V1.EffectsCreateResponse> EffectsCreate { get; } = new("dokime.plugin.host.v1.Effects/Create", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EffectsGetRequest, global::Dokime.Protocol.Plugin.Host.V1.EffectsGetResponse> EffectsGet { get; } = new("dokime.plugin.host.v1.Effects/Get", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Plugin.Host.V1.EffectsSetRequest, global::Dokime.Protocol.Plugin.Host.V1.EffectsSetResponse> EffectsSet { get; } = new("dokime.plugin.host.v1.Effects/Set", Visibility.Public);
}

/// <summary>One unary plugin call or observer tick. Owned resources live until cleanup.</summary>
public sealed class Context
{
  private readonly object? resource;
  internal Context(Harmos.Sdk.Context transport, TestCase testCase, object? resource = null)
  {
    Transport = transport;
    TestCase = testCase.Clone();
    this.resource = resource;
  }

  public Harmos.Sdk.Context Transport { get; }
  public string Execution => Transport.Scope();
  public TestCase TestCase { get; }
  public IReadOnlyDictionary<string, Dokime.Protocol.V1.Value> Parameters => TestCase.Parameters;
  public CancellationToken CancellationToken => Transport.CancellationToken;
  public Timestamp? At => Transport.At?.Clone();
  public ulong Sequence => Transport.Sequence;
  public T Resource<T>() where T : class => resource as T ?? throw new InvalidOperationException("No resource of the requested type is active");

  public Task<TResponse> OperationAsync<TRequest, TResponse>(Operation<TRequest, TResponse> operation, TRequest request,
      OperationPolicy? policy = null) where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => Transport.CallAsync(operation, request, policy);
  public Task<Harmos.Context.V1.TimeSleepResponse> SleepAsync(Duration duration) => Transport.SleepAsync(duration);
  public Task<Timestamp> NowAsync() => Transport.NowAsync();
  public Task<global::Dokime.Protocol.Plugin.Host.V1.PromptsCreateResponse> PromptAsync(Dokime.Protocol.V1.Prompt prompt) => OperationAsync(Host.Prompt,
      new global::Dokime.Protocol.Plugin.Host.V1.PromptsCreateRequest { Prompt = prompt.Metadata.Identifier, PromptType = prompt.Metadata.Identifier, Definition = prompt.Clone() });
  public Task<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendResponse> RecordAsync<T>(Record<T> record, T value, Timestamp? sourceAt = null, string? requestId = null) where T : IMessage<T>, new()
      => OperationAsync(Host.Record, new global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendRequest { Execution = Execution, Kind = record.Identifier, Payload = value.ToByteString(), SourceAt = sourceAt?.Clone(), RequestId = requestId ?? Guid.NewGuid().ToString() });
  public Task<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsListResponse> RecordsAsync(string kind, string step = "", PageRequest? page = null, Position? at = null)
      => OperationAsync(Host.Records, new global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsListRequest { Execution = Execution, Kind = kind, Step = step, Page = page?.Clone(), At = at?.Clone() });
  public Task<global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse> StatisticsAsync(string stream, string field = "", string step = "")
      => OperationAsync(Host.Statistics, new global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsRequest { Execution = Execution, Stream = stream, Field = field, Step = step });
  public Task<SavedSettings> SettingsAsync() => OperationAsync(Host.Settings, new ReadSettings());
  public Task<Secret> SecretAsync(string identifier) => OperationAsync(Host.Secret, new ReadSecret { Identifier = identifier });
  public async Task<global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressSetResponse> ProgressAsync(ulong completed, ulong total, string label = "")
  {
    var current = await OperationAsync(Host.ProgressGet, new global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressGetRequest { Name = Execution });
    return await OperationAsync(Host.ProgressSet, new global::Dokime.Protocol.Plugin.Host.V1.ExecutionsResources.ProgressSetRequest
    {
      Name = Execution,
      Value = new ExecutionProgressValue { Completed = completed, Total = total, Label = label },
      ExpectedRevision = current.Revision,
      RequestId = Guid.NewGuid().ToString()
    });
  }
  public Task<global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.LogsAppendResponse> LogAsync(Severity severity, string message, string? requestId = null)
      => OperationAsync(Host.Log, new global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.LogsAppendRequest { Execution = Execution, Severity = severity, Message = message, RequestId = requestId ?? Guid.NewGuid().ToString() });
}
