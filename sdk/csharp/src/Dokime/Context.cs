// SPDX-License-Identifier: Apache-2.0

using Dokime.Clients.EvidenceResources;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Dokime.Plugins;

namespace Dokime.Templates;

/// <summary>One scoped plugin invocation or observer sample, with execution-owned resources.</summary>
public sealed partial class Context : Dokime.Operations.Context
{
  private readonly Template? template;
  internal object? Instance
  {
    get;
  }

  internal Context(Dokime.Plugin plugin, Harmos.Plugins.Context transport, TestCase testCase, Template? template = null, object? instance = null, Termination? termination = null) : base(plugin, transport)
  {
    TestCase = testCase.Clone();
    this.template = template;
    Instance = instance;
    Termination = termination;
  }

  public string Execution => Transport.Scope();
  public Termination? Termination
  {
    get;
  }
  internal TestCase TestCase
  {
    get;
  }
  public IReadOnlyDictionary<string, object?> Parameters => TestCase.Parameters.ToDictionary(item => item.Key, item => Scalars.Decode(item.Value));
  internal Timestamp? At => Transport.At?.Clone();
  public ulong Sequence => Transport.Sequence;

  public async Task<ulong> RecordAsync<T>(Record<T> record, T value, DateTimeOffset? sourceAt = null,
      string? requestId = null, Dokime.Operations.OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var identity = Resolve(record);
    var payload = Payload<T>.Pack(value).ToByteString();
    var at = sourceAt is null ? await Host.Time.Get(policy?.ToWire(), cancellationToken) : Timestamp.FromDateTimeOffset(sourceAt.Value);
    var reply = await Host.Evidence.Records.Append(Execution, identity, payload, at, Client.Identity(requestId), policy?.ToWire(), cancellationToken);
    return reply.Ordinal;
  }

  public RecordsQuery<T> Records<T>(Record<T> record, Func<Context, Task>? step = null) =>
      new(this, Resolve(record), step is null ? "" : Resolve(step));

  internal RecordsQuery<T> Records<T>(Record<T> record, Step step) => new(this, Resolve(record), Resolve(step));

  public async Task<Statistics> StatisticsAsync(IStream stream, string field = "", Func<Context, Task>? step = null, CancellationToken cancellationToken = default) =>
      Statistics.FromWire((await Host.Evidence.Statistics(Resolve(stream), [], new Dokime.Window(), [], field, step is null ? "" : Resolve(step), cancellationToken: cancellationToken)).Statistics);

  internal async Task<Statistics> StatisticsAsync(IStream stream, string field, Step step) =>
      Statistics.FromWire((await Host.Evidence.Statistics(Resolve(stream), [], new Dokime.Window(), [], field, Resolve(step))).Statistics);

  internal string Resolve(object declaration) => template?.Resolve(declaration, Instance)
      ?? throw new ArgumentException("Resolve Declaration: Call From a Template Operation");

  public Task<string> PromptAsync(string text, string[]? choices = null, CancellationToken cancellationToken = default) => AskAsync(new PromptDeclaration(text, choices), false, cancellationToken);

  public Task<string> PromptAsync(PromptDeclaration declaration, CancellationToken cancellationToken = default) => AskAsync(declaration, true, cancellationToken);
  public Task<string> PromptAsync(Prompt definition, CancellationToken cancellationToken = default) => AskAsync(new PromptDeclaration(definition), false, cancellationToken);

  private async Task<string> AskAsync(PromptDeclaration declaration, bool declared, CancellationToken cancellationToken)
  {
    var identity = declared ? Resolve(declaration) : Identifiers.Build("question", prefix: template?.Definition.Metadata.Identifier
        ?? throw new ArgumentException("Create Prompt: Call From a Template Operation"));
    var definition = declaration.Definition.Clone();
    definition.Metadata ??= new Metadata();
    definition.Metadata.Identifier = identity;
    if (declared && template is not null)
    {
      definition.OwnerSuite = template.OwnerSuite;
      definition.Template = template.Definition.Metadata.Identifier;
    }
    var response = await Host.Prompts.Create(identity, declared ? identity : "", new Dictionary<string, Value>(), definition, cancellationToken: cancellationToken);
    if (!response.Accepted)
      throw new ArgumentException("Create Prompt: Choose an Accepted Operator Answer");
    return definition.Actions.FirstOrDefault(action => action.Identifier == response.Action)?.Label
        ?? throw new ArgumentException("Read Prompt Answer: Choose One of the Declared Actions");
  }

  public async Task LogAsync(string message, Severity severity = Severity.Info, string? requestId = null,
      Dokime.Operations.OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var at = At ?? await Host.Time.Get(policy?.ToWire(), cancellationToken);
    await Host.Evidence.Logs.Append(Execution, severity, message, at, Client.Identity(requestId), policy?.ToWire(), cancellationToken);
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

  public async Task<T> LastAsync(CancellationToken cancellationToken = default)
  {
    var response = await context.Host.Evidence.Records.List(identifier, [], new Dokime.Window(), step, new CapturedRecordSelection(), cancellationToken: cancellationToken);
    if (response.Last is null)
      throw new InvalidOperationException("Read Last Record: Capture a Record Before Reading It");
    return Payload<T>.Unpack(response.Last.Payload);
  }
}
