// @generated from protocol descriptors; do not edit.
#nullable enable
using Dokime;
using Dokime.Clients;
using Dokime.Clients.ContentResources;
using Dokime.Clients.EvidenceResources;
using Dokime.Clients.ExecutionsResources;
using Dokime.Clients.FilesResources;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using Harmos.Clients;
using Harmos.Plugins;
using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
namespace Dokime.Plugins;
public sealed partial class Plugin
{
  internal Plugin RequiresLibrary(Harmos.Plugins.HostOperation operation)
  {
    transport.Requires(operation);
    return this;
  }
}
public enum HostOperation
{
  BackendEnvironment,
  ContentArtifactsCreate,
  DocumentsAppendParts,
  EffectsCreate,
  EffectsGet,
  EffectsSet,
  EvidenceLogsAppend,
  EvidenceRecordsList,
  EvidenceRecordsAppend,
  EvidenceTraceabilityGet,
  EvidenceTraceabilitySet,
  EvidenceStatistics,
  ExecutionsProgressGet,
  ExecutionsProgressSet,
  FilesCapturesStart,
  FilesCapturesStop,
  FilesUploadsCreate,
  FilesUploadsGet,
  FilesUploadsResume,
  FilesUploadsAppend,
  FilesUploadsPublish,
  FilesUploadsDelete,
  FilesGet,
  FilesRead,
  FilesWatch,
  FilesDelete,
  FilesCapture,
  PromptsCreate,
  SecretsGet,
  InvocationsGet,
  InvocationsCancel,
  InvocationsWatch,
  PluginsSettingsList,
  TimeGet,
  TimeSleep,
}
internal static class HostContracts
{
  internal static readonly Operation<BackendEnvironmentRequest, BackendEnvironmentResponse> BackendEnvironment = new("dokime.host.v1.Backend/Environment", Visibility.Public, watch: false);
  internal static readonly Operation<ArtifactsCreateRequest, ArtifactsCreateResponse> ContentArtifactsCreate = new("dokime.host.v1.content.Artifacts/Create", Visibility.Public, watch: false);
  internal static readonly Operation<DocumentsAppendPartsRequest, DocumentsAppendPartsResponse> DocumentsAppendParts = new("dokime.host.v1.Documents/AppendParts", Visibility.Public, watch: false);
  internal static readonly Operation<EffectsCreateRequest, EffectsCreateResponse> EffectsCreate = new("dokime.host.v1.Effects/Create", Visibility.Public, watch: false);
  internal static readonly Operation<EffectsGetRequest, EffectsGetResponse> EffectsGet = new("dokime.host.v1.Effects/Get", Visibility.Public, watch: false);
  internal static readonly Operation<EffectsSetRequest, EffectsSetResponse> EffectsSet = new("dokime.host.v1.Effects/Set", Visibility.Public, watch: false);
  internal static readonly Operation<LogsAppendRequest, LogsAppendResponse> EvidenceLogsAppend = new("dokime.host.v1.evidence.Logs/Append", Visibility.Public, watch: false);
  internal static readonly Operation<RecordsListRequest, RecordsListResponse> EvidenceRecordsList = new("dokime.host.v1.evidence.Records/List", Visibility.Public, watch: false);
  internal static readonly Operation<RecordsAppendRequest, RecordsAppendResponse> EvidenceRecordsAppend = new("dokime.host.v1.evidence.Records/Append", Visibility.Public, watch: false);
  internal static readonly Operation<TraceabilityGetRequest, TraceabilityGetResponse> EvidenceTraceabilityGet = new("dokime.host.v1.evidence.Traceability/Get", Visibility.Public, watch: false);
  internal static readonly Operation<TraceabilitySetRequest, TraceabilitySetResponse> EvidenceTraceabilitySet = new("dokime.host.v1.evidence.Traceability/Set", Visibility.Public, watch: false);
  internal static readonly Operation<EvidenceStatisticsRequest, EvidenceStatisticsResponse> EvidenceStatistics = new("dokime.host.v1.Evidence/Statistics", Visibility.Public, watch: false);
  internal static readonly Operation<ProgressGetRequest, ProgressGetResponse> ExecutionsProgressGet = new("dokime.host.v1.executions.Progress/Get", Visibility.Public, watch: false);
  internal static readonly Operation<ProgressSetRequest, ProgressSetResponse> ExecutionsProgressSet = new("dokime.host.v1.executions.Progress/Set", Visibility.Public, watch: false);
  internal static readonly Operation<CapturesStartRequest, CapturesStartResponse> FilesCapturesStart = new("dokime.host.v1.files.Captures/Start", Visibility.Public, watch: false);
  internal static readonly Operation<CapturesStopRequest, CapturesStopResponse> FilesCapturesStop = new("dokime.host.v1.files.Captures/Stop", Visibility.Public, watch: false);
  internal static readonly Operation<UploadsCreateRequest, UploadsCreateResponse> FilesUploadsCreate = new("dokime.host.v1.files.Uploads/Create", Visibility.Public, watch: false);
  internal static readonly Operation<UploadsGetRequest, UploadsGetResponse> FilesUploadsGet = new("dokime.host.v1.files.Uploads/Get", Visibility.Public, watch: false);
  internal static readonly Operation<UploadsResumeRequest, UploadsResumeResponse> FilesUploadsResume = new("dokime.host.v1.files.Uploads/Resume", Visibility.Public, watch: false);
  internal static readonly Operation<UploadsAppendRequest, UploadsAppendResponse> FilesUploadsAppend = new("dokime.host.v1.files.Uploads/Append", Visibility.Public, watch: false);
  internal static readonly Operation<UploadsPublishRequest, UploadsPublishResponse> FilesUploadsPublish = new("dokime.host.v1.files.Uploads/Publish", Visibility.Public, watch: false);
  internal static readonly Operation<UploadsDeleteRequest, UploadsDeleteResponse> FilesUploadsDelete = new("dokime.host.v1.files.Uploads/Delete", Visibility.Public, watch: false);
  internal static readonly Operation<FilesGetRequest, FilesGetResponse> FilesGet = new("dokime.host.v1.Files/Get", Visibility.Public, watch: false);
  internal static readonly Operation<FilesReadRequest, FilesReadResponse> FilesRead = new("dokime.host.v1.Files/Read", Visibility.Public, watch: false);
  internal static readonly Operation<FilesWatchRequest, FilesWatchResponse> FilesWatch = new("dokime.host.v1.Files/Watch", Visibility.Public, watch: true);
  internal static readonly Operation<FilesDeleteRequest, FilesDeleteResponse> FilesDelete = new("dokime.host.v1.Files/Delete", Visibility.Public, watch: false);
  internal static readonly Operation<FilesCaptureRequest, FilesCaptureResponse> FilesCapture = new("dokime.host.v1.Files/Capture", Visibility.Public, watch: false);
  internal static readonly Operation<PromptsCreateRequest, PromptsCreateResponse> PromptsCreate = new("dokime.host.v1.Prompts/Create", Visibility.Public, watch: false);
  internal static readonly Operation<SecretsGetRequest, SecretsGetResponse> SecretsGet = new("dokime.host.v1.Secrets/Get", Visibility.Public, watch: false);
}
public static class HostRequirements
{
  public static Plugin Requires(this Plugin plugin, HostOperation operation) => operation switch
  {
    HostOperation.BackendEnvironment => plugin.Requires(HostContracts.BackendEnvironment),
    HostOperation.ContentArtifactsCreate => plugin.Requires(HostContracts.ContentArtifactsCreate),
    HostOperation.DocumentsAppendParts => plugin.Requires(HostContracts.DocumentsAppendParts),
    HostOperation.EffectsCreate => plugin.Requires(HostContracts.EffectsCreate),
    HostOperation.EffectsGet => plugin.Requires(HostContracts.EffectsGet),
    HostOperation.EffectsSet => plugin.Requires(HostContracts.EffectsSet),
    HostOperation.EvidenceLogsAppend => plugin.Requires(HostContracts.EvidenceLogsAppend),
    HostOperation.EvidenceRecordsList => plugin.Requires(HostContracts.EvidenceRecordsList),
    HostOperation.EvidenceRecordsAppend => plugin.Requires(HostContracts.EvidenceRecordsAppend),
    HostOperation.EvidenceTraceabilityGet => plugin.Requires(HostContracts.EvidenceTraceabilityGet),
    HostOperation.EvidenceTraceabilitySet => plugin.Requires(HostContracts.EvidenceTraceabilitySet),
    HostOperation.EvidenceStatistics => plugin.Requires(HostContracts.EvidenceStatistics),
    HostOperation.ExecutionsProgressGet => plugin.Requires(HostContracts.ExecutionsProgressGet),
    HostOperation.ExecutionsProgressSet => plugin.Requires(HostContracts.ExecutionsProgressSet),
    HostOperation.FilesCapturesStart => plugin.Requires(HostContracts.FilesCapturesStart),
    HostOperation.FilesCapturesStop => plugin.Requires(HostContracts.FilesCapturesStop),
    HostOperation.FilesUploadsCreate => plugin.Requires(HostContracts.FilesUploadsCreate),
    HostOperation.FilesUploadsGet => plugin.Requires(HostContracts.FilesUploadsGet),
    HostOperation.FilesUploadsResume => plugin.Requires(HostContracts.FilesUploadsResume),
    HostOperation.FilesUploadsAppend => plugin.Requires(HostContracts.FilesUploadsAppend),
    HostOperation.FilesUploadsPublish => plugin.Requires(HostContracts.FilesUploadsPublish),
    HostOperation.FilesUploadsDelete => plugin.Requires(HostContracts.FilesUploadsDelete),
    HostOperation.FilesGet => plugin.Requires(HostContracts.FilesGet),
    HostOperation.FilesRead => plugin.Requires(HostContracts.FilesRead),
    HostOperation.FilesWatch => plugin.Requires(HostContracts.FilesWatch),
    HostOperation.FilesDelete => plugin.Requires(HostContracts.FilesDelete),
    HostOperation.FilesCapture => plugin.Requires(HostContracts.FilesCapture),
    HostOperation.PromptsCreate => plugin.Requires(HostContracts.PromptsCreate),
    HostOperation.SecretsGet => plugin.Requires(HostContracts.SecretsGet),
    HostOperation.InvocationsGet => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.InvocationsGet),
    HostOperation.InvocationsCancel => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.InvocationsCancel),
    HostOperation.InvocationsWatch => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.InvocationsWatch),
    HostOperation.PluginsSettingsList => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.PluginsSettingsList),
    HostOperation.TimeGet => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.TimeGet),
    HostOperation.TimeSleep => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.TimeSleep),
    _ => throw new ArgumentOutOfRangeException(nameof(operation), "Declare Host Requirement: Unknown Operation")
  };
}
public sealed class HostRoot(Context context)
{
  public HostBackendResource Backend { get; } = new(context);
  public HostContentResource Content { get; } = new(context);
  public HostDocumentsResource Documents { get; } = new(context);
  public HostEffectsResource Effects { get; } = new(context);
  public HostEvidenceResource Evidence { get; } = new(context);
  public HostExecutionsResource Executions { get; } = new(context);
  public HostFilesResource Files { get; } = new(context);
  public HostInvocationsResource Invocations { get; } = new(context);
  public HostPluginsResource Plugins { get; } = new(context);
  public HostPromptsResource Prompts { get; } = new(context);
  public HostSecretsResource Secrets { get; } = new(context);
  public HostTimeResource Time { get; } = new(context);
}
public sealed class HostBackendResource(Context context)
{
  public async Task<BackendEnvironmentResponse> Environment(Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new BackendEnvironmentRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.BackendEnvironment, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostContentResource(Context context)
{
  public HostContentArtifactsResource Artifacts { get; } = new(context);
}
public sealed class HostContentArtifactsResource(Context context)
{
  public async Task<ArtifactsCreateResponse> Create(FileReference file, string filename, string content_type, ArtifactScope? scope = null, string? request_id = null, Metadata? metadata = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ArtifactsCreateRequest
    {
      File = file,
      Filename = filename,
      ContentType = content_type,
    };
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    if (metadata is not null)
    {
      request.Metadata = metadata;
    }
    var response = await context.CallAsync(HostContracts.ContentArtifactsCreate, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostDocumentsResource(Context context)
{
  public async Task<Receipt> AppendParts(string execution, string document, string section, Section? subsection = null, Card? card = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new DocumentsAppendPartsRequest
    {
      Execution = execution,
      Document = document,
      Section = section,
    };
    if (subsection is not null)
    {
      request.Subsection = subsection;
    }
    if (card is not null)
    {
      request.Card = card;
    }
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.DocumentsAppendParts, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read DocumentsAppendParts: Missing Payload");
  }
}
public sealed class HostEffectsResource(Context context)
{
  public async Task<EffectsCreateResponse> Create(string resource, string inverse_operation, ByteString inverse, string description, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EffectsCreateRequest
    {
      Resource = resource,
      InverseOperation = inverse_operation,
      Inverse = inverse,
      Description = description,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.EffectsCreate, request, policy, cancellationToken);
    return response;
  }
  public async Task<EffectsGetResponse> Get(string name, Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EffectsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.EffectsGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<EffectsSetResponse> Set(string name, EffectWritable value, ByteString expected_revision, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EffectsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expected_revision,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.EffectsSet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostEvidenceResource(Context context)
{
  public HostEvidenceLogsResource Logs { get; } = new(context);
  public HostEvidenceRecordsResource Records { get; } = new(context);
  public HostEvidenceTraceabilityResource Traceability { get; } = new(context);
  public async Task<EvidenceStatisticsResponse> Statistics(string stream, IReadOnlyList<uint> versions, Window between, IReadOnlyList<Output> outputs, string field, string step, EvidenceScope? scope = null, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EvidenceStatisticsRequest
    {
      Stream = stream,
      Between = between,
      Field = field,
      Step = step,
    };
    request.Versions.Add(versions);
    request.Outputs.Add(outputs);
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.EvidenceStatistics, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostEvidenceLogsResource(Context context)
{
  public async Task<Receipt> Append(string execution, Severity severity, string message, Timestamp at, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new LogsAppendRequest
    {
      Execution = execution,
      Severity = severity,
      Message = message,
      At = at,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.EvidenceLogsAppend, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read EvidenceLogsAppend: Missing Payload");
  }
}
public sealed class HostEvidenceRecordsResource(Context context)
{
  public async Task<RecordsListResponse> List(string record_type, IReadOnlyList<uint> versions, Window between, string step, CapturedRecordSelection selection, RecordScope? scope = null, ReadCut? at = null, PageRequest? page = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new RecordsListRequest
    {
      RecordType = record_type,
      Between = between,
      Step = step,
      Selection = selection,
    };
    request.Versions.Add(versions);
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var response = await context.CallAsync(HostContracts.EvidenceRecordsList, request, policy, cancellationToken);
    return response;
  }
  public async Task<RecordsAppendResponse> Append(string execution, string kind, ByteString payload, Timestamp source_at, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new RecordsAppendRequest
    {
      Execution = execution,
      Kind = kind,
      Payload = payload,
      SourceAt = source_at,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.EvidenceRecordsAppend, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostEvidenceTraceabilityResource(Context context)
{
  public async Task<TraceabilityGetResponse> Get(EvidenceScope? scope = null, PageRequest? page = null, ReadCut? at = null, string? entry = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new TraceabilityGetRequest
    {
    };
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (entry is not null)
    {
      request.Entry = entry;
    }
    var response = await context.CallAsync(HostContracts.EvidenceTraceabilityGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<TraceabilitySetResponse> Set(TraceabilityEntry value, ByteString expected_revision, string entry, string? name = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new TraceabilitySetRequest
    {
      Value = value,
      ExpectedRevision = expected_revision,
      Entry = entry,
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.EvidenceTraceabilitySet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostExecutionsResource(Context context)
{
  public HostExecutionsProgressResource Progress { get; } = new(context);
}
public sealed class HostExecutionsProgressResource(Context context)
{
  public async Task<ProgressGetResponse> Get(string? name = null, Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ProgressGetRequest
    {
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.ExecutionsProgressGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<ProgressSetResponse> Set(ExecutionProgressValue value, ByteString expected_revision, string? name = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ProgressSetRequest
    {
      Value = value,
      ExpectedRevision = expected_revision,
    };
    if (name is not null)
    {
      request.Name = name;
    }
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.ExecutionsProgressSet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostFilesResource(Context context)
{
  public HostFilesCapturesResource Captures { get; } = new(context);
  public HostFilesUploadsResource Uploads { get; } = new(context);
  public async Task<FilesGetResponse> Get(FileReference file, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesGetRequest
    {
      File = file,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.FilesGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<FilesReadResponse> Read(FileReference file, ulong offset, ulong max_bytes, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesReadRequest
    {
      File = file,
      Offset = offset,
      MaxBytes = max_bytes,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.FilesRead, request, policy, cancellationToken);
    return response;
  }
  public async IAsyncEnumerable<FilesWatchResponse> Watch(ulong offset, ulong maxItemBytes, FileReference? file = null, Upload? upload = null, ReadCut? at = null, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new FilesWatchRequest
    {
      Offset = offset,
    };
    if (file is not null)
    {
      request.File = file;
    }
    if (upload is not null)
    {
      request.Upload = upload;
    }
    if (at is not null)
    {
      request.At = at;
    }
    await foreach (var response in context.WatchAsync(HostContracts.FilesWatch, request, maxItemBytes, cancellationToken))
    {
      yield return response;
    }
  }
  public async Task<Receipt> Delete(FileReference file, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesDeleteRequest
    {
      File = file,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.FilesDelete, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read FilesDelete: Missing Payload");
  }
  public async Task<FilesCaptureResponse> Capture(FileSource source, string output, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesCaptureRequest
    {
      Source = source,
      Output = output,
    };
    var response = await context.CallAsync(HostContracts.FilesCapture, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostFilesCapturesResource(Context context)
{
  public async Task<ulong> Start(FileSource source, string output, CaptureMode mode, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new CapturesStartRequest
    {
      Source = source,
      Output = output,
      Mode = mode,
    };
    var response = await context.CallAsync(HostContracts.FilesCapturesStart, request, policy, cancellationToken);
    return response.CaptureId;
  }
  public async Task<CapturesStopResponse> Stop(ulong capture_id, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new CapturesStopRequest
    {
      CaptureId = capture_id,
    };
    var response = await context.CallAsync(HostContracts.FilesCapturesStop, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostFilesUploadsResource(Context context)
{
  public async Task<UploadsCreateResponse> Create(FileMetadata metadata, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsCreateRequest
    {
      Metadata = metadata,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.FilesUploadsCreate, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsGetResponse> Get(Upload upload, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsGetRequest
    {
      Upload = upload,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.FilesUploadsGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsResumeResponse> Resume(Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsResumeRequest
    {
      Upload = upload,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.FilesUploadsResume, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsAppendResponse> Append(Upload upload, ulong offset, ByteString data, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsAppendRequest
    {
      Upload = upload,
      Offset = offset,
      Data = data,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.FilesUploadsAppend, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsPublishResponse> Publish(Upload upload, ulong length, ByteString sha256, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsPublishRequest
    {
      Upload = upload,
      Length = length,
      Sha256 = sha256,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.FilesUploadsPublish, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsDeleteResponse> Delete(Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsDeleteRequest
    {
      Upload = upload,
    };
    if (request_id is not null)
    {
      request.RequestId = request_id;
    }
    var response = await context.CallAsync(HostContracts.FilesUploadsDelete, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostInvocationsResource(Context context)
{
  public async Task<InstanceState> Get(ulong instance, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Invocations.Get(instance, policy, cancellationToken);
  }
  public async Task<InstanceState> Cancel(ulong instance, string reason, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Invocations.Cancel(instance, reason, policy, cancellationToken);
  }
  public async IAsyncEnumerable<InstanceState> Watch(ulong instance, ulong maxItemBytes, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in context.Transport.Host.Invocations.Watch(instance, maxItemBytes, cancellationToken))
    {
      yield return response;
    }
  }
}
public sealed class HostPluginsResource(Context context)
{
  public HostPluginsSettingsResource Settings { get; } = new(context);
}
public sealed class HostPluginsSettingsResource(Context context)
{
  public async Task<SettingsListResponse> List(string? name = null, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Plugins.Settings.List(name, at, policy, cancellationToken);
  }
}
public sealed class HostPromptsResource(Context context)
{
  public async Task<PromptsCreateResponse> Create(string prompt, string prompt_type, IReadOnlyDictionary<string, Value> values, Prompt? definition = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new PromptsCreateRequest
    {
      Prompt = prompt,
      PromptType = prompt_type,
    };
    foreach (var entry in values)
    {
      request.Values.Add(entry.Key, entry.Value);
    }
    if (definition is not null)
    {
      request.Definition = definition;
    }
    var response = await context.CallAsync(HostContracts.PromptsCreate, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostSecretsResource(Context context)
{
  public async Task<SecretsGetResponse> Get(string name, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new SecretsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.CallAsync(HostContracts.SecretsGet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostTimeResource(Context context)
{
  public async Task<Timestamp> Get(OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Time.Get(policy, cancellationToken);
  }
  public async Task<TimeSleepResponse> Sleep(Duration duration, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Time.Sleep(duration, policy, cancellationToken);
  }
}
