// @generated from protocol descriptors; do not edit.
#nullable enable
using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using System.Runtime.CompilerServices;
using Grpc.Core;
using Grpc.Net.Client;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using StorageTransferSettings = global::Harmos.StorageTransferSettings;
using Transfer = global::Harmos.Transfer;
using ArtifactScope = global::Dokime.ArtifactScope;
using AggregatesGetRequest = global::Dokime.Clients.AggregatesGetRequest;
using AggregatesGetResponse = global::Dokime.Clients.AggregatesGetResponse;
using ApplicationSettings = global::Dokime.Clients.ApplicationSettings;
using BackendAppearanceRequest = global::Dokime.Clients.BackendAppearanceRequest;
using BackendAppearanceResponse = global::Dokime.Clients.BackendAppearanceResponse;
using OverviewGetRequest = global::Dokime.Clients.BackendResources.OverviewGetRequest;
using OverviewGetResponse = global::Dokime.Clients.BackendResources.OverviewGetResponse;
using OverviewWatchRequest = global::Dokime.Clients.BackendResources.OverviewWatchRequest;
using OverviewWatchResponse = global::Dokime.Clients.BackendResources.OverviewWatchResponse;
using CasesCreateRequest = global::Dokime.Clients.CasesCreateRequest;
using CasesCreateResponse = global::Dokime.Clients.CasesCreateResponse;
using CasesDeleteRequest = global::Dokime.Clients.CasesDeleteRequest;
using CasesDeleteResponse = global::Dokime.Clients.CasesDeleteResponse;
using CasesGetRequest = global::Dokime.Clients.CasesGetRequest;
using CasesGetResponse = global::Dokime.Clients.CasesGetResponse;
using CasesListRequest = global::Dokime.Clients.CasesListRequest;
using CasesListResponse = global::Dokime.Clients.CasesListResponse;
using CasesParametersRequest = global::Dokime.Clients.CasesParametersRequest;
using CasesParametersResponse = global::Dokime.Clients.CasesParametersResponse;
using CasesWatchRequest = global::Dokime.Clients.CasesWatchRequest;
using CasesWatchResponse = global::Dokime.Clients.CasesWatchResponse;
using ArtifactsCreateRequest = global::Dokime.Clients.ContentResources.ArtifactsCreateRequest;
using ArtifactsCreateResponse = global::Dokime.Clients.ContentResources.ArtifactsCreateResponse;
using ArtifactsGetRequest = global::Dokime.Clients.ContentResources.ArtifactsGetRequest;
using ArtifactsGetResponse = global::Dokime.Clients.ContentResources.ArtifactsGetResponse;
using ArtifactsListRequest = global::Dokime.Clients.ContentResources.ArtifactsListRequest;
using ArtifactsListResponse = global::Dokime.Clients.ContentResources.ArtifactsListResponse;
using ArtifactsWatchRequest = global::Dokime.Clients.ContentResources.ArtifactsWatchRequest;
using ArtifactsWatchResponse = global::Dokime.Clients.ContentResources.ArtifactsWatchResponse;
using ExecutionFilesGetRequest = global::Dokime.Clients.ContentResources.ExecutionFilesGetRequest;
using ExecutionFilesGetResponse = global::Dokime.Clients.ContentResources.ExecutionFilesGetResponse;
using ExecutionFilesListRequest = global::Dokime.Clients.ContentResources.ExecutionFilesListRequest;
using ExecutionFilesListResponse = global::Dokime.Clients.ContentResources.ExecutionFilesListResponse;
using ExecutionFilesWatchRequest = global::Dokime.Clients.ContentResources.ExecutionFilesWatchRequest;
using ExecutionFilesWatchResponse = global::Dokime.Clients.ContentResources.ExecutionFilesWatchResponse;
using CycleFilter = global::Dokime.Clients.CycleFilter;
using CycleOrder = global::Dokime.Clients.CycleOrder;
using CycleSelection = global::Dokime.Clients.CycleSelection;
using CycleWritable = global::Dokime.Clients.CycleWritable;
using CyclesCreateRequest = global::Dokime.Clients.CyclesCreateRequest;
using CyclesCreateResponse = global::Dokime.Clients.CyclesCreateResponse;
using CyclesGetRequest = global::Dokime.Clients.CyclesGetRequest;
using CyclesGetResponse = global::Dokime.Clients.CyclesGetResponse;
using CyclesListRequest = global::Dokime.Clients.CyclesListRequest;
using CyclesListResponse = global::Dokime.Clients.CyclesListResponse;
using CyclesSealRequest = global::Dokime.Clients.CyclesSealRequest;
using CyclesSealResponse = global::Dokime.Clients.CyclesSealResponse;
using CyclesSetRequest = global::Dokime.Clients.CyclesSetRequest;
using CyclesSetResponse = global::Dokime.Clients.CyclesSetResponse;
using CyclesWatchRequest = global::Dokime.Clients.CyclesWatchRequest;
using CyclesWatchResponse = global::Dokime.Clients.CyclesWatchResponse;
using DocumentScope = global::Dokime.Clients.DocumentScope;
using DocumentsGetRequest = global::Dokime.Clients.DocumentsGetRequest;
using DocumentsGetResponse = global::Dokime.Clients.DocumentsGetResponse;
using DocumentsWatchRequest = global::Dokime.Clients.DocumentsWatchRequest;
using DocumentsWatchResponse = global::Dokime.Clients.DocumentsWatchResponse;
using EvidenceArchiveRequest = global::Dokime.Clients.EvidenceArchiveRequest;
using EvidenceArchiveResponse = global::Dokime.Clients.EvidenceArchiveResponse;
using AnnotationOwner = global::Dokime.Clients.EvidenceResources.AnnotationOwner;
using AnnotationSelection = global::Dokime.Clients.EvidenceResources.AnnotationSelection;
using BucketRequest = global::Dokime.Clients.EvidenceResources.BucketRequest;
using CapturedRecordSelection = global::Dokime.Clients.EvidenceResources.CapturedRecordSelection;
using HealthEventsListRequest = global::Dokime.Clients.EvidenceResources.HealthEventsListRequest;
using HealthEventsListResponse = global::Dokime.Clients.EvidenceResources.HealthEventsListResponse;
using HealthEventsWatchRequest = global::Dokime.Clients.EvidenceResources.HealthEventsWatchRequest;
using HealthEventsWatchResponse = global::Dokime.Clients.EvidenceResources.HealthEventsWatchResponse;
using NoteWritable = global::Dokime.Clients.EvidenceResources.NoteWritable;
using NotesCreateRequest = global::Dokime.Clients.EvidenceResources.NotesCreateRequest;
using NotesCreateResponse = global::Dokime.Clients.EvidenceResources.NotesCreateResponse;
using NotesGetRequest = global::Dokime.Clients.EvidenceResources.NotesGetRequest;
using NotesGetResponse = global::Dokime.Clients.EvidenceResources.NotesGetResponse;
using NotesListRequest = global::Dokime.Clients.EvidenceResources.NotesListRequest;
using NotesListResponse = global::Dokime.Clients.EvidenceResources.NotesListResponse;
using NotesSetRequest = global::Dokime.Clients.EvidenceResources.NotesSetRequest;
using NotesSetResponse = global::Dokime.Clients.EvidenceResources.NotesSetResponse;
using NotesWatchRequest = global::Dokime.Clients.EvidenceResources.NotesWatchRequest;
using NotesWatchResponse = global::Dokime.Clients.EvidenceResources.NotesWatchResponse;
using RecordScope = global::Dokime.Clients.EvidenceResources.RecordScope;
using RecordsListRequest = global::Dokime.Clients.EvidenceResources.RecordsListRequest;
using RecordsListResponse = global::Dokime.Clients.EvidenceResources.RecordsListResponse;
using RecordsWatchRequest = global::Dokime.Clients.EvidenceResources.RecordsWatchRequest;
using RecordsWatchResponse = global::Dokime.Clients.EvidenceResources.RecordsWatchResponse;
using SamplesListRequest = global::Dokime.Clients.EvidenceResources.SamplesListRequest;
using SamplesListResponse = global::Dokime.Clients.EvidenceResources.SamplesListResponse;
using SamplesWatchRequest = global::Dokime.Clients.EvidenceResources.SamplesWatchRequest;
using SamplesWatchResponse = global::Dokime.Clients.EvidenceResources.SamplesWatchResponse;
using TraceabilityGetRequest = global::Dokime.Clients.EvidenceResources.TraceabilityGetRequest;
using TraceabilityGetResponse = global::Dokime.Clients.EvidenceResources.TraceabilityGetResponse;
using TraceabilityWatchRequest = global::Dokime.Clients.EvidenceResources.TraceabilityWatchRequest;
using TraceabilityWatchResponse = global::Dokime.Clients.EvidenceResources.TraceabilityWatchResponse;
using EvidenceStatisticsRequest = global::Dokime.Clients.EvidenceStatisticsRequest;
using EvidenceStatisticsResponse = global::Dokime.Clients.EvidenceStatisticsResponse;
using EvidenceSubject = global::Dokime.Clients.EvidenceSubject;
using ExecutionBinding = global::Dokime.Clients.ExecutionBinding;
using ExecutionFilter = global::Dokime.Clients.ExecutionFilter;
using ExecutionMode = global::Dokime.Clients.ExecutionMode;
using ExecutionOrder = global::Dokime.Clients.ExecutionOrder;
using ExecutionParameterSet = global::Dokime.Clients.ExecutionParameterSet;
using ExecutionSelection = global::Dokime.Clients.ExecutionSelection;
using ExecutionWritable = global::Dokime.Clients.ExecutionWritable;
using ExecutionsCancelRequest = global::Dokime.Clients.ExecutionsCancelRequest;
using ExecutionsCancelResponse = global::Dokime.Clients.ExecutionsCancelResponse;
using ExecutionsCreateRequest = global::Dokime.Clients.ExecutionsCreateRequest;
using ExecutionsCreateResponse = global::Dokime.Clients.ExecutionsCreateResponse;
using ExecutionsDeleteRequest = global::Dokime.Clients.ExecutionsDeleteRequest;
using ExecutionsDeleteResponse = global::Dokime.Clients.ExecutionsDeleteResponse;
using ExecutionsGetRequest = global::Dokime.Clients.ExecutionsGetRequest;
using ExecutionsGetResponse = global::Dokime.Clients.ExecutionsGetResponse;
using ExecutionsListRequest = global::Dokime.Clients.ExecutionsListRequest;
using ExecutionsListResponse = global::Dokime.Clients.ExecutionsListResponse;
using ExecutionsParametersRequest = global::Dokime.Clients.ExecutionsParametersRequest;
using ExecutionsParametersResponse = global::Dokime.Clients.ExecutionsParametersResponse;
using CommandsCreateRequest = global::Dokime.Clients.ExecutionsResources.CommandsCreateRequest;
using CommandsCreateResponse = global::Dokime.Clients.ExecutionsResources.CommandsCreateResponse;
using CommandsGetRequest = global::Dokime.Clients.ExecutionsResources.CommandsGetRequest;
using CommandsGetResponse = global::Dokime.Clients.ExecutionsResources.CommandsGetResponse;
using ExecutionCommandAction = global::Dokime.Clients.ExecutionsResources.ExecutionCommandAction;
using ProgressGetRequest = global::Dokime.Clients.ExecutionsResources.ProgressGetRequest;
using ProgressGetResponse = global::Dokime.Clients.ExecutionsResources.ProgressGetResponse;
using ProgressWatchRequest = global::Dokime.Clients.ExecutionsResources.ProgressWatchRequest;
using ProgressWatchResponse = global::Dokime.Clients.ExecutionsResources.ProgressWatchResponse;
using SchemasGetRequest = global::Dokime.Clients.ExecutionsResources.SchemasGetRequest;
using SchemasGetResponse = global::Dokime.Clients.ExecutionsResources.SchemasGetResponse;
using ExecutionsSetRequest = global::Dokime.Clients.ExecutionsSetRequest;
using ExecutionsSetResponse = global::Dokime.Clients.ExecutionsSetResponse;
using ExecutionsStartRequest = global::Dokime.Clients.ExecutionsStartRequest;
using ExecutionsStartResponse = global::Dokime.Clients.ExecutionsStartResponse;
using ExecutionsStepsRequest = global::Dokime.Clients.ExecutionsStepsRequest;
using ExecutionsStepsResponse = global::Dokime.Clients.ExecutionsStepsResponse;
using ExecutionsStopRequest = global::Dokime.Clients.ExecutionsStopRequest;
using ExecutionsStopResponse = global::Dokime.Clients.ExecutionsStopResponse;
using ExecutionsWatchRequest = global::Dokime.Clients.ExecutionsWatchRequest;
using ExecutionsWatchResponse = global::Dokime.Clients.ExecutionsWatchResponse;
using FilesDeleteRequest = global::Dokime.Clients.FilesDeleteRequest;
using FilesDeleteResponse = global::Dokime.Clients.FilesDeleteResponse;
using FilesGetRequest = global::Dokime.Clients.FilesGetRequest;
using FilesGetResponse = global::Dokime.Clients.FilesGetResponse;
using FilesReadRequest = global::Dokime.Clients.FilesReadRequest;
using FilesReadResponse = global::Dokime.Clients.FilesReadResponse;
using UploadsAppendRequest = global::Dokime.Clients.FilesResources.UploadsAppendRequest;
using UploadsAppendResponse = global::Dokime.Clients.FilesResources.UploadsAppendResponse;
using UploadsCreateRequest = global::Dokime.Clients.FilesResources.UploadsCreateRequest;
using UploadsCreateResponse = global::Dokime.Clients.FilesResources.UploadsCreateResponse;
using UploadsDeleteRequest = global::Dokime.Clients.FilesResources.UploadsDeleteRequest;
using UploadsDeleteResponse = global::Dokime.Clients.FilesResources.UploadsDeleteResponse;
using UploadsGetRequest = global::Dokime.Clients.FilesResources.UploadsGetRequest;
using UploadsGetResponse = global::Dokime.Clients.FilesResources.UploadsGetResponse;
using UploadsPublishRequest = global::Dokime.Clients.FilesResources.UploadsPublishRequest;
using UploadsPublishResponse = global::Dokime.Clients.FilesResources.UploadsPublishResponse;
using UploadsResumeRequest = global::Dokime.Clients.FilesResources.UploadsResumeRequest;
using UploadsResumeResponse = global::Dokime.Clients.FilesResources.UploadsResumeResponse;
using FilesWatchRequest = global::Dokime.Clients.FilesWatchRequest;
using FilesWatchResponse = global::Dokime.Clients.FilesWatchResponse;
using CaptureKind = global::Dokime.Clients.MediaResources.CaptureKind;
using CapturesCreateRequest = global::Dokime.Clients.MediaResources.CapturesCreateRequest;
using CapturesCreateResponse = global::Dokime.Clients.MediaResources.CapturesCreateResponse;
using CapturesGetRequest = global::Dokime.Clients.MediaResources.CapturesGetRequest;
using CapturesGetResponse = global::Dokime.Clients.MediaResources.CapturesGetResponse;
using CapturesStopRequest = global::Dokime.Clients.MediaResources.CapturesStopRequest;
using CapturesStopResponse = global::Dokime.Clients.MediaResources.CapturesStopResponse;
using CapturesWatchRequest = global::Dokime.Clients.MediaResources.CapturesWatchRequest;
using CapturesWatchResponse = global::Dokime.Clients.MediaResources.CapturesWatchResponse;
using RecordingAudio = global::Dokime.Clients.MediaResources.RecordingAudio;
using Output = global::Dokime.Clients.Output;
using PlotRequest = global::Dokime.Clients.PlotsResources.PlotRequest;
using PlotSettingsValue = global::Dokime.Clients.PlotsResources.PlotSettingsValue;
using DokimeClientsPlotsResourcesSettingsGetRequest = global::Dokime.Clients.PlotsResources.SettingsGetRequest;
using DokimeClientsPlotsResourcesSettingsGetResponse = global::Dokime.Clients.PlotsResources.SettingsGetResponse;
using DokimeClientsPlotsResourcesSettingsSetRequest = global::Dokime.Clients.PlotsResources.SettingsSetRequest;
using DokimeClientsPlotsResourcesSettingsSetResponse = global::Dokime.Clients.PlotsResources.SettingsSetResponse;
using SnapshotsListRequest = global::Dokime.Clients.PlotsResources.SnapshotsListRequest;
using SnapshotsListResponse = global::Dokime.Clients.PlotsResources.SnapshotsListResponse;
using SnapshotsWatchRequest = global::Dokime.Clients.PlotsResources.SnapshotsWatchRequest;
using SnapshotsWatchResponse = global::Dokime.Clients.PlotsResources.SnapshotsWatchResponse;
using DokimeClientsPluginsResourcesSecretsListRequest = global::Dokime.Clients.PluginsResources.SecretsListRequest;
using DokimeClientsPluginsResourcesSecretsListResponse = global::Dokime.Clients.PluginsResources.SecretsListResponse;
using PromptDefinitionsGetRequest = global::Dokime.Clients.PromptDefinitionsGetRequest;
using PromptDefinitionsGetResponse = global::Dokime.Clients.PromptDefinitionsGetResponse;
using PromptDefinitionsListRequest = global::Dokime.Clients.PromptDefinitionsListRequest;
using PromptDefinitionsListResponse = global::Dokime.Clients.PromptDefinitionsListResponse;
using PromptsGetRequest = global::Dokime.Clients.PromptsGetRequest;
using PromptsGetResponse = global::Dokime.Clients.PromptsGetResponse;
using PromptsListRequest = global::Dokime.Clients.PromptsListRequest;
using PromptsListResponse = global::Dokime.Clients.PromptsListResponse;
using AnswersCreateRequest = global::Dokime.Clients.PromptsResources.AnswersCreateRequest;
using AnswersCreateResponse = global::Dokime.Clients.PromptsResources.AnswersCreateResponse;
using PromptsWatchRequest = global::Dokime.Clients.PromptsWatchRequest;
using PromptsWatchResponse = global::Dokime.Clients.PromptsWatchResponse;
using QueryAggregate = global::Dokime.Clients.QueryAggregate;
using QueryFilter = global::Dokime.Clients.QueryFilter;
using QuerySource = global::Dokime.Clients.QuerySource;
using RecordSchemasGetRequest = global::Dokime.Clients.RecordSchemasGetRequest;
using RecordSchemasGetResponse = global::Dokime.Clients.RecordSchemasGetResponse;
using RecordSchemasListRequest = global::Dokime.Clients.RecordSchemasListRequest;
using RecordSchemasListResponse = global::Dokime.Clients.RecordSchemasListResponse;
using ReportSubject = global::Dokime.Clients.ReportSubject;
using ReportsCreateRequest = global::Dokime.Clients.ReportsCreateRequest;
using ReportsCreateResponse = global::Dokime.Clients.ReportsCreateResponse;
using ReportsGetRequest = global::Dokime.Clients.ReportsGetRequest;
using ReportsGetResponse = global::Dokime.Clients.ReportsGetResponse;
using ReportsPublishRequest = global::Dokime.Clients.ReportsPublishRequest;
using ReportsPublishResponse = global::Dokime.Clients.ReportsPublishResponse;
using SecretImportChoice = global::Dokime.Clients.SecretImportChoice;
using SecretWritable = global::Dokime.Clients.SecretWritable;
using SecretsDeleteRequest = global::Dokime.Clients.SecretsDeleteRequest;
using SecretsDeleteResponse = global::Dokime.Clients.SecretsDeleteResponse;
using SecretsExportRequest = global::Dokime.Clients.SecretsExportRequest;
using SecretsExportResponse = global::Dokime.Clients.SecretsExportResponse;
using SecretsGetRequest = global::Dokime.Clients.SecretsGetRequest;
using SecretsGetResponse = global::Dokime.Clients.SecretsGetResponse;
using SecretsImportRequest = global::Dokime.Clients.SecretsImportRequest;
using SecretsImportResponse = global::Dokime.Clients.SecretsImportResponse;
using DokimeClientsSecretsListRequest = global::Dokime.Clients.SecretsListRequest;
using DokimeClientsSecretsListResponse = global::Dokime.Clients.SecretsListResponse;
using SecretsSetRequest = global::Dokime.Clients.SecretsSetRequest;
using SecretsSetResponse = global::Dokime.Clients.SecretsSetResponse;
using SecretsWatchRequest = global::Dokime.Clients.SecretsWatchRequest;
using SecretsWatchResponse = global::Dokime.Clients.SecretsWatchResponse;
using SelectionPlan = global::Dokime.Clients.SelectionPlan;
using DokimeClientsSettingsGetRequest = global::Dokime.Clients.SettingsGetRequest;
using DokimeClientsSettingsGetResponse = global::Dokime.Clients.SettingsGetResponse;
using SettingsSchemaRequest = global::Dokime.Clients.SettingsSchemaRequest;
using SettingsSchemaResponse = global::Dokime.Clients.SettingsSchemaResponse;
using DokimeClientsSettingsSetRequest = global::Dokime.Clients.SettingsSetRequest;
using DokimeClientsSettingsSetResponse = global::Dokime.Clients.SettingsSetResponse;
using SourcesGetRequest = global::Dokime.Clients.StorageResources.SourcesGetRequest;
using SourcesGetResponse = global::Dokime.Clients.StorageResources.SourcesGetResponse;
using SourcesSetRequest = global::Dokime.Clients.StorageResources.SourcesSetRequest;
using SourcesSetResponse = global::Dokime.Clients.StorageResources.SourcesSetResponse;
using StorageSources = global::Dokime.Clients.StorageResources.StorageSources;
using StreamSchemasGetRequest = global::Dokime.Clients.StreamSchemasGetRequest;
using StreamSchemasGetResponse = global::Dokime.Clients.StreamSchemasGetResponse;
using StreamSchemasListRequest = global::Dokime.Clients.StreamSchemasListRequest;
using StreamSchemasListResponse = global::Dokime.Clients.StreamSchemasListResponse;
using SuitesGetRequest = global::Dokime.Clients.SuitesGetRequest;
using SuitesGetResponse = global::Dokime.Clients.SuitesGetResponse;
using SuitesListRequest = global::Dokime.Clients.SuitesListRequest;
using SuitesListResponse = global::Dokime.Clients.SuitesListResponse;
using EvidenceScope = global::Dokime.EvidenceScope;
using MapValue = global::Dokime.MapValue;
using Metadata = global::Dokime.Metadata;
using Origin = global::Dokime.Origin;
using Window = global::Dokime.Window;
using Duration = global::Google.Protobuf.WellKnownTypes.Duration;
using Timestamp = global::Google.Protobuf.WellKnownTypes.Timestamp;
using ListenerGetResponse = global::Harmos.Clients.ListenerGetResponse;
using ListenerSetResponse = global::Harmos.Clients.ListenerSetResponse;
using ListenerValue = global::Harmos.Clients.ListenerValue;
using PairingsCreateResponse = global::Harmos.Clients.PairingsCreateResponse;
using PairingsDeleteResponse = global::Harmos.Clients.PairingsDeleteResponse;
using PairingsListResponse = global::Harmos.Clients.PairingsListResponse;
using PluginEntry = global::Harmos.Clients.PluginEntry;
using PluginPlacement = global::Harmos.Clients.PluginPlacement;
using PluginValue = global::Harmos.Clients.PluginValue;
using PluginsCreateResponse = global::Harmos.Clients.PluginsCreateResponse;
using PluginsGetResponse = global::Harmos.Clients.PluginsGetResponse;
using PluginsListResponse = global::Harmos.Clients.PluginsListResponse;
using PluginsReplaceResponse = global::Harmos.Clients.PluginsReplaceResponse;
using SettingsListResponse = global::Harmos.Clients.SettingsListResponse;
using HarmosHostPluginsResourcesSettingsSetResponse = global::Harmos.Clients.SettingsSetResponse;
using PluginsSetResponse = global::Harmos.Clients.PluginsSetResponse;
using PluginsWatchResponse = global::Harmos.Clients.PluginsWatchResponse;
using SchemaDescription = global::Harmos.Clients.SchemaDescription;
using StatusGetResponse = global::Harmos.Clients.StatusGetResponse;
using DiagnosticsGetResponse = global::Harmos.Clients.DiagnosticsGetResponse;
using LogsListResponse = global::Harmos.Clients.LogsListResponse;
using LogsWatchResponse = global::Harmos.Clients.LogsWatchResponse;
using TraceEventsListResponse = global::Harmos.Clients.TraceEventsListResponse;
using TraceEventsWatchResponse = global::Harmos.Clients.TraceEventsWatchResponse;
using StatusWatchResponse = global::Harmos.Clients.StatusWatchResponse;
using StreamIdentity = global::Harmos.Clients.StreamIdentity;
using TimeSleepResponse = global::Harmos.Clients.TimeSleepResponse;
using DiscoveredOperation = global::Harmos.DiscoveredOperation;
using FileMetadata = global::Harmos.FileMetadata;
using FileReference = global::Harmos.FileReference;
using InstanceState = global::Harmos.InstanceState;
using PageRequest = global::Harmos.PageRequest;
using Position = global::Harmos.Position;
using ReadCut = global::Harmos.ReadCut;
using Receipt = global::Harmos.Receipt;
using SettingValues = global::Harmos.SettingValues;
using Upload = global::Harmos.Upload;
namespace Dokime;

public sealed class Client : IDisposable
{
  private GrpcChannel? ownedChannel;
  public AccessResource Access { get; }
  public AggregatesResource Aggregates { get; }
  public BackendResource Backend { get; }
  public CasesResource Cases { get; }
  public ContentResource Content { get; }
  public CyclesResource Cycles { get; }
  public DocumentsResource Documents { get; }
  public EvidenceResource Evidence { get; }
  public ExecutionsResource Executions { get; }
  public FilesResource Files { get; }
  public InvocationsResource Invocations { get; }
  public MediaResource Media { get; }
  public OperationsResource Operations { get; }
  public PlotsResource Plots { get; }
  public PluginsResource Plugins { get; }
  public PromptDefinitionsResource PromptDefinitions { get; }
  public PromptsResource Prompts { get; }
  public RecordSchemasResource RecordSchemas { get; }
  public ReportsResource Reports { get; }
  public SchemasResource Schemas { get; }
  public SecretsResource Secrets { get; }
  public SettingsResource Settings { get; }
  public StatusResource Status { get; }
  public StorageResource Storage { get; }
  public StreamSchemasResource StreamSchemas { get; }
  public StreamsResource Streams { get; }
  public SuitesResource Suites { get; }
  public TimeResource Time { get; }
  public Client(CallInvoker invoker)
  {
    Access = new AccessResource(invoker);
    Aggregates = new AggregatesResource(invoker);
    Backend = new BackendResource(invoker);
    Cases = new CasesResource(invoker);
    Content = new ContentResource(invoker);
    Cycles = new CyclesResource(invoker);
    Documents = new DocumentsResource(invoker);
    Evidence = new EvidenceResource(invoker);
    Executions = new ExecutionsResource(invoker);
    Files = new FilesResource(invoker);
    Invocations = new InvocationsResource(invoker);
    Media = new MediaResource(invoker);
    Operations = new OperationsResource(invoker);
    Plots = new PlotsResource(invoker);
    Plugins = new PluginsResource(invoker);
    PromptDefinitions = new PromptDefinitionsResource(invoker);
    Prompts = new PromptsResource(invoker);
    RecordSchemas = new RecordSchemasResource(invoker);
    Reports = new ReportsResource(invoker);
    Schemas = new SchemasResource(invoker);
    Secrets = new SecretsResource(invoker);
    Settings = new SettingsResource(invoker);
    Status = new StatusResource(invoker);
    Storage = new StorageResource(invoker);
    StreamSchemas = new StreamSchemasResource(invoker);
    Streams = new StreamsResource(invoker);
    Suites = new SuitesResource(invoker);
    Time = new TimeResource(invoker);
  }
  public static Client Connect(string address, StorageTransferSettings? transfer = null, int? messageBytes = null)
  {
    var maximum = Transfer.MessageBytes(transfer, messageBytes);
    var channel = GrpcChannel.ForAddress(address, new() { MaxSendMessageSize = maximum, MaxReceiveMessageSize = maximum });
    return new Client(channel.CreateCallInvoker()) { ownedChannel = channel };
  }
  public void Dispose() => ownedChannel?.Dispose();
}
public sealed class AccessResource
{
  public AccessCertificatesResource Certificates { get; }
  public AccessListenerResource Listener { get; }
  public AccessPairingsResource Pairings { get; }
  public AccessResource(CallInvoker invoker)
  {
    Certificates = new AccessCertificatesResource(invoker);
    Listener = new AccessListenerResource(invoker);
    Pairings = new AccessPairingsResource(invoker);
  }
}
public sealed class AccessCertificatesResource
{
  private readonly CallInvoker invoker;
  public AccessCertificatesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<string> Create(CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Access.Certificates.Create(context);
  }
}
public sealed class AccessListenerResource
{
  private readonly CallInvoker invoker;
  public AccessListenerResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ListenerGetResponse> Get(CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Access.Listener.Get(context);
  }
  public async Task<ListenerSetResponse> Set(ListenerValue value, ByteString expectedRevision, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Access.Listener.Set(value, expectedRevision, context);
  }
}
public sealed class AccessPairingsResource
{
  private readonly CallInvoker invoker;
  public AccessPairingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<PairingsCreateResponse> Create(string label, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Access.Pairings.Create(label, context);
  }
  public async Task<PairingsListResponse> List(Position? at = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Access.Pairings.List(at, context);
  }
  public async Task<PairingsDeleteResponse> Delete(string name, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Access.Pairings.Delete(name, context);
  }
}
public sealed class AggregatesResource
{
  private readonly CallInvoker invoker;
  public AggregatesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<AggregatesGetResponse> Get(QuerySource source, IReadOnlyList<QueryFilter> filters, IReadOnlyList<string> groupBy, IReadOnlyList<QueryAggregate> aggregates, uint limit, ReadCut? at = null, CallOptions context = default)
  {
    var request = new AggregatesGetRequest
    {
      Source = source,
      Limit = limit,
    };
    request.Filters.Add(filters);
    request.GroupBy.Add(groupBy);
    request.Aggregates.Add(aggregates);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<AggregatesGetRequest, AggregatesGetResponse>(MethodType.Unary, "dokime.host.v1.Aggregates", "Get",
        Marshallers.Create<AggregatesGetRequest>(value => value.ToByteArray(), bytes => AggregatesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<AggregatesGetResponse>(value => value.ToByteArray(), bytes => AggregatesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    AggregatesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call aggregates.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class BackendResource
{
  private readonly CallInvoker invoker;
  public BackendOverviewResource Overview { get; }
  public BackendResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Overview = new BackendOverviewResource(invoker);
  }
  public async Task<BackendAppearanceResponse> Appearance(ReadCut? at = null, CallOptions context = default)
  {
    var request = new BackendAppearanceRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<BackendAppearanceRequest, BackendAppearanceResponse>(MethodType.Unary, "dokime.host.v1.Backend", "Appearance",
        Marshallers.Create<BackendAppearanceRequest>(value => value.ToByteArray(), bytes => BackendAppearanceRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<BackendAppearanceResponse>(value => value.ToByteArray(), bytes => BackendAppearanceResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    BackendAppearanceResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call backend.appearance: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class BackendOverviewResource
{
  private readonly CallInvoker invoker;
  public BackendOverviewResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<OverviewGetResponse> Get(ReadCut? at = null, CallOptions context = default)
  {
    var request = new OverviewGetRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<OverviewGetRequest, OverviewGetResponse>(MethodType.Unary, "dokime.host.v1.backend.Overview", "Get",
        Marshallers.Create<OverviewGetRequest>(value => value.ToByteArray(), bytes => OverviewGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<OverviewGetResponse>(value => value.ToByteArray(), bytes => OverviewGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    OverviewGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call backend.overview.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<OverviewWatchResponse> Watch(ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new OverviewWatchRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<OverviewWatchRequest, OverviewWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.backend.Overview", "Watch",
        Marshallers.Create<OverviewWatchRequest>(value => value.ToByteArray(), bytes => OverviewWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<OverviewWatchResponse>(value => value.ToByteArray(), bytes => OverviewWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call backend.overview.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class CasesResource
{
  private readonly CallInvoker invoker;
  public CasesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<CasesCreateResponse> Create(SelectionPlan plan, bool validateOnly, string? requestId = null, CallOptions context = default)
  {
    var request = new CasesCreateRequest
    {
      Plan = plan,
      ValidateOnly = validateOnly,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<CasesCreateRequest, CasesCreateResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Create",
        Marshallers.Create<CasesCreateRequest>(value => value.ToByteArray(), bytes => CasesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CasesCreateResponse>(value => value.ToByteArray(), bytes => CasesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CasesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CasesGetResponse> Get(string name, string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new CasesGetRequest
    {
      Name = name,
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CasesGetRequest, CasesGetResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Get",
        Marshallers.Create<CasesGetRequest>(value => value.ToByteArray(), bytes => CasesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CasesGetResponse>(value => value.ToByteArray(), bytes => CasesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CasesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CasesListResponse> List(string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new CasesListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CasesListRequest, CasesListResponse>(MethodType.Unary, "dokime.host.v1.Cases", "List",
        Marshallers.Create<CasesListRequest>(value => value.ToByteArray(), bytes => CasesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CasesListResponse>(value => value.ToByteArray(), bytes => CasesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CasesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<CasesWatchResponse> Watch(string parent, ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new CasesWatchRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CasesWatchRequest, CasesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Cases", "Watch",
        Marshallers.Create<CasesWatchRequest>(value => value.ToByteArray(), bytes => CasesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CasesWatchResponse>(value => value.ToByteArray(), bytes => CasesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call cases.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<Receipt> Delete(string name, string parent, string? requestId = null, CallOptions context = default)
  {
    var request = new CasesDeleteRequest
    {
      Name = name,
      Parent = parent,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<CasesDeleteRequest, CasesDeleteResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Delete",
        Marshallers.Create<CasesDeleteRequest>(value => value.ToByteArray(), bytes => CasesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CasesDeleteResponse>(value => value.ToByteArray(), bytes => CasesDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CasesDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.delete: " + error.Status.Detail), error.Trailers);
    }
    return response.Receipt ?? throw new InvalidOperationException("Read Cases.Delete: Missing Payload");
  }
  public async Task<CasesParametersResponse> Parameters(string name, string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new CasesParametersRequest
    {
      Name = name,
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CasesParametersRequest, CasesParametersResponse>(MethodType.Unary, "dokime.host.v1.Cases", "Parameters",
        Marshallers.Create<CasesParametersRequest>(value => value.ToByteArray(), bytes => CasesParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CasesParametersResponse>(value => value.ToByteArray(), bytes => CasesParametersResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CasesParametersResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cases.parameters: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class ContentResource
{
  public ContentArtifactsResource Artifacts { get; }
  public ContentExecutionFilesResource ExecutionFiles { get; }
  public ContentResource(CallInvoker invoker)
  {
    Artifacts = new ContentArtifactsResource(invoker);
    ExecutionFiles = new ContentExecutionFilesResource(invoker);
  }
}
public sealed class ContentArtifactsResource
{
  private readonly CallInvoker invoker;
  public ContentArtifactsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ArtifactsGetResponse> Get(string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new ArtifactsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ArtifactsGetRequest, ArtifactsGetResponse>(MethodType.Unary, "dokime.host.v1.content.Artifacts", "Get",
        Marshallers.Create<ArtifactsGetRequest>(value => value.ToByteArray(), bytes => ArtifactsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ArtifactsGetResponse>(value => value.ToByteArray(), bytes => ArtifactsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ArtifactsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ArtifactsCreateResponse> Create(FileReference file, ArtifactScope scope, string filename, string contentType, string? requestId = null, Metadata? metadata = null, CallOptions context = default)
  {
    var request = new ArtifactsCreateRequest
    {
      File = file,
      Scope = scope,
      Filename = filename,
      ContentType = contentType,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    if (metadata is not null)
    {
      request.Metadata = metadata;
    }
    var method = new Method<ArtifactsCreateRequest, ArtifactsCreateResponse>(MethodType.Unary, "dokime.host.v1.content.Artifacts", "Create",
        Marshallers.Create<ArtifactsCreateRequest>(value => value.ToByteArray(), bytes => ArtifactsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ArtifactsCreateResponse>(value => value.ToByteArray(), bytes => ArtifactsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ArtifactsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ArtifactsListResponse> List(ArtifactScope scope, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new ArtifactsListRequest
    {
      Scope = scope,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<ArtifactsListRequest, ArtifactsListResponse>(MethodType.Unary, "dokime.host.v1.content.Artifacts", "List",
        Marshallers.Create<ArtifactsListRequest>(value => value.ToByteArray(), bytes => ArtifactsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ArtifactsListResponse>(value => value.ToByteArray(), bytes => ArtifactsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ArtifactsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<ArtifactsWatchResponse> Watch(ArtifactScope scope, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new ArtifactsWatchRequest
    {
      Scope = scope,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<ArtifactsWatchRequest, ArtifactsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.content.Artifacts", "Watch",
        Marshallers.Create<ArtifactsWatchRequest>(value => value.ToByteArray(), bytes => ArtifactsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ArtifactsWatchResponse>(value => value.ToByteArray(), bytes => ArtifactsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call content.artifacts.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class ContentExecutionFilesResource
{
  private readonly CallInvoker invoker;
  public ContentExecutionFilesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ExecutionFilesGetResponse> Get(string parent, string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new ExecutionFilesGetRequest
    {
      Parent = parent,
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ExecutionFilesGetRequest, ExecutionFilesGetResponse>(MethodType.Unary, "dokime.host.v1.content.ExecutionFiles", "Get",
        Marshallers.Create<ExecutionFilesGetRequest>(value => value.ToByteArray(), bytes => ExecutionFilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionFilesGetResponse>(value => value.ToByteArray(), bytes => ExecutionFilesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionFilesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.execution_files.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionFilesListResponse> List(string parent, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new ExecutionFilesListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<ExecutionFilesListRequest, ExecutionFilesListResponse>(MethodType.Unary, "dokime.host.v1.content.ExecutionFiles", "List",
        Marshallers.Create<ExecutionFilesListRequest>(value => value.ToByteArray(), bytes => ExecutionFilesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionFilesListResponse>(value => value.ToByteArray(), bytes => ExecutionFilesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionFilesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call content.execution_files.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<ExecutionFilesWatchResponse> Watch(string parent, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new ExecutionFilesWatchRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<ExecutionFilesWatchRequest, ExecutionFilesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.content.ExecutionFiles", "Watch",
        Marshallers.Create<ExecutionFilesWatchRequest>(value => value.ToByteArray(), bytes => ExecutionFilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionFilesWatchResponse>(value => value.ToByteArray(), bytes => ExecutionFilesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call content.execution_files.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class CyclesResource
{
  private readonly CallInvoker invoker;
  public CyclesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<CyclesGetResponse> Get(string name, Position? at = null, CallOptions context = default)
  {
    var request = new CyclesGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CyclesGetRequest, CyclesGetResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Get",
        Marshallers.Create<CyclesGetRequest>(value => value.ToByteArray(), bytes => CyclesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CyclesGetResponse>(value => value.ToByteArray(), bytes => CyclesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CyclesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CyclesSetResponse> Set(string name, CycleWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new CyclesSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<CyclesSetRequest, CyclesSetResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Set",
        Marshallers.Create<CyclesSetRequest>(value => value.ToByteArray(), bytes => CyclesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CyclesSetResponse>(value => value.ToByteArray(), bytes => CyclesSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CyclesSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CyclesListResponse> List(CycleSelection selection, CycleFilter filter, CycleOrder order, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new CyclesListRequest
    {
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<CyclesListRequest, CyclesListResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "List",
        Marshallers.Create<CyclesListRequest>(value => value.ToByteArray(), bytes => CyclesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CyclesListResponse>(value => value.ToByteArray(), bytes => CyclesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CyclesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<CyclesWatchResponse> Watch(CycleSelection selection, CycleFilter filter, CycleOrder order, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new CyclesWatchRequest
    {
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<CyclesWatchRequest, CyclesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Cycles", "Watch",
        Marshallers.Create<CyclesWatchRequest>(value => value.ToByteArray(), bytes => CyclesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CyclesWatchResponse>(value => value.ToByteArray(), bytes => CyclesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call cycles.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<CyclesCreateResponse> Create(IReadOnlyList<Origin> origins, Metadata metadata, bool continueAutomatically, string sourceCycle, string? requestId = null, CallOptions context = default)
  {
    var request = new CyclesCreateRequest
    {
      Metadata = metadata,
      ContinueAutomatically = continueAutomatically,
      SourceCycle = sourceCycle,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Origins.Add(origins);
    var method = new Method<CyclesCreateRequest, CyclesCreateResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Create",
        Marshallers.Create<CyclesCreateRequest>(value => value.ToByteArray(), bytes => CyclesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CyclesCreateResponse>(value => value.ToByteArray(), bytes => CyclesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CyclesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CyclesSealResponse> Seal(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new CyclesSealRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<CyclesSealRequest, CyclesSealResponse>(MethodType.Unary, "dokime.host.v1.Cycles", "Seal",
        Marshallers.Create<CyclesSealRequest>(value => value.ToByteArray(), bytes => CyclesSealRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CyclesSealResponse>(value => value.ToByteArray(), bytes => CyclesSealResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CyclesSealResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call cycles.seal: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class DocumentsResource
{
  private readonly CallInvoker invoker;
  public DocumentsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<DocumentsGetResponse> Get(DocumentScope scope, string documentId, bool structureOnly, IReadOnlyList<string> cardIds, string pageToken, ulong maxBytes, uint maxFrames, ReadCut? at = null, CallOptions context = default)
  {
    var request = new DocumentsGetRequest
    {
      Scope = scope,
      DocumentId = documentId,
      StructureOnly = structureOnly,
      PageToken = pageToken,
      MaxBytes = maxBytes,
      MaxFrames = maxFrames,
    };
    request.CardIds.Add(cardIds);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<DocumentsGetRequest, DocumentsGetResponse>(MethodType.Unary, "dokime.host.v1.Documents", "Get",
        Marshallers.Create<DocumentsGetRequest>(value => value.ToByteArray(), bytes => DocumentsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DocumentsGetResponse>(value => value.ToByteArray(), bytes => DocumentsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DocumentsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call documents.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<DocumentsWatchResponse> Watch(DocumentScope scope, string documentId, bool structureOnly, IReadOnlyList<string> cardIds, string pageToken, ulong maxBytes, uint maxFrames, ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new DocumentsWatchRequest
    {
      Scope = scope,
      DocumentId = documentId,
      StructureOnly = structureOnly,
      PageToken = pageToken,
      MaxBytes = maxBytes,
      MaxFrames = maxFrames,
    };
    request.CardIds.Add(cardIds);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<DocumentsWatchRequest, DocumentsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Documents", "Watch",
        Marshallers.Create<DocumentsWatchRequest>(value => value.ToByteArray(), bytes => DocumentsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DocumentsWatchResponse>(value => value.ToByteArray(), bytes => DocumentsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call documents.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class EvidenceResource
{
  private readonly CallInvoker invoker;
  public EvidenceHealthEventsResource HealthEvents { get; }
  public EvidenceNotesResource Notes { get; }
  public EvidenceRecordsResource Records { get; }
  public EvidenceSamplesResource Samples { get; }
  public EvidenceTraceabilityResource Traceability { get; }
  public EvidenceResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    HealthEvents = new EvidenceHealthEventsResource(invoker);
    Notes = new EvidenceNotesResource(invoker);
    Records = new EvidenceRecordsResource(invoker);
    Samples = new EvidenceSamplesResource(invoker);
    Traceability = new EvidenceTraceabilityResource(invoker);
  }
  public async Task<EvidenceStatisticsResponse> Statistics(EvidenceScope scope, string stream, IReadOnlyList<uint> versions, Window between, IReadOnlyList<Output> outputs, string field, string step, ReadCut? at = null, CallOptions context = default)
  {
    var request = new EvidenceStatisticsRequest
    {
      Scope = scope,
      Stream = stream,
      Between = between,
      Field = field,
      Step = step,
    };
    request.Versions.Add(versions);
    request.Outputs.Add(outputs);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<EvidenceStatisticsRequest, EvidenceStatisticsResponse>(MethodType.Unary, "dokime.host.v1.Evidence", "Statistics",
        Marshallers.Create<EvidenceStatisticsRequest>(value => value.ToByteArray(), bytes => EvidenceStatisticsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<EvidenceStatisticsResponse>(value => value.ToByteArray(), bytes => EvidenceStatisticsResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    EvidenceStatisticsResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.statistics: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<EvidenceArchiveResponse> Archive(EvidenceSubject subject, ReadCut? at = null, CallOptions context = default)
  {
    var request = new EvidenceArchiveRequest
    {
      Subject = subject,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<EvidenceArchiveRequest, EvidenceArchiveResponse>(MethodType.Unary, "dokime.host.v1.Evidence", "Archive",
        Marshallers.Create<EvidenceArchiveRequest>(value => value.ToByteArray(), bytes => EvidenceArchiveRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<EvidenceArchiveResponse>(value => value.ToByteArray(), bytes => EvidenceArchiveResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    EvidenceArchiveResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.archive: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class EvidenceHealthEventsResource
{
  private readonly CallInvoker invoker;
  public EvidenceHealthEventsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<HealthEventsListResponse> List(AnnotationSelection selection, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new HealthEventsListRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<HealthEventsListRequest, HealthEventsListResponse>(MethodType.Unary, "dokime.host.v1.evidence.HealthEvents", "List",
        Marshallers.Create<HealthEventsListRequest>(value => value.ToByteArray(), bytes => HealthEventsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<HealthEventsListResponse>(value => value.ToByteArray(), bytes => HealthEventsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    HealthEventsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.health_events.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<HealthEventsWatchResponse> Watch(AnnotationSelection selection, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new HealthEventsWatchRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<HealthEventsWatchRequest, HealthEventsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.HealthEvents", "Watch",
        Marshallers.Create<HealthEventsWatchRequest>(value => value.ToByteArray(), bytes => HealthEventsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<HealthEventsWatchResponse>(value => value.ToByteArray(), bytes => HealthEventsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.health_events.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class EvidenceNotesResource
{
  private readonly CallInvoker invoker;
  public EvidenceNotesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<NotesGetResponse> Get(string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new NotesGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<NotesGetRequest, NotesGetResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "Get",
        Marshallers.Create<NotesGetRequest>(value => value.ToByteArray(), bytes => NotesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<NotesGetResponse>(value => value.ToByteArray(), bytes => NotesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    NotesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<NotesSetResponse> Set(string name, NoteWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new NotesSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<NotesSetRequest, NotesSetResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "Set",
        Marshallers.Create<NotesSetRequest>(value => value.ToByteArray(), bytes => NotesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<NotesSetResponse>(value => value.ToByteArray(), bytes => NotesSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    NotesSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<NotesListResponse> List(AnnotationSelection selection, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new NotesListRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<NotesListRequest, NotesListResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "List",
        Marshallers.Create<NotesListRequest>(value => value.ToByteArray(), bytes => NotesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<NotesListResponse>(value => value.ToByteArray(), bytes => NotesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    NotesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<NotesWatchResponse> Watch(AnnotationSelection selection, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new NotesWatchRequest
    {
      Selection = selection,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<NotesWatchRequest, NotesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Notes", "Watch",
        Marshallers.Create<NotesWatchRequest>(value => value.ToByteArray(), bytes => NotesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<NotesWatchResponse>(value => value.ToByteArray(), bytes => NotesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<NotesCreateResponse> Create(AnnotationOwner owner, NoteWritable value, string? requestId = null, CallOptions context = default)
  {
    var request = new NotesCreateRequest
    {
      Owner = owner,
      Value = value,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<NotesCreateRequest, NotesCreateResponse>(MethodType.Unary, "dokime.host.v1.evidence.Notes", "Create",
        Marshallers.Create<NotesCreateRequest>(value => value.ToByteArray(), bytes => NotesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<NotesCreateResponse>(value => value.ToByteArray(), bytes => NotesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    NotesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.notes.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class EvidenceRecordsResource
{
  private readonly CallInvoker invoker;
  public EvidenceRecordsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<RecordsListResponse> List(RecordScope scope, string recordType, IReadOnlyList<uint> versions, Window between, string step, CapturedRecordSelection selection, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new RecordsListRequest
    {
      Scope = scope,
      RecordType = recordType,
      Between = between,
      Step = step,
      Selection = selection,
    };
    request.Versions.Add(versions);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<RecordsListRequest, RecordsListResponse>(MethodType.Unary, "dokime.host.v1.evidence.Records", "List",
        Marshallers.Create<RecordsListRequest>(value => value.ToByteArray(), bytes => RecordsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<RecordsListResponse>(value => value.ToByteArray(), bytes => RecordsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    RecordsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.records.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<RecordsWatchResponse> Watch(RecordScope scope, string recordType, IReadOnlyList<uint> versions, Window between, string step, CapturedRecordSelection selection, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new RecordsWatchRequest
    {
      Scope = scope,
      RecordType = recordType,
      Between = between,
      Step = step,
      Selection = selection,
    };
    request.Versions.Add(versions);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<RecordsWatchRequest, RecordsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Records", "Watch",
        Marshallers.Create<RecordsWatchRequest>(value => value.ToByteArray(), bytes => RecordsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<RecordsWatchResponse>(value => value.ToByteArray(), bytes => RecordsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.records.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class EvidenceSamplesResource
{
  private readonly CallInvoker invoker;
  public EvidenceSamplesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SamplesListResponse> List(EvidenceScope scope, IReadOnlyList<string> streams, IReadOnlyList<uint> versions, IReadOnlyList<string> fields, IReadOnlyList<string> sources, IReadOnlyList<string> signals, Window between, BucketRequest bucket, bool latest, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new SamplesListRequest
    {
      Scope = scope,
      Between = between,
      Bucket = bucket,
      Latest = latest,
    };
    request.Streams.Add(streams);
    request.Versions.Add(versions);
    request.Fields.Add(fields);
    request.Sources.Add(sources);
    request.Signals.Add(signals);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<SamplesListRequest, SamplesListResponse>(MethodType.Unary, "dokime.host.v1.evidence.Samples", "List",
        Marshallers.Create<SamplesListRequest>(value => value.ToByteArray(), bytes => SamplesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SamplesListResponse>(value => value.ToByteArray(), bytes => SamplesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SamplesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.samples.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<SamplesWatchResponse> Watch(EvidenceScope scope, IReadOnlyList<string> streams, IReadOnlyList<uint> versions, IReadOnlyList<string> fields, IReadOnlyList<string> sources, IReadOnlyList<string> signals, Window between, BucketRequest bucket, bool latest, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new SamplesWatchRequest
    {
      Scope = scope,
      Between = between,
      Bucket = bucket,
      Latest = latest,
    };
    request.Streams.Add(streams);
    request.Versions.Add(versions);
    request.Fields.Add(fields);
    request.Sources.Add(sources);
    request.Signals.Add(signals);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<SamplesWatchRequest, SamplesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Samples", "Watch",
        Marshallers.Create<SamplesWatchRequest>(value => value.ToByteArray(), bytes => SamplesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SamplesWatchResponse>(value => value.ToByteArray(), bytes => SamplesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.samples.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class EvidenceTraceabilityResource
{
  private readonly CallInvoker invoker;
  public EvidenceTraceabilityResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<TraceabilityGetResponse> Get(EvidenceScope scope, PageRequest? page = null, ReadCut? at = null, string? entry = null, CallOptions context = default)
  {
    var request = new TraceabilityGetRequest
    {
      Scope = scope,
    };
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
    var method = new Method<TraceabilityGetRequest, TraceabilityGetResponse>(MethodType.Unary, "dokime.host.v1.evidence.Traceability", "Get",
        Marshallers.Create<TraceabilityGetRequest>(value => value.ToByteArray(), bytes => TraceabilityGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<TraceabilityGetResponse>(value => value.ToByteArray(), bytes => TraceabilityGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    TraceabilityGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call evidence.traceability.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<TraceabilityWatchResponse> Watch(EvidenceScope scope, PageRequest? page = null, ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new TraceabilityWatchRequest
    {
      Scope = scope,
    };
    if (page is not null)
    {
      request.Page = page;
    }
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<TraceabilityWatchRequest, TraceabilityWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.evidence.Traceability", "Watch",
        Marshallers.Create<TraceabilityWatchRequest>(value => value.ToByteArray(), bytes => TraceabilityWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<TraceabilityWatchResponse>(value => value.ToByteArray(), bytes => TraceabilityWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call evidence.traceability.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class ExecutionsResource
{
  private readonly CallInvoker invoker;
  public ExecutionsCommandsResource Commands { get; }
  public ExecutionsProgressResource Progress { get; }
  public ExecutionsSchemasResource Schemas { get; }
  public ExecutionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Commands = new ExecutionsCommandsResource(invoker);
    Progress = new ExecutionsProgressResource(invoker);
    Schemas = new ExecutionsSchemasResource(invoker);
  }
  public async Task<ExecutionsGetResponse> Get(string name, Position? at = null, CallOptions context = default)
  {
    var request = new ExecutionsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ExecutionsGetRequest, ExecutionsGetResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Get",
        Marshallers.Create<ExecutionsGetRequest>(value => value.ToByteArray(), bytes => ExecutionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsGetResponse>(value => value.ToByteArray(), bytes => ExecutionsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsSetResponse> Set(string name, ExecutionWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new ExecutionsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<ExecutionsSetRequest, ExecutionsSetResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Set",
        Marshallers.Create<ExecutionsSetRequest>(value => value.ToByteArray(), bytes => ExecutionsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsSetResponse>(value => value.ToByteArray(), bytes => ExecutionsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsListResponse> List(string parent, ExecutionSelection selection, ExecutionFilter filter, ExecutionOrder order, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new ExecutionsListRequest
    {
      Parent = parent,
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<ExecutionsListRequest, ExecutionsListResponse>(MethodType.Unary, "dokime.host.v1.Executions", "List",
        Marshallers.Create<ExecutionsListRequest>(value => value.ToByteArray(), bytes => ExecutionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsListResponse>(value => value.ToByteArray(), bytes => ExecutionsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<ExecutionsWatchResponse> Watch(string parent, ExecutionSelection selection, ExecutionFilter filter, ExecutionOrder order, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new ExecutionsWatchRequest
    {
      Parent = parent,
      Selection = selection,
      Filter = filter,
      Order = order,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<ExecutionsWatchRequest, ExecutionsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Executions", "Watch",
        Marshallers.Create<ExecutionsWatchRequest>(value => value.ToByteArray(), bytes => ExecutionsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsWatchResponse>(value => value.ToByteArray(), bytes => ExecutionsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call executions.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<ExecutionsCreateResponse> Create(string parent, IReadOnlyList<ExecutionBinding> executions, string? requestId = null, CallOptions context = default)
  {
    var request = new ExecutionsCreateRequest
    {
      Parent = parent,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Executions.Add(executions);
    var method = new Method<ExecutionsCreateRequest, ExecutionsCreateResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Create",
        Marshallers.Create<ExecutionsCreateRequest>(value => value.ToByteArray(), bytes => ExecutionsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsCreateResponse>(value => value.ToByteArray(), bytes => ExecutionsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsStartResponse> Start(string name, ExecutionMode mode, string? requestId = null, CallOptions context = default)
  {
    var request = new ExecutionsStartRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
      Mode = mode,
    };
    var method = new Method<ExecutionsStartRequest, ExecutionsStartResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Start",
        Marshallers.Create<ExecutionsStartRequest>(value => value.ToByteArray(), bytes => ExecutionsStartRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsStartResponse>(value => value.ToByteArray(), bytes => ExecutionsStartResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsStartResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.start: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsStopResponse> Stop(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new ExecutionsStopRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<ExecutionsStopRequest, ExecutionsStopResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Stop",
        Marshallers.Create<ExecutionsStopRequest>(value => value.ToByteArray(), bytes => ExecutionsStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsStopResponse>(value => value.ToByteArray(), bytes => ExecutionsStopResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsStopResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.stop: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsCancelResponse> Cancel(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new ExecutionsCancelRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<ExecutionsCancelRequest, ExecutionsCancelResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Cancel",
        Marshallers.Create<ExecutionsCancelRequest>(value => value.ToByteArray(), bytes => ExecutionsCancelRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsCancelResponse>(value => value.ToByteArray(), bytes => ExecutionsCancelResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsCancelResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.cancel: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsDeleteResponse> Delete(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new ExecutionsDeleteRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<ExecutionsDeleteRequest, ExecutionsDeleteResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Delete",
        Marshallers.Create<ExecutionsDeleteRequest>(value => value.ToByteArray(), bytes => ExecutionsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsDeleteResponse>(value => value.ToByteArray(), bytes => ExecutionsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsParametersResponse> Parameters(string name, ExecutionParameterSet set, Position? at = null, CallOptions context = default)
  {
    var request = new ExecutionsParametersRequest
    {
      Name = name,
      Set = set,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ExecutionsParametersRequest, ExecutionsParametersResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Parameters",
        Marshallers.Create<ExecutionsParametersRequest>(value => value.ToByteArray(), bytes => ExecutionsParametersRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsParametersResponse>(value => value.ToByteArray(), bytes => ExecutionsParametersResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsParametersResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.parameters: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ExecutionsStepsResponse> Steps(string name, Position? at = null, CallOptions context = default)
  {
    var request = new ExecutionsStepsRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ExecutionsStepsRequest, ExecutionsStepsResponse>(MethodType.Unary, "dokime.host.v1.Executions", "Steps",
        Marshallers.Create<ExecutionsStepsRequest>(value => value.ToByteArray(), bytes => ExecutionsStepsRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ExecutionsStepsResponse>(value => value.ToByteArray(), bytes => ExecutionsStepsResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ExecutionsStepsResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.steps: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class ExecutionsCommandsResource
{
  private readonly CallInvoker invoker;
  public ExecutionsCommandsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<CommandsGetResponse> Get(string name, string parent, Position? at = null, CallOptions context = default)
  {
    var request = new CommandsGetRequest
    {
      Name = name,
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CommandsGetRequest, CommandsGetResponse>(MethodType.Unary, "dokime.host.v1.executions.Commands", "Get",
        Marshallers.Create<CommandsGetRequest>(value => value.ToByteArray(), bytes => CommandsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CommandsGetResponse>(value => value.ToByteArray(), bytes => CommandsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CommandsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.commands.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CommandsCreateResponse> Create(string parent, string commandId, ExecutionCommandAction action, string? requestId = null, string? stepId = null, CallOptions context = default)
  {
    var request = new CommandsCreateRequest
    {
      Parent = parent,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
      CommandId = commandId,
      Action = action,
    };
    if (stepId is not null)
    {
      request.StepId = stepId;
    }
    var method = new Method<CommandsCreateRequest, CommandsCreateResponse>(MethodType.Unary, "dokime.host.v1.executions.Commands", "Create",
        Marshallers.Create<CommandsCreateRequest>(value => value.ToByteArray(), bytes => CommandsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CommandsCreateResponse>(value => value.ToByteArray(), bytes => CommandsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CommandsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.commands.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class ExecutionsProgressResource
{
  private readonly CallInvoker invoker;
  public ExecutionsProgressResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ProgressGetResponse> Get(string name, Position? at = null, CallOptions context = default)
  {
    var request = new ProgressGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ProgressGetRequest, ProgressGetResponse>(MethodType.Unary, "dokime.host.v1.executions.Progress", "Get",
        Marshallers.Create<ProgressGetRequest>(value => value.ToByteArray(), bytes => ProgressGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ProgressGetResponse>(value => value.ToByteArray(), bytes => ProgressGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ProgressGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.progress.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<ProgressWatchResponse> Watch(string name, Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new ProgressWatchRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ProgressWatchRequest, ProgressWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.executions.Progress", "Watch",
        Marshallers.Create<ProgressWatchRequest>(value => value.ToByteArray(), bytes => ProgressWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ProgressWatchResponse>(value => value.ToByteArray(), bytes => ProgressWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call executions.progress.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class ExecutionsSchemasResource
{
  private readonly CallInvoker invoker;
  public ExecutionsSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SchemasGetResponse> Get(string name, Position? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new SchemasGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<SchemasGetRequest, SchemasGetResponse>(MethodType.Unary, "dokime.host.v1.executions.Schemas", "Get",
        Marshallers.Create<SchemasGetRequest>(value => value.ToByteArray(), bytes => SchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SchemasGetResponse>(value => value.ToByteArray(), bytes => SchemasGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SchemasGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call executions.schemas.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class FilesResource
{
  private readonly CallInvoker invoker;
  public FilesUploadsResource Uploads { get; }
  public FilesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Uploads = new FilesUploadsResource(invoker);
  }
  public async Task<FilesGetResponse> Get(FileReference file, ReadCut? at = null, CallOptions context = default)
  {
    var request = new FilesGetRequest
    {
      File = file,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<FilesGetRequest, FilesGetResponse>(MethodType.Unary, "dokime.host.v1.Files", "Get",
        Marshallers.Create<FilesGetRequest>(value => value.ToByteArray(), bytes => FilesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<FilesGetResponse>(value => value.ToByteArray(), bytes => FilesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    FilesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<FilesReadResponse> Read(FileReference file, ulong offset, ulong maxBytes, ReadCut? at = null, CallOptions context = default)
  {
    var request = new FilesReadRequest
    {
      File = file,
      Offset = offset,
      MaxBytes = maxBytes,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<FilesReadRequest, FilesReadResponse>(MethodType.Unary, "dokime.host.v1.Files", "Read",
        Marshallers.Create<FilesReadRequest>(value => value.ToByteArray(), bytes => FilesReadRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<FilesReadResponse>(value => value.ToByteArray(), bytes => FilesReadResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    FilesReadResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.read: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<FilesWatchResponse> Watch(ulong offset, FileReference? file = null, Upload? upload = null, ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
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
    var method = new Method<FilesWatchRequest, FilesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Files", "Watch",
        Marshallers.Create<FilesWatchRequest>(value => value.ToByteArray(), bytes => FilesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<FilesWatchResponse>(value => value.ToByteArray(), bytes => FilesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call files.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<Receipt> Delete(FileReference file, string? requestId = null, CallOptions context = default)
  {
    var request = new FilesDeleteRequest
    {
      File = file,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<FilesDeleteRequest, FilesDeleteResponse>(MethodType.Unary, "dokime.host.v1.Files", "Delete",
        Marshallers.Create<FilesDeleteRequest>(value => value.ToByteArray(), bytes => FilesDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<FilesDeleteResponse>(value => value.ToByteArray(), bytes => FilesDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    FilesDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.delete: " + error.Status.Detail), error.Trailers);
    }
    return response.Receipt ?? throw new InvalidOperationException("Read Files.Delete: Missing Payload");
  }
}
public sealed class FilesUploadsResource
{
  private readonly CallInvoker invoker;
  public FilesUploadsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<UploadsCreateResponse> Create(FileMetadata metadata, string? requestId = null, CallOptions context = default)
  {
    var request = new UploadsCreateRequest
    {
      Metadata = metadata,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<UploadsCreateRequest, UploadsCreateResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Create",
        Marshallers.Create<UploadsCreateRequest>(value => value.ToByteArray(), bytes => UploadsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<UploadsCreateResponse>(value => value.ToByteArray(), bytes => UploadsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    UploadsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<UploadsGetResponse> Get(Upload upload, ReadCut? at = null, CallOptions context = default)
  {
    var request = new UploadsGetRequest
    {
      Upload = upload,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<UploadsGetRequest, UploadsGetResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Get",
        Marshallers.Create<UploadsGetRequest>(value => value.ToByteArray(), bytes => UploadsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<UploadsGetResponse>(value => value.ToByteArray(), bytes => UploadsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    UploadsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<UploadsResumeResponse> Resume(Upload upload, string? requestId = null, CallOptions context = default)
  {
    var request = new UploadsResumeRequest
    {
      Upload = upload,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<UploadsResumeRequest, UploadsResumeResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Resume",
        Marshallers.Create<UploadsResumeRequest>(value => value.ToByteArray(), bytes => UploadsResumeRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<UploadsResumeResponse>(value => value.ToByteArray(), bytes => UploadsResumeResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    UploadsResumeResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.resume: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<UploadsAppendResponse> Append(Upload upload, ulong offset, ByteString data, string? requestId = null, CallOptions context = default)
  {
    var request = new UploadsAppendRequest
    {
      Upload = upload,
      Offset = offset,
      Data = data,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<UploadsAppendRequest, UploadsAppendResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Append",
        Marshallers.Create<UploadsAppendRequest>(value => value.ToByteArray(), bytes => UploadsAppendRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<UploadsAppendResponse>(value => value.ToByteArray(), bytes => UploadsAppendResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    UploadsAppendResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.append: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<UploadsPublishResponse> Publish(Upload upload, ulong length, ByteString sha256, string? requestId = null, CallOptions context = default)
  {
    var request = new UploadsPublishRequest
    {
      Upload = upload,
      Length = length,
      Sha256 = sha256,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<UploadsPublishRequest, UploadsPublishResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Publish",
        Marshallers.Create<UploadsPublishRequest>(value => value.ToByteArray(), bytes => UploadsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<UploadsPublishResponse>(value => value.ToByteArray(), bytes => UploadsPublishResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    UploadsPublishResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.publish: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<UploadsDeleteResponse> Delete(Upload upload, string? requestId = null, CallOptions context = default)
  {
    var request = new UploadsDeleteRequest
    {
      Upload = upload,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<UploadsDeleteRequest, UploadsDeleteResponse>(MethodType.Unary, "dokime.host.v1.files.Uploads", "Delete",
        Marshallers.Create<UploadsDeleteRequest>(value => value.ToByteArray(), bytes => UploadsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<UploadsDeleteResponse>(value => value.ToByteArray(), bytes => UploadsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    UploadsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call files.uploads.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class InvocationsResource
{
  private readonly CallInvoker invoker;
  public InvocationsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ByteString> Create(string operation, ByteString fingerprint, ByteString payload, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Invocations.Create(operation, fingerprint, payload, context);
  }
  public async Task<InstanceState> Get(ulong instance, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Invocations.Get(instance, context);
  }
  public async Task<InstanceState> Cancel(ulong instance, string reason, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Invocations.Cancel(instance, reason, context);
  }
  public async IAsyncEnumerable<InstanceState> Watch(ulong instance, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in new Harmos.Client(invoker).Invocations.Watch(instance, context, cancellationToken)) { yield return response; }
  }
}
public sealed class MediaResource
{
  public MediaCapturesResource Captures { get; }
  public MediaResource(CallInvoker invoker)
  {
    Captures = new MediaCapturesResource(invoker);
  }
}
public sealed class MediaCapturesResource
{
  private readonly CallInvoker invoker;
  public MediaCapturesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<CapturesCreateResponse> Create(ArtifactScope scope, CaptureKind kind, RecordingAudio audio, uint? input = null, uint? camera = null, uint? display = null, uint? maximumSeconds = null, string? requestId = null, CallOptions context = default)
  {
    var request = new CapturesCreateRequest
    {
      Scope = scope,
      Kind = kind,
      Audio = audio,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    if (input is not null)
    {
      request.Input = input.Value;
    }
    if (camera is not null)
    {
      request.Camera = camera.Value;
    }
    if (display is not null)
    {
      request.Display = display.Value;
    }
    if (maximumSeconds is not null)
    {
      request.MaximumSeconds = maximumSeconds.Value;
    }
    var method = new Method<CapturesCreateRequest, CapturesCreateResponse>(MethodType.Unary, "dokime.host.v1.media.Captures", "Create",
        Marshallers.Create<CapturesCreateRequest>(value => value.ToByteArray(), bytes => CapturesCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CapturesCreateResponse>(value => value.ToByteArray(), bytes => CapturesCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CapturesCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call media.captures.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<CapturesGetResponse> Get(string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new CapturesGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CapturesGetRequest, CapturesGetResponse>(MethodType.Unary, "dokime.host.v1.media.Captures", "Get",
        Marshallers.Create<CapturesGetRequest>(value => value.ToByteArray(), bytes => CapturesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CapturesGetResponse>(value => value.ToByteArray(), bytes => CapturesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CapturesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call media.captures.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<CapturesWatchResponse> Watch(string name, ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new CapturesWatchRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<CapturesWatchRequest, CapturesWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.media.Captures", "Watch",
        Marshallers.Create<CapturesWatchRequest>(value => value.ToByteArray(), bytes => CapturesWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CapturesWatchResponse>(value => value.ToByteArray(), bytes => CapturesWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call media.captures.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<CapturesStopResponse> Stop(string name, string? requestId = null, CallOptions context = default)
  {
    var request = new CapturesStopRequest
    {
      Name = name,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<CapturesStopRequest, CapturesStopResponse>(MethodType.Unary, "dokime.host.v1.media.Captures", "Stop",
        Marshallers.Create<CapturesStopRequest>(value => value.ToByteArray(), bytes => CapturesStopRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<CapturesStopResponse>(value => value.ToByteArray(), bytes => CapturesStopResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    CapturesStopResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call media.captures.stop: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class OperationsResource
{
  private readonly CallInvoker invoker;
  public OperationsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<IReadOnlyList<DiscoveredOperation>> List(CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Operations.List(context);
  }
  public async Task<DiscoveredOperation> Get(string id, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Operations.Get(id, context);
  }
}
public sealed class PlotsResource
{
  public PlotsSettingsResource Settings { get; }
  public PlotsSnapshotsResource Snapshots { get; }
  public PlotsResource(CallInvoker invoker)
  {
    Settings = new PlotsSettingsResource(invoker);
    Snapshots = new PlotsSnapshotsResource(invoker);
  }
}
public sealed class PlotsSettingsResource
{
  private readonly CallInvoker invoker;
  public PlotsSettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<DokimeClientsPlotsResourcesSettingsGetResponse> Get(DocumentScope scope, string documentId, string plotId, ReadCut? at = null, CallOptions context = default)
  {
    var request = new DokimeClientsPlotsResourcesSettingsGetRequest
    {
      Scope = scope,
      DocumentId = documentId,
      PlotId = plotId,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<DokimeClientsPlotsResourcesSettingsGetRequest, DokimeClientsPlotsResourcesSettingsGetResponse>(MethodType.Unary, "dokime.host.v1.plots.Settings", "Get",
        Marshallers.Create<DokimeClientsPlotsResourcesSettingsGetRequest>(value => value.ToByteArray(), bytes => DokimeClientsPlotsResourcesSettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DokimeClientsPlotsResourcesSettingsGetResponse>(value => value.ToByteArray(), bytes => DokimeClientsPlotsResourcesSettingsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DokimeClientsPlotsResourcesSettingsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plots.settings.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<DokimeClientsPlotsResourcesSettingsSetResponse> Set(DocumentScope scope, string documentId, string plotId, PlotSettingsValue value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new DokimeClientsPlotsResourcesSettingsSetRequest
    {
      Scope = scope,
      DocumentId = documentId,
      PlotId = plotId,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<DokimeClientsPlotsResourcesSettingsSetRequest, DokimeClientsPlotsResourcesSettingsSetResponse>(MethodType.Unary, "dokime.host.v1.plots.Settings", "Set",
        Marshallers.Create<DokimeClientsPlotsResourcesSettingsSetRequest>(value => value.ToByteArray(), bytes => DokimeClientsPlotsResourcesSettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DokimeClientsPlotsResourcesSettingsSetResponse>(value => value.ToByteArray(), bytes => DokimeClientsPlotsResourcesSettingsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DokimeClientsPlotsResourcesSettingsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plots.settings.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class PlotsSnapshotsResource
{
  private readonly CallInvoker invoker;
  public PlotsSnapshotsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SnapshotsListResponse> List(IReadOnlyList<PlotRequest> requests, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new SnapshotsListRequest
    {
    };
    request.Requests.Add(requests);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<SnapshotsListRequest, SnapshotsListResponse>(MethodType.Unary, "dokime.host.v1.plots.Snapshots", "List",
        Marshallers.Create<SnapshotsListRequest>(value => value.ToByteArray(), bytes => SnapshotsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SnapshotsListResponse>(value => value.ToByteArray(), bytes => SnapshotsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SnapshotsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plots.snapshots.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<SnapshotsWatchResponse> Watch(IReadOnlyList<PlotRequest> requests, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new SnapshotsWatchRequest
    {
    };
    request.Requests.Add(requests);
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<SnapshotsWatchRequest, SnapshotsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.plots.Snapshots", "Watch",
        Marshallers.Create<SnapshotsWatchRequest>(value => value.ToByteArray(), bytes => SnapshotsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SnapshotsWatchResponse>(value => value.ToByteArray(), bytes => SnapshotsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call plots.snapshots.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class PluginsResource
{
  private readonly CallInvoker invoker;
  public PluginsSecretsResource Secrets { get; }
  public PluginsSettingsResource Settings { get; }
  public PluginsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Secrets = new PluginsSecretsResource(invoker);
    Settings = new PluginsSettingsResource(invoker);
  }
  public async Task<PluginsGetResponse> Get(string name, ReadCut? at = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Get(name, at, context);
  }
  public async Task<PluginsListResponse> List(ReadCut? at = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.List(at, context);
  }
  public async IAsyncEnumerable<PluginsWatchResponse> Watch(string? name = null, ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in new Harmos.Client(invoker).Plugins.Watch(name, at, context, cancellationToken)) { yield return response; }
  }
  public async Task<PluginsCreateResponse> Create(string name, PluginPlacement placement, PluginValue value, string? requestId = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Create(name, placement, value, requestId, context);
  }
  public async Task<PluginsSetResponse> Set(string name, PluginValue value, ByteString expectedRevision, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Set(name, value, expectedRevision, context);
  }
  public async Task<Receipt> Delete(string name, ByteString expectedRevision, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Delete(name, expectedRevision, context);
  }
  public async Task<PluginEntry> Restart(string name, ByteString expectedRevision, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Restart(name, expectedRevision, context);
  }
  public async Task<PluginsReplaceResponse> Replace(string name, PluginPlacement placement, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Replace(name, placement, expectedRevision, requestId, context);
  }
}
public sealed class PluginsSecretsResource
{
  private readonly CallInvoker invoker;
  public PluginsSecretsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<DokimeClientsPluginsResourcesSecretsListResponse> List(string? name = null, ReadCut? at = null, CallOptions context = default)
  {
    var request = new DokimeClientsPluginsResourcesSecretsListRequest
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
    var method = new Method<DokimeClientsPluginsResourcesSecretsListRequest, DokimeClientsPluginsResourcesSecretsListResponse>(MethodType.Unary, "dokime.host.v1.plugins.Secrets", "List",
        Marshallers.Create<DokimeClientsPluginsResourcesSecretsListRequest>(value => value.ToByteArray(), bytes => DokimeClientsPluginsResourcesSecretsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DokimeClientsPluginsResourcesSecretsListResponse>(value => value.ToByteArray(), bytes => DokimeClientsPluginsResourcesSecretsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DokimeClientsPluginsResourcesSecretsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call plugins.secrets.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class PluginsSettingsResource
{
  private readonly CallInvoker invoker;
  public PluginsSettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SettingsListResponse> List(string name, ReadCut? at = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Settings.List(name, at, context);
  }
  public async Task<HarmosHostPluginsResourcesSettingsSetResponse> Set(string name, SettingValues value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Plugins.Settings.Set(name, value, expectedRevision, requestId, context);
  }
}
public sealed class PromptDefinitionsResource
{
  private readonly CallInvoker invoker;
  public PromptDefinitionsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<PromptDefinitionsGetResponse> Get(string name, string parent, uint schemaVersion, ReadCut? at = null, CallOptions context = default)
  {
    var request = new PromptDefinitionsGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<PromptDefinitionsGetRequest, PromptDefinitionsGetResponse>(MethodType.Unary, "dokime.host.v1.PromptDefinitions", "Get",
        Marshallers.Create<PromptDefinitionsGetRequest>(value => value.ToByteArray(), bytes => PromptDefinitionsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<PromptDefinitionsGetResponse>(value => value.ToByteArray(), bytes => PromptDefinitionsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    PromptDefinitionsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompt_definitions.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<PromptDefinitionsListResponse> List(string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new PromptDefinitionsListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<PromptDefinitionsListRequest, PromptDefinitionsListResponse>(MethodType.Unary, "dokime.host.v1.PromptDefinitions", "List",
        Marshallers.Create<PromptDefinitionsListRequest>(value => value.ToByteArray(), bytes => PromptDefinitionsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<PromptDefinitionsListResponse>(value => value.ToByteArray(), bytes => PromptDefinitionsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    PromptDefinitionsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompt_definitions.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class PromptsResource
{
  private readonly CallInvoker invoker;
  public PromptsAnswersResource Answers { get; }
  public PromptsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Answers = new PromptsAnswersResource(invoker);
  }
  public async Task<PromptsGetResponse> Get(string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new PromptsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<PromptsGetRequest, PromptsGetResponse>(MethodType.Unary, "dokime.host.v1.Prompts", "Get",
        Marshallers.Create<PromptsGetRequest>(value => value.ToByteArray(), bytes => PromptsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<PromptsGetResponse>(value => value.ToByteArray(), bytes => PromptsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    PromptsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompts.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<PromptsListResponse> List(string parent, bool pendingOnly, string? cycleId = null, string? executionId = null, ReadCut? at = null, PageRequest? page = null, CallOptions context = default)
  {
    var request = new PromptsListRequest
    {
      Parent = parent,
      PendingOnly = pendingOnly,
    };
    if (cycleId is not null)
    {
      request.CycleId = cycleId;
    }
    if (executionId is not null)
    {
      request.ExecutionId = executionId;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<PromptsListRequest, PromptsListResponse>(MethodType.Unary, "dokime.host.v1.Prompts", "List",
        Marshallers.Create<PromptsListRequest>(value => value.ToByteArray(), bytes => PromptsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<PromptsListResponse>(value => value.ToByteArray(), bytes => PromptsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    PromptsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompts.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<PromptsWatchResponse> Watch(string parent, bool pendingOnly, string? cycleId = null, string? executionId = null, ReadCut? at = null, PageRequest? page = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new PromptsWatchRequest
    {
      Parent = parent,
      PendingOnly = pendingOnly,
    };
    if (cycleId is not null)
    {
      request.CycleId = cycleId;
    }
    if (executionId is not null)
    {
      request.ExecutionId = executionId;
    }
    if (at is not null)
    {
      request.At = at;
    }
    if (page is not null)
    {
      request.Page = page;
    }
    var method = new Method<PromptsWatchRequest, PromptsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Prompts", "Watch",
        Marshallers.Create<PromptsWatchRequest>(value => value.ToByteArray(), bytes => PromptsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<PromptsWatchResponse>(value => value.ToByteArray(), bytes => PromptsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call prompts.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
}
public sealed class PromptsAnswersResource
{
  private readonly CallInvoker invoker;
  public PromptsAnswersResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<AnswersCreateResponse> Create(string parent, string action, MapValue values, bool accepted, string? requestId = null, CallOptions context = default)
  {
    var request = new AnswersCreateRequest
    {
      Parent = parent,
      Action = action,
      Values = values,
      Accepted = accepted,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<AnswersCreateRequest, AnswersCreateResponse>(MethodType.Unary, "dokime.host.v1.prompts.Answers", "Create",
        Marshallers.Create<AnswersCreateRequest>(value => value.ToByteArray(), bytes => AnswersCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<AnswersCreateResponse>(value => value.ToByteArray(), bytes => AnswersCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    AnswersCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call prompts.answers.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class RecordSchemasResource
{
  private readonly CallInvoker invoker;
  public RecordSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<RecordSchemasGetResponse> Get(string name, string parent, uint schemaVersion, ReadCut? at = null, CallOptions context = default)
  {
    var request = new RecordSchemasGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<RecordSchemasGetRequest, RecordSchemasGetResponse>(MethodType.Unary, "dokime.host.v1.RecordSchemas", "Get",
        Marshallers.Create<RecordSchemasGetRequest>(value => value.ToByteArray(), bytes => RecordSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<RecordSchemasGetResponse>(value => value.ToByteArray(), bytes => RecordSchemasGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    RecordSchemasGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call record_schemas.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<RecordSchemasListResponse> List(string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new RecordSchemasListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<RecordSchemasListRequest, RecordSchemasListResponse>(MethodType.Unary, "dokime.host.v1.RecordSchemas", "List",
        Marshallers.Create<RecordSchemasListRequest>(value => value.ToByteArray(), bytes => RecordSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<RecordSchemasListResponse>(value => value.ToByteArray(), bytes => RecordSchemasListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    RecordSchemasListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call record_schemas.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class ReportsResource
{
  private readonly CallInvoker invoker;
  public ReportsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<ReportsGetResponse> Get(ReportSubject subject, string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new ReportsGetRequest
    {
      Subject = subject,
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ReportsGetRequest, ReportsGetResponse>(MethodType.Unary, "dokime.host.v1.Reports", "Get",
        Marshallers.Create<ReportsGetRequest>(value => value.ToByteArray(), bytes => ReportsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ReportsGetResponse>(value => value.ToByteArray(), bytes => ReportsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ReportsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call reports.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ReportsCreateResponse> Create(ReportSubject subject, string documentId, ReadCut? at = null, string? requestId = null, CallOptions context = default)
  {
    var request = new ReportsCreateRequest
    {
      Subject = subject,
      DocumentId = documentId,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<ReportsCreateRequest, ReportsCreateResponse>(MethodType.Unary, "dokime.host.v1.Reports", "Create",
        Marshallers.Create<ReportsCreateRequest>(value => value.ToByteArray(), bytes => ReportsCreateRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ReportsCreateResponse>(value => value.ToByteArray(), bytes => ReportsCreateResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ReportsCreateResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call reports.create: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<ReportsPublishResponse> Publish(string name, ReportSubject subject, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new ReportsPublishRequest
    {
      Name = name,
      Subject = subject,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<ReportsPublishRequest, ReportsPublishResponse>(MethodType.Unary, "dokime.host.v1.Reports", "Publish",
        Marshallers.Create<ReportsPublishRequest>(value => value.ToByteArray(), bytes => ReportsPublishRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<ReportsPublishResponse>(value => value.ToByteArray(), bytes => ReportsPublishResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    ReportsPublishResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call reports.publish: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class SchemasResource
{
  private readonly CallInvoker invoker;
  public SchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<IReadOnlyList<SchemaDescription>> List(string? schema = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Schemas.List(schema, context);
  }
}
public sealed class SecretsResource
{
  private readonly CallInvoker invoker;
  public SecretsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SecretsGetResponse> Get(string name, ReadCut? at = null, CallOptions context = default)
  {
    var request = new SecretsGetRequest
    {
      Name = name,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SecretsGetRequest, SecretsGetResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Get",
        Marshallers.Create<SecretsGetRequest>(value => value.ToByteArray(), bytes => SecretsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SecretsGetResponse>(value => value.ToByteArray(), bytes => SecretsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SecretsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<SecretsSetResponse> Set(string name, SecretWritable value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new SecretsSetRequest
    {
      Name = name,
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<SecretsSetRequest, SecretsSetResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Set",
        Marshallers.Create<SecretsSetRequest>(value => value.ToByteArray(), bytes => SecretsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SecretsSetResponse>(value => value.ToByteArray(), bytes => SecretsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SecretsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<DokimeClientsSecretsListResponse> List(ReadCut? at = null, CallOptions context = default)
  {
    var request = new DokimeClientsSecretsListRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<DokimeClientsSecretsListRequest, DokimeClientsSecretsListResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "List",
        Marshallers.Create<DokimeClientsSecretsListRequest>(value => value.ToByteArray(), bytes => DokimeClientsSecretsListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DokimeClientsSecretsListResponse>(value => value.ToByteArray(), bytes => DokimeClientsSecretsListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DokimeClientsSecretsListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async IAsyncEnumerable<SecretsWatchResponse> Watch(ReadCut? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    var request = new SecretsWatchRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SecretsWatchRequest, SecretsWatchResponse>(MethodType.ServerStreaming, "dokime.host.v1.Secrets", "Watch",
        Marshallers.Create<SecretsWatchRequest>(value => value.ToByteArray(), bytes => SecretsWatchRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SecretsWatchResponse>(value => value.ToByteArray(), bytes => SecretsWatchResponse.Parser.ParseFrom(bytes)));
    using var linked = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken, cancellationToken);
    using var call = invoker.AsyncServerStreamingCall(method, null, context.WithCancellationToken(linked.Token), request);
    while (true)
    {
      bool available;
      try
      {
        available = await call.ResponseStream.MoveNext(linked.Token);
      }
      catch (RpcException error)
      {
        throw new RpcException(new Status(error.StatusCode, "Call secrets.watch: " + error.Status.Detail), error.Trailers);
      }
      if (!available)
      {
        yield break;
      }
      var response = call.ResponseStream.Current;
      yield return response;
    }
  }
  public async Task<SecretsDeleteResponse> Delete(IReadOnlyList<string> names, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new SecretsDeleteRequest
    {
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Names.Add(names);
    var method = new Method<SecretsDeleteRequest, SecretsDeleteResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Delete",
        Marshallers.Create<SecretsDeleteRequest>(value => value.ToByteArray(), bytes => SecretsDeleteRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SecretsDeleteResponse>(value => value.ToByteArray(), bytes => SecretsDeleteResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SecretsDeleteResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.delete: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<SecretsExportResponse> Export(IReadOnlyList<string> names, ulong generation, ReadCut? at = null, CallOptions context = default)
  {
    var request = new SecretsExportRequest
    {
      Generation = generation,
    };
    request.Names.Add(names);
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SecretsExportRequest, SecretsExportResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Export",
        Marshallers.Create<SecretsExportRequest>(value => value.ToByteArray(), bytes => SecretsExportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SecretsExportResponse>(value => value.ToByteArray(), bytes => SecretsExportResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SecretsExportResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.export: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<SecretsImportResponse> Import(ByteString data, IReadOnlyList<SecretImportChoice> choices, ulong generation, bool validateOnly, string? requestId = null, CallOptions context = default)
  {
    var request = new SecretsImportRequest
    {
      Data = data,
      Generation = generation,
      ValidateOnly = validateOnly,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    request.Choices.Add(choices);
    var method = new Method<SecretsImportRequest, SecretsImportResponse>(MethodType.Unary, "dokime.host.v1.Secrets", "Import",
        Marshallers.Create<SecretsImportRequest>(value => value.ToByteArray(), bytes => SecretsImportRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SecretsImportResponse>(value => value.ToByteArray(), bytes => SecretsImportResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SecretsImportResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call secrets.import: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class SettingsResource
{
  private readonly CallInvoker invoker;
  public SettingsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<DokimeClientsSettingsGetResponse> Get(ReadCut? at = null, CallOptions context = default)
  {
    var request = new DokimeClientsSettingsGetRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<DokimeClientsSettingsGetRequest, DokimeClientsSettingsGetResponse>(MethodType.Unary, "dokime.host.v1.Settings", "Get",
        Marshallers.Create<DokimeClientsSettingsGetRequest>(value => value.ToByteArray(), bytes => DokimeClientsSettingsGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DokimeClientsSettingsGetResponse>(value => value.ToByteArray(), bytes => DokimeClientsSettingsGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DokimeClientsSettingsGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call settings.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<DokimeClientsSettingsSetResponse> Set(ApplicationSettings value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new DokimeClientsSettingsSetRequest
    {
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<DokimeClientsSettingsSetRequest, DokimeClientsSettingsSetResponse>(MethodType.Unary, "dokime.host.v1.Settings", "Set",
        Marshallers.Create<DokimeClientsSettingsSetRequest>(value => value.ToByteArray(), bytes => DokimeClientsSettingsSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<DokimeClientsSettingsSetResponse>(value => value.ToByteArray(), bytes => DokimeClientsSettingsSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    DokimeClientsSettingsSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call settings.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<SettingsSchemaResponse> Schema(ReadCut? at = null, CallOptions context = default)
  {
    var request = new SettingsSchemaRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SettingsSchemaRequest, SettingsSchemaResponse>(MethodType.Unary, "dokime.host.v1.Settings", "Schema",
        Marshallers.Create<SettingsSchemaRequest>(value => value.ToByteArray(), bytes => SettingsSchemaRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SettingsSchemaResponse>(value => value.ToByteArray(), bytes => SettingsSchemaResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SettingsSchemaResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call settings.schema: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class StatusResource
{
  private readonly CallInvoker invoker;
  public StatusDiagnosticsResource Diagnostics { get; }
  public StatusLogsResource Logs { get; }
  public StatusTraceEventsResource TraceEvents { get; }
  public StatusResource(CallInvoker invoker)
  {
    this.invoker = invoker;
    Diagnostics = new StatusDiagnosticsResource(invoker);
    Logs = new StatusLogsResource(invoker);
    TraceEvents = new StatusTraceEventsResource(invoker);
  }
  public async Task<StatusGetResponse> Get(CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Status.Get(context);
  }
  public async IAsyncEnumerable<StatusWatchResponse> Watch(Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in new Harmos.Client(invoker).Status.Watch(at, context, cancellationToken)) { yield return response; }
  }
}
public sealed class StatusDiagnosticsResource
{
  private readonly CallInvoker invoker;
  public StatusDiagnosticsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<DiagnosticsGetResponse> Get(string name, ulong offset, ulong maxBytes, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Status.Diagnostics.Get(name, offset, maxBytes, context);
  }
}
public sealed class StatusLogsResource
{
  private readonly CallInvoker invoker;
  public StatusLogsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<LogsListResponse> List(Position? at = null, PageRequest? page = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Status.Logs.List(at, page, context);
  }
  public async IAsyncEnumerable<LogsWatchResponse> Watch(ulong maxBytes, uint maxEvents, Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in new Harmos.Client(invoker).Status.Logs.Watch(maxBytes, maxEvents, at, context, cancellationToken)) { yield return response; }
  }
}
public sealed class StatusTraceEventsResource
{
  private readonly CallInvoker invoker;
  public StatusTraceEventsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<TraceEventsListResponse> List(Position? at = null, PageRequest? page = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Status.TraceEvents.List(at, page, context);
  }
  public async IAsyncEnumerable<TraceEventsWatchResponse> Watch(ulong maxBytes, uint maxEvents, Position? at = null, CallOptions context = default, [EnumeratorCancellation] CancellationToken cancellationToken = default)
  {
    await foreach (var response in new Harmos.Client(invoker).Status.TraceEvents.Watch(maxBytes, maxEvents, at, context, cancellationToken)) { yield return response; }
  }
}
public sealed class StorageResource
{
  public StorageSourcesResource Sources { get; }
  public StorageResource(CallInvoker invoker)
  {
    Sources = new StorageSourcesResource(invoker);
  }
}
public sealed class StorageSourcesResource
{
  private readonly CallInvoker invoker;
  public StorageSourcesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SourcesGetResponse> Get(ReadCut? at = null, CallOptions context = default)
  {
    var request = new SourcesGetRequest
    {
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SourcesGetRequest, SourcesGetResponse>(MethodType.Unary, "dokime.host.v1.storage.Sources", "Get",
        Marshallers.Create<SourcesGetRequest>(value => value.ToByteArray(), bytes => SourcesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SourcesGetResponse>(value => value.ToByteArray(), bytes => SourcesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SourcesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call storage.sources.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<SourcesSetResponse> Set(StorageSources value, ByteString expectedRevision, string? requestId = null, CallOptions context = default)
  {
    var request = new SourcesSetRequest
    {
      Value = value,
      ExpectedRevision = expectedRevision,
      RequestId = requestId ?? Guid.NewGuid().ToString("N"),
    };
    var method = new Method<SourcesSetRequest, SourcesSetResponse>(MethodType.Unary, "dokime.host.v1.storage.Sources", "Set",
        Marshallers.Create<SourcesSetRequest>(value => value.ToByteArray(), bytes => SourcesSetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SourcesSetResponse>(value => value.ToByteArray(), bytes => SourcesSetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SourcesSetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call storage.sources.set: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class StreamSchemasResource
{
  private readonly CallInvoker invoker;
  public StreamSchemasResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<StreamSchemasGetResponse> Get(string name, string parent, uint schemaVersion, ReadCut? at = null, CallOptions context = default)
  {
    var request = new StreamSchemasGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<StreamSchemasGetRequest, StreamSchemasGetResponse>(MethodType.Unary, "dokime.host.v1.StreamSchemas", "Get",
        Marshallers.Create<StreamSchemasGetRequest>(value => value.ToByteArray(), bytes => StreamSchemasGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<StreamSchemasGetResponse>(value => value.ToByteArray(), bytes => StreamSchemasGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    StreamSchemasGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call stream_schemas.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<StreamSchemasListResponse> List(string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new StreamSchemasListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<StreamSchemasListRequest, StreamSchemasListResponse>(MethodType.Unary, "dokime.host.v1.StreamSchemas", "List",
        Marshallers.Create<StreamSchemasListRequest>(value => value.ToByteArray(), bytes => StreamSchemasListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<StreamSchemasListResponse>(value => value.ToByteArray(), bytes => StreamSchemasListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    StreamSchemasListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call stream_schemas.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class StreamsResource
{
  private readonly CallInvoker invoker;
  public StreamsResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<IReadOnlyList<StreamIdentity>> List(string? schema = null, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Streams.List(schema, context);
  }
}
public sealed class SuitesResource
{
  private readonly CallInvoker invoker;
  public SuitesResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<SuitesGetResponse> Get(string name, string parent, uint schemaVersion, ReadCut? at = null, CallOptions context = default)
  {
    var request = new SuitesGetRequest
    {
      Name = name,
      Parent = parent,
      SchemaVersion = schemaVersion,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SuitesGetRequest, SuitesGetResponse>(MethodType.Unary, "dokime.host.v1.Suites", "Get",
        Marshallers.Create<SuitesGetRequest>(value => value.ToByteArray(), bytes => SuitesGetRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SuitesGetResponse>(value => value.ToByteArray(), bytes => SuitesGetResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SuitesGetResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call suites.get: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
  public async Task<SuitesListResponse> List(string parent, ReadCut? at = null, CallOptions context = default)
  {
    var request = new SuitesListRequest
    {
      Parent = parent,
    };
    if (at is not null)
    {
      request.At = at;
    }
    var method = new Method<SuitesListRequest, SuitesListResponse>(MethodType.Unary, "dokime.host.v1.Suites", "List",
        Marshallers.Create<SuitesListRequest>(value => value.ToByteArray(), bytes => SuitesListRequest.Parser.ParseFrom(bytes)),
        Marshallers.Create<SuitesListResponse>(value => value.ToByteArray(), bytes => SuitesListResponse.Parser.ParseFrom(bytes)));
    using var call = invoker.AsyncUnaryCall(method, null, context, request);
    SuitesListResponse response;
    try
    {
      response = await call.ResponseAsync;
    }
    catch (RpcException error)
    {
      throw new RpcException(new Status(error.StatusCode, "Call suites.list: " + error.Status.Detail), error.Trailers);
    }
    return response;
  }
}
public sealed class TimeResource
{
  private readonly CallInvoker invoker;
  public TimeResource(CallInvoker invoker)
  {
    this.invoker = invoker;
  }
  public async Task<Timestamp> Get(CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Time.Get(context);
  }
  public async Task<TimeSleepResponse> Sleep(Duration duration, CallOptions context = default)
  {
    return await new Harmos.Client(invoker).Time.Sleep(duration, context);
  }
}
