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
  public static Operation<OperatorQuestion, OperatorAnswer> Prompt { get; } = new("dokime.execution.prompt", Visibility.Public);
  public static Operation<PublishRecord, RecordReceipt> Record { get; } = new("dokime.execution.record", Visibility.Public);
  public static Operation<RecordsRequest, Records> Records { get; } = new("dokime.execution.records", Visibility.Public);
  public static Operation<StatisticsRequest, Statistics> Statistics { get; } = new("dokime.execution.statistics", Visibility.Public);
  public static Operation<ReadSettings, SavedSettings> Settings { get; } = new("dokime.settings.read", Visibility.Public);
  public static Operation<ReadSecret, Secret> Secret { get; } = new("dokime.secrets.read", Visibility.Public);
  public static Operation<ReportProgress, ReportProgressResponse> Progress { get; } = new("dokime.execution.progress", Visibility.Public);
  public static Operation<WriteLog, WriteLogResponse> Log { get; } = new("dokime.execution.log", Visibility.Public);
  public static Operation<SetTraceability, SetTraceabilityResponse> Traceability { get; } = new("dokime.execution.traceability", Visibility.Public);
  public static Operation<AddDocumentPart, AddDocumentPartResponse> Document { get; } = new("dokime.execution.document", Visibility.Public);
  public static Operation<PublishArtifact, ArtifactReceipt> Artifact { get; } = new("dokime.execution.artifact", Visibility.Public);
  public static Operation<CopyFiles, CapturedFiles> CopyFiles { get; } = new("dokime.files.copy_files", Visibility.Public);
  public static Operation<StartCapture, CaptureStarted> StartCapture { get; } = new("dokime.files.start_capture", Visibility.Public);
  public static Operation<StopCapture, CapturedFiles> StopCapture { get; } = new("dokime.files.stop_capture", Visibility.Public);
  public static Operation<ReadEnvironment, V1.Environment> Environment { get; } = new("dokime.execution.environment", Visibility.Public);
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
  public Task<SleepResponse> SleepAsync(Duration duration) => Transport.SleepAsync(duration);
  public Task<Timestamp> NowAsync() => Transport.NowAsync();
  public Task<OperatorAnswer> PromptAsync(V1.Prompt prompt) => OperationAsync(Host.Prompt,
      new OperatorQuestion { Prompt = prompt.Metadata.Identifier, PromptType = prompt.Metadata.Identifier, Definition = prompt.Clone() });
  public Task<RecordReceipt> RecordAsync<T>(Record<T> record, T value, Timestamp? at = null) where T : IMessage<T>, new()
      => OperationAsync(Host.Record, new PublishRecord { Execution = Execution, Kind = record.Identifier, Payload = value.ToByteString(), At = at?.Clone() });
  public Task<Records> RecordsAsync(string kind, string step = "", uint limit = 0)
      => OperationAsync(Host.Records, new RecordsRequest { Execution = Execution, Kind = kind, Step = step, Limit = limit });
  public Task<Statistics> StatisticsAsync(string stream, string field = "", string step = "")
      => OperationAsync(Host.Statistics, new StatisticsRequest { Execution = Execution, Stream = stream, Field = field, Step = step });
  public Task<SavedSettings> SettingsAsync() => OperationAsync(Host.Settings, new ReadSettings());
  public Task<Secret> SecretAsync(string identifier) => OperationAsync(Host.Secret, new ReadSecret { Identifier = identifier });
  public Task<ReportProgressResponse> ProgressAsync(ulong completed, ulong total, string label = "")
      => OperationAsync(Host.Progress, new ReportProgress { Execution = Execution, Completed = completed, Total = total, Label = label });
  public Task<WriteLogResponse> LogAsync(Severity severity, string message)
      => OperationAsync(Host.Log, new WriteLog { Execution = Execution, Severity = severity, Message = message });
}
