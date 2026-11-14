// @generated from protocol descriptors; do not edit.
#nullable enable
using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using System.Runtime.CompilerServices;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using BackendEnvironmentRequest = global::Dokime.Clients.BackendEnvironmentRequest;
using BackendEnvironmentResponse = global::Dokime.Clients.BackendEnvironmentResponse;
using ArtifactsCreateRequest = global::Dokime.Clients.ContentResources.ArtifactsCreateRequest;
using ArtifactsCreateResponse = global::Dokime.Clients.ContentResources.ArtifactsCreateResponse;
using DocumentsAppendPartsRequest = global::Dokime.Clients.DocumentsAppendPartsRequest;
using DocumentsAppendPartsResponse = global::Dokime.Clients.DocumentsAppendPartsResponse;
using EffectsCreateRequest = global::Dokime.Clients.EffectsCreateRequest;
using EffectsCreateResponse = global::Dokime.Clients.EffectsCreateResponse;
using EffectsGetRequest = global::Dokime.Clients.EffectsGetRequest;
using EffectsGetResponse = global::Dokime.Clients.EffectsGetResponse;
using EffectsSetRequest = global::Dokime.Clients.EffectsSetRequest;
using EffectsSetResponse = global::Dokime.Clients.EffectsSetResponse;
using LogsAppendRequest = global::Dokime.Clients.EvidenceResources.LogsAppendRequest;
using LogsAppendResponse = global::Dokime.Clients.EvidenceResources.LogsAppendResponse;
using RecordsListRequest = global::Dokime.Clients.EvidenceResources.RecordsListRequest;
using RecordsListResponse = global::Dokime.Clients.EvidenceResources.RecordsListResponse;
using RecordsAppendRequest = global::Dokime.Clients.EvidenceResources.RecordsAppendRequest;
using RecordsAppendResponse = global::Dokime.Clients.EvidenceResources.RecordsAppendResponse;
using TraceabilityGetRequest = global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest;
using TraceabilityGetResponse = global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse;
using TraceabilitySetRequest = global::Dokime.Clients.EvidenceResources.TraceabilitySetRequest;
using TraceabilitySetResponse = global::Dokime.Clients.EvidenceResources.TraceabilitySetResponse;
using EvidenceStatisticsRequest = global::Dokime.Clients.EvidenceStatisticsRequest;
using EvidenceStatisticsResponse = global::Dokime.Clients.EvidenceStatisticsResponse;
using ProgressGetRequest = global::Dokime.Clients.ExecutionsResources.ProgressGetRequest;
using ProgressGetResponse = global::Dokime.Clients.ExecutionsResources.ProgressGetResponse;
using ProgressSetRequest = global::Dokime.Clients.ExecutionsResources.ProgressSetRequest;
using ProgressSetResponse = global::Dokime.Clients.ExecutionsResources.ProgressSetResponse;
using CapturesStartRequest = global::Dokime.Clients.FilesResources.CapturesStartRequest;
using CapturesStartResponse = global::Dokime.Clients.FilesResources.CapturesStartResponse;
using CapturesStopRequest = global::Dokime.Clients.FilesResources.CapturesStopRequest;
using CapturesStopResponse = global::Dokime.Clients.FilesResources.CapturesStopResponse;
using UploadsCreateRequest = global::Dokime.Clients.FilesResources.UploadsCreateRequest;
using UploadsCreateResponse = global::Dokime.Clients.FilesResources.UploadsCreateResponse;
using UploadsGetRequest = global::Dokime.Clients.FilesResources.UploadsGetRequest;
using UploadsGetResponse = global::Dokime.Clients.FilesResources.UploadsGetResponse;
using UploadsResumeRequest = global::Dokime.Clients.FilesResources.UploadsResumeRequest;
using UploadsResumeResponse = global::Dokime.Clients.FilesResources.UploadsResumeResponse;
using UploadsAppendRequest = global::Dokime.Clients.FilesResources.UploadsAppendRequest;
using UploadsAppendResponse = global::Dokime.Clients.FilesResources.UploadsAppendResponse;
using UploadsPublishRequest = global::Dokime.Clients.FilesResources.UploadsPublishRequest;
using UploadsPublishResponse = global::Dokime.Clients.FilesResources.UploadsPublishResponse;
using UploadsDeleteRequest = global::Dokime.Clients.FilesResources.UploadsDeleteRequest;
using UploadsDeleteResponse = global::Dokime.Clients.FilesResources.UploadsDeleteResponse;
using FilesGetRequest = global::Dokime.Clients.FilesGetRequest;
using FilesGetResponse = global::Dokime.Clients.FilesGetResponse;
using FilesReadRequest = global::Dokime.Clients.FilesReadRequest;
using FilesReadResponse = global::Dokime.Clients.FilesReadResponse;
using FilesWatchRequest = global::Dokime.Clients.FilesWatchRequest;
using FilesWatchResponse = global::Dokime.Clients.FilesWatchResponse;
using FilesDeleteRequest = global::Dokime.Clients.FilesDeleteRequest;
using FilesDeleteResponse = global::Dokime.Clients.FilesDeleteResponse;
using FilesCaptureRequest = global::Dokime.Clients.FilesCaptureRequest;
using FilesCaptureResponse = global::Dokime.Clients.FilesCaptureResponse;
using PromptsCreateRequest = global::Dokime.Clients.PromptsCreateRequest;
using PromptsCreateResponse = global::Dokime.Clients.PromptsCreateResponse;
using SecretsGetRequest = global::Dokime.Clients.SecretsGetRequest;
using SecretsGetResponse = global::Dokime.Clients.SecretsGetResponse;
using SettingsListResponse = global::Harmos.Clients.SettingsListResponse;
using TimeSleepResponse = global::Harmos.Clients.TimeSleepResponse;
using Position = global::Harmos.Position;
using FileReference = global::Harmos.FileReference;
using ArtifactScope = global::Dokime.ArtifactScope;
using Metadata = global::Dokime.Metadata;
using Section = global::Dokime.Section;
using Card = global::Dokime.Card;
using Receipt = global::Harmos.Receipt;
using EffectWritable = global::Dokime.Clients.EffectWritable;
using Severity = global::Dokime.Severity;
using Timestamp = global::Google.Protobuf.WellKnownTypes.Timestamp;
using RecordScope = global::Dokime.Clients.EvidenceResources.RecordScope;
using Window = global::Dokime.Window;
using CapturedRecordSelection = global::Dokime.Clients.EvidenceResources.CapturedRecordSelection;
using ReadCut = global::Harmos.ReadCut;
using PageRequest = global::Harmos.PageRequest;
using EvidenceScope = global::Dokime.EvidenceScope;
using TraceabilityEntry = global::Dokime.TraceabilityEntry;
using Output = global::Dokime.Clients.Output;
using ExecutionProgressValue = global::Dokime.ExecutionProgressValue;
using FileSource = global::Dokime.FileSource;
using CaptureMode = global::Dokime.Clients.FilesResources.CaptureMode;
using FileMetadata = global::Harmos.FileMetadata;
using Upload = global::Harmos.Upload;
using Value = global::Dokime.Value;
using Prompt = global::Dokime.Prompt;
using InstanceState = global::Harmos.InstanceState;
using Duration = global::Google.Protobuf.WellKnownTypes.Duration;
namespace Dokime.Plugins;
using Harmos;
using Harmos.Plugins;
using Dokime.Clients;
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
  public async Task<BackendEnvironmentResponse> Environment(global::Harmos.Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new BackendEnvironmentRequest
    {
    };
    if (at is not null) request.At = at;
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
  public async Task<ArtifactsCreateResponse> Create(global::Harmos.FileReference file, string filename, string content_type, global::Dokime.ArtifactScope? scope = null, string? request_id = null, global::Dokime.Metadata? metadata = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ArtifactsCreateRequest
    {
      File = file,
      Filename = filename,
      ContentType = content_type,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (scope is not null) request.Scope = scope;
    if (request_id is not null) request.RequestId = request_id;
    if (metadata is not null) request.Metadata = metadata;
    var response = await context.CallAsync(HostContracts.ContentArtifactsCreate, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostDocumentsResource(Context context)
{
  public async Task<global::Harmos.Receipt> AppendParts(string execution, string document, string section, global::Dokime.Section? subsection = null, global::Dokime.Card? card = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    if ((subsection is null ? 0 : 1) + (card is null ? 0 : 1) != 1)
    {
      throw new ArgumentException("Choose Exactly One part Variant");
    }
    var request = new DocumentsAppendPartsRequest
    {
      Execution = execution,
      Document = document,
      Section = section,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (subsection is not null) request.Subsection = subsection;
    if (card is not null) request.Card = card;
    if (request_id is not null) request.RequestId = request_id;
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
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.EffectsCreate, request, policy, cancellationToken);
    return response;
  }
  public async Task<EffectsGetResponse> Get(string name, global::Harmos.Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EffectsGetRequest
    {
      Name = name,
    };
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.EffectsGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<EffectsSetResponse> Set(string name, global::Dokime.Clients.EffectWritable value, ByteString expected_revision, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EffectsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expected_revision,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.EffectsSet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostEvidenceResource(Context context)
{
  public HostEvidenceLogsResource Logs { get; } = new(context);
  public HostEvidenceRecordsResource Records { get; } = new(context);
  public HostEvidenceTraceabilityResource Traceability { get; } = new(context);
  public async Task<EvidenceStatisticsResponse> Statistics(string stream, IReadOnlyList<uint> versions, global::Dokime.Window between, IReadOnlyList<global::Dokime.Clients.Output> outputs, string field, string step, global::Dokime.EvidenceScope? scope = null, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
    if (scope is not null) request.Scope = scope;
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.EvidenceStatistics, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostEvidenceLogsResource(Context context)
{
  public async Task<global::Harmos.Receipt> Append(string execution, global::Dokime.Severity severity, string message, global::Google.Protobuf.WellKnownTypes.Timestamp at, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new LogsAppendRequest
    {
      Execution = execution,
      Severity = severity,
      Message = message,
      At = at,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.EvidenceLogsAppend, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read EvidenceLogsAppend: Missing Payload");
  }
}
public sealed class HostEvidenceRecordsResource(Context context)
{
  public async Task<RecordsListResponse> List(string record_type, IReadOnlyList<uint> versions, global::Dokime.Window between, string step, global::Dokime.Clients.EvidenceResources.CapturedRecordSelection selection, global::Dokime.Clients.EvidenceResources.RecordScope? scope = null, global::Harmos.ReadCut? at = null, global::Harmos.PageRequest? page = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new RecordsListRequest
    {
      RecordType = record_type,
      Between = between,
      Step = step,
      Selection = selection,
    };
    request.Versions.Add(versions);
    if (scope is not null) request.Scope = scope;
    if (at is not null) request.At = at;
    if (page is not null) request.Page = page;
    var response = await context.CallAsync(HostContracts.EvidenceRecordsList, request, policy, cancellationToken);
    return response;
  }
  public async Task<RecordsAppendResponse> Append(string execution, string kind, ByteString payload, global::Google.Protobuf.WellKnownTypes.Timestamp source_at, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new RecordsAppendRequest
    {
      Execution = execution,
      Kind = kind,
      Payload = payload,
      SourceAt = source_at,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.EvidenceRecordsAppend, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostEvidenceTraceabilityResource(Context context)
{
  public async Task<TraceabilityGetResponse> Get(global::Dokime.EvidenceScope? scope = null, global::Harmos.PageRequest? page = null, global::Harmos.ReadCut? at = null, string? entry = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new TraceabilityGetRequest
    {
    };
    if (scope is not null) request.Scope = scope;
    if (page is not null) request.Page = page;
    if (at is not null) request.At = at;
    if (entry is not null) request.Entry = entry;
    var response = await context.CallAsync(HostContracts.EvidenceTraceabilityGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<TraceabilitySetResponse> Set(global::Dokime.TraceabilityEntry value, ByteString expected_revision, string entry, string? name = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new TraceabilitySetRequest
    {
      Value = value,
      ExpectedRevision = expected_revision,
      Entry = entry,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (name is not null) request.Name = name;
    if (request_id is not null) request.RequestId = request_id;
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
  public async Task<ProgressGetResponse> Get(string? name = null, global::Harmos.Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ProgressGetRequest
    {
    };
    if (name is not null) request.Name = name;
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.ExecutionsProgressGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<ProgressSetResponse> Set(global::Dokime.ExecutionProgressValue value, ByteString expected_revision, string? name = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ProgressSetRequest
    {
      Value = value,
      ExpectedRevision = expected_revision,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (name is not null) request.Name = name;
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.ExecutionsProgressSet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostFilesResource(Context context)
{
  public HostFilesCapturesResource Captures { get; } = new(context);
  public HostFilesUploadsResource Uploads { get; } = new(context);
  public async Task<FilesGetResponse> Get(global::Harmos.FileReference file, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesGetRequest
    {
      File = file,
    };
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.FilesGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<FilesReadResponse> Read(global::Harmos.FileReference file, ulong offset, ulong max_bytes, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesReadRequest
    {
      File = file,
      Offset = offset,
      MaxBytes = max_bytes,
    };
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.FilesRead, request, policy, cancellationToken);
    return response;
  }
  public async IAsyncEnumerable<FilesWatchResponse> Watch(ulong offset, ulong maxItemBytes, global::Harmos.FileReference? file = null, global::Harmos.Upload? upload = null, global::Harmos.ReadCut? at = null, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    if ((file is null ? 0 : 1) + (upload is null ? 0 : 1) != 1)
    {
      throw new ArgumentException("Choose Exactly One target Variant");
    }
    var request = new FilesWatchRequest
    {
      Offset = offset,
    };
    if (file is not null) request.File = file;
    if (upload is not null) request.Upload = upload;
    if (at is not null) request.At = at;
    await foreach (var response in context.WatchAsync(HostContracts.FilesWatch, request, maxItemBytes, cancellationToken))
    {
      yield return response;
    }
  }
  public async Task<global::Harmos.Receipt> Delete(global::Harmos.FileReference file, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesDeleteRequest
    {
      File = file,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesDelete, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read FilesDelete: Missing Payload");
  }
  public async Task<FilesCaptureResponse> Capture(global::Dokime.FileSource source, string output, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<ulong> Start(global::Dokime.FileSource source, string output, global::Dokime.Clients.FilesResources.CaptureMode mode, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<UploadsCreateResponse> Create(global::Harmos.FileMetadata metadata, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsCreateRequest
    {
      Metadata = metadata,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesUploadsCreate, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsGetResponse> Get(global::Harmos.Upload upload, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsGetRequest
    {
      Upload = upload,
    };
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.FilesUploadsGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsResumeResponse> Resume(global::Harmos.Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsResumeRequest
    {
      Upload = upload,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesUploadsResume, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsAppendResponse> Append(global::Harmos.Upload upload, ulong offset, ByteString data, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsAppendRequest
    {
      Upload = upload,
      Offset = offset,
      Data = data,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesUploadsAppend, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsPublishResponse> Publish(global::Harmos.Upload upload, ulong length, ByteString sha256, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsPublishRequest
    {
      Upload = upload,
      Length = length,
      Sha256 = sha256,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesUploadsPublish, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsDeleteResponse> Delete(global::Harmos.Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsDeleteRequest
    {
      Upload = upload,
    };
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesUploadsDelete, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostInvocationsResource(Context context)
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
    await foreach (var response in context.Transport.Host.Invocations.Watch(instance, maxItemBytes, cancellationToken)) { yield return response; }
  }
}
public sealed class HostPluginsResource(Context context)
{
  public HostPluginsSettingsResource Settings { get; } = new(context);
}
public sealed class HostPluginsSettingsResource(Context context)
{
  public async Task<SettingsListResponse> List(string? name = null, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Plugins.Settings.List(name, at, policy, cancellationToken);
  }
}
public sealed class HostPromptsResource(Context context)
{
  public async Task<PromptsCreateResponse> Create(string prompt, string prompt_type, IReadOnlyDictionary<string, global::Dokime.Value> values, global::Dokime.Prompt? definition = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
    if (definition is not null) request.Definition = definition;
    var response = await context.CallAsync(HostContracts.PromptsCreate, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostSecretsResource(Context context)
{
  public async Task<SecretsGetResponse> Get(string name, global::Harmos.ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new SecretsGetRequest
    {
      Name = name,
    };
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.SecretsGet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostTimeResource(Context context)
{
  public async Task<global::Google.Protobuf.WellKnownTypes.Timestamp> Get(OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Time.Get(policy, cancellationToken);
  }
  public async Task<TimeSleepResponse> Sleep(global::Google.Protobuf.WellKnownTypes.Duration duration, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    return await context.Transport.Host.Time.Sleep(duration, policy, cancellationToken);
  }
}
