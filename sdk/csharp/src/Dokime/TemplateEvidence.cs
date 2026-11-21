/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

using Dokime.Clients;

namespace Dokime.Templates;

public sealed partial class Context
{
  public Task<Note> NoteAsync(string text, string? requestId = null, CancellationToken cancellationToken = default) =>
    throw new InvalidOperationException("Attach Execution Note: Publish the Plugin Notes/Create Admission Contract and Regenerate Bindings");

  public async Task<Artifact> ArtifactAsync(string name, byte[] data, string mime = "application/octet-stream", string? requestId = null,
    CancellationToken cancellationToken = default)
  {
    var identity = Client.Identity(requestId);
    var file = await FileTransfers.UploadBytesAsync(Host.Files, data, identity, cancellationToken);
    var reply = await Host.Content.Artifacts.Create(file, name, mime, new ArtifactScope { ExecutionId = Execution }, identity + "/artifact", cancellationToken: cancellationToken);
    return Artifact.FromWire(reply.Artifact);
  }
  public async Task ProgressAsync(ulong completed, ulong total, string label = "", string? requestId = null, CancellationToken cancellationToken = default)
  {
    var current = await Host.Executions.Progress.Get(cancellationToken: cancellationToken);
    var value = new ExecutionProgressValue { Completed = completed, Total = total, Label = label };
    await Host.Executions.Progress.Set(value, current.Revision, request_id: Client.Identity(requestId), cancellationToken: cancellationToken);
  }
  public async Task TraceabilityAsync(string identifier, object value, string label = "", string source = "", string unit = "",
    string? requestId = null, CancellationToken cancellationToken = default)
  {
    var current = await Host.Evidence.Traceability.Get(entry: identifier, cancellationToken: cancellationToken);
    var entry = new TraceabilityEntry { Metadata = new Metadata { Identifier = identifier, Label = label }, Value = Scalars.Encode(value), Source = source, Unit = unit };
    await Host.Evidence.Traceability.Set(entry, current.Revision, identifier, request_id: Client.Identity(requestId), cancellationToken: cancellationToken);
  }
  public async Task DocumentAsync(string document, Section part, string section = "", string? requestId = null, CancellationToken cancellationToken = default)
  {
    await Host.Documents.AppendParts(Execution, document, section, subsection: part.Clone(), request_id: Client.Identity(requestId), cancellationToken: cancellationToken);
  }
  public async Task DocumentAsync(string document, Card part, string section = "", string? requestId = null, CancellationToken cancellationToken = default)
  {
    await Host.Documents.AppendParts(Execution, document, section, card: part.Clone(), request_id: Client.Identity(requestId), cancellationToken: cancellationToken);
  }
  public async Task<CaptureResult> CaptureAsync(FileSource source, string output, CancellationToken cancellationToken = default)
  {
    var reply = await Host.Files.Capture(source, output, cancellationToken: cancellationToken);
    return new CaptureResult(reply.Artifacts.Select(value => value.Clone()).ToArray(), reply.Gaps.Select(value => value.Clone()).ToArray());
  }
  public async Task<ulong> StartCaptureAsync(FileSource source, string output, CaptureMode mode, CancellationToken cancellationToken = default)
  {
    var reply = await Host.Files.Captures.Start(source, output, mode.ToWire(), cancellationToken: cancellationToken);
    return reply;
  }
  public async Task<CaptureResult> StopCaptureAsync(ulong captureId, CancellationToken cancellationToken = default)
  {
    var reply = await Host.Files.Captures.Stop(captureId, cancellationToken: cancellationToken);
    return new CaptureResult(reply.Artifacts.Select(value => value.Clone()).ToArray(), reply.Gaps.Select(value => value.Clone()).ToArray());
  }
  public async Task<Environment> EnvironmentAsync(CancellationToken cancellationToken = default)
  {
    var reply = await Host.Backend.Environment(cancellationToken: cancellationToken);
    return new Environment(reply.BackendPid, reply.InStep);
  }
}

public sealed record CaptureResult(IReadOnlyList<CapturedArtifact> Artifacts, IReadOnlyList<CaptureGap> Gaps);
public sealed record Environment(uint BackendPid, bool InStep);


public sealed record CaptureMode(StreamCapture? Stream = null, MirrorCapture? Mirror = null)
{
  internal Dokime.Clients.FilesResources.CaptureMode ToWire()
  {
    if ((Stream is null) == (Mirror is null))
      throw new ArgumentException("Start File Capture: Select Exactly One Stream or Mirror Mode");
    if (Mirror is not null)
      return new()
      {
        Mirror = Mirror.IntervalSeconds is { } seconds ? new()
        {
          IntervalS = seconds
        } : new()
      };
    var stream = new Dokime.Clients.FilesResources.StreamCapture();
    foreach (var matcher in Stream!.Matchers ?? [])
    {
      var value = new Dokime.Clients.FilesResources.LineMatcher { Pattern = matcher.Pattern, Stream = matcher.Stream };
      value.Fields.Add((matcher.Fields ?? []).Select(field => new Dokime.Clients.FilesResources.LineField { Name = field.Name, Type = field.Type }));
      stream.Matchers.Add(value);
    }
    stream.Markers.Add((Stream.Markers ?? []).Select(marker => new Dokime.Clients.FilesResources.LineMarker { Pattern = marker.Pattern, RecordType = marker.RecordType }));
    return new()
    {
      Stream = stream
    };
  }
}

public sealed record StreamCapture(IReadOnlyList<LineMatcher>? Matchers = null, IReadOnlyList<LineMarker>? Markers = null);
public sealed record MirrorCapture(uint? IntervalSeconds = null);
public sealed record LineMatcher(string Pattern, string Stream, IReadOnlyList<LineField>? Fields = null);
public sealed record LineField(string Name, ValueType Type);
public sealed record LineMarker(string Pattern, string RecordType);
