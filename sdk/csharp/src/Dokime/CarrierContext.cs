// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Harmos;

namespace Dokime.Plugins;

internal sealed class Context(Harmos.Plugins.Context transport)
{
  internal Harmos.Plugins.Context Transport => transport;
  internal HostRoot Host => new(this);
  internal async Task<TResponse> CallAsync<TRequest, TResponse>(Harmos.Plugins.Operation<TRequest, TResponse> operation, TRequest request,
    OperationPolicy? policy = null, CancellationToken cancellationToken = default)
    where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
  {
    try
    {
      return await transport.CallAsync(operation, request, policy, cancellationToken);
    }
    catch (Harmos.PluginFailureException error)
    {
      throw new SdkError(Failure.FromWire(error.Failure), cause: error);
    }
  }
  internal IAsyncEnumerable<TResponse> WatchAsync<TRequest, TResponse>(Harmos.Plugins.Operation<TRequest, TResponse> operation, TRequest request,
    ulong maxItemBytes, CancellationToken cancellationToken = default)
    where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new() =>
    transport.WatchAsync(operation, request, maxItemBytes, cancellationToken);
}
