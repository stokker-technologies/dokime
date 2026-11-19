/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Net.Security;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using System.Text.Json;
using System.Text.RegularExpressions;
using Dokime.Clients;
using Dokime.Clients.EvidenceResources;
using Google.Protobuf;
using Grpc.Core;
using Grpc.Net.Client;
using Harmos;

namespace Dokime;

public sealed class Client : IAsyncDisposable
{
  private readonly GrpcChannel? channel;
  private readonly Grpc.Core.Metadata? headers;
  private Task? closing;
  private readonly CancellationTokenSource promptCancellation = new();
  private Func<PromptQuestion, Task<PromptAnswer>>? promptHandler;
  private Task? promptTask;
  internal Transport.Client Transport
  {
    get;
  }
  internal Transport.FrozenSchemaCache Schemas { get; } = new(new PageRequest());
  public PluginCollection Plugins
  {
    get;
  }
  public Cases Cases
  {
    get;
  }
  public Cycles Cycles
  {
    get;
  }
  public Executions Executions
  {
    get;
  }

  internal Client(CallInvoker invoker, GrpcChannel? channel = null, Grpc.Core.Metadata? headers = null)
  {
    Transport = new Transport.Client(new FailureInvoker(invoker));
    this.channel = channel;
    this.headers = headers;
    Plugins = new PluginCollection(this);
    Cases = new Cases(this);
    Cycles = new Cycles(this);
    Executions = new Executions(this);
  }

  public static async Task<Client> ConnectAsync(ConnectionSettings? settings = null, CancellationToken cancellationToken = default)
  {
    settings ??= new ConnectionSettings();
    if (settings.ConnectTimeout <= TimeSpan.Zero)
      throw new ArgumentException("Connect: Supply a Positive ConnectTimeout");
    var selected = Configuration.Resolve("DOKIME_REMOTE", settings.Remote, Configuration.Load());
    string[] endpoints = [selected];
    string? certificate = null;
    string? token = null;
    string? expectedHost = null;
    if (selected.EndsWith(".pair", StringComparison.Ordinal))
    {
      using var document = JsonDocument.Parse(await System.IO.File.ReadAllBytesAsync(selected, cancellationToken));
      var pairing = document.RootElement;
      if (pairing.GetProperty("version").GetInt32() != 1)
        throw new ArgumentException("Connect: Export a Version 1 Pairing from the Intended Host");
      endpoints = pairing.GetProperty("endpoints").EnumerateArray().Select(value => value.GetString()!).ToArray();
      certificate = pairing.TryGetProperty("certificate_pem", out var pem) ? pem.GetString() : null;
      token = pairing.TryGetProperty("access_key", out var key) ? key.GetString() : null;
      expectedHost = pairing.GetProperty("backend_identifier").GetString();
      if (endpoints.Length == 0 || string.IsNullOrWhiteSpace(expectedHost))
        throw new ArgumentException("Connect: Export a Pairing with Endpoints and a Host Identity");
    }
    List<Exception> failures = [];
    foreach (var endpoint in endpoints)
    {
      cancellationToken.ThrowIfCancellationRequested();
      GrpcChannel? channel = null;
      try
      {
        var uri = Endpoint(endpoint);
        var handler = new HttpClientHandler();
        if (uri.Scheme == "https")
        {
          if (certificate is null)
            throw new ArgumentException("Connect: Select a Pairing with a Pinned HTTPS Certificate");
          using var pinned = X509Certificate2.CreateFromPem(certificate);
          var certificateBytes = pinned.RawData;
          handler.ServerCertificateCustomValidationCallback = (_, observed, _, errors) =>
            observed is not null && CryptographicOperations.FixedTimeEquals(observed.RawData, certificateBytes)
            && (errors & (SslPolicyErrors.RemoteCertificateNameMismatch | SslPolicyErrors.RemoteCertificateNotAvailable)) == 0
            && observed.NotBefore.ToUniversalTime() <= DateTime.UtcNow && observed.NotAfter.ToUniversalTime() >= DateTime.UtcNow;
        }
        else if (certificate is not null)
          throw new ArgumentException("Connect: HTTP Cannot Carry a Certificate");
        channel = GrpcChannel.ForAddress(uri, new GrpcChannelOptions { HttpHandler = handler });
        var headers = token is null ? null : new Grpc.Core.Metadata { { "authorization", $"Bearer {token}" } };
        var client = new Client(channel.CreateCallInvoker(), channel, headers);
        var options = new CallOptions(headers, DateTime.UtcNow.Add(settings.ConnectTimeout), cancellationToken);
        var status = await client.Transport.Status.Get(options);
        if (expectedHost is not null && status.Status.Identity.HostId != expectedHost)
          throw new ArgumentException("Connect: Export a Pairing from the Intended Host; Its Identity Changed");
        return client;
      }
      catch (Exception failure) when (failure is SdkError or RpcException or ArgumentException or HttpRequestException)
      {
        channel?.Dispose();
        failures.Add(new InvalidOperationException($"Connect to {endpoint}: Check the Host Listener or Export a New Pairing", failure));
      }
      catch
      {
        channel?.Dispose();
        throw;
      }
    }
    throw new AggregateException("Connect: Check the Host Listener or Export a New Pairing", failures);
  }

  internal static Uri Endpoint(string value)
  {
    if (value.Any(character => char.IsWhiteSpace(character) || char.IsControl(character))
      || !Uri.TryCreate(value, UriKind.Absolute, out var uri) || uri.Scheme is not ("http" or "https")
      || uri.Host.Length == 0 || uri.Port == 0 || !Regex.IsMatch(value, @"^https?://(?:\[[^\]]+\]|[^/:?#@]+):[0-9]+$") || uri.AbsolutePath != "/" || value.EndsWith('/')
      || uri.UserInfo.Length != 0 || uri.Query.Length != 0 || uri.Fragment.Length != 0)
      throw new ArgumentException("Connect: Supply an HTTP(S) Origin with an Explicit Port or Select a .pair File");
    return uri;
  }

  internal CallOptions Options(CancellationToken cancellationToken = default)
  {
    if (closing is not null)
      throw new InvalidOperationException("Call Client: Connect a New Client After Close");
    return new CallOptions(headers: headers, cancellationToken: cancellationToken);
  }

  internal CallOptions CleanupOptions(CancellationToken cancellationToken = default) => new(headers: headers, cancellationToken: cancellationToken);
  internal static string Identity(string? requestId)
  {
    if (requestId == "")
      throw new ArgumentException("Mutate Resource: Supply a Nonempty Request Identity");
    return requestId ?? Guid.NewGuid().ToString();
  }

  public async Task<Cycle> CycleAsync(string label, string? requestId = null, CancellationToken cancellationToken = default)
  {
    var result = await Transport.Cycles.Create([], new Metadata { Label = label }, false, "", Identity(requestId), Options(cancellationToken));
    return new Cycle(this, result.Cycle);
  }

  public Operations.BoundOperation<TRequest, TResponse> Operation<TRequest, TResponse>(Operations.Operation<TRequest, TResponse> operation) => new(this, operation);

  public void OnPrompt(Func<PromptQuestion, Task<PromptAnswer>>? handler)
  {
    _ = Options();
    promptHandler = handler;
    if (handler is not null)
      promptTask ??= AnswerPromptsAsync(promptCancellation.Token);
  }

  private async Task AnswerPromptsAsync(CancellationToken cancellationToken)
  {
    HashSet<string> answered = [];
    await foreach (var frame in Transport.Prompts.Watch("", true, context: Options(cancellationToken), cancellationToken: cancellationToken))
    {
      if (frame.ResyncRequired is not null)
        throw new InvalidDataException("Observe Prompts: Reconnect After Resynchronization");
      foreach (var view in frame.Prompts)
      {
        var prompt = view.Prompt;
        var handler = promptHandler;
        if (handler is null || view.Response is not null || answered.Contains(prompt.PromptId))
          continue;
        var question = new PromptQuestion(prompt.PromptId, prompt.Definition.Message, prompt.Definition.Actions.Select(action => action.Label).ToArray(), prompt.Scope.ExecutionId);
        var answer = await handler(question);
        var actions = prompt.Definition.Actions.Where(action => action.Label == answer.Choice).ToArray();
        if (actions.Length != 1)
          throw new ArgumentException("Answer Prompt: Choose One Supplied Choice Label");
        await Transport.Prompts.Answers.Create(prompt.PromptId, actions[0].Identifier, new MapValue(), true, Identity(null), Options(cancellationToken));
        answered.Add(prompt.PromptId);
      }
    }
  }

  public Task CloseAsync(CancellationToken cancellationToken = default)
  {
    closing ??= CloseCoreAsync();
    return closing.WaitAsync(cancellationToken);
  }

  private async Task CloseCoreAsync()
  {
    List<Exception> failures = [];
    promptHandler = null;
    await promptCancellation.CancelAsync();
    if (promptTask is not null)
    {
      try
      {
        await promptTask;
      }
      catch (OperationCanceledException) when (promptCancellation.IsCancellationRequested)
      {
      }
      catch (RpcException failure) when (promptCancellation.IsCancellationRequested && failure.StatusCode == StatusCode.Cancelled)
      {
      }
      catch (Exception failure)
      {
        failures.Add(failure);
      }
    }
    try
    {
      await Plugins.StopRegistrationsAsync();
    }
    catch (Exception failure)
    {
      failures.Add(failure);
    }
    foreach (var plugin in Plugins.Owned.ToArray())
    {
      try
      {
        await plugin.UnregisterAsync();
      }
      catch (Exception failure)
      {
        failures.Add(failure);
      }
    }
    channel?.Dispose();
    if (failures.Count > 0)
      throw new AggregateException("Close Client: Resolve Each Plugin Cleanup Failure", failures);
  }

  public async ValueTask DisposeAsync() => await CloseAsync();

  internal async Task<byte[]> ArchiveAsync(EvidenceSubject subject, CancellationToken cancellationToken)
  {
    var reply = await Transport.Evidence.Archive(subject, context: Options(cancellationToken));
    return await DownloadAsync(reply.File ?? throw new InvalidDataException("Read Evidence: Request a Retained Archive"), cancellationToken);
  }

  internal async Task<byte[]> DownloadAsync(FileReference file, CancellationToken cancellationToken)
  {
    var selected = await Transport.Files.Get(file, context: Options(cancellationToken));
    if (selected.File != file || selected.ChunkBytes == 0 || selected.ChunkBytes >= selected.MessageBytes)
      throw new InvalidDataException("Read File: Request Valid Transfer Limits and an Unchanged File");
    using var data = new MemoryStream();
    while ((ulong)data.Length < file.Length)
    {
      var reply = await Transport.Files.Read(file, (ulong)data.Length, selected.ChunkBytes, selected.ResolvedAt, Options(cancellationToken));
      if (reply.TotalLength != file.Length || reply.Sha256 != file.Sha256 || reply.Data.Length == 0
        || reply.ResolvedAt != selected.ResolvedAt || (ulong)reply.Data.Length > file.Length - (ulong)data.Length)
        throw new InvalidDataException("Read File: Request an Unchanged File at Its Retained Cut");
      await data.WriteAsync(reply.Data.Memory, cancellationToken);
    }
    var bytes = data.ToArray();
    if (!CryptographicOperations.FixedTimeEquals(SHA256.HashData(bytes), file.Sha256.Span))
      throw new InvalidDataException("Read File: Export the Archive Again; SHA-256 Verification Failed");
    return bytes;
  }

  internal async Task<Note> NoteAsync(string cycle, string? execution, string text, string? requestId, CancellationToken cancellationToken)
  {
    var owner = execution is null ? new AnnotationOwner { Owner = new AnnotationOwnerOwner { CycleId = cycle } }
      : new AnnotationOwner { Owner = new AnnotationOwnerOwner { Execution = new ExecutionNoteOwner { CycleId = cycle, ExecutionId = execution } } };
    var reply = await Transport.Evidence.Notes.Create(owner, new NoteWritable { Message = text }, Identity(requestId), Options(cancellationToken));
    var note = reply.Note;
    return new Note(note.NoteId, note.Message, note.HasAuthor ? note.Author : null, note.CreatedAt.ToDateTimeOffset(), note.Scope.CycleId, string.IsNullOrEmpty(note.Scope.ExecutionId) ? null : note.Scope.ExecutionId);
  }

  internal async Task<Artifact> ArtifactAsync(ArtifactScope scope, string name, byte[] data, string mime, string? requestId, CancellationToken cancellationToken)
  {
    var identity = Identity(requestId);
    var options = Options(cancellationToken);
    var reply = await Transport.Files.Uploads.Create(new FileMetadata { ExpectedLength = (ulong)data.Length }, identity + "/create", options);
    var upload = reply.Upload;
    try
    {
      if (reply.ChunkBytes == 0 || reply.ChunkBytes >= reply.MessageBytes || reply.ChunkBytes > int.MaxValue)
        throw new InvalidDataException("Upload Artifact: Request Valid Transfer Limits");
      for (var offset = 0; offset < data.Length;)
      {
        var size = Math.Min((int)reply.ChunkBytes, data.Length - offset);
        var appended = await Transport.Files.Uploads.Append(upload, (ulong)offset, ByteString.CopyFrom(data, offset, size), identity + $"/append/{offset}", options);
        offset += size;
        upload = appended.Upload;
        if (upload.AcceptedLength != (ulong)offset)
          throw new InvalidDataException("Upload Artifact: Resume from the Host-Acknowledged Offset");
      }
      var digest = ByteString.CopyFrom(SHA256.HashData(data));
      var published = await Transport.Files.Uploads.Publish(upload, (ulong)data.Length, digest, identity + "/publish", options);
      if (published.Upload.Length != (ulong)data.Length || published.Upload.Sha256 != digest)
        throw new InvalidDataException("Upload Artifact: Publish the Unchanged Length and SHA-256");
      var attached = await Transport.Content.Artifacts.Create(published.Upload, name, mime, scope, identity + "/artifact", context: options);
      return Artifact.FromWire(attached.Artifact);
    }
    catch
    {
      await Transport.Files.Uploads.Delete(upload, identity + "/abandon", CleanupOptions());
      throw;
    }
  }
}

public sealed record Progress(string Phase, ulong Completed, ulong Total, string Label, bool Sealed);
public sealed record Outcome(ResultStatus Status, string Attribution, string Reason, Termination? Termination = null, string? Importer = null)
{
  internal static Outcome FromWire(ExecutionOutcome value) => value.AttributionCase switch
  {
    ExecutionOutcome.AttributionOneofCase.Evaluated => new(value.Evaluated.Status, "evaluated", value.Evaluated.Reason),
    ExecutionOutcome.AttributionOneofCase.Declared => new(value.Declared.Status, "declared", value.Declared.Reason, Importer: value.Declared.Importer),
    ExecutionOutcome.AttributionOneofCase.NotEvaluated => new(ResultStatus.Pending, "not_evaluated", value.NotEvaluated.Reason, value.NotEvaluated.Termination),
    _ => throw new InvalidDataException("Seal Execution: Request a Stable Outcome from the Host")
  };
}
public sealed record Note(string Identifier, string Text, string? Author, DateTimeOffset CreatedAt, string Cycle, string? Execution);
public sealed record PromptQuestion(string Identifier, string Text, IReadOnlyList<string> Choices, string Execution);
public sealed record PromptAnswer(string Choice);
public sealed record Artifact(string Identifier, string Name, string Mime, ulong SizeBytes, byte[] Sha256, string? Cycle, string? Execution)
{
  internal static Artifact FromWire(ArtifactMetadata value) => new(value.ArtifactId, value.Filename, value.ContentType, value.SizeBytes,
    Convert.FromHexString(value.Sha256), string.IsNullOrEmpty(value.Scope?.CycleId) ? null : value.Scope.CycleId, string.IsNullOrEmpty(value.Scope?.ExecutionId) ? null : value.Scope.ExecutionId);
}
