// SPDX-License-Identifier: Apache-2.0

using System.Security.Cryptography;
using Dokime.Clients.FilesResources;
using Dokime.Plugins;
using Google.Protobuf;
using Grpc.Core;
using Harmos;
using Dokime.Transport;

namespace Dokime;

internal static class FileTransfers
{
  public static Task<FileReference> UploadAsync(this FilesResource files, string path, CallOptions context = default)
      => UploadAsync(path, context.CancellationToken,
          metadata => files.Uploads.Create(metadata, context: context),
          (upload, offset, data) => files.Uploads.Append(upload, offset, data, context: context),
          (upload, length, digest) => files.Uploads.Publish(upload, length, digest, context: context),
          upload => files.Uploads.Delete(upload, context: new CallOptions(headers: context.Headers)));

  public static Task<FileReference> UploadAsync(this HostFilesResource files, string path,
      OperationPolicy? policy = null, CancellationToken cancellationToken = default)
      => UploadAsync(path, cancellationToken,
          metadata => files.Uploads.Create(metadata, policy: policy, cancellationToken: cancellationToken),
          (upload, offset, data) => files.Uploads.Append(upload, offset, data, policy: policy, cancellationToken: cancellationToken),
          (upload, length, digest) => files.Uploads.Publish(upload, length, digest, policy: policy, cancellationToken: cancellationToken),
          upload => files.Uploads.Delete(upload));

  internal static async Task<FileReference> UploadBytesAsync(this HostFilesResource files, byte[] data, string requestId, CancellationToken cancellationToken)
  {
    using var source = new MemoryStream(data, writable: false);
    return await UploadAsync(source, cancellationToken,
      metadata => files.Uploads.Create(metadata, request_id: requestId + "/create", cancellationToken: cancellationToken),
      (upload, offset, bytes) => files.Uploads.Append(upload, offset, bytes, request_id: requestId + $"/append/{offset}", cancellationToken: cancellationToken),
      (upload, length, digest) => files.Uploads.Publish(upload, length, digest, request_id: requestId + "/publish", cancellationToken: cancellationToken),
      upload => files.Uploads.Delete(upload, request_id: requestId + "/abandon"));
  }

  private static async Task<FileReference> UploadAsync(string path, CancellationToken cancellationToken,
      Func<FileMetadata, Task<UploadsCreateResponse>> create,
      Func<Upload, ulong, ByteString, Task<UploadsAppendResponse>> append,
      Func<Upload, ulong, ByteString, Task<UploadsPublishResponse>> publish,
      Func<Upload, Task<UploadsDeleteResponse>> delete)
  {
    await using var source = System.IO.File.OpenRead(path);
    return await UploadAsync(source, cancellationToken, create, append, publish, delete);
  }

  private static async Task<FileReference> UploadAsync(System.IO.Stream source, CancellationToken cancellationToken,
    Func<FileMetadata, Task<UploadsCreateResponse>> create,
    Func<Upload, ulong, ByteString, Task<UploadsAppendResponse>> append,
    Func<Upload, ulong, ByteString, Task<UploadsPublishResponse>> publish,
    Func<Upload, Task<UploadsDeleteResponse>> delete)
  {
    var created = await create(new FileMetadata { ExpectedLength = (ulong)source.Length });
    var upload = created.Upload ?? throw new InvalidDataException("Upload File: Missing Upload");
    try
    {
      if (created.ChunkBytes == 0 || created.ChunkBytes >= created.MessageBytes || created.ChunkBytes > int.MaxValue)
        throw new InvalidDataException("Upload File: Invalid Transfer Limits");
      var buffer = new byte[(int)created.ChunkBytes];
      using var digest = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
      ulong offset = 0;
      while (true)
      {
        var size = await source.ReadAsync(buffer, cancellationToken).ConfigureAwait(false);
        if (size == 0)
          break;
        var reply = await append(upload, offset, ByteString.CopyFrom(buffer, 0, size)).ConfigureAwait(false);
        offset = checked(offset + (ulong)size);
        if (reply.Upload is not { } accepted || accepted.AcceptedLength != offset)
          throw new InvalidDataException("Upload File: Invalid Accepted Offset");
        upload = accepted;
        digest.AppendData(buffer, 0, size);
      }
      var hash = ByteString.CopyFrom(digest.GetHashAndReset());
      var result = await publish(upload, offset, hash).ConfigureAwait(false);
      var published = result.Upload ?? throw new InvalidDataException("Upload File: Missing Published Reference");
      if (published.Length != offset || published.Sha256 != hash)
        throw new InvalidDataException("Upload File: Published Length or SHA-256 Changed");
      return published;
    }
    catch (Exception failure)
    {
      try
      {
        await delete(upload).ConfigureAwait(false);
      }
      catch (Exception cleanup)
      {
        throw new AggregateException("Upload File: Transfer and Abandon Failed", failure, cleanup);
      }
      throw;
    }
  }
}
