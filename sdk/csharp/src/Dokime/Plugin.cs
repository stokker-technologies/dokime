// SPDX-License-Identifier: Apache-2.0

using System.Collections.Concurrent;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.Plugins;
using Harmos;
using Dokime.Templates;

namespace Dokime.Plugins;

/// <summary>Composed suites served through the exact-revision Harmos SDK.</summary>
internal sealed partial class Plugin(Dokime.Plugin author, string identifier, string version, params SuiteBinding[] suites)
{
  private readonly Harmos.Plugins.Plugin transport = new(identifier, version);
  private readonly ConcurrentDictionary<string, Lazy<Task<ExecutionResource>>> resources = new();
  private readonly RecordContracts records = new();
  private readonly SecretsDeclarations secrets = new();
  private readonly PromptDefinitions prompts = new();
  private bool built;
  public IReadOnlyList<SuiteBinding> Suites { get; } = BindSuites(suites);

  private static IReadOnlyList<SuiteBinding> BindSuites(SuiteBinding[] suites)
  {
    var identifiers = new HashSet<string>(StringComparer.Ordinal);
    var bound = new List<SuiteBinding>();
    foreach (var suite in suites)
    {
      var identity = Identifiers.Build("", suite.Metadata.Identifier);
      Unique(identifiers, identity);
      var templates = suite.Templates.Select(template => template.Bind(identity)).ToArray();
      var templateIdentifiers = new HashSet<string>(StringComparer.Ordinal);
      foreach (var template in templates)
        Unique(templateIdentifiers, template.Definition.Metadata.Identifier);
      bound.Add(new SuiteBinding(suite.Metadata, templates));
    }
    return bound;
  }

  public Plugin Lane(Dokime.Lane lane)
  {
    if (built)
      throw new InvalidOperationException("Declare Lane: Create the Plugin Before Serving It");
    transport.Lane(lane.Native);
    return this;
  }

  public Plugin Setting(string name, SettingValue defaultValue, string title = "", string description = "")
  {
    if (built)
      throw new InvalidOperationException("Declare Setting: Plugin Is Already Built");
    transport.Register(registrar => registrar.Register(name, defaultValue, title, description));
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

  public Plugin Operation<TRequest, TResponse>(Dokime.Operations.Operation<TRequest, TResponse> operation,
      Func<Dokime.Operations.Context, TRequest, Task<TResponse>> handler)
  {
    if (built)
      throw new InvalidOperationException("Declare Operation: Create the Plugin Before Serving It");
    transport.Register(registrar => operation.Register(registrar, raw => new Dokime.Operations.Context(author, raw), handler));
    return this;
  }

  public Plugin Requires<TRequest, TResponse>(Operation<TRequest, TResponse> operation)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
  {
    transport.Register(registrar => registrar.Register(operation));
    return this;
  }

  /// <summary>Add instrument operations or requirements before serving.</summary>
  internal Harmos.Plugins.Plugin Build()
  {
    if (built)
      return transport;
    transport.Register(Register);
    built = true;
    return transport;
  }

  private void Register(Registrar registrar)
  {
    foreach (var type in author.GetType().Assembly.GetTypes())
      foreach (var field in type.GetFields(System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic))
        if (typeof(Dokime.Operations.IOperationReference).IsAssignableFrom(field.FieldType)
          && field.GetValue(null) is Dokime.Operations.IOperationReference reference)
          reference.Declare(registrar);
    foreach (var operation in System.Enum.GetValues<HostOperation>())
      this.Requires(operation);
    var identifiers = new HashSet<string>(StringComparer.Ordinal);
    var section = new Suites();
    foreach (var suite in Suites)
    {
      Unique(identifiers, suite.Metadata.Identifier);
      var definition = new Dokime.SuiteDefinition { Metadata = suite.Metadata.Clone() };
      foreach (var template in suite.Templates)
      {
        Unique(identifiers, template.Definition.Metadata.Identifier);
        Register(registrar, template, definition, identifiers);
      }
      section.Suites_.Add(definition);
    }
    if (Suites.Count > 0)
      registrar.Register(section);
    if (records.Records.Count > 0)
      registrar.Register(records);
    if (prompts.Prompts.Count > 0)
      registrar.Register(prompts);
    if (secrets.Secrets.Count > 0)
      registrar.Register(secrets);
    registrar.Register(new Operation<Empty, Empty>($"{identifier}.initialize", Visibility.Protected), async (raw, request) =>
    {
      await author.InitializeAsync(new Dokime.Operations.Context(author, raw), raw.CancellationToken);
      return new Empty();
    });
    registrar.Register(new Operation<Empty, Empty>($"{identifier}.finalize", Visibility.Protected), async (raw, request) =>
    {
      try
      {
        await author.FinalizeAsync(new Dokime.Operations.Context(author, raw), raw.CancellationToken);
        return new Empty();
      }
      finally
      {
        await DisposeResourcesAsync();
      }
    });
  }

  public async Task RunAsync(string? address = null, bool remote = false, CancellationToken cancellationToken = default)
  {
    await Build().RunAsync(address, remote, cancellationToken);
  }

  private void Register(Registrar registrar, Template template, Dokime.SuiteDefinition suite, HashSet<string> identifiers)
  {
    var name = template.Definition.Metadata.Identifier;
    var definition = template.Definition.Clone();
    definition.Materialize = Identifiers.Build("", "materialize", name);
    // Cleanup Always Exists to Release SDK-Owned Resources.
    definition.Cleanup = Identifiers.Build("", "cleanup", name);
    if (template.InitializeBody is not null)
      definition.Initialize = Identifiers.Build("", "initialize", name);
    suite.Templates.Add(definition);

    var testCase = new TestCase();
    var local = new HashSet<string>(StringComparer.Ordinal);
    foreach (var step in template.Steps)
    {
      var operation = template.Resolve(step);
      Unique(local, operation);
      testCase.Steps.Add(new Dokime.Step { Metadata = new Metadata { Identifier = operation, Label = step.Label }, Operation = operation, Repeatable = step.Repeatable });
      // Declare the Procedure on the Template So Catalogs Show It Before Materialization.
      definition.Steps.Add(testCase.Steps[^1].Clone());
      registrar.Register(new Operation<StepsExecuteRequest, StepsExecuteResponse>(operation, Visibility.Protected, policy: step.Policy, replaySafe: step.ReplaySafe), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, operation);
        if (!request.TestCase.Steps.Any(item => item.Metadata.Identifier == request.Step && item.Operation == operation))
          throw new ArgumentException($"{operation}: Step Does Not Match the Frozen Case");
        var context = await GetContextAsync(raw, template, request.TestCase);
        try
        {
          await step.Run(context);
        }
        catch (SdkError error)
        {
          throw new PluginFailureException(error.Failure.ToWire());
        }
        return new StepsExecuteResponse();
      });
    }
    foreach (var criterion in template.Criteria)
    {
      var operation = template.Resolve(criterion);
      Unique(local, operation);
      testCase.Criteria.Add(new Dokime.Criterion { Metadata = new Metadata { Identifier = operation, Label = criterion.Label }, Operation = operation });
      registrar.Register(new Operation<CriteriaEvaluateRequest, CriteriaEvaluateResponse>(operation, Visibility.Protected, policy: criterion.Policy), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, operation);
        if (!request.TestCase.Criteria.Any(item => item.Metadata.Identifier == request.Criterion && item.Operation == operation))
          throw new ArgumentException($"{operation}: Criterion Does Not Match the Frozen Case");
        var context = await GetContextAsync(raw, template, request.TestCase);
        try
        {
          return new CriteriaEvaluateResponse { Passed = await criterion.Evaluate(context) };
        }
        catch (SdkError error)
        {
          throw new PluginFailureException(error.Failure.ToWire());
        }
      });
    }
    foreach (var stream in template.Streams)
    {
      var identity = template.Resolve(stream);
      Unique(identifiers, identity);
      testCase.Streams.Add(identity);
      stream.Register(registrar, identity, raw => ObserverContext(raw, template));
    }
    foreach (var record in template.Records)
    {
      var identity = template.Resolve(record);
      Unique(identifiers, identity);
      record.Register(registrar, records, identity);
    }
    foreach (var prompt in template.Prompts)
    {
      var identity = template.Resolve(prompt);
      Unique(identifiers, identity);
      var value = prompt.Definition.Clone();
      value.Metadata ??= new Metadata();
      value.Metadata.Identifier = identity;
      value.OwnerSuite = template.OwnerSuite;
      value.Template = name;
      prompts.Prompts.Add(value);
    }
    registrar.Register(new Operation<TestCasesMaterializeRequest, TestCasesMaterializeResponse>(definition.Materialize, Visibility.Protected, policy: template.MaterializePolicy), (raw, request) =>
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
      var materialized = new HashSet<string>(StringComparer.Ordinal);
      foreach (var step in result.Steps)
      {
        if (!testCase.Steps.Any(known => known.Metadata.Identifier == step.Metadata.Identifier))
          step.Metadata.Identifier = Identifiers.Build("", step.Metadata.Identifier, name);
        Unique(materialized, step.Metadata.Identifier);
      }
      foreach (var criterion in result.Criteria)
      {
        if (!testCase.Criteria.Any(known => known.Metadata.Identifier == criterion.Metadata.Identifier))
          criterion.Metadata.Identifier = Identifiers.Build("", criterion.Metadata.Identifier, name);
        Unique(materialized, criterion.Metadata.Identifier);
      }
      if (result.Steps.Any(item => !testCase.Steps.Any(known => known.Operation == item.Operation)) ||
              result.Criteria.Any(item => !testCase.Criteria.Any(known => known.Operation == item.Operation)) ||
              result.Streams.Any(item => !testCase.Streams.Contains(item)))
        throw new ArgumentException($"{definition.Materialize}: Materialized Case Refers to an Unregistered Operation or Stream");
      return Task.FromResult(new TestCasesMaterializeResponse { TestCase = result });
    });
    if (definition.HasInitialize)
      registrar.Register(new Operation<ExecutionsInitializeRequest, ExecutionsInitializeResponse>(definition.Initialize, Visibility.Protected, policy: template.InitializePolicy), async (raw, request) =>
      {
        Validate(raw, request.Execution, name, request.TestCase, request.Origin, definition.Initialize);
        var context = await GetContextAsync(raw, template, request.TestCase);
        if (template.InitializeBody is not null)
          await template.InitializeBody(context);
        return new ExecutionsInitializeResponse();
      });
    registrar.Register(new Operation<ExecutionsCleanupRequest, ExecutionsCleanupResponse>(definition.Cleanup, Visibility.Protected, policy: template.CleanupPolicy), async (raw, request) =>
    {
      Validate(raw, request.Execution, name, request.TestCase, request.Origin, definition.Cleanup);
      var key = $"{name}/{raw.Scope()}";
      resources.TryGetValue(key, out var pending);
      var owned = pending is null ? null : await pending.Value;
      try
      {
        if (template.CleanupBody is not null)
          await template.CleanupBody(new Dokime.Templates.Context(author, raw, request.TestCase, template, owned?.Instance, request.Termination));
      }
      finally
      {
        if (resources.TryRemove(key, out _) && owned?.Instance is IAsyncDisposable resource)
          await resource.DisposeAsync();
      }
      return new ExecutionsCleanupResponse();
    });
  }

  private async Task<Dokime.Templates.Context> GetContextAsync(Harmos.Plugins.Context raw, Template template, TestCase testCase)
  {
    var key = $"{template.Definition.Metadata.Identifier}/{raw.Scope()}";
    var pending = resources.GetOrAdd(key, _ => new Lazy<Task<ExecutionResource>>(() => Task.FromResult(new ExecutionResource(testCase.Clone(), template.CreateInstance()))));
    var owned = await pending.Value;
    if (!owned.TestCase.Equals(testCase))
      throw new ArgumentException("Read Execution Resource: Frozen Case Changed Within the Same Scope");
    return new Dokime.Templates.Context(author, raw, testCase, template, owned.Instance);
  }

  private Dokime.Templates.Context ObserverContext(Harmos.Plugins.Context raw, Template template)
  {
    if (resources.TryGetValue($"{template.Definition.Metadata.Identifier}/{raw.Scope()}", out var pending) && pending.IsValueCreated && pending.Value.IsCompletedSuccessfully)
    {
      var owned = pending.Value.Result;
      return new Dokime.Templates.Context(author, raw, owned.TestCase, template, owned.Instance);
    }
    return new Dokime.Templates.Context(author, raw, new TestCase(), template: template, instance: template.CreateInstance());
  }

  private void Validate(Harmos.Plugins.Context raw, string execution, string template, TestCase testCase, Origin origin, string action)
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
        if (owned.Instance is IAsyncDisposable resource)
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
  private sealed record ExecutionResource(TestCase TestCase, object? Instance);
}
