// SPDX-License-Identifier: Apache-2.0

using Dokime.Clients.EvidenceResources;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using Harmos.Plugins;

namespace Dokime.Plugins;

/// <summary>One scoped plugin invocation or observer sample, with execution-owned resources.</summary>
public sealed class Context
{
  private readonly object? resource;
  private readonly Template? template;
  internal object? Instance
  {
    get;
  }

  internal Context(Harmos.Plugins.Context transport, TestCase testCase, object? resource = null, Template? template = null, object? instance = null)
  {
    Transport = transport;
    TestCase = testCase.Clone();
    this.resource = resource;
    this.template = template;
    Instance = instance;
  }

  public Harmos.Plugins.Context Transport
  {
    get;
  }
  public HostRoot Host => new(this);
  public string Execution => Transport.Scope();
  public TestCase TestCase
  {
    get;
  }
  public IReadOnlyDictionary<string, Value> Parameters => TestCase.Parameters;
  public CancellationToken CancellationToken => Transport.CancellationToken;
  public Timestamp? At => Transport.At?.Clone();
  public ulong Sequence => Transport.Sequence;

  public T Resource<T>() where T : class => resource as T
      ?? throw new InvalidOperationException("Read Execution Resource: No Resource of the Requested Type Is Active");

  public Task<TResponse> CallAsync<TRequest, TResponse>(Operation<TRequest, TResponse> operation, TRequest request,
      OperationPolicy? policy = null, CancellationToken cancellationToken = default)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => Transport.CallAsync(operation, request, policy, cancellationToken);

  public IAsyncEnumerable<TResponse> WatchAsync<TRequest, TResponse>(Operation<TRequest, TResponse> operation,
      TRequest request, ulong maxItemBytes, CancellationToken cancellationToken = default)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => Transport.WatchAsync(operation, request, maxItemBytes, cancellationToken);

  public async Task<RecordsAppendResponse> RecordAsync<T>(Record<T> record, T value, Timestamp? sourceAt = null,
      string? requestId = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var identity = Resolve(record);
    var payload = Payload<T>.Pack(value).ToByteString();
    var at = sourceAt ?? await Host.Time.Get(policy, cancellationToken);
    return await Host.Evidence.Records.Append(Execution, identity, payload, at, requestId, policy, cancellationToken);
  }

  public RecordsQuery<T> Records<T>(Record<T> record, Func<Context, Task>? step = null) =>
      new(this, Resolve(record), step is null ? "" : Resolve(step));

  public RecordsQuery<T> Records<T>(Record<T> record, Step step) => new(this, Resolve(record), Resolve(step));

  public Task<Dokime.Clients.EvidenceStatisticsResponse> StatisticsAsync(IStream stream, string field = "", Func<Context, Task>? step = null) =>
      Host.Evidence.Statistics(Resolve(stream), [], new Dokime.Window(), [], field, step is null ? "" : Resolve(step));

  public Task<Dokime.Clients.EvidenceStatisticsResponse> StatisticsAsync(IStream stream, string field, Step step) =>
      Host.Evidence.Statistics(Resolve(stream), [], new Dokime.Window(), [], field, Resolve(step));

  internal string Resolve(object declaration) => template?.Resolve(declaration, Instance)
      ?? throw new ArgumentException("Resolve Declaration: Call From a Template Operation");

  public Task<string> PromptAsync(string text, string[]? choices = null) => AskAsync(new PromptDeclaration(text, choices), false);

  public Task<string> PromptAsync(PromptDeclaration declaration) => AskAsync(declaration, true);

  private async Task<string> AskAsync(PromptDeclaration declaration, bool declared)
  {
    var identity = declared ? Resolve(declaration) : "question";
    var definition = declaration.Definition.Clone();
    definition.Metadata ??= new Metadata();
    definition.Metadata.Identifier = identity;
    if (declared && template is not null)
    {
      definition.OwnerSuite = template.OwnerSuite;
      definition.Template = template.Definition.Metadata.Identifier;
    }
    var response = await Host.Prompts.Create(identity, declared ? identity : "", new Dictionary<string, Value>(), definition);
    if (!response.Accepted)
      throw new ArgumentException("Create Prompt: Choose an Accepted Operator Answer");
    return definition.Actions.FirstOrDefault(action => action.Identifier == response.Action)?.Label
        ?? throw new ArgumentException("Read Prompt Answer: Choose One of the Declared Actions");
  }

  public Task<TResponse> CallAsync<TRequest, TResponse>(Dokime.Authoring.Operation<TRequest, TResponse> operation, TRequest request) =>
      operation.CallAsync(this, request);

  public async Task<Receipt> LogAsync(Severity severity, string message, string? requestId = null,
      OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var at = At ?? await Host.Time.Get(policy, cancellationToken);
    return await Host.Evidence.Logs.Append(Execution, severity, message, at, requestId, policy, cancellationToken);
  }
}


public sealed class RecordsQuery<T>
{
  private readonly Context context;
  private readonly string identifier;
  private readonly string step;

  internal RecordsQuery(Context context, string identifier, string step)
  {
    this.context = context;
    this.identifier = identifier;
    this.step = step;
  }

  public async Task<T> LastAsync()
  {
    var response = await context.Host.Evidence.Records.List(identifier, [], new Dokime.Window(), step, new CapturedRecordSelection());
    if (response.Last is null)
      throw new InvalidOperationException("Read Last Record: Capture a Record Before Reading It");
    return Payload<T>.Unpack(response.Last.Payload);
  }
}
