// SPDX-License-Identifier: Apache-2.0

using System.Security.Cryptography;
using Dokime.Clients;
using Google.Protobuf;
using Google.Protobuf.Reflection;
using Grpc.Core;
using Harmos;

namespace Dokime;

/// <summary>Fetch and verify descriptor pages once per immutable schema digest.</summary>
public sealed class FrozenSchemaCache(PageRequest page)
{
  private readonly PageRequest limits = page.Clone();
  private readonly Dictionary<ByteString, FileDescriptorSet> schemas = [];
  private readonly SemaphoreSlim gate = new(1, 1);

  public async Task<FileDescriptorSet> GetAsync(Client client, ExecutionDetail detail, CallOptions context = default)
  {
    var digest = detail.Schemas?.Digest ?? throw new InvalidDataException("Read Frozen Schemas: Missing Digest");
    var execution = detail.Summary?.ExecutionId ?? throw new InvalidDataException("Read Frozen Schemas: Missing Execution");
    await gate.WaitAsync(context.CancellationToken).ConfigureAwait(false);
    try
    {
      if (schemas.TryGetValue(digest, out var cached))
        return cached.Clone();
      var result = new FileDescriptorSet();
      var request = limits.Clone();
      request.PageToken = "";
      Position? at = null;
      HashSet<string> tokens = [];
      do
      {
        var response = await client.Executions.Schemas.Get(execution, at, request, context).ConfigureAwait(false);
        if (response.Digest != digest || response.Types_ is null || response.ResolvedAt is null)
          throw new InvalidDataException("Read Frozen Schemas: Missing or Changed Descriptor Boundary");
        if (at is not null && !at.Equals(response.ResolvedAt))
          throw new InvalidDataException("Read Frozen Schemas: Descriptor Boundary Changed");
        at = response.ResolvedAt.Clone();
        result.File.AddRange(response.Types_.File.Select(file => file.Clone()));
        request.PageToken = response.NextPageToken;
        if (request.PageToken.Length > 0 && !tokens.Add(request.PageToken))
          throw new InvalidDataException("Read Frozen Schemas: Repeated Continuation");
      } while (request.PageToken.Length > 0);
      var actual = ByteString.CopyFrom(SHA256.HashData(result.ToByteArray()));
      if (actual != digest)
        throw new InvalidDataException("Read Frozen Schemas: SHA-256 Does Not Match the Frozen Digest");
      schemas.Add(digest, result);
      return result.Clone();
    }
    finally
    {
      gate.Release();
    }
  }
}
