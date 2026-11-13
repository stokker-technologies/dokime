// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.Sdk;
using Harmos.V1;
using Dokime.V1;

namespace Dokime;

/// <summary>Typed public host operations. Scope is inherited from the parent invocation.</summary>
public static class Host
{
  public static Operation<global::Dokime.Protocol.Context.V1.PromptsCreateRequest, global::Dokime.Protocol.Context.V1.PromptsCreateResponse> Prompt { get; } = new("dokime.context.v1.Prompts/Create", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.Evidence.RecordsAppendRequest, global::Dokime.Protocol.Context.V1.Evidence.RecordsAppendResponse> Record { get; } = new("dokime.context.v1.evidence.Records/Append", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.Evidence.RecordsListRequest, global::Dokime.Protocol.Context.V1.Evidence.RecordsListResponse> Records { get; } = new("dokime.context.v1.evidence.Records/List", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.EvidenceStatisticsRequest, global::Dokime.Protocol.Context.V1.EvidenceStatisticsResponse> Statistics { get; } = new("dokime.context.v1.Evidence/Statistics", Visibility.Public);
  public static Operation<ReadSettings, SavedSettings> Settings { get; } = new("dokime.settings.read", Visibility.Public);
  public static Operation<ReadSecret, Secret> Secret { get; } = new("dokime.secrets.read", Visibility.Public);
  public static Operation<ReportProgress, ReportProgressResponse> Progress { get; } = new("dokime.execution.progress", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.Evidence.LogsAppendRequest, global::Dokime.Protocol.Context.V1.Evidence.LogsAppendResponse> Log { get; } = new("dokime.context.v1.evidence.Logs/Append", Visibility.Public);
  public static Operation<SetTraceability, SetTraceabilityResponse> Traceability { get; } = new("dokime.execution.traceability", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.DocumentsAppendPartsRequest, global::Dokime.Protocol.Context.V1.DocumentsAppendPartsResponse> Document { get; } = new("dokime.context.v1.Documents/AppendParts", Visibility.Public);
  public static Operation<PublishArtifact, ArtifactReceipt> Artifact { get; } = new("dokime.execution.artifact", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.FilesCaptureRequest, global::Dokime.Protocol.Context.V1.FilesCaptureResponse> CopyFiles { get; } = new("dokime.context.v1.Files/Capture", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.Files.CapturesStartRequest, global::Dokime.Protocol.Context.V1.Files.CapturesStartResponse> StartCapture { get; } = new("dokime.context.v1.files.Captures/Start", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.Files.CapturesStopRequest, global::Dokime.Protocol.Context.V1.Files.CapturesStopResponse> StopCapture { get; } = new("dokime.context.v1.files.Captures/Stop", Visibility.Public);
  public static Operation<global::Dokime.Protocol.Context.V1.BackendEnvironmentRequest, global::Dokime.Protocol.Context.V1.BackendEnvironmentResponse> Environment { get; } = new("dokime.context.v1.Backend/Environment", Visibility.Public);
  public static Operation<PrepareEffect, Ticket> PrepareEffect { get; } = new("dokime.execution.effect.prepare", Visibility.Public);
  public static Operation<ConfirmEffect, ConfirmEffectResponse> ConfirmEffect { get; } = new("dokime.execution.effect.applied", Visibility.Public);
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
  public IReadOnlyDictionary<string, V1.Value> Parameters => TestCase.Parameters;
  public CancellationToken CancellationToken => Transport.CancellationToken;
  public Timestamp? At => Transport.At?.Clone();
  public ulong Sequence => Transport.Sequence;
  public T Resource<T>() where T : class => resource as T ?? throw new InvalidOperationException("No resource of the requested type is active");

  public Task<TResponse> OperationAsync<TRequest, TResponse>(Operation<TRequest, TResponse> operation, TRequest request,
      OperationPolicy? policy = null) where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => Transport.CallAsync(operation, request, policy);
  public Task<Harmos.Context.V1.TimeSleepResponse> SleepAsync(Duration duration) => Transport.SleepAsync(duration);
  public Task<Timestamp> NowAsync() => Transport.NowAsync();
  public Task<global::Dokime.Protocol.Context.V1.PromptsCreateResponse> PromptAsync(V1.Prompt prompt) => OperationAsync(Host.Prompt,
      new global::Dokime.Protocol.Context.V1.PromptsCreateRequest { Prompt = prompt.Metadata.Identifier, PromptType = prompt.Metadata.Identifier, Definition = prompt.Clone() });
  public Task<global::Dokime.Protocol.Context.V1.Evidence.RecordsAppendResponse> RecordAsync<T>(Record<T> record, T value, Timestamp? at = null) where T : IMessage<T>, new()
      => OperationAsync(Host.Record, new global::Dokime.Protocol.Context.V1.Evidence.RecordsAppendRequest { Execution = Execution, Kind = record.Identifier, Payload = value.ToByteString(), At = at?.Clone() });
  public Task<global::Dokime.Protocol.Context.V1.Evidence.RecordsListResponse> RecordsAsync(string kind, string step = "", uint limit = 0)
      => OperationAsync(Host.Records, new global::Dokime.Protocol.Context.V1.Evidence.RecordsListRequest { Execution = Execution, Kind = kind, Step = step, Limit = limit });
  public Task<global::Dokime.Protocol.Context.V1.EvidenceStatisticsResponse> StatisticsAsync(string stream, string field = "", string step = "")
      => OperationAsync(Host.Statistics, new global::Dokime.Protocol.Context.V1.EvidenceStatisticsRequest { Execution = Execution, Stream = stream, Field = field, Step = step });
  public Task<SavedSettings> SettingsAsync() => OperationAsync(Host.Settings, new ReadSettings());
  public Task<Secret> SecretAsync(string identifier) => OperationAsync(Host.Secret, new ReadSecret { Identifier = identifier });
  public Task<ReportProgressResponse> ProgressAsync(ulong completed, ulong total, string label = "")
      => OperationAsync(Host.Progress, new ReportProgress { Execution = Execution, Completed = completed, Total = total, Label = label });
  public Task<global::Dokime.Protocol.Context.V1.Evidence.LogsAppendResponse> LogAsync(Severity severity, string message)
      => OperationAsync(Host.Log, new global::Dokime.Protocol.Context.V1.Evidence.LogsAppendRequest { Execution = Execution, Severity = severity, Message = message });
}
