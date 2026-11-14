// SPDX-License-Identifier: Apache-2.0

using Google.Protobuf;
using Grpc.Core;
using System.Security.Cryptography;
using Dokime.Protocol.Client.V1.FilesResources;

namespace Dokime;

public static class FileTransfers
{
  public static async Task<Harmos.Storage.V1.FileReference> UploadAsync(this Sdk.FilesResource files, string path, CallOptions options = default)
  {
    await using var source = File.OpenRead(path);
    var requestId = Guid.NewGuid().ToString("N");
    using var opening = files.Uploads.Create(new UploadsCreateRequest { Metadata = new Harmos.Storage.V1.FileMetadata { ExpectedLength = (ulong)source.Length }, RequestId = $"{requestId}/create" }, options);
    var created = await opening.ResponseAsync.ConfigureAwait(false);
    var session = created.Production?.Session ?? throw new InvalidDataException("Upload File: Missing Writer Session");
    try
    {
      if (created.ChunkBytes == 0 || created.ChunkBytes >= created.MessageBytes || created.ChunkBytes > int.MaxValue) throw new InvalidDataException("Upload File: Invalid Transfer Limits");
      var buffer = new byte[(int)created.ChunkBytes];
      using var digest = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
      ulong offset = 0;
      while (true)
      {
        var size = await source.ReadAsync(buffer, options.CancellationToken).ConfigureAwait(false);
        if (size == 0) break;
        using var append = files.Uploads.Append(new UploadsAppendRequest { Session = session, Offset = offset, Data = ByteString.CopyFrom(buffer, 0, size), RequestId = $"{requestId}/append/{offset}" }, options);
        var reply = await append.ResponseAsync.ConfigureAwait(false);
        offset += (ulong)size;
        if (reply.Production?.Length != offset) throw new InvalidDataException("Upload File: Invalid Accepted Offset");
        digest.AppendData(buffer, 0, size);
      }
      var hash = ByteString.CopyFrom(digest.GetHashAndReset());
      using var publish = files.Uploads.Publish(new UploadsPublishRequest { Session = session, Length = offset, Sha256 = hash, RequestId = $"{requestId}/publish" }, options);
      var published = (await publish.ResponseAsync.ConfigureAwait(false)).File ?? throw new InvalidDataException("Upload File: Missing Published Reference");
      if (published.Length != offset || published.Sha256 != hash) throw new InvalidDataException("Upload File: Published Length or SHA-256 Changed");
      return published;
    }
    catch
    {
      using var abandon = files.Uploads.Delete(new UploadsDeleteRequest { Session = session, RequestId = $"{requestId}/abandon" }, options);
      await abandon.ResponseAsync.ConfigureAwait(false);
      throw;
    }
  }
}
