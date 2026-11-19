// SPDX-License-Identifier: Apache-2.0

using System.Runtime.CompilerServices;
using System.Text.RegularExpressions;
using Dokime.Clients.EvidenceResources;
using Dokime.Plugins;
using Google.Protobuf;
using Google.Protobuf.Reflection;
using Google.Protobuf.WellKnownTypes;
using Harmos;

namespace Dokime;

public sealed class RecordsReader<T>
{
  private readonly Execution execution;
  private readonly string identifier;
  private readonly string step;
  private ReadCut? at;
  internal RecordsReader(Execution execution, string identifier, string step)
  {
    ArgumentException.ThrowIfNullOrWhiteSpace(identifier);
    this.execution = execution;
    this.identifier = identifier;
    this.step = step;
  }
  private async Task<RecordsListResponse> PageAsync(PageRequest page, CancellationToken cancellationToken)
  {
    await execution.PrepareAsync<T>(identifier, false, cancellationToken);
    var reply = await execution.Client.Transport.Evidence.Records.List(identifier, [], new Window(), step, new CapturedRecordSelection(),
      new RecordScope { Execution = execution.Identifier }, at, page, execution.Client.Options(cancellationToken));
    at = Paging.Cut(at, reply.ResolvedAt);
    return reply;
  }
  private RecordRow<T> Decode(CapturedRecord value)
  {
    if (value.RecordType != identifier || value.Scope?.ExecutionId != execution.Identifier)
      throw new InvalidDataException("Read Records: Select Records from the Bound Execution and Record Type");
    return new RecordRow<T>(value.RecordId, value.Sequence, EvidenceTime.Nanoseconds(value.Timestamp), EvidenceTime.Nanoseconds(value.ReceivedAt),
      value.RecordType, value.SchemaVersion, value.Source, Payload<T>.Unpack(value.Payload));
  }
  public async IAsyncEnumerable<IReadOnlyList<RecordRow<T>>> PagesAsync(int pageSize = 0, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var page = Paging.Page(pageSize);
    HashSet<string> tokens = [];
    do
    {
      var reply = await PageAsync(page, cancellationToken);
      yield return reply.Records.Select(Decode).ToArray();
      page.PageToken = Paging.Token(tokens, reply.NextPageToken);
    } while (page.PageToken.Length > 0);
  }
  public async Task<IReadOnlyList<RecordRow<T>>> ReadAsync(int limit = 0, CancellationToken cancellationToken = default)
  {
    if (limit == 0)
      return (await PageAsync(new PageRequest(), cancellationToken)).Records.Select(Decode).ToArray();
    _ = Paging.Page(limit);
    var rows = new List<RecordRow<T>>();
    await foreach (var page in PagesAsync(limit, cancellationToken))
    {
      rows.AddRange(page.Take(limit - rows.Count));
      if (rows.Count == limit)
        break;
    }
    return rows;
  }
  public async Task<ulong> CountAsync(CancellationToken cancellationToken = default) =>
    (await PageAsync(new PageRequest { PageSize = 1 }, cancellationToken)).Count;
  public async Task<T> LastAsync(CancellationToken cancellationToken = default)
  {
    var reply = await PageAsync(new PageRequest { PageSize = 1 }, cancellationToken);
    if (reply.Last is null)
      throw new InvalidOperationException("Read Last Record: Capture a Record Before Reading It");
    return Decode(reply.Last).Value;
  }
}

public sealed class SamplesReader<T>
{
  private readonly Execution execution;
  private readonly string identifier;
  private readonly IReadOnlyList<string> fields;
  private readonly Window between;
  private ReadCut? at;
  internal SamplesReader(Execution execution, string identifier, IReadOnlyList<string>? fields, DateTimeOffset? start, DateTimeOffset? end)
  {
    ArgumentException.ThrowIfNullOrWhiteSpace(identifier);
    this.execution = execution;
    this.identifier = identifier;
    this.fields = fields?.ToArray() ?? [];
    between = new Window
    {
      Start = start is null ? null : Timestamp.FromDateTimeOffset(start.Value),
      End = end is null ? null : Timestamp.FromDateTimeOffset(end.Value)
    };
  }
  public async IAsyncEnumerable<IReadOnlyList<SampleRow<T>>> PagesAsync(int pageSize = 0, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await execution.PrepareAsync<T>(identifier, true, cancellationToken);
    var page = Paging.Page(pageSize);
    HashSet<string> tokens = [];
    do
    {
      var reply = await execution.Client.Transport.Evidence.Samples.List(new EvidenceScope { ExecutionId = execution.Identifier }, [identifier], [],
        fields, [], [], between, new BucketRequest(), false, at, page, execution.Client.Options(cancellationToken));
      at = Paging.Cut(at, reply.ResolvedAt);
      var rows = new List<SampleRow<T>>();
      foreach (var value in reply.Samples)
      {
        var raw = value.Raw;
        if (raw is null || raw.Stream != identifier || raw.Scope?.ExecutionId != execution.Identifier)
          throw new InvalidDataException("Read Samples: Select Raw Samples from the Bound Execution and Stream");
        rows.Add(new SampleRow<T>(raw.SampleId, raw.Sequence, EvidenceTime.Nanoseconds(raw.Timestamp), EvidenceTime.Nanoseconds(raw.ReceivedAt), raw.Stream,
          raw.SchemaVersion, raw.Source, Payload<T>.Unpack(raw.Payload), raw.Values.Values.ToDictionary(item => item.Key, item => Scalars.Decode(item.Value))));
      }
      yield return rows;
      page.PageToken = Paging.Token(tokens, reply.NextPageToken);
    } while (page.PageToken.Length > 0);
  }
  public async Task<IReadOnlyList<SampleRow<T>>> CollectAsync(int maxRows = 100_000, CancellationToken cancellationToken = default)
  {
    if (maxRows <= 0)
      throw new ArgumentException("Read Samples: Supply a Positive Row Bound");
    var rows = new List<SampleRow<T>>();
    await foreach (var page in PagesAsync(cancellationToken: cancellationToken))
    {
      rows.AddRange(page);
      if (rows.Count > maxRows)
        throw new InvalidOperationException("Read Samples: Narrow the Selection or Increase the Explicit Row Bound");
    }
    return rows;
  }
}

internal static class SchemaBinding
{
  internal static void Validate<T>(FileDescriptorSet files, string name)
  {
    var declared = Payload<T>.Descriptor;
    var frozen = Find(files, name);
    if (declared.FullName != name || frozen is null || !Normalize(frozen).Equals(Normalize(declared.ToProto())))
      throw new InvalidDataException("Decode Evidence: Use the Payload Type Matching the Frozen Message Binding");
  }
  private static DescriptorProto? Find(FileDescriptorSet files, string name)
  {
    var pending = files.File.SelectMany(file => file.MessageType.Select(message => (Name: $"{file.Package}.{message.Name}", Message: message))).ToList();
    for (var index = 0; index < pending.Count; index++)
    {
      var item = pending[index];
      if (item.Name == name)
        return item.Message;
      pending.AddRange(item.Message.NestedType.Select(message => ($"{item.Name}.{message.Name}", message)));
    }
    return null;
  }
  private static DescriptorProto Normalize(DescriptorProto input)
  {
    var value = input.Clone();
    var pending = new List<DescriptorProto> { value };
    for (var index = 0; index < pending.Count; index++)
    {
      foreach (var field in pending[index].Field)
        if (field.JsonName.Length == 0)
          field.JsonName = Regex.Replace(field.Name, "_([a-z])", match => match.Groups[1].Value.ToUpperInvariant());
      pending.AddRange(pending[index].NestedType);
    }
    return value;
  }
}
