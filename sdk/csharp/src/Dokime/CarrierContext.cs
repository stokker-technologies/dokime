// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Harmos;

namespace Dokime.Plugins;

internal sealed class Context(Harmos.Plugins.Context transport)
{
  internal Harmos.Plugins.Context Transport => transport;
  internal HostRoot Host => new(this);
  internal Task<TResponse> Operation<TRequest, TResponse>(Harmos.Plugins.Operation<TRequest, TResponse> operation, TRequest request,
    OperationPolicy? policy = null, CancellationToken cancellationToken = default)
    where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
  {
    return transport.Operation(operation, request, policy, cancellationToken);
  }
  internal IAsyncEnumerable<TResponse> WatchAsync<TRequest, TResponse>(Harmos.Plugins.Operation<TRequest, TResponse> operation, TRequest request,
    ulong maxItemBytes, CancellationToken cancellationToken = default)
    where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new() =>
    transport.WatchAsync(operation, request, maxItemBytes, cancellationToken);
}
