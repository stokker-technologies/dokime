/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Reflection;
using System.Text.Json.Nodes;
using Dokime.Plugins;
using Google.Protobuf.WellKnownTypes;

namespace Dokime.Operations;

[AttributeUsage(AttributeTargets.Method)]
public sealed class OperationAttribute(string identifier) : PolicyAttribute
{
  public string Identifier { get; } = identifier;
  public Visibility Visibility { get; set; } = Visibility.Public;
}

public sealed record SecretDeclaration(string Identifier, string Label, bool Required = true);
public sealed record SettingDeclaration(string Identifier, SettingValue Value, string Label = "", string Description = "");

public sealed record SettingValue
{
  public bool? Flag
  {
    get; init;
  }
  public long? Integer
  {
    get; init;
  }
  public double? Number
  {
    get; init;
  }
  public string? Text
  {
    get; init;
  }
  public TimeSpan? Duration
  {
    get; init;
  }

  internal Harmos.SettingValue ToWire()
  {
    var supplied = new object?[] { Flag, Integer, Number, Text, Duration }.Count(value => value is not null);
    if (supplied != 1)
      throw new ArgumentException("Declare Setting: Supply Exactly One Typed Value");
    if (Flag is { } flag)
      return new Harmos.SettingValue { Flag = flag };
    if (Integer is { } integer)
      return new Harmos.SettingValue { Integer = integer };
    if (Number is { } number)
      return new Harmos.SettingValue { Number = number };
    if (Text is { } text)
      return new Harmos.SettingValue { Text = text };
    return new Harmos.SettingValue { Duration = Google.Protobuf.WellKnownTypes.Duration.FromTimeSpan(Duration!.Value) };
  }

  internal static SettingValue FromWire(Harmos.SettingValue value) => value.ValueCase switch
  {
    Harmos.SettingValue.ValueOneofCase.Flag => new() { Flag = value.Flag },
    Harmos.SettingValue.ValueOneofCase.Integer => new() { Integer = value.Integer },
    Harmos.SettingValue.ValueOneofCase.Number => new() { Number = value.Number },
    Harmos.SettingValue.ValueOneofCase.Text => new() { Text = value.Text },
    Harmos.SettingValue.ValueOneofCase.Duration => new() { Duration = value.Duration.ToTimeSpan() },
    _ => throw new InvalidDataException("Read Setting: Select a Typed Saved Value")
  };
}

public class Context
{
  internal Context(Dokime.Plugin plugin, Harmos.Plugins.Context transport)
  {
    Plugin = plugin;
    Transport = transport;
    Carrier = new Plugins.Context(transport);
  }

  public Dokime.Plugin Plugin
  {
    get;
  }
  public uint Attempt => Transport.Attempt;
  internal Harmos.Plugins.Context Transport
  {
    get;
  }
  internal Plugins.Context Carrier
  {
    get;
  }
  internal HostRoot Host => Carrier.Host;
  public CancellationToken CancellationToken => Transport.CancellationToken;
  public Task<string> CancelledAsync(CancellationToken cancellationToken = default) => Transport.CancelledAsync().WaitAsync(cancellationToken);
  public async Task<DateTimeOffset> NowAsync(CancellationToken cancellationToken = default) =>
    (await Host.Time.Get(cancellationToken: cancellationToken)).ToDateTimeOffset();
  public async Task SleepAsync(TimeSpan duration, CancellationToken cancellationToken = default)
  {
    await Host.Time.Sleep(Duration.FromTimeSpan(duration), cancellationToken: cancellationToken);
  }
  public async Task<byte[]> SecretAsync(string identifier, CancellationToken cancellationToken = default)
  {
    if (!Plugin.Secrets.Any(secret => secret.Identifier == identifier))
      throw new ArgumentException("Read Secret: Declare the Secret on This Plugin");
    var reply = await Host.Secrets.Get(identifier, cancellationToken: cancellationToken);
    return reply.Secret.ToByteArray();
  }
  public async Task<IReadOnlyDictionary<string, SettingValue>> SettingsAsync(CancellationToken cancellationToken = default)
  {
    var reply = await Host.Plugins.Settings.List(Plugin.Identifier, cancellationToken: cancellationToken);
    return reply.Value.Values.ToDictionary(item => item.Key, item => SettingValue.FromWire(item.Value));
  }
  public BoundOperation<TRequest, TResponse> Operation<TRequest, TResponse>(Operation<TRequest, TResponse> operation) => new(this, operation);
  public Task<JsonNode?> Operation(string identifier, object? payload = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default) =>
    Transport.Operation(identifier, payload, policy?.ToWire(), cancellationToken);
  public Task<StreamSubscription<T>> SubscribeAsync<T>(Dokime.Templates.Stream<T> stream, CancellationToken cancellationToken = default)
    =>
    throw new InvalidOperationException("Subscribe to Samples: Upgrade the Library to Provide Scoped Sample Demand, Cancellation and Subscription Close");
  internal Task<TResponse> CallAsync<TRequest, TResponse>(Harmos.Plugins.Operation<TRequest, TResponse> operation, TRequest request)
    where TRequest : Google.Protobuf.IMessage<TRequest>, new() where TResponse : Google.Protobuf.IMessage<TResponse>, new() => Transport.CallAsync(operation, request);
}

public interface StreamSubscription<T> : IAsyncEnumerable<SampleRow<T>>, IAsyncDisposable;

public sealed class BoundOperation<TRequest, TResponse>
{
  private readonly Context? context;
  private readonly Dokime.Client? client;
  private readonly Operation<TRequest, TResponse> operation;
  internal BoundOperation(Context context, Operation<TRequest, TResponse> operation)
  {
    this.context = context;
    this.operation = operation;
  }
  internal BoundOperation(Dokime.Client client, Operation<TRequest, TResponse> operation)
  {
    this.client = client;
    this.operation = operation;
  }
  public Task<TResponse> CallAsync(TRequest request, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    if (client is null)
      return operation.CallAsync(context!, request, policy, cancellationToken);
    if (policy is not null)
      throw new InvalidOperationException("Call Client Operation: Publish the Host Invocation Policy Override Contract and Regenerate Bindings");
    return operation.CallAsync(client, request, cancellationToken);
  }
}

internal static class OperationRegistration
{
  internal static void Register(Plugins.Plugin plugin, Delegate handler)
  {
    var method = handler.Method;
    var attribute = method.GetCustomAttribute<OperationAttribute>();
    var parameters = method.GetParameters();
    var valid = method.IsStatic && attribute is not null && parameters.Length is 2 or 3 && parameters[0].ParameterType == typeof(Context)
      && method.ReturnType.IsGenericType && method.ReturnType.GetGenericTypeDefinition() == typeof(Task<>);
    if (!valid || parameters.Length == 3 && parameters[2].ParameterType != typeof(CancellationToken))
      throw new ArgumentException("Declare Operation: List an Annotated Static Task<T> Handler Taking Context, Request and Optional CancellationToken");
    typeof(OperationRegistration).GetMethod(nameof(RegisterTyped), BindingFlags.Static | BindingFlags.NonPublic)!
      .MakeGenericMethod(parameters[1].ParameterType, method.ReturnType.GenericTypeArguments[0]).Invoke(null, [plugin, handler, attribute]);
  }

  private static void RegisterTyped<TRequest, TResponse>(Plugins.Plugin plugin, Delegate handler, OperationAttribute attribute)
  {
    var operation = new Operation<TRequest, TResponse>(attribute.Identifier, attribute.Visibility, attribute.Policy(), attribute.ReplaySafe);
    if (handler.Method.GetParameters().Length == 3)
    {
      var body = handler.Method.CreateDelegate<Func<Context, TRequest, CancellationToken, Task<TResponse>>>();
      plugin.Operation(operation, (context, request) => body(context, request, context.CancellationToken));
      return;
    }
    var unary = handler.Method.CreateDelegate<Func<Context, TRequest, Task<TResponse>>>();
    plugin.Operation(operation, unary);
  }
}
