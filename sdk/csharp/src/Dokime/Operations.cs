/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using System.Reflection;
using Google.Protobuf;
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
  public async Task<ulong> RecordAsync<T>(Dokime.Templates.Record<T> record, T value, CancellationToken cancellationToken = default)
  {
    var identity = Plugin.Adapter.RecordIdentity(record)
      ?? throw new ArgumentException("Write Plugin Record: Declare the Record on This Plugin");
    var at = await new Harmos.Plugins.HostRoot(Transport).Time.Get(cancellationToken: cancellationToken);
    var reply = await Host.Evidence.Records.Append("", identity, Payload<T>.Pack(value).ToByteString(), at, Client.Identity(null), cancellationToken: cancellationToken);
    return reply.Ordinal;
  }
  public Task<T> Resource<T>() where T : Resource, new() => Plugin.Adapter.Resource<T>(this);
  public T Settings<T>() where T : Settings => Plugin.Adapter.Settings<T>();
  public BoundOperation<TRequest, TResponse> Operation<TRequest, TResponse>(Operation<TRequest, TResponse> operation) => new(this, operation);
  public Task<JsonNode?> Operation(string identifier, object? payload = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default) =>
    Transport.Operation(identifier, payload, policy?.ToWire(), cancellationToken);
  public Task<StreamSubscription<T>> SubscribeAsync<T>(Dokime.Templates.Stream<T> stream, CancellationToken cancellationToken = default)
    =>
    throw new InvalidOperationException("Subscribe to Samples: Upgrade the Library to Provide Scoped Sample Demand, Cancellation and Subscription Close");
  internal Task<TResponse> CallAsync<TRequest, TResponse>(Harmos.Plugins.Operation<TRequest, TResponse> operation, TRequest request)
    where TRequest : Google.Protobuf.IMessage<TRequest>, new() where TResponse : Google.Protobuf.IMessage<TResponse>, new() => Transport.Operation(operation, request);
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
