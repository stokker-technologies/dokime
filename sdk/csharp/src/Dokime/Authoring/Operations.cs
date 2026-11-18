// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Dokime.Plugins;

namespace Dokime.Authoring;

public enum Visibility
{
  Private,
  Protected,
  Public
}

/// <summary>A Plain or Structured Operation Using the SDK's Shared Payload Conversion.</summary>
public sealed class Operation<TRequest, TResponse>(string identifier, Visibility visibility)
{
  internal void Register(Harmos.Plugins.Plugin plugin, Func<Harmos.Plugins.Context, Context> context,
      Func<Context, TRequest, Task<TResponse>> handler)
  {
    GetType().GetMethod(nameof(RegisterWire), System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!
        .MakeGenericMethod(Payload<TRequest>.Descriptor.ClrType, Payload<TResponse>.Descriptor.ClrType)
        .Invoke(this, [plugin, context, handler]);
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
    return new Harmos.Plugins.Operation<TWireRequest, TWireResponse>(identifier, audience);
  }

  private void RegisterWire<TWireRequest, TWireResponse>(Harmos.Plugins.Plugin plugin,
      Func<Harmos.Plugins.Context, Context> context, Func<Context, TRequest, Task<TResponse>> handler)
      where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new()
  {
    plugin.Operation(Wire<TWireRequest, TWireResponse>(), async (raw, request) =>
    {
      var value = await handler(context(raw), Payload<TRequest>.Unpack(request.ToByteString()));
      return (TWireResponse)Payload<TResponse>.Pack(value);
    });
  }

  internal Task<TResponse> CallAsync(Context context, TRequest request)
  {
    return (Task<TResponse>)GetType().GetMethod(nameof(CallWireAsync), System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!
        .MakeGenericMethod(Payload<TRequest>.Descriptor.ClrType, Payload<TResponse>.Descriptor.ClrType)
        .Invoke(this, [context, request])!;
  }

  private async Task<TResponse> CallWireAsync<TWireRequest, TWireResponse>(Context context, TRequest request)
      where TWireRequest : IMessage<TWireRequest>, new() where TWireResponse : IMessage<TWireResponse>, new()
  {
    var reply = await context.CallAsync(Wire<TWireRequest, TWireResponse>(), (TWireRequest)Payload<TRequest>.Pack(request));
    return Payload<TResponse>.Unpack(reply.ToByteString());
  }
}
