// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Google.Protobuf.Reflection;
using Grpc.Core;
using System.Security.Cryptography;
using Grpc.Net.Client;

namespace Dokime;

/// <summary>Owns a gRPC channel and creates generated clients over the same transport.</summary>
public sealed partial class Client : IAsyncDisposable
{
  private readonly GrpcChannel? channel;
  public Client(CallInvoker calls) => Calls = calls;
  private Client(GrpcChannel channel) { this.channel = channel; Calls = channel.CreateCallInvoker(); }
  public CallInvoker Calls { get; }
  public static Client Connect(string address, GrpcChannelOptions? options = null)
      => new(GrpcChannel.ForAddress(address, options ?? new GrpcChannelOptions()));
  public T Service<T>(Func<CallInvoker, T> create) => create(Calls);
  public ValueTask DisposeAsync() { channel?.Dispose(); return ValueTask.CompletedTask; }
}

/// <summary>Caller-owned cache of complete frozen descriptors, shared by digest across executions.</summary>
public sealed class FrozenSchemaCache
{
  private readonly Dictionary<ByteString, FileDescriptorSet> sets = new();
  private readonly SemaphoreSlim gate = new(1, 1);

  public static string MessageName(Protocol.Client.V1.ExecutionDetail detail, string identity)
  {
    var schemas = detail.Schemas ?? throw new ArgumentException("Decode Evidence: Supply Frozen Bindings");
    var name = schemas.Records.FirstOrDefault(record => record.RecordType == identity)?.MessageName
      ?? schemas.Streams.FirstOrDefault(stream => stream.Stream == identity)?.MessageName;
    return !string.IsNullOrEmpty(name) ? name : throw new InvalidDataException("Decode Evidence: Missing Frozen Binding");
  }

  public async Task<FileDescriptorSet> GetAsync(Client client, Protocol.Client.V1.ExecutionDetail detail, CallOptions options = default)
  {
    var digest = detail.Schemas?.Digest ?? throw new ArgumentException("Read Schemas: Supply Frozen Schema Identities");
    if (digest.Length != 32) throw new ArgumentException("Read Schemas: Supply a SHA-256 Digest");
    await gate.WaitAsync(options.CancellationToken).ConfigureAwait(false);
    try
    {
      if (sets.TryGetValue(digest, out var cached)) return cached.Clone();
      var request = new Protocol.Client.V1.ExecutionsResources.SchemasGetRequest
      {
        Name = detail.Summary?.ExecutionId ?? throw new ArgumentException("Read Schemas: Supply an Execution Identity"),
        At = detail.ResolvedAt?.Clone(),
        Page = new Harmos.V1.PageRequest { PageSize = 1 }
      };
      var types = new FileDescriptorSet();
      var tokens = new HashSet<string>();
      while (true)
      {
        using var call = client.Executions.Schemas.Get(request, options);
        var page = await call.ResponseAsync.ConfigureAwait(false);
        if (page.Digest != digest || page.ResolvedAt is null || (request.At is not null && !page.ResolvedAt.Equals(request.At)))
          throw new InvalidDataException("Read Schemas: Descriptor Digest or Read Cut Changed");
        if (!page.Records.Equals(detail.Schemas.Records) || !page.Streams.Equals(detail.Schemas.Streams))
          throw new InvalidDataException("Read Schemas: Frozen Message Bindings Changed");
        types.File.Add(page.Types_?.File ?? throw new InvalidDataException("Read Schemas: Missing Descriptor Page"));
        if (page.NextPageToken.Length == 0) break;
        if (!tokens.Add(page.NextPageToken)) throw new InvalidDataException("Read Schemas: Repeated Descriptor Page");
        request.At = page.ResolvedAt;
        request.Page = new Harmos.V1.PageRequest { PageSize = 1, PageToken = page.NextPageToken };
      }
      if (!SHA256.HashData(types.ToByteArray()).AsSpan().SequenceEqual(digest.Span))
        throw new InvalidDataException("Read Schemas: Frozen Descriptor Digest Does Not Match");
      sets.Add(digest, types.Clone());
      return types;
    }
    finally { gate.Release(); }
  }
}
