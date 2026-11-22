// @generated from protocol descriptors; do not edit.
#nullable enable
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
internal sealed partial class Plugin
{
  internal Plugin RequiresLibrary(Harmos.Plugins.HostOperation operation)
  {
    transport.Register(ctx => ctx.Register(operation));
    return this;
  }
}
internal enum HostOperation
{
  BackendEnvironment,
  ContentArtifactsCreate,
  DocumentsAppendParts,
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
  OperationsList,
  OperationsGet,
  PluginsSettingsList,
  PluginsStateGet,
  PluginsStatePut,
  SubscriptionsSubscribe,
  TimeGet,
  TimeSleep,
}
internal static class HostContracts
{
  internal static readonly Operation<global::Dokime.Clients.BackendEnvironmentRequest, global::Dokime.Clients.BackendEnvironmentResponse> BackendEnvironment = new("dokime.host.v1.Backend/Environment", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.ContentResources.ArtifactsCreateRequest, global::Dokime.Clients.ContentResources.ArtifactsCreateResponse> ContentArtifactsCreate = new("dokime.host.v1.content.Artifacts/Create", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.DocumentsAppendPartsRequest, global::Dokime.Clients.DocumentsAppendPartsResponse> DocumentsAppendParts = new("dokime.host.v1.Documents/AppendParts", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.EvidenceResources.LogsAppendRequest, global::Dokime.Clients.EvidenceResources.LogsAppendResponse> EvidenceLogsAppend = new("dokime.host.v1.evidence.Logs/Append", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.EvidenceResources.RecordsListRequest, global::Dokime.Clients.EvidenceResources.RecordsListResponse> EvidenceRecordsList = new("dokime.host.v1.evidence.Records/List", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.EvidenceResources.RecordsAppendRequest, global::Dokime.Clients.EvidenceResources.RecordsAppendResponse> EvidenceRecordsAppend = new("dokime.host.v1.evidence.Records/Append", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest, global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse> EvidenceTraceabilityGet = new("dokime.host.v1.evidence.Traceability/Get", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.EvidenceResources.TraceabilitySetRequest, global::Dokime.Clients.EvidenceResources.TraceabilitySetResponse> EvidenceTraceabilitySet = new("dokime.host.v1.evidence.Traceability/Set", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.EvidenceStatisticsRequest, global::Dokime.Clients.EvidenceStatisticsResponse> EvidenceStatistics = new("dokime.host.v1.Evidence/Statistics", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.ExecutionsResources.ProgressGetRequest, global::Dokime.Clients.ExecutionsResources.ProgressGetResponse> ExecutionsProgressGet = new("dokime.host.v1.executions.Progress/Get", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.ExecutionsResources.ProgressSetRequest, global::Dokime.Clients.ExecutionsResources.ProgressSetResponse> ExecutionsProgressSet = new("dokime.host.v1.executions.Progress/Set", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.CapturesStartRequest, global::Dokime.Clients.FilesResources.CapturesStartResponse> FilesCapturesStart = new("dokime.host.v1.files.Captures/Start", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.CapturesStopRequest, global::Dokime.Clients.FilesResources.CapturesStopResponse> FilesCapturesStop = new("dokime.host.v1.files.Captures/Stop", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.UploadsCreateRequest, global::Dokime.Clients.FilesResources.UploadsCreateResponse> FilesUploadsCreate = new("dokime.host.v1.files.Uploads/Create", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.UploadsGetRequest, global::Dokime.Clients.FilesResources.UploadsGetResponse> FilesUploadsGet = new("dokime.host.v1.files.Uploads/Get", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.UploadsResumeRequest, global::Dokime.Clients.FilesResources.UploadsResumeResponse> FilesUploadsResume = new("dokime.host.v1.files.Uploads/Resume", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.UploadsAppendRequest, global::Dokime.Clients.FilesResources.UploadsAppendResponse> FilesUploadsAppend = new("dokime.host.v1.files.Uploads/Append", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.UploadsPublishRequest, global::Dokime.Clients.FilesResources.UploadsPublishResponse> FilesUploadsPublish = new("dokime.host.v1.files.Uploads/Publish", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesResources.UploadsDeleteRequest, global::Dokime.Clients.FilesResources.UploadsDeleteResponse> FilesUploadsDelete = new("dokime.host.v1.files.Uploads/Delete", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesGetRequest, global::Dokime.Clients.FilesGetResponse> FilesGet = new("dokime.host.v1.Files/Get", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesReadRequest, global::Dokime.Clients.FilesReadResponse> FilesRead = new("dokime.host.v1.Files/Read", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesWatchRequest, global::Dokime.Clients.FilesWatchResponse> FilesWatch = new("dokime.host.v1.Files/Watch", Visibility.Public, watch: true);
  internal static readonly Operation<global::Dokime.Clients.FilesDeleteRequest, global::Dokime.Clients.FilesDeleteResponse> FilesDelete = new("dokime.host.v1.Files/Delete", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.FilesCaptureRequest, global::Dokime.Clients.FilesCaptureResponse> FilesCapture = new("dokime.host.v1.Files/Capture", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.PromptsCreateRequest, global::Dokime.Clients.PromptsCreateResponse> PromptsCreate = new("dokime.host.v1.Prompts/Create", Visibility.Public, watch: false);
  internal static readonly Operation<global::Dokime.Clients.SecretsGetRequest, global::Dokime.Clients.SecretsGetResponse> SecretsGet = new("dokime.host.v1.Secrets/Get", Visibility.Public, watch: false);
}
internal static class HostRequirements
{
  public static Plugin Requires(this Plugin plugin, HostOperation operation) => operation switch
  {
    HostOperation.BackendEnvironment => plugin.Requires(HostContracts.BackendEnvironment),
    HostOperation.ContentArtifactsCreate => plugin.Requires(HostContracts.ContentArtifactsCreate),
    HostOperation.DocumentsAppendParts => plugin.Requires(HostContracts.DocumentsAppendParts),
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
    HostOperation.OperationsList => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.OperationsList),
    HostOperation.OperationsGet => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.OperationsGet),
    HostOperation.PluginsSettingsList => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.PluginsSettingsList),
    HostOperation.PluginsStateGet => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.PluginsStateGet),
    HostOperation.PluginsStatePut => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.PluginsStatePut),
    HostOperation.SubscriptionsSubscribe => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.SubscriptionsSubscribe),
    HostOperation.TimeGet => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.TimeGet),
    HostOperation.TimeSleep => plugin.RequiresLibrary(Harmos.Plugins.HostOperation.TimeSleep),
    _ => throw new ArgumentOutOfRangeException(nameof(operation), "Declare Host Requirement: Unknown Operation")
  };
}
internal sealed class HostRoot(Context context)
{
  public HostBackendResource Backend { get; } = new(context);
  public HostContentResource Content { get; } = new(context);
  public HostDocumentsResource Documents { get; } = new(context);
  public HostEvidenceResource Evidence { get; } = new(context);
  public HostExecutionsResource Executions { get; } = new(context);
  public HostFilesResource Files { get; } = new(context);
  public HostInvocationsResource Invocations { get; } = new(context);
  public HostOperationsResource Operations { get; } = new(context);
  public HostPluginsResource Plugins { get; } = new(context);
  public HostPromptsResource Prompts { get; } = new(context);
  public HostSecretsResource Secrets { get; } = new(context);
  public HostTimeResource Time { get; } = new(context);
}
internal sealed class HostBackendResource(Context context)
{
  public async Task<global::Dokime.Clients.BackendEnvironmentResponse> Environment(global::Harmos.Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.BackendEnvironmentRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.Operation(HostContracts.BackendEnvironment, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostContentResource(Context context)
{
  public HostContentArtifactsResource Artifacts { get; } = new(context);
}
internal sealed class HostContentArtifactsResource(Context context)
{
  public async Task<global::Dokime.Clients.ContentResources.ArtifactsCreateResponse> Create(global::Harmos.FileReference file, string filename, string content_type, global::Dokime.ArtifactScope? scope = null, string? request_id = null, global::Dokime.Metadata? metadata = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ContentResources.ArtifactsCreateRequest
    {
      File = file,
      Filename = filename,
      ContentType = content_type,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    if (scope is not null)
    {
      request.Scope = scope;
    }
    if (metadata is not null)
    {
      request.Metadata = metadata;
    }
    var response = await context.Operation(HostContracts.ContentArtifactsCreate, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostDocumentsResource(Context context)
{
  public async Task<global::Harmos.Receipt> AppendParts(string execution, string document, string section, global::Dokime.Section? subsection = null, global::Dokime.Card? card = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.DocumentsAppendPartsRequest
    {
      Execution = execution,
      Document = document,
      Section = section,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    if (subsection is not null)
    {
      request.Subsection = subsection;
    }
    if (card is not null)
    {
      request.Card = card;
    }
    var response = await context.Operation(HostContracts.DocumentsAppendParts, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read DocumentsAppendParts: Missing Payload");
  }
}
internal sealed class HostEvidenceResource(Context context)
{
  public HostEvidenceLogsResource Logs { get; } = new(context);
  public HostEvidenceRecordsResource Records { get; } = new(context);
  public HostEvidenceTraceabilityResource Traceability { get; } = new(context);
  public async Task<global::Dokime.Clients.EvidenceStatisticsResponse> Statistics(string stream, IReadOnlyList<uint> versions, global::Dokime.Window between, IReadOnlyList<global::Dokime.Clients.Output> outputs, string field, string step, global::Dokime.EvidenceScope? scope = null, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceStatisticsRequest
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
    var response = await context.Operation(HostContracts.EvidenceStatistics, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostEvidenceLogsResource(Context context)
{
  public async Task<global::Harmos.Receipt> Append(string execution, global::Dokime.Severity severity, string message, global::Google.Protobuf.WellKnownTypes.Timestamp at, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.LogsAppendRequest
    {
      Execution = execution,
      Severity = severity,
      Message = message,
      At = at,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.EvidenceLogsAppend, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read EvidenceLogsAppend: Missing Payload");
  }
}
internal sealed class HostEvidenceRecordsResource(Context context)
{
  public async Task<global::Dokime.Clients.EvidenceResources.RecordsListResponse> List(string record_type, IReadOnlyList<uint> versions, global::Dokime.Window between, string step, global::Dokime.Clients.EvidenceResources.CapturedRecordSelection selection, global::Dokime.Clients.EvidenceResources.RecordScope? scope = null, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.RecordsListRequest
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
    var response = await context.Operation(HostContracts.EvidenceRecordsList, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.RecordsAppendResponse> Append(string execution, string kind, ByteString payload, global::Google.Protobuf.WellKnownTypes.Timestamp source_at, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.RecordsAppendRequest
    {
      Execution = execution,
      Kind = kind,
      Payload = payload,
      SourceAt = source_at,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.EvidenceRecordsAppend, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostEvidenceTraceabilityResource(Context context)
{
  public async Task<global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse> Get(global::Dokime.EvidenceScope? scope = null, global::Harmos.PageRequest? page = null, global::Harmos.ReadCut? at = null, string? entry = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest
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
    var response = await context.Operation(HostContracts.EvidenceTraceabilityGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.EvidenceResources.TraceabilitySetResponse> Set(global::Dokime.TraceabilityEntry value, ByteString expected_revision, string entry, string? name = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.EvidenceResources.TraceabilitySetRequest
    {
      Value = value,
      ExpectedRevision = expected_revision,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
      Entry = entry,
    };
    if (name is not null)
    {
      request.Name = name;
    }
    var response = await context.Operation(HostContracts.EvidenceTraceabilitySet, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostExecutionsResource(Context context)
{
  public HostExecutionsProgressResource Progress { get; } = new(context);
}
internal sealed class HostExecutionsProgressResource(Context context)
{
  public async Task<global::Dokime.Clients.ExecutionsResources.ProgressGetResponse> Get(string? name = null, global::Harmos.Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.ProgressGetRequest
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
    var response = await context.Operation(HostContracts.ExecutionsProgressGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.ExecutionsResources.ProgressSetResponse> Set(global::Dokime.ExecutionProgressValue value, ByteString expected_revision, string? name = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.ExecutionsResources.ProgressSetRequest
    {
      Value = value,
      ExpectedRevision = expected_revision,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    if (name is not null)
    {
      request.Name = name;
    }
    var response = await context.Operation(HostContracts.ExecutionsProgressSet, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostFilesResource(Context context)
{
  public HostFilesCapturesResource Captures { get; } = new(context);
  public HostFilesUploadsResource Uploads { get; } = new(context);
  public async Task<global::Dokime.Clients.FilesGetResponse> Get(global::Harmos.FileReference file, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesGetRequest
    {
      File = file,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.Operation(HostContracts.FilesGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.FilesReadResponse> Read(global::Harmos.FileReference file, ulong offset, ulong max_bytes, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesReadRequest
    {
      File = file,
      Offset = offset,
      MaxBytes = max_bytes,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.Operation(HostContracts.FilesRead, request, policy, cancellationToken);
    return response;
  }
  public async IAsyncEnumerable<global::Dokime.Clients.FilesWatchResponse> Watch(ulong offset, ulong maxItemBytes, global::Harmos.FileReference? file = null, global::Harmos.Upload? upload = null, global::Harmos.ReadCut? at = null, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesWatchRequest
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
  public async Task<global::Harmos.Receipt> Delete(global::Harmos.FileReference file, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesDeleteRequest
    {
      File = file,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.FilesDelete, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read FilesDelete: Missing Payload");
  }
  public async Task<global::Dokime.Clients.FilesCaptureResponse> Capture(global::Dokime.FileSource source, string output, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesCaptureRequest
    {
      Source = source,
      Output = output,
    };
    var response = await context.Operation(HostContracts.FilesCapture, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostFilesCapturesResource(Context context)
{
  public async Task<ulong> Start(global::Dokime.FileSource source, string output, global::Dokime.Clients.FilesResources.CaptureMode mode, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.CapturesStartRequest
    {
      Source = source,
      Output = output,
      Mode = mode,
    };
    var response = await context.Operation(HostContracts.FilesCapturesStart, request, policy, cancellationToken);
    return response.CaptureId;
  }
  public async Task<global::Dokime.Clients.FilesResources.CapturesStopResponse> Stop(ulong capture_id, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.CapturesStopRequest
    {
      CaptureId = capture_id,
    };
    var response = await context.Operation(HostContracts.FilesCapturesStop, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostFilesUploadsResource(Context context)
{
  public async Task<global::Dokime.Clients.FilesResources.UploadsCreateResponse> Create(global::Harmos.FileMetadata metadata, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsCreateRequest
    {
      Metadata = metadata,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.FilesUploadsCreate, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsGetResponse> Get(global::Harmos.Upload upload, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsGetRequest
    {
      Upload = upload,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.Operation(HostContracts.FilesUploadsGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsResumeResponse> Resume(global::Harmos.Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsResumeRequest
    {
      Upload = upload,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.FilesUploadsResume, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsAppendResponse> Append(global::Harmos.Upload upload, ulong offset, ByteString data, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsAppendRequest
    {
      Upload = upload,
      Offset = offset,
      Data = data,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.FilesUploadsAppend, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsPublishResponse> Publish(global::Harmos.Upload upload, ulong length, ByteString sha256, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsPublishRequest
    {
      Upload = upload,
      Length = length,
      Sha256 = sha256,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.FilesUploadsPublish, request, policy, cancellationToken);
    return response;
  }
  public async Task<global::Dokime.Clients.FilesResources.UploadsDeleteResponse> Delete(global::Harmos.Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.FilesResources.UploadsDeleteRequest
    {
      Upload = upload,
      RequestId = request_id ?? Guid.NewGuid().ToString("N"),
    };
    var response = await context.Operation(HostContracts.FilesUploadsDelete, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostInvocationsResource(Context context)
{
  public async Task<global::Harmos.InstanceState> Get(ulong instance, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Invocations.Get(instance, policy, cancellationToken);
  }
  public async Task<global::Harmos.InstanceState> Cancel(ulong instance, string reason, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Invocations.Cancel(instance, reason, policy, cancellationToken);
  }
  public async IAsyncEnumerable<global::Harmos.InstanceState> Watch(ulong instance, ulong maxItemBytes, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in context.Transport.Host.Invocations.Watch(instance, maxItemBytes, cancellationToken))
    {
      yield return response;
    }
  }
}
internal sealed class HostOperationsResource(Context context)
{
  public async Task<IReadOnlyList<global::Harmos.DiscoveredOperation>> List(OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Operations.List(policy, cancellationToken);
  }
  public async Task<global::Harmos.DiscoveredOperation> Get(string id, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Operations.Get(id, policy, cancellationToken);
  }
}
internal sealed class HostPluginsResource(Context context)
{
  public HostPluginsSettingsResource Settings { get; } = new(context);
  public HostPluginsStateResource State { get; } = new(context);
}
internal sealed class HostPluginsSettingsResource(Context context)
{
  public async Task<global::Harmos.Clients.SettingsListResponse> List(string? name = null, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Plugins.Settings.List(name, at, policy, cancellationToken);
  }
}
internal sealed class HostPluginsStateResource(Context context)
{
  public async Task<global::Harmos.Clients.StateGetResponse> Get(OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Plugins.State.Get(policy, cancellationToken);
  }
  public async Task<ulong> Put(ulong expected_revision, ByteString value, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Plugins.State.Put(expected_revision, value, policy, cancellationToken);
  }
}
internal sealed class HostPromptsResource(Context context)
{
  public async Task<global::Dokime.Clients.PromptsCreateResponse> Create(string prompt, string prompt_type, IReadOnlyDictionary<string, global::Dokime.Value> values, global::Dokime.Prompt? definition = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.PromptsCreateRequest
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
    var response = await context.Operation(HostContracts.PromptsCreate, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostSecretsResource(Context context)
{
  public async Task<global::Dokime.Clients.SecretsGetResponse> Get(string name, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new global::Dokime.Clients.SecretsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var response = await context.Operation(HostContracts.SecretsGet, request, policy, cancellationToken);
    return response;
  }
}
internal sealed class HostTimeResource(Context context)
{
  public async Task<global::Google.Protobuf.WellKnownTypes.Timestamp> Get(OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Time.Get(policy, cancellationToken);
  }
  public async Task<global::Harmos.Clients.TimeSleepResponse> Sleep(global::Google.Protobuf.WellKnownTypes.Duration duration, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Time.Sleep(duration, policy, cancellationToken);
  }
}
