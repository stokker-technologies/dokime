// SPDX-License-Identifier: Apache-2.0

using Grpc.Core;
using Grpc.Net.Client;

namespace Dokime;

/// <summary>Owns a gRPC channel and creates generated clients over the same transport.</summary>
public sealed class Client : IAsyncDisposable
{
  private readonly GrpcChannel? channel;
  public Client(CallInvoker calls) => Calls = calls;
  private Client(GrpcChannel channel) { this.channel = channel; Calls = channel.CreateCallInvoker(); }
  public CallInvoker Calls { get; }
  public Harmos.Sdk.AccessResource Access => new(Calls);
  public Harmos.Sdk.StatusResource Status => new(Calls);
  public Harmos.Sdk.StorageResource Storage => new(Calls);
  public static Client Connect(string address, GrpcChannelOptions? options = null)
      => new(GrpcChannel.ForAddress(address, options ?? new GrpcChannelOptions()));
  public T Service<T>(Func<CallInvoker, T> create) => create(Calls);
  public ValueTask DisposeAsync() { channel?.Dispose(); return ValueTask.CompletedTask; }
}
