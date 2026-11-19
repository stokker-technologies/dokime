// SPDX-License-Identifier: Apache-2.0

using System.Globalization;
using System.Reflection;
using System.Runtime.ExceptionServices;
using System.Text.RegularExpressions;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using Dokime.Plugins;

namespace Dokime.Templates;

internal sealed record Step(string Label, Func<Context, Task> Run, string? Identifier = null,
    OperationPolicy? Policy = null, bool ReplaySafe = false, bool Repeatable = false);
internal sealed record Criterion(string Label, Func<Context, Task<bool>> Evaluate, string? Identifier = null, OperationPolicy? Policy = null);

public interface IRecord
{
  string? Identifier
  {
    get;
  }
  string Label
  {
    get;
  }
  internal void Register(Harmos.Plugins.Plugin plugin, RecordContracts contracts, string identifier);
}

public sealed class Record<T>(string label = "", string? identifier = null) : IRecord
{
  public string? Identifier { get; } = identifier;
  public string Label { get; } = label;

  void IRecord.Register(Harmos.Plugins.Plugin plugin, RecordContracts contracts, string identity)
  {
    Payload<T>.Register(plugin);
    contracts.Records.Add(new RecordContract
    {
      Metadata = new Metadata { Identifier = identity, Label = Label.Length == 0 ? identity : Label },
      Message = Payload<T>.Descriptor.FullName
    });
  }
}

public sealed class PromptDeclaration
{
  public PromptDeclaration(string text, string[]? choices = null, string? identifier = null, string label = "")
  {
    Identifier = identifier;
    Definition = new Prompt
    {
      Message = text,
      Archetype = PromptArchetype.Choice,
      Metadata = new Metadata { Label = label.Length == 0 ? text : label },
    };
    var actions = choices is { Length: > 0 } ? choices : ["OK"];
    if (actions.Distinct(StringComparer.Ordinal).Count() != actions.Length)
      throw new ArgumentException("Declare Prompt: Supply Distinct Choice Labels");
    foreach (var (choice, index) in actions.Select((choice, index) => (choice, index)))
      Definition.Actions.Add(new PromptAction { Identifier = index.ToString(CultureInfo.InvariantCulture), Label = choice });
  }

  public PromptDeclaration(Prompt definition, string? identifier = null)
  {
    Definition = definition.Clone();
    Identifier = identifier ?? (definition.Metadata?.Identifier is { Length: > 0 } identity ? identity : null);
  }

  public string? Identifier
  {
    get;
  }
  internal Prompt Definition
  {
    get;
  }
}

[AttributeUsage(AttributeTargets.Method)]
public sealed class StepAttribute(string label) : Dokime.Operations.PolicyAttribute
{
  public string Label { get; } = label;
  public string? Identifier
  {
    get; set;
  }
  public bool Repeatable
  {
    get; set;
  }
}

[AttributeUsage(AttributeTargets.Method)]
public sealed class CriterionAttribute(string label) : Dokime.Operations.PolicyAttribute
{
  public string Label { get; } = label;
  public string? Identifier
  {
    get; set;
  }
}

internal static class Identifiers
{
  internal static string Build(string name, string? identifier = null, string prefix = "", bool type = false)
  {
    if (identifier is null)
    {
      if (type)
        name = Regex.Replace(name, "(?:TestTemplate|Plugin|Suite)$", "");
      name = Regex.Replace(name, "([A-Z]+)([A-Z][a-z])", "$1_$2");
      name = Regex.Replace(name, "([a-z0-9])([A-Z])", "$1_$2");
      identifier = name.ToLowerInvariant();
    }
    if (string.IsNullOrWhiteSpace(identifier))
      throw new ArgumentException("Build Plugin: Supply a Nonempty Declaration Identifier");
    return prefix.Length == 0 ? identifier : $"{prefix}.{identifier}";
  }
}

public interface IStream
{
  string? Identifier
  {
    get;
  }
  internal void Register(Harmos.Plugins.Plugin plugin, string identifier, Func<Harmos.Plugins.Context, Context> context);
}

/// <summary>Registers One Observer Form for Its Stream.</summary>
internal interface ISampler
{
  void Register(Harmos.Plugins.Plugin plugin, string stream, Func<Harmos.Plugins.Context, Context> context);
}

/// <summary>A device-driven stream; an Observer supplies samples in host-granted windows instead.</summary>
public sealed class Stream<T>(string? identifier = null, Func<Context, Publisher<T>, Task>? produce = null) : IStream
{
  public string? Identifier
  {
    get;
  } = identifier;
  internal ISampler? Observer
  {
    get;
    set;
  }
  void IStream.Register(Harmos.Plugins.Plugin plugin, string identifier, Func<Harmos.Plugins.Context, Context> context)
  {
    if (Observer is { } observer)
      observer.Register(plugin, identifier, context);
    else if (produce is not null)
      GetType().GetMethod(nameof(RegisterProducer), BindingFlags.Instance | BindingFlags.NonPublic)!
        .MakeGenericMethod(Payload<T>.Descriptor.ClrType).Invoke(this, [plugin, identifier, context]);
    else
      throw new ArgumentException($"Declare Stream {Identifier}: Supply a Producer or Observer");
  }

  private void RegisterProducer<TWire>(Harmos.Plugins.Plugin plugin, string identifier, Func<Harmos.Plugins.Context, Context> context)
    where TWire : IMessage<TWire>, new()
  {
    plugin.Stream<TWire>(identifier, (raw, publisher) => produce!(context(raw), new Publisher<T>((value, at) =>
      publisher.PublishAsync((TWire)Payload<T>.Pack(value), at is null ? null : Timestamp.FromDateTimeOffset(at.Value)))));
  }
}

/// <summary>Samples One Row per Call; Each Host Window Calls It Once per Sample, in Order.</summary>
/// <remarks><c>windowSetting</c> names a plugin Duration setting that replaces the host's window length.</remarks>
public sealed class Observer<T> : ISampler
{
  public Observer(Stream<T> stream, TimeSpan? interval, Func<Context, Task<T>> read, string intervalSetting = "", string windowSetting = "")
  {
    if (stream.Observer is not null)
      throw new ArgumentException($"Declare Observer {stream.Identifier}: Remove the Duplicate Observer");
    Stream = stream;
    Interval = interval;
    Read = read;
    IntervalSetting = intervalSetting;
    WindowSetting = windowSetting;
    stream.Observer = this;
  }
  void ISampler.Register(Harmos.Plugins.Plugin plugin, string stream, Func<Harmos.Plugins.Context, Context> context) =>
      GetType().GetMethod(nameof(RegisterWire), BindingFlags.Instance | BindingFlags.NonPublic)!
      .MakeGenericMethod(Payload<T>.Descriptor.ClrType).Invoke(this, [plugin, stream, context]);

  private void RegisterWire<TWire>(Harmos.Plugins.Plugin plugin, string stream, Func<Harmos.Plugins.Context, Context> context)
    where TWire : IMessage<TWire>, new() =>
    plugin.Observer<TWire>(stream, Interval is null ? null : Duration.FromTimeSpan(Interval.Value), async raw => (TWire)Payload<T>.Pack(await Read(context(raw))),
      intervalSetting: IntervalSetting, windowSetting: WindowSetting);
  public string WindowSetting
  {
    get;
  }
  public Stream<T> Stream
  {
    get;
  }
  public TimeSpan? Interval
  {
    get;
  }
  public string IntervalSetting
  {
    get;
  }
  public Func<Context, Task<T>> Read
  {
    get;
  }
}

/// <summary>Answers a Whole Host Window per Call With Columns That Repeat Every Field of the Row.</summary>
/// <remarks>Sample <c>i</c> is due at <c>window.Start + i * window.Period</c>; every column holds <c>window.Count</c> values.</remarks>
public sealed class WindowObserver<T, TColumns> : ISampler
    where TColumns : IMessage<TColumns>, new()
{
  public WindowObserver(Stream<T> stream, TimeSpan? interval, Func<Context, SampleWindow, Task<TColumns>> read,
      string intervalSetting = "", string windowSetting = "")
  {
    if (stream.Observer is not null)
      throw new ArgumentException($"Declare Observer {stream.Identifier}: Remove the Duplicate Observer");
    Stream = stream;
    Interval = interval;
    Read = read;
    IntervalSetting = intervalSetting;
    WindowSetting = windowSetting;
    stream.Observer = this;
  }
  void ISampler.Register(Harmos.Plugins.Plugin plugin, string stream, Func<Harmos.Plugins.Context, Context> context) =>
      GetType().GetMethod(nameof(RegisterWire), BindingFlags.Instance | BindingFlags.NonPublic)!
      .MakeGenericMethod(Payload<T>.Descriptor.ClrType).Invoke(this, [plugin, stream, context]);

  private void RegisterWire<TWire>(Harmos.Plugins.Plugin plugin, string stream, Func<Harmos.Plugins.Context, Context> context)
    where TWire : IMessage<TWire>, new() =>
    plugin.WindowObserver<TWire, TColumns>(stream, Interval is null ? null : Duration.FromTimeSpan(Interval.Value),
      (raw, window) => Read(context(raw), new SampleWindow(window.Start.ToDateTimeOffset(), window.Period.ToTimeSpan(), window.Count)),
      intervalSetting: IntervalSetting, windowSetting: WindowSetting);
  public Stream<T> Stream
  {
    get;
  }
  public TimeSpan? Interval
  {
    get;
  }
  public string IntervalSetting
  {
    get;
  }
  public string WindowSetting
  {
    get;
  }
  public Func<Context, SampleWindow, Task<TColumns>> Read
  {
    get;
  }
}

internal sealed class Template(string identifier, string label)
{
  private System.Type? author;
  private readonly Dictionary<object, string> names = new();
  private readonly Dictionary<object, string> identifiers = new();
  internal List<IRecord> Records { get; } = [];
  internal List<PromptDeclaration> Prompts { get; } = [];
  internal string OwnerSuite { get; private set; } = "";

  public static Template From<T>() where T : TestTemplate, new()
  {
    var instance = new T();
    var template = new Template(instance.Identifier, instance.Label);
    template.author = typeof(T);
    template.Definition.Metadata.Label = instance.Label;
    template.Definition.Parameters.Add(instance.Parameters.Select(value => value.Clone()));
    template.Definition.Documents.Add(instance.Documents.Select(value => value.Clone()));
    template.InitializeBody = context => ((TestTemplate)context.Instance!).SetupAsync(context, context.CancellationToken);
    template.CleanupBody = context => ((TestTemplate)context.Instance!).CleanupAsync(context, context.CancellationToken);
    foreach (var field in typeof(T).GetFields(BindingFlags.Instance | BindingFlags.Static | BindingFlags.Public))
    {
      var declaration = field.GetValue(instance);
      if (declaration is IRecord record)
        template.Records.Add(record);
      else if (declaration is PromptDeclaration prompt)
        template.Prompts.Add(prompt);
      else if (declaration is IStream stream)
        template.Streams.Add(stream);
      else
        continue;
      template.names.Add(declaration, field.Name);
    }
    foreach (var method in typeof(T).GetMethods().OrderBy(method => method.MetadataToken))
    {
      if (method.GetCustomAttribute<StepAttribute>() is { } step)
      {
        ValidateHandler(method, typeof(Task));
        var declaration = new Step(step.Label, async context =>
        {
          var arguments = method.GetParameters().Length == 2 ? new object[] { context, context.CancellationToken } : new object[] { context };
          var result = Invoke(method, context.Instance, arguments);
          if (result is Task task)
            await task;
          else if (method.ReturnType != typeof(void))
            throw new ArgumentException("Run Step: Return Task or void");
        }, step.Identifier, Policy: step.Policy()?.ToWire(), ReplaySafe: step.ReplaySafe, Repeatable: step.Repeatable);
        template.Steps.Add(declaration);
        template.names.Add(declaration, method.Name);
        template.names.Add(method, method.Name);
      }
      if (method.GetCustomAttribute<CriterionAttribute>() is { } criterion)
      {
        ValidateHandler(method, typeof(Task<bool>));
        var declaration = new Criterion(criterion.Label, async context =>
        {
          var arguments = method.GetParameters().Length == 2 ? new object[] { context, context.CancellationToken } : new object[] { context };
          var result = Invoke(method, context.Instance, arguments);
          return result is Task<bool> task ? await task : result is bool passed ? passed
              : throw new ArgumentException("Evaluate Criterion: Return bool or Task<bool>");
        }, criterion.Identifier, criterion.Policy()?.ToWire());
        template.Criteria.Add(declaration);
        template.names.Add(declaration, method.Name);
        template.names.Add(method, method.Name);
      }
    }
    return template;
  }

  private static object? Invoke(MethodInfo method, object? instance, object[] arguments)
  {
    try
    {
      return method.Invoke(instance, arguments);
    }
    catch (TargetInvocationException error) when (error.InnerException is not null)
    {
      ExceptionDispatchInfo.Capture(error.InnerException).Throw();
      throw;
    }
  }

  private static void ValidateHandler(MethodInfo method, System.Type result)
  {
    var parameters = method.GetParameters();
    if (method.IsStatic || method.ReturnType != result || parameters.Length is < 1 or > 2 || parameters[0].ParameterType != typeof(Context)
      || parameters.Length == 2 && parameters[1].ParameterType != typeof(CancellationToken))
      throw new ArgumentException($"Declare Template Member {method.Name}: Use an Instance Task Handler Taking Context and an Optional CancellationToken");
  }

  internal object? CreateInstance() => author is null ? null : Activator.CreateInstance(author);

  public string Resolve(object declaration)
  {
    if (declaration is Delegate handler)
      declaration = handler.Method;
    return identifiers.TryGetValue(declaration, out var identity) ? identity
        : throw new ArgumentException("Resolve Declaration: Supply a Member of This Template");
  }

  internal string Resolve(object declaration, object? instance)
  {
    if (identifiers.ContainsKey(declaration) || declaration is Delegate)
      return Resolve(declaration);
    var field = author?.GetFields(BindingFlags.Instance | BindingFlags.Public)
        .FirstOrDefault(field => ReferenceEquals(field.GetValue(instance), declaration));
    var prototype = field is null ? null : names.FirstOrDefault(item => item.Value == field.Name).Key;
    return prototype is null ? Resolve(declaration) : Resolve(prototype);
  }

  internal Template Bind(string suite)
  {
    var bound = new Template(Identifiers.Build("", Definition.Metadata.Identifier, suite), Definition.Metadata.Label)
    {
      author = author,
      OwnerSuite = suite,
      InitializeBody = InitializeBody,
      CleanupBody = CleanupBody,
      MaterializePolicy = MaterializePolicy,
      InitializePolicy = InitializePolicy,
      CleanupPolicy = CleanupPolicy
    };
    bound.Definition.Parameters.Add(Definition.Parameters.Select(item => item.Clone()));
    bound.Definition.Documents.Add(Definition.Documents.Select(item => item.Clone()));
    bound.Steps.AddRange(Steps);
    bound.Criteria.AddRange(Criteria);
    bound.Streams.AddRange(Streams);
    bound.Records.AddRange(Records);
    bound.Prompts.AddRange(Prompts);
    foreach (var item in names)
      bound.names.Add(item.Key, item.Value);
    var unique = new HashSet<string>(StringComparer.Ordinal);
    foreach (var declaration in Steps.Cast<object>().Concat(Criteria).Concat(Streams).Concat(Records).Concat(Prompts))
    {
      var (explicitId, codeName) = declaration switch
      {
        Step step => (step.Identifier, step.Run.Method.Name),
        Criterion criterion => (criterion.Identifier, criterion.Evaluate.Method.Name),
        IStream stream => (stream.Identifier, ""),
        IRecord record => (record.Identifier, ""),
        PromptDeclaration prompt => (prompt.Identifier, ""),
        _ => throw new ArgumentException("Bind Template: Supply a Typed Declaration")
      };
      var identity = Identifiers.Build(names.GetValueOrDefault(declaration, codeName), explicitId, bound.Definition.Metadata.Identifier);
      if (!unique.Add(identity))
        throw new ArgumentException($"Build Plugin: Rename Duplicate Declaration {identity}");
      bound.identifiers.Add(declaration, identity);
      if (declaration is Step stepDeclaration)
        bound.identifiers.TryAdd(stepDeclaration.Run.Method, identity);
      if (declaration is Criterion criterionDeclaration)
        bound.identifiers.TryAdd(criterionDeclaration.Evaluate.Method, identity);
      if (names.TryGetValue(declaration, out var member) && author?.GetMethod(member) is { } method)
        bound.identifiers.Add(method, identity);
    }
    return bound;
  }

  public TestCaseTemplate Definition
  {
    get;
  } = new()
  {
    Metadata = new Metadata { Identifier = identifier, Label = label }
  };
  internal List<Step> Steps
  {
    get;
  } = [];
  internal List<Criterion> Criteria
  {
    get;
  } = [];
  internal List<IStream> Streams
  {
    get;
  } = [];
  internal Func<Context, Task>? InitializeBody
  {
    get;
    private set;
  }
  internal Func<Context, Task>? CleanupBody
  {
    get;
    private set;
  }
  internal OperationPolicy? MaterializePolicy
  {
    get;
    private set;
  }
  internal OperationPolicy? InitializePolicy
  {
    get;
    private set;
  }
  internal OperationPolicy? CleanupPolicy
  {
    get;
    private set;
  }
}
