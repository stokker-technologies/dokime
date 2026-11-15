// SPDX-License-Identifier: Apache-2.0

using Dokime.Clients.EvidenceResources;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using Harmos.Plugins;
using Value = Dokime.Value;
using TransportContext = Harmos.Plugins.Context;

namespace Dokime.Plugins;

/// <summary>One scoped plugin invocation or host tick, with execution-owned resources.</summary>
public sealed class Context
{
  private readonly object? resource;

  internal Context(TransportContext transport, TestCase testCase, object? resource = null)
  {
    Transport = transport;
    TestCase = testCase.Clone();
    this.resource = resource;
  }

  public TransportContext Transport
  {
    get;
  }
  public HostRoot Host => new(this);
  public string Execution => Transport.Scope();
  public TestCase TestCase
  {
    get;
  }
  public IReadOnlyDictionary<string, Value> Parameters => TestCase.Parameters;
  public CancellationToken CancellationToken => Transport.CancellationToken;
  public Timestamp? At => Transport.At?.Clone();
  public ulong Sequence => Transport.Sequence;

  public T Resource<T>() where T : class => resource as T
      ?? throw new InvalidOperationException("Read Execution Resource: No Resource of the Requested Type Is Active");

  public Task<TResponse> CallAsync<TRequest, TResponse>(Operation<TRequest, TResponse> operation, TRequest request,
      OperationPolicy? policy = null, CancellationToken cancellationToken = default)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => Transport.CallAsync(operation, request, policy, cancellationToken);

  public IAsyncEnumerable<TResponse> WatchAsync<TRequest, TResponse>(Operation<TRequest, TResponse> operation,
      TRequest request, ulong maxItemBytes, CancellationToken cancellationToken = default)
      where TRequest : IMessage<TRequest>, new() where TResponse : IMessage<TResponse>, new()
      => Transport.WatchAsync(operation, request, maxItemBytes, cancellationToken);

  public Task<RecordsAppendResponse> RecordAsync<T>(Record<T> record, T value, Timestamp sourceAt,
      string? requestId = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
      where T : IMessage<T>, new()
      => Host.Evidence.Records.Append(Execution, record.Identifier, value.ToByteString(), sourceAt,
          requestId, policy, cancellationToken);

  public async Task<Receipt> LogAsync(Severity severity, string message, string? requestId = null,
      OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var at = At ?? await Host.Time.Get(policy, cancellationToken);
    return await Host.Evidence.Logs.Append(Execution, severity, message, at, requestId, policy, cancellationToken);
  }
}
