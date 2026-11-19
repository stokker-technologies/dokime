// SPDX-License-Identifier: Apache-2.0

using System.Reflection;
using Google.Protobuf;
using Dokime.Plugins;

namespace Dokime.Operations;

public enum Visibility
{
  Private,
  Protected,
  Public
}

/// <summary>A Plain or Structured Operation Using the SDK's Shared Payload Conversion.</summary>
public sealed class Operation<TRequest, TResponse> : IOperationReference
{
  private readonly string identifier;
  private readonly Visibility visibility;
  private readonly OperationPolicy? policy;
  private readonly bool replaySafe;
  private readonly Lane? lane;
  public Operation(string identifier, Visibility visibility = Visibility.Public, OperationPolicy? policy = null, bool replaySafe = false, Lane? lane = null)
  {
    ArgumentException.ThrowIfNullOrWhiteSpace(identifier);
    this.identifier = identifier;
    this.visibility = visibility;
    this.policy = policy;
    this.replaySafe = replaySafe;
    this.lane = lane;
    _ = policy?.ToWire();
  }

  internal void Register(Harmos.Plugins.Registrar registrar, Func<Harmos.Plugins.Context, Context> context,
      Func<Context, TRequest, Task<TResponse>> handler)
  {
    GetType().GetMethod(nameof(RegisterWire), BindingFlags.Instance | BindingFlags.NonPublic)!
        .MakeGenericMethod(Payload<TRequest>.Descriptor.ClrType, Payload<TResponse>.Descriptor.ClrType)
        .Invoke(this, [registrar, context, handler]);
  }

  private Harmos.Plugins.Operation<TWireRequest, TWireResponse> Wire<TWireRequest, TWireResponse>()
      where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new()
  {
    var audience = visibility switch
    {
      Visibility.Private => Harmos.Visibility.Private,
      Visibility.Protected => Harmos.Visibility.Protected,
      Visibility.Public => Harmos.Visibility.Public,
      _ => throw new ArgumentException("Declare Operation: Choose a Visibility")
    };
    return new Harmos.Plugins.Operation<TWireRequest, TWireResponse>(identifier, audience, policy?.ToWire(), replaySafe: replaySafe, lane: lane?.Native);
  }

  private void RegisterWire<TWireRequest, TWireResponse>(Harmos.Plugins.Registrar registrar,
      Func<Harmos.Plugins.Context, Context> context, Func<Context, TRequest, Task<TResponse>> handler)
      where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new()
  {
    registrar.Register(Wire<TWireRequest, TWireResponse>(), async (raw, request) =>
    {
      try
      {
        var value = await handler(context(raw), Payload<TRequest>.Unpack(request.ToByteString()));
        return (TWireResponse)Payload<TResponse>.Pack(value);
      }
      catch (SdkError error)
      {
        throw new Harmos.PluginFailureException(error.Failure.ToWire());
      }
    });
  }

  internal Task<TResponse> CallAsync(Context context, TRequest request, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return (Task<TResponse>)GetType().GetMethod(nameof(CallWireAsync), BindingFlags.Instance | BindingFlags.NonPublic)!
        .MakeGenericMethod(Payload<TRequest>.Descriptor.ClrType, Payload<TResponse>.Descriptor.ClrType)
        .Invoke(this, [context, request, policy, cancellationToken])!;
  }

  void IOperationReference.Declare(Harmos.Plugins.Registrar registrar)
  {
    GetType().GetMethod(nameof(DeclareWire), BindingFlags.Instance | BindingFlags.NonPublic)!
      .MakeGenericMethod(Payload<TRequest>.Descriptor.ClrType, Payload<TResponse>.Descriptor.ClrType).Invoke(this, [registrar]);
  }

  private void DeclareWire<TWireRequest, TWireResponse>(Harmos.Plugins.Registrar registrar)
    where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new()
  {
    if (visibility != Visibility.Private)
      registrar.Register(Wire<TWireRequest, TWireResponse>());
  }

  internal async Task<TResponse> CallAsync(Dokime.Client client, TRequest request, CancellationToken cancellationToken)
  {
    var operation = (Harmos.OperationContract)GetType().GetMethod(nameof(Contract), BindingFlags.Instance | BindingFlags.NonPublic)!
      .MakeGenericMethod(Payload<TRequest>.Descriptor.ClrType, Payload<TResponse>.Descriptor.ClrType).Invoke(this, null)!;
    var observed = await client.Transport.Operations.Get(identifier, client.Options(cancellationToken));
    if (observed.Contract.Visibility != Harmos.Visibility.Public || observed.Contract.Fingerprint != operation.Fingerprint)
      throw new InvalidDataException("Call Operation: Select the Matching Public Operation Contract");
    var reply = await client.Transport.Invocations.Create(identifier, operation.Fingerprint, Payload<TRequest>.Pack(request).ToByteString(), client.Options(cancellationToken));
    return Payload<TResponse>.Unpack(reply);
  }

  private Harmos.OperationContract Contract<TWireRequest, TWireResponse>()
    where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new() => Wire<TWireRequest, TWireResponse>().Contract;

  private async Task<TResponse> CallWireAsync<TWireRequest, TWireResponse>(Context context, TRequest request, OperationPolicy? policy, CancellationToken cancellationToken)
      where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new()
  {
    var reply = await context.Carrier.CallAsync(Wire<TWireRequest, TWireResponse>(), (TWireRequest)Payload<TRequest>.Pack(request), policy?.ToWire(), cancellationToken);
    return Payload<TResponse>.Unpack(reply.ToByteString());
  }
}

internal interface IOperationReference
{
  void Declare(Harmos.Plugins.Registrar registrar);
}
