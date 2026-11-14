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
using PromptsCreateRequest = global::Dokime.Clients.PromptsCreateRequest;
using PromptsCreateResponse = global::Dokime.Clients.PromptsCreateResponse;
using SecretsGetRequest = global::Dokime.Clients.SecretsGetRequest;
using SecretsGetResponse = global::Dokime.Clients.SecretsGetResponse;
using InvocationsGetRequest = global::Harmos.Clients.InvocationsGetRequest;
using InvocationsGetResponse = global::Harmos.Clients.InvocationsGetResponse;
using InvocationsCancelRequest = global::Harmos.Clients.InvocationsCancelRequest;
using InvocationsCancelResponse = global::Harmos.Clients.InvocationsCancelResponse;
using InvocationsWatchRequest = global::Harmos.Clients.InvocationsWatchRequest;
using InvocationsWatchResponse = global::Harmos.Clients.InvocationsWatchResponse;
using SettingsListRequest = global::Harmos.Clients.SettingsListRequest;
using SettingsListResponse = global::Harmos.Clients.SettingsListResponse;
using TimeGetRequest = global::Harmos.Clients.TimeGetRequest;
using TimeGetResponse = global::Harmos.Clients.TimeGetResponse;
using TimeSleepRequest = global::Harmos.Clients.TimeSleepRequest;
using TimeSleepResponse = global::Harmos.Clients.TimeSleepResponse;
namespace Dokime.Plugins;
using Harmos;
using Harmos.Plugins;
using Dokime.Clients;
public enum HostOperation
{
  BackendEnvironment,
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
  internal static readonly Operation<PromptsCreateRequest, PromptsCreateResponse> PromptsCreate = new("dokime.host.v1.Prompts/Create", Visibility.Public, watch: false);
  internal static readonly Operation<SecretsGetRequest, SecretsGetResponse> SecretsGet = new("dokime.host.v1.Secrets/Get", Visibility.Public, watch: false);
  internal static readonly Operation<InvocationsGetRequest, InvocationsGetResponse> InvocationsGet = new("harmos.host.v1.Invocations/Get", Visibility.Public, watch: false);
  internal static readonly Operation<InvocationsCancelRequest, InvocationsCancelResponse> InvocationsCancel = new("harmos.host.v1.Invocations/Cancel", Visibility.Public, watch: false);
  internal static readonly Operation<InvocationsWatchRequest, InvocationsWatchResponse> InvocationsWatch = new("harmos.host.v1.Invocations/Watch", Visibility.Public, watch: true);
  internal static readonly Operation<SettingsListRequest, SettingsListResponse> PluginsSettingsList = new("harmos.host.v1.plugins.Settings/List", Visibility.Public, watch: false);
  internal static readonly Operation<TimeGetRequest, TimeGetResponse> TimeGet = new("harmos.host.v1.Time/Get", Visibility.Public, watch: false);
  internal static readonly Operation<TimeSleepRequest, TimeSleepResponse> TimeSleep = new("harmos.host.v1.Time/Sleep", Visibility.Public, watch: false);
}
public static class HostRequirements
{
  public static Plugin Requires(this Plugin plugin, HostOperation operation) => operation switch
  {
    HostOperation.BackendEnvironment => plugin.Requires(HostContracts.BackendEnvironment),
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
    HostOperation.PromptsCreate => plugin.Requires(HostContracts.PromptsCreate),
    HostOperation.SecretsGet => plugin.Requires(HostContracts.SecretsGet),
    HostOperation.InvocationsGet => plugin.Requires(HostContracts.InvocationsGet),
    HostOperation.InvocationsCancel => plugin.Requires(HostContracts.InvocationsCancel),
    HostOperation.InvocationsWatch => plugin.Requires(HostContracts.InvocationsWatch),
    HostOperation.PluginsSettingsList => plugin.Requires(HostContracts.PluginsSettingsList),
    HostOperation.TimeGet => plugin.Requires(HostContracts.TimeGet),
    HostOperation.TimeSleep => plugin.Requires(HostContracts.TimeSleep),
    _ => throw new ArgumentOutOfRangeException(nameof(operation), "Declare Host Requirement: Unknown Operation")
  };
}
public sealed class HostRoot(Context context)
{
  public HostBackendResource Backend { get; } = new(context);
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
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.BackendEnvironment, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostDocumentsResource(Context context)
{
  public async Task<Receipt> AppendParts(string execution, string document, string section, Section? subsection = null, Card? card = null, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<EffectsGetResponse> Get(string name, Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new EffectsGetRequest
    {
      Name = name,
    };
    if (at is not null) request.At = at;
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
    if (scope is not null) request.Scope = scope;
    if (at is not null) request.At = at;
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
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
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
    if (scope is not null) request.Scope = scope;
    if (at is not null) request.At = at;
    if (page is not null) request.Page = page;
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
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
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
    if (scope is not null) request.Scope = scope;
    if (page is not null) request.Page = page;
    if (at is not null) request.At = at;
    if (entry is not null) request.Entry = entry;
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
  public async Task<ProgressGetResponse> Get(string? name = null, Position? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new ProgressGetRequest
    {
    };
    if (name is not null) request.Name = name;
    if (at is not null) request.At = at;
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
  public async Task<FilesGetResponse> Get(FileReference file, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new FilesGetRequest
    {
      File = file,
    };
    if (at is not null) request.At = at;
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
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.FilesRead, request, policy, cancellationToken);
    return response;
  }
  public async IAsyncEnumerable<FilesWatchResponse> Watch(FileReference file, ulong offset, ulong maxItemBytes, ReadCut? at = null, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new FilesWatchRequest
    {
      File = file,
      Offset = offset,
    };
    if (at is not null) request.At = at;
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
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesDelete, request, policy, cancellationToken);
    return response.Receipt ?? throw new InvalidOperationException("Read FilesDelete: Missing Payload");
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
    request.RequestId = request_id ?? Guid.NewGuid().ToString("N");
    if (request_id is not null) request.RequestId = request_id;
    var response = await context.CallAsync(HostContracts.FilesUploadsCreate, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsGetResponse> Get(Upload upload, ReadCut? at = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new UploadsGetRequest
    {
      Upload = upload,
    };
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.FilesUploadsGet, request, policy, cancellationToken);
    return response;
  }
  public async Task<UploadsResumeResponse> Resume(Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<UploadsAppendResponse> Append(Upload upload, ulong offset, ByteString data, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<UploadsPublishResponse> Publish(Upload upload, ulong length, ByteString sha256, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<UploadsDeleteResponse> Delete(Upload upload, string? request_id = null, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
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
  public async Task<InstanceState> Get(ulong instance, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new InvocationsGetRequest
    {
      Instance = instance,
    };
    var response = await context.CallAsync(HostContracts.InvocationsGet, request, policy, cancellationToken);
    return response.Invocation ?? throw new InvalidOperationException("Read InvocationsGet: Missing Payload");
  }
  public async Task<InstanceState> Cancel(ulong instance, string reason, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new InvocationsCancelRequest
    {
      Instance = instance,
      Reason = reason,
    };
    var response = await context.CallAsync(HostContracts.InvocationsCancel, request, policy, cancellationToken);
    return response.Invocation ?? throw new InvalidOperationException("Read InvocationsCancel: Missing Payload");
  }
  public async IAsyncEnumerable<InstanceState> Watch(ulong instance, ulong maxItemBytes, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new InvocationsWatchRequest
    {
      Instance = instance,
    };
    await foreach (var response in context.WatchAsync(HostContracts.InvocationsWatch, request, maxItemBytes, cancellationToken))
    {
      yield return response.Invocation ?? throw new InvalidOperationException("Read InvocationsWatch: Missing Payload");
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
    var request = new SettingsListRequest
    {
    };
    if (name is not null) request.Name = name;
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.PluginsSettingsList, request, policy, cancellationToken);
    return response;
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
    if (definition is not null) request.Definition = definition;
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
    if (at is not null) request.At = at;
    var response = await context.CallAsync(HostContracts.SecretsGet, request, policy, cancellationToken);
    return response;
  }
}
public sealed class HostTimeResource(Context context)
{
  public async Task<Timestamp> Get(OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new TimeGetRequest
    {
    };
    var response = await context.CallAsync(HostContracts.TimeGet, request, policy, cancellationToken);
    return response.Now ?? throw new InvalidOperationException("Read TimeGet: Missing Payload");
  }
  public async Task<TimeSleepResponse> Sleep(Duration duration, OperationPolicy? policy = null, CancellationToken cancellationToken = default)
  {
    var request = new TimeSleepRequest
    {
      Duration = duration,
    };
    var response = await context.CallAsync(HostContracts.TimeSleep, request, policy, cancellationToken);
    return response;
  }
}
