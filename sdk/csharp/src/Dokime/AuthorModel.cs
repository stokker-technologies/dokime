/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Reflection;
using System.Text.RegularExpressions;
using Dokime.Templates;

namespace Dokime;

public sealed class Lane
{
  private Lane(Harmos.Plugins.Lane native) => Native = native;
  internal Harmos.Plugins.Lane Native
  {
    get;
  }
  public static Lane Serial(string name) => new(Harmos.Plugins.Lane.Serial(name));
  public static Lane Concurrent(string name, uint width) => new(Harmos.Plugins.Lane.Concurrent(name, width));
  public static Lane Unbounded(string name) => new(Harmos.Plugins.Lane.Unbounded(name));
}

public abstract class Plugin
{
  private Plugins.Plugin? adapter;
  public virtual string Identifier => Identifiers.Build(GetType().Name, type: true);
  public virtual string Label => AuthorNames.Label(GetType().Name, "Plugin");
  public virtual string Version => GetType().Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion.Split('+')[0]
    ?? throw new ArgumentException("Describe Plugin: Set the Plugin Project Version");
  public virtual Task InitializeAsync(Dokime.Operations.Context context, CancellationToken cancellationToken = default) => Task.CompletedTask;
  public virtual Task FinalizeAsync(Dokime.Operations.Context context, CancellationToken cancellationToken = default) => Task.CompletedTask;
  public virtual IReadOnlyList<Type> Suites => [];
  public virtual IReadOnlyList<Delegate> Operations => [];
  public virtual IReadOnlyList<Dokime.Operations.SettingDeclaration> Settings => [];
  public virtual IReadOnlyList<Dokime.Operations.SecretDeclaration> Secrets => [];

  internal Plugins.Plugin Adapter
  {
    get
    {
      if (adapter is not null)
        return adapter;
      var suites = new List<Plugins.SuiteBinding>();
      foreach (var type in Suites)
      {
        if (Activator.CreateInstance(type) is not Suite suite)
          throw new ArgumentException("Describe Plugin: Declare Suite Classes");
        var templates = new List<Template>();
        foreach (var template in suite.Templates)
        {
          if (!typeof(TestTemplate).IsAssignableFrom(template))
            throw new ArgumentException("Describe Suite: Declare TestTemplate Classes");
          var factory = typeof(Template).GetMethod(nameof(Template.From))!.MakeGenericMethod(template);
          templates.Add((Template)factory.Invoke(null, null)!);
        }
        suites.Add(new Plugins.SuiteBinding(new Metadata { Identifier = suite.Identifier, Label = suite.Label }, templates));
      }
      var value = new Plugins.Plugin(this, Identifier, Version, suites.ToArray());
      foreach (var setting in Settings)
        value.Setting(setting.Identifier, setting.Value.ToWire(), setting.Label, setting.Description);
      foreach (var secret in Secrets)
        value.Secret(secret.Identifier, secret.Label, secret.Required);
      foreach (var handler in Operations)
        Dokime.Operations.OperationRegistration.Register(value, handler);
      adapter = value;
      return value;
    }
  }

  public Plugin Lane(Lane lane)
  {
    Adapter.Lane(lane);
    return this;
  }

  public Plugin Operation<TRequest, TResponse>(Dokime.Operations.Operation<TRequest, TResponse> operation,
    Func<Dokime.Operations.Context, TRequest, Task<TResponse>> handler)
  {
    Adapter.Operation(operation, handler);
    return this;
  }

  public PluginDescription Describe()
  {
    var declarations = Adapter.Build().Declarations();
    return new PluginDescription(Identifier, Label, Version,
      Adapter.Suites.Select(suite => new SuiteDescription(suite.Metadata.Identifier, suite.Metadata.Label,
        suite.Templates.Select(template => new TemplateDescription(template.Definition.Metadata.Identifier,
          template.Definition.Metadata.Label, template.Steps.Select(declaration => template.Resolve(declaration)).ToArray(),
          template.Criteria.Select(declaration => template.Resolve(declaration)).ToArray(), template.Records.Select(declaration => template.Resolve(declaration)).ToArray())).ToArray())).ToArray(),
      declarations.Operations.Select(operation => new OperationDescription(operation.Id, operation.Lane)).ToArray(),
      declarations.Lanes.Select(lane => new LaneDescription(lane.Name, lane.Serial,
        lane.CapacityCase == Harmos.LaneDeclaration.CapacityOneofCase.Concurrent ? lane.Concurrent : null, lane.Unbounded)).ToArray());
  }

  public Task RunAsync(ConnectionSettings? settings = null, CancellationToken cancellationToken = default)
  {
    if (System.Environment.GetEnvironmentVariable("HARMOS_ADDRESS") is not null)
      return Adapter.ServeAsync(cancellationToken: cancellationToken);
    var values = Configuration.Load();
    var address = Configuration.Resolve("DOKIME_PLUGIN_ADDRESS", settings?.PluginAddress, values);
    return Adapter.ServeAsync(address, remote: true, cancellationToken: cancellationToken);
  }
}

public abstract class Suite
{
  public virtual string Identifier => Identifiers.Build(GetType().Name, type: true);
  public virtual string Label => AuthorNames.Label(GetType().Name, "Suite");
  public abstract IReadOnlyList<Type> Templates
  {
    get;
  }
}

public sealed record PluginDescription(string Identifier, string Label, string Version, IReadOnlyList<SuiteDescription> Suites,
  IReadOnlyList<OperationDescription> Operations, IReadOnlyList<LaneDescription> Lanes);
public sealed record OperationDescription(string Identifier, string Lane);
public sealed record LaneDescription(string Name, bool Serial, uint? Concurrent, bool Unbounded);
public sealed record SuiteDescription(string Identifier, string Label, IReadOnlyList<TemplateDescription> Templates);
public sealed record TemplateDescription(string Identifier, string Label, IReadOnlyList<string> Steps,
  IReadOnlyList<string> Criteria, IReadOnlyList<string> Records);

internal static class AuthorNames
{
  internal static string Label(string name, string suffix)
  {
    var stem = name.EndsWith(suffix, StringComparison.Ordinal) ? name[..^suffix.Length] : name;
    var words = Regex.Replace(Regex.Replace(stem, "([A-Z]+)([A-Z][a-z])", "$1 $2"), "([a-z0-9])([A-Z])", "$1 $2").Replace('_', ' ')
      .Split(' ', StringSplitOptions.RemoveEmptyEntries);
    var minor = new HashSet<string>(["a", "an", "the", "and", "but", "or", "nor", "as", "at", "by", "for", "in", "of", "on", "per", "to", "via"]);
    return string.Join(' ', words.Select((word, index) => word.All(char.IsUpper) ? word
      : index > 0 && index < words.Length - 1 && minor.Contains(word.ToLowerInvariant()) ? word.ToLowerInvariant()
      : char.ToUpperInvariant(word[0]) + word[1..].ToLowerInvariant()));
  }
}
