// SPDX-License-Identifier: Apache-2.0

using System.Collections.Concurrent;
using Dokime;
using Dokime.Plugins;
using Google.Protobuf;
using Harmos.Plugins;
using Harmos;
using ModelAnalyses = Dokime.Analyses;
using ModelCriterion = Dokime.Criterion;
using ModelStep = Dokime.Step;
using ModelSuite = Dokime.Suite;
using TransportContext = Harmos.Plugins.Context;
using TransportPlugin = Harmos.Plugins.Plugin;

namespace Dokime.Plugins;

/// <summary>Composed suites served through the exact-revision Harmos SDK.</summary>
public sealed partial class Plugin(string identifier, string version, params Suite[] suites)
{
  private readonly TransportPlugin transport = new(identifier, version);
  private readonly ConcurrentDictionary<string, Lazy<Task<ExecutionResource>>> resources = new();
  private readonly RecordContracts records = new();
  private readonly SecretsDeclarations secrets = new();
  private readonly ModelAnalyses analyses = new();
  private bool built;

  public Plugin Setting(string name, SettingValue defaultValue, string title = "", string description = "")
  {
    if (built)
      throw new InvalidOperationException("Declare Setting: Plugin Is Already Built");
    transport.Setting(name, defaultValue, title, description);
    return this;
  }

  public Plugin Secret(string name, string label, bool required = true)
  {
    if (built)
      throw new InvalidOperationException("Declare Secret: Plugin Is Already Built");
    ArgumentException.ThrowIfNullOrWhiteSpace(name);
    if (secrets.Secrets.Any(item => item.Metadata.Identifier == name))
      throw new ArgumentException($"Declare Secret: Duplicate Name {name}");
    secrets.Secrets.Add(new SecretDeclaration
    {
      Metadata = new Metadata { Identifier = name, Label = label },
      Required = required
    });
    return this;
  }

  public Plugin Analysis(Analysis definition,
      Func<Context, AnalysisSamples, MapValue, Task<IReadOnlyList<AnalysisValues>>> compute,
      OperationPolicy? policy = null)
  {
    if (built)
      throw new InvalidOperationException("Declare Analysis: Plugin Is Already Built");
    var declaration = definition.Clone();
    var operation = declaration.Metadata.Identifier;
    if (analyses.Analyses_.Any(item => item.Metadata.Identifier == operation))
      throw new ArgumentException($"Declare Analysis: Duplicate Identifier {operation}");
    analyses.Analyses_.Add(declaration);
    transport.Operation(new Operation<AnalysesComputeRequest, AnalysesComputeResponse>(operation, Visibility.Protected, policy),
        async (raw, request) =>
        {
          var outputs = await compute(new Context(raw, new TestCase()), request.Samples, request.Parameters);
          var response = new AnalysesComputeResponse();
          response.Outputs.AddRange(outputs.Select(output => output.Clone()));
          return response;
        });
    return this;
  }

  public Plugin Record<T>(Record<T> record) where T : IMessage<T>, new()
  {
    if (built)
      throw new InvalidOperationException("Declare Record: Plugin Is Already Built");
    if (records.Records.Any(item => item.Metadata.Identifier == record.Identifier))
      throw new ArgumentException($"Declare Record: Duplicate Identifier {record.Identifier}");
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
  public TransportPlugin Build()
  {
    if (built)
      return transport;
    var identifiers = new HashSet<string>(StringComparer.Ordinal);
    var section = new Suites();
    foreach (var suite in suites)
    {
      Unique(identifiers, suite.Metadata.Identifier);
      var definition = new ModelSuite { Metadata = suite.Metadata.Clone() };
      foreach (var template in suite.Templates)
      {
        Unique(identifiers, template.Definition.Metadata.Identifier);
        Register(template, definition, identifiers);
      }
      section.Suites_.Add(definition);
    }
    if (suites.Length > 0)
      transport.Section(section);
    if (records.Records.Count > 0)
      transport.Section(records);
    if (secrets.Secrets.Count > 0)
      transport.Section(secrets);
    if (analyses.Analyses_.Count > 0)
      transport.Section(analyses);
    transport.OnStop(DisposeResourcesAsync);
    built = true;
    return transport;
  }

  public async Task ServeAsync(string? address = null, bool remote = false, CancellationToken cancellationToken = default)
  {
    try
    {
      await Build().ServeAsync(address, remote, cancellationToken);
    }
    finally
    {
      await DisposeResourcesAsync();
    }
  }

  private void Register(Template template, ModelSuite suite, HashSet<string> identifiers)
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
      testCase.Steps.Add(new ModelStep { Metadata = new Metadata { Identifier = step.Identifier, Label = step.Label }, Operation = operation, Repeatable = step.Repeatable });
      // Declare the Procedure on the Template So Catalogs Show It Before Materialization.
      definition.Steps.Add(testCase.Steps[^1].Clone());
      transport.Operation(new Operation<StepsExecuteRequest, StepsExecuteResponse>(operation, Visibility.Protected, policy: step.Policy, replaySafe: step.ReplaySafe), async (raw, request) =>
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
      testCase.Criteria.Add(new ModelCriterion { Metadata = new Metadata { Identifier = criterion.Identifier, Label = criterion.Label }, Operation = operation });
      transport.Operation(new Operation<CriteriaEvaluateRequest, CriteriaEvaluateResponse>(operation, Visibility.Protected, policy: criterion.Policy), async (raw, request) =>
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
    transport.Operation(new Operation<TestCasesMaterializeRequest, TestCasesMaterializeResponse>(definition.Materialize, Visibility.Protected, policy: template.MaterializePolicy), (raw, request) =>
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
      transport.Operation(new Operation<ExecutionsInitializeRequest, ExecutionsInitializeResponse>(definition.Initialize, Visibility.Protected, policy: template.InitializePolicy), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, definition.Initialize);
        var context = await GetContextAsync(raw, template, request.TestCase);
        if (template.InitializeBody is not null)
          await template.InitializeBody(context);
        return new ExecutionsInitializeResponse();
      });
    transport.Operation(new Operation<ExecutionsCleanupRequest, ExecutionsCleanupResponse>(definition.Cleanup, Visibility.Protected, policy: template.CleanupPolicy), async (raw, request) =>
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
        if (resources.TryRemove(key, out _) && owned?.Resource is { } resource)
          await resource.DisposeAsync();
      }
      return new ExecutionsCleanupResponse();
    });
  }

  private async Task<Context> GetContextAsync(TransportContext raw, Template template, TestCase testCase)
  {
    var key = $"{template.Definition.Metadata.Identifier}/{raw.Scope()}";
    var pending = resources.GetOrAdd(key, _ => new Lazy<Task<ExecutionResource>>(async () =>
    {
      var context = new Context(raw, testCase);
      var resource = template.CreateResource is null ? null : await template.CreateResource(context);
      return new ExecutionResource(testCase.Clone(), resource);
    }));
    var owned = await pending.Value;
    if (!owned.TestCase.Equals(testCase))
      throw new ArgumentException("Read Execution Resource: Frozen Case Changed Within the Same Scope");
    return new Context(raw, testCase, owned.Resource);
  }

  private Context ObserverContext(TransportContext raw, string template)
  {
    if (resources.TryGetValue($"{template}/{raw.Scope()}", out var pending) && pending.IsValueCreated && pending.Value.IsCompletedSuccessfully)
    {
      var owned = pending.Value.Result;
      return new Context(raw, owned.TestCase, owned.Resource);
    }
    return new Context(raw, new TestCase());
  }

  private void Validate(TransportContext raw, string execution, string template, TestCase testCase, Origin origin, string action)
  {
    if (string.IsNullOrWhiteSpace(execution) || raw.Scope() != execution || testCase is null ||
        origin?.PluginRun is not { } run || string.IsNullOrWhiteSpace(run.Plugin) || run.Template != template || !testCase.Equals(run.TestCase))
      throw new ArgumentException($"{action}: Execution Scope, Origin and Frozen Case Must Agree");
  }

  private async Task DisposeResourcesAsync()
  {
    List<Exception> failures = [];
    foreach (var key in resources.Keys)
    {
      if (!resources.TryRemove(key, out var pending))
        continue;
      try
      {
        var owned = await pending.Value;
        if (owned.Resource is { } resource)
          await resource.DisposeAsync();
      }
      catch (Exception failure)
      {
        failures.Add(failure);
      }
    }
    if (failures.Count > 0)
      throw new AggregateException("Dispose Execution Resources: Cleanup Failed", failures);
  }

  private static void Unique(HashSet<string> identifiers, string identifier)
  {
    ArgumentException.ThrowIfNullOrWhiteSpace(identifier);
    if (!identifiers.Add(identifier))
      throw new ArgumentException($"Build Plugin: Duplicate Identifier {identifier}");
  }
  private sealed record ExecutionResource(TestCase TestCase, IAsyncDisposable? Resource);
}
