/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Reflection;
using System.Runtime.CompilerServices;
using Dokime.Clients;
using Dokime.Clients.EvidenceResources;
using Dokime.Plugins;
using Dokime.Templates;
using Google.Protobuf;
using Harmos;

namespace Dokime;

public sealed class Cases
{
  private readonly Client client;
  internal Cases(Client client) => this.client = client;
  public async Task<IReadOnlyList<Case>> MaterializeAsync<T>(IReadOnlyDictionary<string, IReadOnlyList<object?>>? parameters = null,
    IReadOnlyDictionary<string, string?>? files = null, string? plugin = null, string? suite = null,
    string? requestId = null, CancellationToken cancellationToken = default) where T : TestTemplate, new()
  {
    var matches = client.Plugins.Bindings.Where(binding => binding.Type == typeof(T)
      && (plugin is null || binding.Plugin == plugin) && (suite is null || binding.Suite == suite)).ToArray();
    if (matches.Length != 1)
      throw new ArgumentException("Materialize Cases: Register One Unambiguous Plugin and Suite for This Template");
    var binding = matches[0];
    var selection = new RegisteredSelection { SuiteId = binding.Suite, TemplateId = binding.Template };
    foreach (var (key, values) in parameters ?? new Dictionary<string, IReadOnlyList<object?>>())
    {
      var value = new SelectionValues();
      value.Values.Add(values.Select(Scalars.Encode));
      selection.Parameters.Add(new RegisteredSelectionParametersEntry { Key = key, Value = value });
    }
    foreach (var (key, value) in files ?? new Dictionary<string, string?>())
      selection.Files.Add(new RegisteredSelectionFilesEntry
      {
        Key = key,
        Value = new SelectedFile { Kind = value is null ? new SelectedFileKind { Clear = new Null() } : new SelectedFileKind { Value = value } }
      });
    var plan = new SelectionPlan();
    plan.Selections.Add(selection);
    var reply = await client.Transport.Cases.Create(plan, false, Client.Identity(requestId), client.Options(cancellationToken));
    return reply.Cases.Select(value => new Case(client, value.CaseId, value.Metadata?.Label ?? "", value.SuiteId, value.TemplateId)).ToArray();
  }
  public async Task<Case> GetAsync(string identifier, string cycle = "", CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Cases.Get(identifier, cycle, context: client.Options(cancellationToken));
    var value = reply.Value.Summary;
    return new Case(client, value.CaseId, value.Metadata?.Label ?? "", value.SuiteId, value.TemplateId, cycle, reply.ResolvedAt);
  }
}

public sealed class Case
{
  internal Client Client
  {
    get;
  }
  private readonly string cycle;
  private ReadCut? at;
  internal Case(Client client, string identifier, string label, string suite, string template, string cycle = "", ReadCut? at = null)
  {
    Client = client;
    Identifier = identifier;
    Label = label;
    Suite = suite;
    Template = template;
    this.cycle = cycle;
    this.at = at;
  }
  public string Identifier
  {
    get;
  }
  public string Label
  {
    get; private set;
  }
  public string Suite
  {
    get;
  }
  public string Template
  {
    get;
  }
  public async Task<IReadOnlyDictionary<string, object?>> ParametersAsync(CancellationToken cancellationToken = default)
  {
    if (at is null)
      await RefreshAsync(cancellationToken);
    var reply = await Client.Transport.Cases.Parameters(Identifier, cycle, at, Client.Options(cancellationToken));
    return reply.Parameters.ToDictionary(item => item.Name, item => Scalars.Decode(item.Value));
  }
  public async Task<Case> RefreshAsync(CancellationToken cancellationToken = default)
  {
    var observed = await Client.Cases.GetAsync(Identifier, cycle, cancellationToken);
    Label = observed.Label;
    at = observed.at;
    return this;
  }
}

public sealed class Cycles
{
  private readonly Client client;
  internal Cycles(Client client) => this.client = client;
  public async Task<Cycle> GetAsync(string identifier, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Cycles.Get(identifier, context: client.Options(cancellationToken));
    return new Cycle(client, reply.Cycle);
  }
  public async IAsyncEnumerable<Cycle> ListAsync(string text = "", int pageSize = 0, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var page = Paging.Page(pageSize);
    ReadCut? at = null;
    HashSet<string> tokens = [];
    do
    {
      var reply = await client.Transport.Cycles.List(new CycleSelection(), new CycleFilter { Text = text }, new CycleOrder(), at, page, client.Options(cancellationToken));
      at = Paging.Cut(at, reply.ResolvedAt);
      foreach (var value in reply.Cycles)
        yield return new Cycle(client, value);
      page.PageToken = Paging.Token(tokens, reply.NextPageToken);
    } while (page.PageToken.Length > 0);
  }
}

public sealed class Cycle
{
  private readonly Client client;
  private CycleSummary summary;
  internal Cycle(Client client, CycleSummary summary)
  {
    if (string.IsNullOrEmpty(summary?.CycleId))
      throw new InvalidDataException("Read Cycle: Request a Complete Cycle Identity");
    this.client = client;
    this.summary = summary.Clone();
  }
  public string Identifier => summary.CycleId;
  public string Label => summary.Metadata?.Label ?? "";
  public string Phase => summary.Phase.ToString();
  public async Task<Execution> ExecutionAsync(Case testCase, string? requestId = null, CancellationToken cancellationToken = default)
  {
    if (testCase.Client != client)
      throw new ArgumentException("Create Execution: Select a Case from This Client");
    var identity = Client.Identity(requestId);
    var reply = await client.Transport.Executions.Create(Identifier,
      [new ExecutionBinding { CaseId = testCase.Identifier, ExecutionId = MutationIdentity.Execution(Identifier, identity) }], identity, client.Options(cancellationToken));
    if (reply.Executions.Count != 1)
      throw new InvalidDataException("Create Execution: Request Exactly One Bound Execution");
    return new Execution(client, reply.Executions[0], this, testCase);
  }
  public async Task<Cycle> RefreshAsync(CancellationToken cancellationToken = default)
  {
    summary = (await client.Cycles.GetAsync(Identifier, cancellationToken)).summary;
    return this;
  }
  public async Task SealAsync(string? requestId = null, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Cycles.Seal(Identifier, Client.Identity(requestId), client.Options(cancellationToken));
    if (reply.Seal.Length > 0)
    {
      summary.Phase = CyclePhase.Sealed;
      return;
    }
    await foreach (var frame in client.Transport.Cycles.Watch(new CycleSelection { CycleIds = { Identifier } }, new CycleFilter(), new CycleOrder(), context: client.Options(cancellationToken), cancellationToken: cancellationToken))
    {
      if (frame.ResyncRequired is not null)
        throw new InvalidDataException("Seal Cycle: Reconnect and Read the Cycle After Resynchronization");
      foreach (var observed in frame.Cycles.Where(value => value.CycleId == Identifier))
      {
        summary = observed.Clone();
        if (summary.Phase == CyclePhase.Sealed)
          return;
        if (summary.Phase == CyclePhase.SealFailed)
          throw new InvalidOperationException($"Seal Cycle: Resolve the Seal Failure and Retry: {summary.SealFailure}");
      }
    }
    throw new InvalidDataException("Seal Cycle: Reconnect; Observation Ended Before Seal Settlement");
  }
  public Task<byte[]> EvidenceAsync(CancellationToken cancellationToken = default) => client.ArchiveAsync(new EvidenceSubject { Cycle = Identifier }, cancellationToken);
  public Task<Note> NoteAsync(string text, string? requestId = null, CancellationToken cancellationToken = default) =>
    client.NoteAsync(Identifier, null, text, requestId, cancellationToken);
  public Task<Artifact> ArtifactAsync(string name, byte[] data, string mime = "application/octet-stream", string? requestId = null, CancellationToken cancellationToken = default) =>
    client.ArtifactAsync(new ArtifactScope { CycleId = Identifier }, name, data, mime, requestId, cancellationToken);
}

public sealed class Executions
{
  private readonly Client client;
  internal Executions(Client client) => this.client = client;
  public async Task<Execution> GetAsync(string identifier, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Executions.Get(identifier, context: client.Options(cancellationToken));
    return new Execution(client, reply.Execution);
  }
  public async IAsyncEnumerable<Execution> ListAsync(string? cycle = null, string? template = null, string? state = null, int pageSize = 0,
    [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var selection = new ExecutionSelection();
    if (cycle is not null)
      selection.CycleIds.Add(cycle);
    if (template is not null)
      selection.TemplateIds.Add(template);
    if (state is not null)
      selection.Statuses.Add(state.ToLowerInvariant() switch
      {
        "open" => ExecutionState.Open,
        "closed" => ExecutionState.Closed,
        "sealed" => ExecutionState.Sealed,
        _ => throw new ArgumentException("List Executions: Select Open, Closed or Sealed")
      });
    var page = Paging.Page(pageSize);
    ReadCut? at = null;
    HashSet<string> tokens = [];
    do
    {
      var reply = await client.Transport.Executions.List(cycle ?? "", selection, new ExecutionFilter(), new ExecutionOrder(), at, page, client.Options(cancellationToken));
      at = Paging.Cut(at, reply.ResolvedAt);
      foreach (var value in reply.Executions)
        yield return new Execution(client, value);
      page.PageToken = Paging.Token(tokens, reply.NextPageToken);
    } while (page.PageToken.Length > 0);
  }
}

public sealed class Execution
{
  private readonly Client client;
  private ExecutionDetail detail;
  internal Execution(Client client, ExecutionDetail detail, Cycle? cycle = null, Case? testCase = null)
  {
    if (string.IsNullOrEmpty(detail?.Summary?.ExecutionId) || string.IsNullOrEmpty(detail.CycleId) || string.IsNullOrEmpty(detail.Summary.CaseId))
      throw new InvalidDataException("Read Execution: Request a Complete Execution, Cycle and Case Binding");
    this.client = client;
    this.detail = detail.Clone();
    Cycle = cycle ?? new Cycle(client, new CycleSummary { CycleId = detail.CycleId });
    Case = testCase ?? new Case(client, detail.Summary.CaseId, "", detail.AdmittedSuite?.Identifier ?? "", detail.AdmittedTemplate?.Identifier ?? "", detail.CycleId);
    Observe(detail);
  }
  public string Identifier => detail.Summary.ExecutionId;
  internal Client Client => client;
  public Cycle Cycle
  {
    get;
  }
  public Case Case
  {
    get;
  }
  private void Observe(ExecutionDetail observed)
  {
    if (observed.Summary?.ExecutionId != Identifier || observed.CycleId != Cycle.Identifier || observed.Summary.CaseId != Case.Identifier)
      throw new InvalidDataException("Read Execution: Reconnect; the Host Changed the Bound Execution, Cycle or Case");
    detail = observed.Clone();
  }
  public async Task<Execution> RefreshAsync(CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Executions.Get(Identifier, context: client.Options(cancellationToken));
    Observe(reply.Execution);
    return this;
  }
  public async Task StartAsync(string? requestId = null, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Executions.Start(Identifier, ExecutionMode.Automatic, Client.Identity(requestId), client.Options(cancellationToken));
    Observe(reply.Execution);
  }
  public async IAsyncEnumerable<Progress> WatchAsync([EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var frame in client.Transport.Executions.Progress.Watch(Identifier, context: client.Options(cancellationToken), cancellationToken: cancellationToken))
    {
      yield return new Progress(frame.Phase, frame.Value?.Completed ?? 0, frame.Value?.Total ?? 0, frame.Value?.Label ?? "", frame.Sealed);
      if (frame.Sealed)
        yield break;
    }
    throw new InvalidDataException("Watch Execution: Reconnect; Observation Ended Without a Sealed Frame");
  }
  public async IAsyncEnumerable<Progress> RunAsync(string? requestId = null, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await StartAsync(requestId, cancellationToken);
    await foreach (var progress in WatchAsync(cancellationToken))
      yield return progress;
  }
  public async Task StopAsync(string? requestId = null, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Executions.Stop(Identifier, Client.Identity(requestId), client.Options(cancellationToken));
    Observe(reply.Execution);
  }
  public async Task CancelAsync(string? requestId = null, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Executions.Cancel(Identifier, Client.Identity(requestId), client.Options(cancellationToken));
    Observe(reply.Execution);
  }
  public async Task<Outcome> SealAsync(string? requestId = null, CancellationToken cancellationToken = default)
  {
    var reply = await client.Transport.Executions.Seal(Identifier, Client.Identity(requestId), client.Options(cancellationToken));
    return Outcome.FromWire(reply.Outcome);
  }
  public Task<byte[]> EvidenceAsync(CancellationToken cancellationToken = default) => client.ArchiveAsync(new EvidenceSubject { Execution = Identifier }, cancellationToken);
  public Task<Note> NoteAsync(string text, string? requestId = null, CancellationToken cancellationToken = default) =>
    client.NoteAsync(Cycle.Identifier, Identifier, text, requestId, cancellationToken);
  public Task<Artifact> ArtifactAsync(string name, byte[] data, string mime = "application/octet-stream", string? requestId = null, CancellationToken cancellationToken = default) =>
    client.ArtifactAsync(new ArtifactScope { ExecutionId = Identifier }, name, data, mime, requestId, cancellationToken);
  public RecordsReader<T> Records<T>(Record<T> declaration, string step = "") => new(this, Resolve(declaration), step);
  public RecordsReader<T> Records<T>(string identifier, string step = "") => new(this, identifier, step);
  public SamplesReader<T> Samples<T>(Dokime.Templates.Stream<T> declaration, IReadOnlyList<string>? fields = null,
    DateTimeOffset? start = null, DateTimeOffset? end = null) => new(this, Resolve(declaration), fields, start, end);
  public SamplesReader<T> Samples<T>(string identifier, IReadOnlyList<string>? fields = null, DateTimeOffset? start = null, DateTimeOffset? end = null) =>
    new(this, identifier, fields, start, end);

  private string Resolve(object declaration)
  {
    var bindings = client.Plugins.Bindings.Where(binding => binding.Template == Case.Template).ToArray();
    if (bindings.Length != 1)
      throw new ArgumentException("Read Evidence: Select a Declaration Bound to This Execution's Registered Template");
    return bindings[0].Contract.Resolve(declaration);
  }

  internal async Task PrepareAsync<T>(string identifier, bool stream, CancellationToken cancellationToken)
  {
    if (detail.Schemas is null)
      await RefreshAsync(cancellationToken);
    var schemas = detail.Schemas ?? throw new InvalidDataException("Read Evidence: Refresh an Execution with Frozen Schemas");
    var name = stream ? schemas.Streams.SingleOrDefault(value => value.Stream == identifier)?.MessageName
      : schemas.Records.SingleOrDefault(value => value.RecordType == identifier)?.MessageName;
    if (string.IsNullOrEmpty(name))
      throw new InvalidDataException("Read Evidence: Select a Frozen Message Binding from This Execution");
    var files = await client.Schemas.GetAsync(client.Transport, detail, client.Options(cancellationToken));
    SchemaBinding.Validate<T>(files, name);
  }
}

internal static class Paging
{
  internal static PageRequest Page(int size) => size >= 0 ? new PageRequest { PageSize = (uint)size }
    : throw new ArgumentException("List Resources: Supply a Nonnegative Page Size");
  internal static ReadCut Cut(ReadCut? previous, ReadCut current) => previous is null || previous.Equals(current) ? current
    : throw new InvalidDataException("List Resources: Reconnect; the Retained Read Cut Changed");
  internal static string Token(HashSet<string> tokens, string token) => token.Length == 0 || tokens.Add(token) ? token
    : throw new InvalidDataException("List Resources: Restart the Read; the Host Repeated a Page Token");
}

internal static class MutationIdentity
{
  internal static string Execution(string cycle, string request)
  {
    var namespaceId = new Guid("6ba7b811-9dad-11d1-80b4-00c04fd430c8");
    var prefix = namespaceId.ToByteArray(bigEndian: true);
    var name = System.Text.Encoding.UTF8.GetBytes($"dokime/execution/{cycle}/{request}");
    var hash = System.Security.Cryptography.SHA1.HashData([.. prefix, .. name]);
    hash[6] = (byte)((hash[6] & 15) | 80);
    hash[8] = (byte)((hash[8] & 63) | 128);
    return new Guid(hash.AsSpan(0, 16), bigEndian: true).ToString();
  }
}

internal static class Scalars
{
  internal static object? Optional(Value? value) => value is null ? null : Decode(value);
  internal static Value Encode(object? value) => value switch
  {
    null => new() { Null = new Null() },
    Rational rational => new() { Exact = rational.ToWire() },
    byte[] bytes => new() { Data = ByteString.CopyFrom(bytes) },
    DateTimeOffset at => new() { Datetime = Google.Protobuf.WellKnownTypes.Timestamp.FromDateTimeOffset(at) },
    IReadOnlyList<object?> items => new() { List = new ValueList { Values = { items.Select(Encode) } } },
    bool flag => new() { Boolean = flag },
    string text => new() { Text = text },
    int number => new() { Integer = number },
    long number => new() { Integer = number },
    ulong number => new() { UnsignedInteger = number },
    double number => new() { Number = number },
    _ => throw new ArgumentException("Set Parameter: Supply a Supported Scalar Value")
  };
  internal static object? Decode(Value value) => value.KindCase switch
  {
    Value.KindOneofCase.Null => null,
    Value.KindOneofCase.Data => value.Data.ToByteArray(),
    Value.KindOneofCase.Datetime => value.Datetime.ToDateTimeOffset(),
    Value.KindOneofCase.List => value.List.Values.Select(Decode).ToArray(),
    Value.KindOneofCase.Exact => new Rational(System.Numerics.BigInteger.Parse(value.Exact.Numerator) * (value.Exact.Negative ? -1 : 1), System.Numerics.BigInteger.Parse(value.Exact.Denominator)),
    Value.KindOneofCase.Boolean => value.Boolean,
    Value.KindOneofCase.Text => value.Text,
    Value.KindOneofCase.Integer => value.Integer,
    Value.KindOneofCase.UnsignedInteger => value.UnsignedInteger,
    Value.KindOneofCase.Number => value.Number,
    _ => throw new InvalidDataException("Read Parameter: Request a Supported Scalar Value")
  };
}
