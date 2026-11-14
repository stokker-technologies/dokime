// SPDX-License-Identifier: Apache-2.0

using System.Collections.Concurrent;
using Dokime.Protocol.V1;
using Dokime.Protocol.Plugin.V1;
using Google.Protobuf;
using Harmos.Sdk;
using Harmos.V1;

namespace Dokime;

/// <summary>Composed suites served through the exact-revision Harmos SDK.</summary>
public sealed class Plugin(string identifier, string version, params Suite[] suites)
{
  private readonly Harmos.Sdk.Plugin transport = new(identifier, version);
  private readonly ConcurrentDictionary<string, Lazy<Task<ExecutionResource>>> resources = new();
  private readonly RecordContracts records = new();
  private bool built;

  public Plugin Record<T>(Record<T> record) where T : IMessage<T>, new()
  {
    if (built) throw new InvalidOperationException("Plugin is already built");
    if (records.Records.Any(item => item.Metadata.Identifier == record.Identifier))
      throw new ArgumentException($"Duplicate record: {record.Identifier}");
    transport.Message<T>();
    records.Records.Add(new RecordContract
    {
      Metadata = new Metadata { Identifier = record.Identifier, Label = record.Label },
      Message = new T().Descriptor.FullName
    });
    return this;
  }

  public Plugin Requires<TRequest, TResponse>(Operation<TRequest, TResponse> operation)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
  {
    transport.Requires(operation);
    return this;
  }

  /// <summary>Add instrument operations or requirements before serving.</summary>
  public Harmos.Sdk.Plugin Build()
  {
    if (built) return transport;
    var identifiers = new HashSet<string>(StringComparer.Ordinal);
    var section = new Suites();
    foreach (var suite in suites)
    {
      Unique(identifiers, suite.Metadata.Identifier);
      var definition = new Dokime.Protocol.V1.Suite { Metadata = suite.Metadata.Clone() };
      foreach (var template in suite.Templates)
      {
        Unique(identifiers, template.Definition.Metadata.Identifier);
        Register(template, definition, identifiers);
      }
      section.Suites_.Add(definition);
    }
    if (suites.Length > 0) transport.Section(section);
    if (records.Records.Count > 0) transport.Section(records);
    transport.OnStop(DisposeResourcesAsync);
    built = true;
    return transport;
  }

  public async Task ServeAsync(string? address = null, bool remote = false, CancellationToken cancellationToken = default)
  {
    try { await Build().ServeAsync(address, remote, cancellationToken); }
    finally { await DisposeResourcesAsync(); }
  }

  private void Register(Template template, Dokime.Protocol.V1.Suite suite, HashSet<string> identifiers)
  {
    var name = template.Definition.Metadata.Identifier;
    var definition = template.Definition.Clone();
    definition.Materialize = $"{name}.materialize";
    // Cleanup Always Exists to Release SDK-Owned Resources.
    definition.Cleanup = $"{name}.cleanup";
    if (template.InitializeBody is not null || template.CreateResource is not null)
      definition.Initialize = $"{name}.initialize";
    suite.Templates.Add(definition);

    var testCase = new TestCase();
    var local = new HashSet<string>(StringComparer.Ordinal);
    foreach (var (step, index) in template.Steps.Select((step, index) => (step, index)))
    {
      Unique(local, step.Identifier);
      var operation = $"{name}.step.{index}";
      testCase.Steps.Add(new Dokime.Protocol.V1.Step { Metadata = new Metadata { Identifier = step.Identifier, Label = step.Label }, Operation = operation, Repeatable = step.Repeatable });
      var policy = step.Retries == 0 ? null : new OperationPolicy { Retries = new RetryPolicy { TotalAttempts = checked(step.Retries + 1), Initial = new Google.Protobuf.WellKnownTypes.Duration { Nanos = 100_000_000 } } };
      transport.Operation(new Operation<StepsExecuteRequest, StepsExecuteResponse>(operation, Visibility.Protected, policy: policy, replaySafe: step.Retries > 0), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, operation);
        if (!request.TestCase.Steps.Any(item => item.Metadata.Identifier == request.Step && item.Operation == operation))
          throw new ArgumentException($"{operation}: Step Does Not Match the Frozen Case");
        var context = await GetContextAsync(raw, template, request.TestCase);
        await step.Run(context);
        return new StepsExecuteResponse();
      });
    }
    foreach (var (criterion, index) in template.Criteria.Select((criterion, index) => (criterion, index)))
    {
      Unique(local, criterion.Identifier);
      var operation = $"{name}.criterion.{index}";
      testCase.Criteria.Add(new Dokime.Protocol.V1.Criterion { Metadata = new Metadata { Identifier = criterion.Identifier, Label = criterion.Label }, Operation = operation });
      transport.Operation(new Operation<CriteriaEvaluateRequest, CriteriaEvaluateResponse>(operation, Visibility.Protected), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, operation);
        if (!request.TestCase.Criteria.Any(item => item.Metadata.Identifier == request.Criterion && item.Operation == operation))
          throw new ArgumentException($"{operation}: Criterion Does Not Match the Frozen Case");
        var context = await GetContextAsync(raw, template, request.TestCase);
        return new CriteriaEvaluateResponse { Passed = await criterion.Evaluate(context) };
      });
    }
    foreach (var stream in template.Streams)
    {
      Unique(identifiers, stream.Identifier);
      testCase.Streams.Add(stream.Identifier);
      stream.Register(transport, raw => ObserverContext(raw, name));
    }
    transport.Operation(new Operation<TestCasesMaterializeRequest, TestCasesMaterializeResponse>(definition.Materialize, Visibility.Protected), (raw, request) =>
    {
      try
      {
        raw.Scope();
        throw new ArgumentException($"{definition.Materialize}: Materialization Requires No Execution Scope");
      }
      catch (PluginFailureException error) when (error.Failure.Code == "scope_required") { }
      if (request.Template != name)
        throw new ArgumentException($"{definition.Materialize}: Materialization Requires the Named Template and No Execution Scope");
      var result = testCase.Clone();
      result.Parameters.Add(request.Parameters.ToDictionary(item => item.Key, item => item.Value.Clone()));
      result = template.MaterializeBody?.Invoke(result, result.Parameters) ?? result;
      if (result.Steps.Any(item => !testCase.Steps.Any(known => known.Operation == item.Operation)) ||
              result.Criteria.Any(item => !testCase.Criteria.Any(known => known.Operation == item.Operation)) ||
              result.Streams.Any(item => !testCase.Streams.Contains(item)))
        throw new ArgumentException($"{definition.Materialize}: Materialized Case Refers to an Unregistered Operation or Stream");
      return Task.FromResult(new TestCasesMaterializeResponse { TestCase = result });
    });
    if (definition.HasInitialize)
      transport.Operation(new Operation<ExecutionsInitializeRequest, ExecutionsInitializeResponse>(definition.Initialize, Visibility.Protected), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, definition.Initialize);
        var context = await GetContextAsync(raw, template, request.TestCase);
        if (template.InitializeBody is not null) await template.InitializeBody(context);
        return new ExecutionsInitializeResponse();
      });
    transport.Operation(new Operation<ExecutionsCleanupRequest, ExecutionsCleanupResponse>(definition.Cleanup, Visibility.Protected), async (raw, request) =>
    {
      Validate(raw, request.Execution, name, request.TestCase, request.Origin, definition.Cleanup);
      var key = $"{name}/{raw.Scope()}";
      resources.TryGetValue(key, out var pending);
      var owned = pending is null ? null : await pending.Value;
      try
      {
        if (template.CleanupBody is not null)
          await template.CleanupBody(new Context(raw, request.TestCase, owned?.Resource));
      }
      finally
      {
        if (resources.TryRemove(key, out _) && owned?.Resource is { } resource) await resource.DisposeAsync();
      }
      return new ExecutionsCleanupResponse();
    });
  }

  private async Task<Context> GetContextAsync(Harmos.Sdk.Context raw, Template template, TestCase testCase)
  {
    var key = $"{template.Definition.Metadata.Identifier}/{raw.Scope()}";
    var pending = resources.GetOrAdd(key, _ => new Lazy<Task<ExecutionResource>>(async () =>
    {
      var context = new Context(raw, testCase);
      var resource = template.CreateResource is null ? null : await template.CreateResource(context);
      return new ExecutionResource(testCase.Clone(), resource);
    }));
    var owned = await pending.Value;
    if (!owned.TestCase.Equals(testCase)) throw new ArgumentException("Execution case changed within the same scope");
    return new Context(raw, testCase, owned.Resource);
  }

  private Context ObserverContext(Harmos.Sdk.Context raw, string template)
  {
    if (resources.TryGetValue($"{template}/{raw.Scope()}", out var pending) && pending.IsValueCreated && pending.Value.IsCompletedSuccessfully)
    {
      var owned = pending.Value.Result;
      return new Context(raw, owned.TestCase, owned.Resource);
    }
    return new Context(raw, new TestCase());
  }

  private void Validate(Harmos.Sdk.Context raw, string execution, string template, TestCase testCase, Origin origin, string action)
  {
    if (string.IsNullOrWhiteSpace(execution) || raw.Scope() != execution || testCase is null ||
        origin?.PluginRun is not { } run || run.Plugin != identifier || run.Template != template || !testCase.Equals(run.TestCase))
      throw new ArgumentException($"{action}: Execution Scope, Origin and Frozen Case Must Agree");
  }

  private async Task DisposeResourcesAsync()
  {
    List<Exception> failures = [];
    foreach (var key in resources.Keys)
    {
      if (!resources.TryRemove(key, out var pending)) continue;
      try
      {
        var owned = await pending.Value;
        if (owned.Resource is { } resource) await resource.DisposeAsync();
      }
      catch (Exception failure) { failures.Add(failure); }
    }
    if (failures.Count > 0) throw new AggregateException(failures);
  }

  private static void Unique(HashSet<string> identifiers, string identifier)
  {
    ArgumentException.ThrowIfNullOrWhiteSpace(identifier);
    if (!identifiers.Add(identifier)) throw new ArgumentException($"Duplicate identifier: {identifier}");
  }
  private sealed record ExecutionResource(TestCase TestCase, IAsyncDisposable? Resource);
}
