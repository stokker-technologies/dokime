// @generated from protocol descriptors; do not edit.
use dokime_protocol::host::AggregatesGetRequest;
use dokime_protocol::host::AggregatesGetResponse;
use dokime_protocol::host::ApplicationSettings;
use dokime_protocol::host::BackendAppearanceRequest;
use dokime_protocol::host::BackendAppearanceResponse;
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CasesCreateResponse;
use dokime_protocol::host::CasesDeleteRequest;
use dokime_protocol::host::CasesDeleteResponse;
use dokime_protocol::host::CasesGetRequest;
use dokime_protocol::host::CasesGetResponse;
use dokime_protocol::host::CasesListRequest;
use dokime_protocol::host::CasesListResponse;
use dokime_protocol::host::CasesParametersRequest;
use dokime_protocol::host::CasesParametersResponse;
use dokime_protocol::host::CasesWatchRequest;
use dokime_protocol::host::CasesWatchResponse;
use dokime_protocol::host::CycleFilter;
use dokime_protocol::host::CycleOrder;
use dokime_protocol::host::CycleSelection;
use dokime_protocol::host::CycleWritable;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::CyclesCreateResponse;
use dokime_protocol::host::CyclesGetRequest;
use dokime_protocol::host::CyclesGetResponse;
use dokime_protocol::host::CyclesListRequest;
use dokime_protocol::host::CyclesListResponse;
use dokime_protocol::host::CyclesSealRequest;
use dokime_protocol::host::CyclesSealResponse;
use dokime_protocol::host::CyclesSetRequest;
use dokime_protocol::host::CyclesSetResponse;
use dokime_protocol::host::CyclesWatchRequest;
use dokime_protocol::host::CyclesWatchResponse;
use dokime_protocol::host::DocumentScope;
use dokime_protocol::host::DocumentsGetRequest;
use dokime_protocol::host::DocumentsGetResponse;
use dokime_protocol::host::DocumentsWatchRequest;
use dokime_protocol::host::DocumentsWatchResponse;
use dokime_protocol::host::EvidenceArchiveRequest;
use dokime_protocol::host::EvidenceArchiveResponse;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_protocol::host::EvidenceSubject;
use dokime_protocol::host::ExecutionBinding;
use dokime_protocol::host::ExecutionFilter;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::ExecutionOrder;
use dokime_protocol::host::ExecutionParameterSet;
use dokime_protocol::host::ExecutionSelection;
use dokime_protocol::host::ExecutionWritable;
use dokime_protocol::host::ExecutionsCancelRequest;
use dokime_protocol::host::ExecutionsCancelResponse;
use dokime_protocol::host::ExecutionsCreateRequest;
use dokime_protocol::host::ExecutionsCreateResponse;
use dokime_protocol::host::ExecutionsDeleteRequest;
use dokime_protocol::host::ExecutionsDeleteResponse;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsGetResponse;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::ExecutionsListResponse;
use dokime_protocol::host::ExecutionsParametersRequest;
use dokime_protocol::host::ExecutionsParametersResponse;
use dokime_protocol::host::ExecutionsSealRequest;
use dokime_protocol::host::ExecutionsSealResponse;
use dokime_protocol::host::ExecutionsSetRequest;
use dokime_protocol::host::ExecutionsSetResponse;
use dokime_protocol::host::ExecutionsStartRequest;
use dokime_protocol::host::ExecutionsStartResponse;
use dokime_protocol::host::ExecutionsStepsRequest;
use dokime_protocol::host::ExecutionsStepsResponse;
use dokime_protocol::host::ExecutionsStopRequest;
use dokime_protocol::host::ExecutionsStopResponse;
use dokime_protocol::host::ExecutionsWatchRequest;
use dokime_protocol::host::ExecutionsWatchResponse;
use dokime_protocol::host::FilesDeleteRequest;
use dokime_protocol::host::FilesDeleteResponse;
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesGetResponse;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::FilesReadResponse;
use dokime_protocol::host::FilesWatchRequest;
use dokime_protocol::host::FilesWatchResponse;
use dokime_protocol::host::Output;
use dokime_protocol::host::PromptDefinitionsGetRequest;
use dokime_protocol::host::PromptDefinitionsGetResponse;
use dokime_protocol::host::PromptDefinitionsListRequest;
use dokime_protocol::host::PromptDefinitionsListResponse;
use dokime_protocol::host::PromptsGetRequest;
use dokime_protocol::host::PromptsGetResponse;
use dokime_protocol::host::PromptsListRequest;
use dokime_protocol::host::PromptsListResponse;
use dokime_protocol::host::PromptsWatchRequest;
use dokime_protocol::host::PromptsWatchResponse;
use dokime_protocol::host::QueryAggregate;
use dokime_protocol::host::QueryFilter;
use dokime_protocol::host::QuerySource;
use dokime_protocol::host::RecordSchemasGetRequest;
use dokime_protocol::host::RecordSchemasGetResponse;
use dokime_protocol::host::RecordSchemasListRequest;
use dokime_protocol::host::RecordSchemasListResponse;
use dokime_protocol::host::ReportSubject;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsGetResponse;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::ReportsPublishResponse;
use dokime_protocol::host::SecretImportChoice;
use dokime_protocol::host::SecretWritable;
use dokime_protocol::host::SecretsDeleteRequest;
use dokime_protocol::host::SecretsDeleteResponse;
use dokime_protocol::host::SecretsExportRequest;
use dokime_protocol::host::SecretsExportResponse;
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::SecretsGetResponse;
use dokime_protocol::host::SecretsImportRequest;
use dokime_protocol::host::SecretsImportResponse;
use dokime_protocol::host::SecretsSetRequest;
use dokime_protocol::host::SecretsSetResponse;
use dokime_protocol::host::SecretsWatchRequest;
use dokime_protocol::host::SecretsWatchResponse;
use dokime_protocol::host::SelectionPlan;
use dokime_protocol::host::SettingsSchemaRequest;
use dokime_protocol::host::SettingsSchemaResponse;
use dokime_protocol::host::StreamSchemasGetRequest;
use dokime_protocol::host::StreamSchemasGetResponse;
use dokime_protocol::host::StreamSchemasListRequest;
use dokime_protocol::host::StreamSchemasListResponse;
use dokime_protocol::host::SuitesGetRequest;
use dokime_protocol::host::SuitesGetResponse;
use dokime_protocol::host::SuitesListRequest;
use dokime_protocol::host::SuitesListResponse;
use dokime_protocol::host::backend::OverviewGetRequest;
use dokime_protocol::host::backend::OverviewGetResponse;
use dokime_protocol::host::backend::OverviewWatchRequest;
use dokime_protocol::host::backend::OverviewWatchResponse;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ArtifactsCreateResponse;
use dokime_protocol::host::content::ArtifactsGetRequest;
use dokime_protocol::host::content::ArtifactsGetResponse;
use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::content::ArtifactsListResponse;
use dokime_protocol::host::content::ArtifactsWatchRequest;
use dokime_protocol::host::content::ArtifactsWatchResponse;
use dokime_protocol::host::content::ExecutionFilesGetRequest;
use dokime_protocol::host::content::ExecutionFilesGetResponse;
use dokime_protocol::host::content::ExecutionFilesListRequest;
use dokime_protocol::host::content::ExecutionFilesListResponse;
use dokime_protocol::host::content::ExecutionFilesWatchRequest;
use dokime_protocol::host::content::ExecutionFilesWatchResponse;
use dokime_protocol::host::evidence::AnnotationOwner;
use dokime_protocol::host::evidence::AnnotationSelection;
use dokime_protocol::host::evidence::BucketRequest;
use dokime_protocol::host::evidence::CapturedRecordSelection;
use dokime_protocol::host::evidence::HealthEventsListRequest;
use dokime_protocol::host::evidence::HealthEventsListResponse;
use dokime_protocol::host::evidence::HealthEventsWatchRequest;
use dokime_protocol::host::evidence::HealthEventsWatchResponse;
use dokime_protocol::host::evidence::NoteWritable;
use dokime_protocol::host::evidence::NotesCreateRequest;
use dokime_protocol::host::evidence::NotesCreateResponse;
use dokime_protocol::host::evidence::NotesGetRequest;
use dokime_protocol::host::evidence::NotesGetResponse;
use dokime_protocol::host::evidence::NotesListRequest;
use dokime_protocol::host::evidence::NotesListResponse;
use dokime_protocol::host::evidence::NotesSetRequest;
use dokime_protocol::host::evidence::NotesSetResponse;
use dokime_protocol::host::evidence::NotesWatchRequest;
use dokime_protocol::host::evidence::NotesWatchResponse;
use dokime_protocol::host::evidence::RecordScope;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::RecordsListResponse;
use dokime_protocol::host::evidence::RecordsWatchRequest;
use dokime_protocol::host::evidence::RecordsWatchResponse;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::evidence::SamplesListResponse;
use dokime_protocol::host::evidence::SamplesWatchRequest;
use dokime_protocol::host::evidence::SamplesWatchResponse;
use dokime_protocol::host::evidence::TraceabilityGetRequest;
use dokime_protocol::host::evidence::TraceabilityGetResponse;
use dokime_protocol::host::evidence::TraceabilityWatchRequest;
use dokime_protocol::host::evidence::TraceabilityWatchResponse;
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::CommandsCreateResponse;
use dokime_protocol::host::executions::CommandsGetRequest;
use dokime_protocol::host::executions::CommandsGetResponse;
use dokime_protocol::host::executions::ExecutionCommandAction;
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::executions::ProgressGetResponse;
use dokime_protocol::host::executions::ProgressWatchRequest;
use dokime_protocol::host::executions::ProgressWatchResponse;
use dokime_protocol::host::executions::SchemasGetRequest;
use dokime_protocol::host::executions::SchemasGetResponse;
use dokime_protocol::host::files::UploadsAppendRequest;
use dokime_protocol::host::files::UploadsAppendResponse;
use dokime_protocol::host::files::UploadsCreateRequest;
use dokime_protocol::host::files::UploadsCreateResponse;
use dokime_protocol::host::files::UploadsDeleteRequest;
use dokime_protocol::host::files::UploadsDeleteResponse;
use dokime_protocol::host::files::UploadsGetRequest;
use dokime_protocol::host::files::UploadsGetResponse;
use dokime_protocol::host::files::UploadsPublishRequest;
use dokime_protocol::host::files::UploadsPublishResponse;
use dokime_protocol::host::files::UploadsResumeRequest;
use dokime_protocol::host::files::UploadsResumeResponse;
use dokime_protocol::host::media::CaptureKind;
use dokime_protocol::host::media::CapturesCreateRequest;
use dokime_protocol::host::media::CapturesCreateResponse;
use dokime_protocol::host::media::CapturesGetRequest;
use dokime_protocol::host::media::CapturesGetResponse;
use dokime_protocol::host::media::CapturesStopRequest;
use dokime_protocol::host::media::CapturesStopResponse;
use dokime_protocol::host::media::CapturesWatchRequest;
use dokime_protocol::host::media::CapturesWatchResponse;
use dokime_protocol::host::media::RecordingAudio;
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::host::plots::PlotSettingsValue;
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsListResponse;
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_protocol::host::plots::SnapshotsWatchResponse;
use dokime_protocol::host::prompts::AnswersCreateRequest;
use dokime_protocol::host::prompts::AnswersCreateResponse;
use dokime_protocol::host::storage::SourcesGetRequest;
use dokime_protocol::host::storage::SourcesGetResponse;
use dokime_protocol::host::storage::SourcesSetRequest;
use dokime_protocol::host::storage::SourcesSetResponse;
use dokime_protocol::model::ArtifactScope;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::MapValue;
use dokime_protocol::model::Metadata;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Window;
use harmos::client::{CallOptions, ClientTransport, Watching, action_error, request_id};
use harmos_protocol::host::InvocationsCancelRequest;
use harmos_protocol::host::InvocationsCancelResponse;
use harmos_protocol::host::InvocationsCreateRequest;
use harmos_protocol::host::InvocationsCreateResponse;
use harmos_protocol::host::InvocationsGetRequest;
use harmos_protocol::host::InvocationsGetResponse;
use harmos_protocol::host::InvocationsWatchRequest;
use harmos_protocol::host::InvocationsWatchResponse;
use harmos_protocol::host::OperationsGetRequest;
use harmos_protocol::host::OperationsGetResponse;
use harmos_protocol::host::OperationsListRequest;
use harmos_protocol::host::OperationsListResponse;
use harmos_protocol::host::PluginEntry;
use harmos_protocol::host::PluginPlacement;
use harmos_protocol::host::PluginValue;
use harmos_protocol::host::PluginsCreateRequest;
use harmos_protocol::host::PluginsCreateResponse;
use harmos_protocol::host::PluginsDeleteRequest;
use harmos_protocol::host::PluginsDeleteResponse;
use harmos_protocol::host::PluginsGetRequest;
use harmos_protocol::host::PluginsGetResponse;
use harmos_protocol::host::PluginsListRequest;
use harmos_protocol::host::PluginsListResponse;
use harmos_protocol::host::PluginsReplaceRequest;
use harmos_protocol::host::PluginsReplaceResponse;
use harmos_protocol::host::PluginsRestartRequest;
use harmos_protocol::host::PluginsRestartResponse;
use harmos_protocol::host::PluginsSetRequest;
use harmos_protocol::host::PluginsSetResponse;
use harmos_protocol::host::PluginsWatchRequest;
use harmos_protocol::host::PluginsWatchResponse;
use harmos_protocol::host::SchemaDescription;
use harmos_protocol::host::SchemasListRequest;
use harmos_protocol::host::SchemasListResponse;
use harmos_protocol::host::StatusGetRequest;
use harmos_protocol::host::StatusGetResponse;
use harmos_protocol::host::StatusWatchRequest;
use harmos_protocol::host::StatusWatchResponse;
use harmos_protocol::host::StreamIdentity;
use harmos_protocol::host::StreamsListRequest;
use harmos_protocol::host::StreamsListResponse;
use harmos_protocol::host::TimeGetRequest;
use harmos_protocol::host::TimeGetResponse;
use harmos_protocol::host::TimeSleepRequest;
use harmos_protocol::host::TimeSleepResponse;
use harmos_protocol::host::access::CertificatesCreateRequest;
use harmos_protocol::host::access::CertificatesCreateResponse;
use harmos_protocol::host::access::ListenerGetRequest;
use harmos_protocol::host::access::ListenerGetResponse;
use harmos_protocol::host::access::ListenerSetRequest;
use harmos_protocol::host::access::ListenerSetResponse;
use harmos_protocol::host::access::ListenerValue;
use harmos_protocol::host::access::PairingsCreateRequest;
use harmos_protocol::host::access::PairingsCreateResponse;
use harmos_protocol::host::access::PairingsDeleteRequest;
use harmos_protocol::host::access::PairingsDeleteResponse;
use harmos_protocol::host::access::PairingsListRequest;
use harmos_protocol::host::access::PairingsListResponse;
use harmos_protocol::host::plugins::SettingsListRequest;
use harmos_protocol::host::plugins::SettingsListResponse;
use harmos_protocol::host::status::DiagnosticsGetRequest;
use harmos_protocol::host::status::DiagnosticsGetResponse;
use harmos_protocol::host::status::LogsListRequest;
use harmos_protocol::host::status::LogsListResponse;
use harmos_protocol::host::status::LogsWatchRequest;
use harmos_protocol::host::status::LogsWatchResponse;
use harmos_protocol::host::status::TraceEventsListRequest;
use harmos_protocol::host::status::TraceEventsListResponse;
use harmos_protocol::host::status::TraceEventsWatchRequest;
use harmos_protocol::host::status::TraceEventsWatchResponse;
use harmos_protocol::model::DiscoveredOperation;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::InstanceState;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::Position;
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::Receipt;
use harmos_protocol::model::SettingValues;
use harmos_protocol::model::Upload;
use prost_reflect::ReflectMessage;
use prost_types::Duration;
use prost_types::Timestamp;
use tonic::codegen::http::uri::PathAndQuery;
#[derive(Default)]
pub struct AggregatesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for AggregatesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CyclesGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for CyclesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CyclesSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for CyclesSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CyclesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for CyclesListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CyclesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for CyclesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CyclesCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for CyclesCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CyclesSealOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for CyclesSealOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct BackendOverviewGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for BackendOverviewGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct BackendOverviewWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for BackendOverviewWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct BackendAppearanceOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for BackendAppearanceOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct RecordSchemasGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for RecordSchemasGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct RecordSchemasListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for RecordSchemasListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for ExecutionsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ExecutionsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ExecutionsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsStartOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsStartOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsStopOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsStopOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsCancelOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsCancelOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsSealOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsSealOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsParametersOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for ExecutionsParametersOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsStepsOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for ExecutionsStepsOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CasesCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for CasesCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CasesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for CasesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CasesListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for CasesListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CasesWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for CasesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CasesDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for CasesDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct CasesParametersOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for CasesParametersOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentArtifactsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for ContentArtifactsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentArtifactsCreateOptions {
    pub scope: Option<ArtifactScope>,
    pub request_id: Option<String>,
    pub metadata: Option<Metadata>,
    pub context: CallOptions,
}
impl Clone for ContentArtifactsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            request_id: self.request_id.clone(),
            metadata: self.metadata.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentArtifactsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ContentArtifactsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentArtifactsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ContentArtifactsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentExecutionFilesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for ContentExecutionFilesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentExecutionFilesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ContentExecutionFilesListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ContentExecutionFilesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ContentExecutionFilesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PlotsSnapshotsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for PlotsSnapshotsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PlotsSnapshotsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for PlotsSnapshotsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct DocumentsGetOptions {
    pub structure_only: Option<bool>,
    pub card_ids: Option<Vec<String>>,
    pub page_token: Option<String>,
    pub max_bytes: Option<u64>,
    pub max_frames: Option<u32>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for DocumentsGetOptions {
    fn clone(&self) -> Self {
        Self {
            structure_only: self.structure_only,
            card_ids: self.card_ids.clone(),
            page_token: self.page_token.clone(),
            max_bytes: self.max_bytes,
            max_frames: self.max_frames,
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct DocumentsWatchOptions {
    pub structure_only: Option<bool>,
    pub card_ids: Option<Vec<String>>,
    pub page_token: Option<String>,
    pub max_bytes: Option<u64>,
    pub max_frames: Option<u32>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for DocumentsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            structure_only: self.structure_only,
            card_ids: self.card_ids.clone(),
            page_token: self.page_token.clone(),
            max_bytes: self.max_bytes,
            max_frames: self.max_frames,
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceNotesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for EvidenceNotesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceNotesSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for EvidenceNotesSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceNotesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceNotesListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceNotesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceNotesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceNotesCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for EvidenceNotesCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceHealthEventsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceHealthEventsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceHealthEventsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceHealthEventsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceRecordsListOptions {
    pub scope: Option<RecordScope>,
    pub versions: Option<Vec<u32>>,
    pub between: Option<Window>,
    pub step: Option<String>,
    pub selection: Option<CapturedRecordSelection>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceRecordsListOptions {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            versions: self.versions.clone(),
            between: self.between,
            step: self.step.clone(),
            selection: self.selection.clone(),
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceRecordsWatchOptions {
    pub versions: Option<Vec<u32>>,
    pub between: Option<Window>,
    pub step: Option<String>,
    pub selection: Option<CapturedRecordSelection>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceRecordsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            versions: self.versions.clone(),
            between: self.between,
            step: self.step.clone(),
            selection: self.selection.clone(),
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceSamplesListOptions {
    pub versions: Option<Vec<u32>>,
    pub fields: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub signals: Option<Vec<String>>,
    pub between: Option<Window>,
    pub bucket: Option<BucketRequest>,
    pub latest: Option<bool>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceSamplesListOptions {
    fn clone(&self) -> Self {
        Self {
            versions: self.versions.clone(),
            fields: self.fields.clone(),
            sources: self.sources.clone(),
            signals: self.signals.clone(),
            between: self.between,
            bucket: self.bucket,
            latest: self.latest,
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceSamplesWatchOptions {
    pub versions: Option<Vec<u32>>,
    pub fields: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub signals: Option<Vec<String>>,
    pub between: Option<Window>,
    pub bucket: Option<BucketRequest>,
    pub latest: Option<bool>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for EvidenceSamplesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            versions: self.versions.clone(),
            fields: self.fields.clone(),
            sources: self.sources.clone(),
            signals: self.signals.clone(),
            between: self.between,
            bucket: self.bucket,
            latest: self.latest,
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceTraceabilityGetOptions {
    pub scope: Option<EvidenceScope>,
    pub page: Option<PageRequest>,
    pub at: Option<ReadCut>,
    pub entry: Option<String>,
    pub context: CallOptions,
}
impl Clone for EvidenceTraceabilityGetOptions {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            page: self.page.clone(),
            at: self.at.clone(),
            entry: self.entry.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceTraceabilityWatchOptions {
    pub page: Option<PageRequest>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for EvidenceTraceabilityWatchOptions {
    fn clone(&self) -> Self {
        Self {
            page: self.page.clone(),
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceStatisticsOptions {
    pub scope: Option<EvidenceScope>,
    pub versions: Option<Vec<u32>>,
    pub between: Option<Window>,
    pub outputs: Option<Vec<Output>>,
    pub step: Option<String>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for EvidenceStatisticsOptions {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            versions: self.versions.clone(),
            between: self.between,
            outputs: self.outputs.clone(),
            step: self.step.clone(),
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct EvidenceArchiveOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for EvidenceArchiveOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsCommandsGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for ExecutionsCommandsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsCommandsCreateOptions {
    pub request_id: Option<String>,
    pub step_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ExecutionsCommandsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            step_id: self.step_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsProgressGetOptions {
    pub name: Option<String>,
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for ExecutionsProgressGetOptions {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsProgressWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for ExecutionsProgressWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ExecutionsSchemasGetOptions {
    pub at: Option<Position>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for ExecutionsSchemasGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesUploadsCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for FilesUploadsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesUploadsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for FilesUploadsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesUploadsResumeOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for FilesUploadsResumeOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesUploadsAppendOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for FilesUploadsAppendOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesUploadsPublishOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for FilesUploadsPublishOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesUploadsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for FilesUploadsDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for FilesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesReadOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for FilesReadOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesWatchOptions {
    pub file: Option<FileReference>,
    pub upload: Option<Upload>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for FilesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            upload: self.upload.clone(),
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct FilesDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for FilesDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct MediaCapturesCreateOptions {
    pub input: Option<u32>,
    pub camera: Option<u32>,
    pub display: Option<u32>,
    pub maximum_seconds: Option<u32>,
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for MediaCapturesCreateOptions {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            camera: self.camera,
            display: self.display,
            maximum_seconds: self.maximum_seconds,
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct MediaCapturesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for MediaCapturesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct MediaCapturesWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for MediaCapturesWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct MediaCapturesStopOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for MediaCapturesStopOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PlotsSettingsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PlotsSettingsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PlotsSettingsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for PlotsSettingsSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsSecretsListOptions {
    pub name: Option<String>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PluginsSecretsListOptions {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PromptDefinitionsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PromptDefinitionsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PromptDefinitionsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PromptDefinitionsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PromptsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PromptsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PromptsListOptions {
    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for PromptsListOptions {
    fn clone(&self) -> Self {
        Self {
            cycle_id: self.cycle_id.clone(),
            execution_id: self.execution_id.clone(),
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PromptsWatchOptions {
    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for PromptsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            cycle_id: self.cycle_id.clone(),
            execution_id: self.execution_id.clone(),
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PromptsAnswersCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for PromptsAnswersCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SuitesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SuitesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SuitesListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SuitesListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ReportsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for ReportsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ReportsCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ReportsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct ReportsPublishOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for ReportsPublishOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SecretsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for SecretsSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SecretsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SecretsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for SecretsDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsExportOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SecretsExportOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SecretsImportOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for SecretsImportOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SettingsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SettingsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SettingsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for SettingsSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SettingsSchemaOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for SettingsSchemaOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StorageSourcesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for StorageSourcesGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StorageSourcesSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for StorageSourcesSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StreamSchemasGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for StreamSchemasGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StreamSchemasListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for StreamSchemasListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct AccessCertificatesCreateOptions {
    pub context: CallOptions,
}
impl Clone for AccessCertificatesCreateOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct AccessListenerGetOptions {
    pub context: CallOptions,
}
impl Clone for AccessListenerGetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct AccessListenerSetOptions {
    pub context: CallOptions,
}
impl Clone for AccessListenerSetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct AccessPairingsCreateOptions {
    pub context: CallOptions,
}
impl Clone for AccessPairingsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct AccessPairingsListOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for AccessPairingsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct AccessPairingsDeleteOptions {
    pub context: CallOptions,
}
impl Clone for AccessPairingsDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct InvocationsCreateOptions {
    pub context: CallOptions,
}
impl Clone for InvocationsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct InvocationsGetOptions {
    pub context: CallOptions,
}
impl Clone for InvocationsGetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct InvocationsCancelOptions {
    pub context: CallOptions,
}
impl Clone for InvocationsCancelOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct InvocationsWatchOptions {
    pub context: CallOptions,
}
impl Clone for InvocationsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct OperationsListOptions {
    pub context: CallOptions,
}
impl Clone for OperationsListOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct OperationsGetOptions {
    pub context: CallOptions,
}
impl Clone for OperationsGetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsSettingsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PluginsSettingsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsSettingsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for PluginsSettingsSetOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PluginsGetOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PluginsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsWatchOptions {
    pub name: Option<String>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
impl Clone for PluginsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsCreateOptions {
    pub request_id: Option<String>,
    pub origin: Option<String>,
    pub context: CallOptions,
}
impl Clone for PluginsCreateOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            origin: self.origin.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsSetOptions {
    pub context: CallOptions,
}
impl Clone for PluginsSetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsDeleteOptions {
    pub context: CallOptions,
}
impl Clone for PluginsDeleteOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsRestartOptions {
    pub context: CallOptions,
}
impl Clone for PluginsRestartOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct PluginsReplaceOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
impl Clone for PluginsReplaceOptions {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct SchemasListOptions {
    pub schema: Option<String>,
    pub context: CallOptions,
}
impl Clone for SchemasListOptions {
    fn clone(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusDiagnosticsGetOptions {
    pub context: CallOptions,
}
impl Clone for StatusDiagnosticsGetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusLogsListOptions {
    pub at: Option<Position>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for StatusLogsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusLogsWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for StatusLogsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusTraceEventsListOptions {
    pub at: Option<Position>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
impl Clone for StatusTraceEventsListOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            page: self.page.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusTraceEventsWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for StatusTraceEventsWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusGetOptions {
    pub context: CallOptions,
}
impl Clone for StatusGetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StatusWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
impl Clone for StatusWatchOptions {
    fn clone(&self) -> Self {
        Self {
            at: self.at.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct StreamsListOptions {
    pub schema: Option<String>,
    pub context: CallOptions,
}
impl Clone for StreamsListOptions {
    fn clone(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct TimeGetOptions {
    pub context: CallOptions,
}
impl Clone for TimeGetOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Default)]
pub struct TimeSleepOptions {
    pub context: CallOptions,
}
impl Clone for TimeSleepOptions {
    fn clone(&self) -> Self {
        Self {
            context: CallOptions {
                metadata: self.context.metadata.clone(),
                timeout: self.context.timeout,
            },
        }
    }
}
#[derive(Clone)]
pub struct Root {
    pub access: Access,
    pub aggregates: Aggregates,
    pub backend: Backend,
    pub cases: Cases,
    pub content: Content,
    pub cycles: Cycles,
    pub documents: Documents,
    pub evidence: Evidence,
    pub executions: Executions,
    pub files: Files,
    pub invocations: Invocations,
    pub media: Media,
    pub operations: Operations,
    pub plots: Plots,
    pub plugins: Plugins,
    pub prompt_definitions: PromptDefinitions,
    pub prompts: Prompts,
    pub record_schemas: RecordSchemas,
    pub reports: Reports,
    pub schemas: Schemas,
    pub secrets: Secrets,
    pub settings: Settings,
    pub status: Status,
    pub storage: Storage,
    pub stream_schemas: StreamSchemas,
    pub streams: Streams,
    pub suites: Suites,
    pub time: Time,
}
impl Root {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            access: Access::new(client.clone()),
            aggregates: Aggregates::new(client.clone()),
            backend: Backend::new(client.clone()),
            cases: Cases::new(client.clone()),
            content: Content::new(client.clone()),
            cycles: Cycles::new(client.clone()),
            documents: Documents::new(client.clone()),
            evidence: Evidence::new(client.clone()),
            executions: Executions::new(client.clone()),
            files: Files::new(client.clone()),
            invocations: Invocations::new(client.clone()),
            media: Media::new(client.clone()),
            operations: Operations::new(client.clone()),
            plots: Plots::new(client.clone()),
            plugins: Plugins::new(client.clone()),
            prompt_definitions: PromptDefinitions::new(client.clone()),
            prompts: Prompts::new(client.clone()),
            record_schemas: RecordSchemas::new(client.clone()),
            reports: Reports::new(client.clone()),
            schemas: Schemas::new(client.clone()),
            secrets: Secrets::new(client.clone()),
            settings: Settings::new(client.clone()),
            status: Status::new(client.clone()),
            storage: Storage::new(client.clone()),
            stream_schemas: StreamSchemas::new(client.clone()),
            streams: Streams::new(client.clone()),
            suites: Suites::new(client.clone()),
            time: Time::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct Access {
    pub certificates: AccessCertificates,
    pub listener: AccessListener,
    pub pairings: AccessPairings,
}
impl Access {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            certificates: AccessCertificates::new(client.clone()),
            listener: AccessListener::new(client.clone()),
            pairings: AccessPairings::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct AccessCertificates {
    client: ClientTransport,
}
impl AccessCertificates {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self) -> Result<String, tonic::Status> {
        self.create_with(AccessCertificatesCreateOptions::default()).await
    }
    pub async fn create_with(&self, options: AccessCertificatesCreateOptions) -> Result<String, tonic::Status> {
        let value = CertificatesCreateRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CertificatesCreateRequest, CertificatesCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Certificates/Create"),
                request,
                CertificatesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.certificates.create", error))?;
        let response = response.into_inner();
        Ok(response.certificate_pem)
    }
}
#[derive(Clone)]
pub struct AccessListener {
    client: ClientTransport,
}
impl AccessListener {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<ListenerGetResponse, tonic::Status> {
        self.get_with(AccessListenerGetOptions::default()).await
    }
    pub async fn get_with(&self, options: AccessListenerGetOptions) -> Result<ListenerGetResponse, tonic::Status> {
        let value = ListenerGetRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ListenerGetRequest, ListenerGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Listener/Get"),
                request,
                ListenerGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.listener.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(&self, value: ListenerValue, expected_revision: Vec<u8>) -> Result<ListenerSetResponse, tonic::Status> {
        self.set_with(value, expected_revision, AccessListenerSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: ListenerValue, expected_revision: Vec<u8>, options: AccessListenerSetOptions,
    ) -> Result<ListenerSetResponse, tonic::Status> {
        let value = ListenerSetRequest { value: Some(value), expected_revision };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ListenerSetRequest, ListenerSetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Listener/Set"),
                request,
                ListenerSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.listener.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct AccessPairings {
    client: ClientTransport,
}
impl AccessPairings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, label: impl Into<String>) -> Result<PairingsCreateResponse, tonic::Status> {
        self.create_with(label, AccessPairingsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, label: impl Into<String>, options: AccessPairingsCreateOptions,
    ) -> Result<PairingsCreateResponse, tonic::Status> {
        let value = PairingsCreateRequest { label: label.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PairingsCreateRequest, PairingsCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Pairings/Create"),
                request,
                PairingsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.pairings.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self) -> Result<PairingsListResponse, tonic::Status> {
        self.list_with(AccessPairingsListOptions::default()).await
    }
    pub async fn list_with(&self, options: AccessPairingsListOptions) -> Result<PairingsListResponse, tonic::Status> {
        let value = PairingsListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PairingsListRequest, PairingsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Pairings/List"),
                request,
                PairingsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.pairings.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>) -> Result<PairingsDeleteResponse, tonic::Status> {
        self.delete_with(name, AccessPairingsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, options: AccessPairingsDeleteOptions,
    ) -> Result<PairingsDeleteResponse, tonic::Status> {
        let value = PairingsDeleteRequest { name: name.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PairingsDeleteRequest, PairingsDeleteResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Pairings/Delete"),
                request,
                PairingsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.pairings.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Aggregates {
    client: ClientTransport,
}
impl Aggregates {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, source: QuerySource, filters: Vec<QueryFilter>, group_by: Vec<String>, aggregates: Vec<QueryAggregate>, limit: u32,
    ) -> Result<AggregatesGetResponse, tonic::Status> {
        self.get_with(source, filters, group_by, aggregates, limit, AggregatesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, source: QuerySource, filters: Vec<QueryFilter>, group_by: Vec<String>, aggregates: Vec<QueryAggregate>, limit: u32,
        options: AggregatesGetOptions,
    ) -> Result<AggregatesGetResponse, tonic::Status> {
        let value = AggregatesGetRequest {
            source: source as i32,
            filters,
            group_by,
            aggregates,
            limit,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AggregatesGetRequest, AggregatesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Aggregates/Get"),
                request,
                AggregatesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("aggregates.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Backend {
    client: ClientTransport,
    pub overview: BackendOverview,
}
impl Backend {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            overview: BackendOverview::new(client.clone()),
            client,
        }
    }
    pub async fn appearance(&self) -> Result<BackendAppearanceResponse, tonic::Status> {
        self.appearance_with(BackendAppearanceOptions::default()).await
    }
    pub async fn appearance_with(&self, options: BackendAppearanceOptions) -> Result<BackendAppearanceResponse, tonic::Status> {
        let value = BackendAppearanceRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<BackendAppearanceRequest, BackendAppearanceResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Backend/Appearance"),
                request,
                BackendAppearanceResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("backend.appearance", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct BackendOverview {
    client: ClientTransport,
}
impl BackendOverview {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<OverviewGetResponse, tonic::Status> {
        self.get_with(BackendOverviewGetOptions::default()).await
    }
    pub async fn get_with(&self, options: BackendOverviewGetOptions) -> Result<OverviewGetResponse, tonic::Status> {
        let value = OverviewGetRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<OverviewGetRequest, OverviewGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.backend.Overview/Get"),
                request,
                OverviewGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("backend.overview.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<OverviewWatchResponse>, tonic::Status> {
        self.watch_with(BackendOverviewWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: BackendOverviewWatchOptions) -> Result<Watching<OverviewWatchResponse>, tonic::Status> {
        let value = OverviewWatchRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<OverviewWatchRequest, OverviewWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.backend.Overview/Watch"),
                request,
                OverviewWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("backend.overview.watch", error))?;
        let response = response.with_action("backend.overview.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Cases {
    client: ClientTransport,
}
impl Cases {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, plan: SelectionPlan, validate_only: bool) -> Result<CasesCreateResponse, tonic::Status> {
        self.create_with(plan, validate_only, CasesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, plan: SelectionPlan, validate_only: bool, options: CasesCreateOptions,
    ) -> Result<CasesCreateResponse, tonic::Status> {
        let value = CasesCreateRequest {
            plan: Some(plan),
            validate_only,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesCreateRequest, CasesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Create"),
                request,
                CasesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn get(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<CasesGetResponse, tonic::Status> {
        self.get_with(name, parent, CasesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: CasesGetOptions,
    ) -> Result<CasesGetResponse, tonic::Status> {
        let value = CasesGetRequest {
            name: name.into(),
            parent: parent.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesGetRequest, CasesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Get"),
                request,
                CasesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<CasesListResponse, tonic::Status> {
        self.list_with(parent, CasesListOptions::default()).await
    }
    pub async fn list_with(&self, parent: impl Into<String>, options: CasesListOptions) -> Result<CasesListResponse, tonic::Status> {
        let value = CasesListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesListRequest, CasesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/List"),
                request,
                CasesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, parent: impl Into<String>) -> Result<Watching<CasesWatchResponse>, tonic::Status> {
        self.watch_with(parent, CasesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, options: CasesWatchOptions,
    ) -> Result<Watching<CasesWatchResponse>, tonic::Status> {
        let value = CasesWatchRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<CasesWatchRequest, CasesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Watch"),
                request,
                CasesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.watch", error))?;
        let response = response.with_action("cases.watch");
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<Receipt, tonic::Status> {
        self.delete_with(name, parent, CasesDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: CasesDeleteOptions,
    ) -> Result<Receipt, tonic::Status> {
        let value = CasesDeleteRequest {
            name: name.into(),
            parent: parent.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesDeleteRequest, CasesDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Delete"),
                request,
                CasesDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| tonic::Status::data_loss("Read Cases.Delete: Missing Payload"))
    }
    pub async fn parameters(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<CasesParametersResponse, tonic::Status> {
        self.parameters_with(name, parent, CasesParametersOptions::default()).await
    }
    pub async fn parameters_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: CasesParametersOptions,
    ) -> Result<CasesParametersResponse, tonic::Status> {
        let value = CasesParametersRequest {
            name: name.into(),
            parent: parent.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesParametersRequest, CasesParametersResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Parameters"),
                request,
                CasesParametersResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.parameters", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Content {
    pub artifacts: ContentArtifacts,
    pub execution_files: ContentExecutionFiles,
}
impl Content {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            artifacts: ContentArtifacts::new(client.clone()),
            execution_files: ContentExecutionFiles::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct ContentArtifacts {
    client: ClientTransport,
}
impl ContentArtifacts {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<ArtifactsGetResponse, tonic::Status> {
        self.get_with(name, ContentArtifactsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, options: ContentArtifactsGetOptions,
    ) -> Result<ArtifactsGetResponse, tonic::Status> {
        let value = ArtifactsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ArtifactsGetRequest, ArtifactsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/Get"),
                request,
                ArtifactsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn create(
        &self, file: FileReference, filename: impl Into<String>, content_type: impl Into<String>,
    ) -> Result<ArtifactsCreateResponse, tonic::Status> {
        self.create_with(file, filename, content_type, ContentArtifactsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, file: FileReference, filename: impl Into<String>, content_type: impl Into<String>, options: ContentArtifactsCreateOptions,
    ) -> Result<ArtifactsCreateResponse, tonic::Status> {
        let value = ArtifactsCreateRequest {
            file: Some(file),
            scope: options.scope,
            filename: filename.into(),
            content_type: content_type.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            metadata: options.metadata,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ArtifactsCreateRequest, ArtifactsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/Create"),
                request,
                ArtifactsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, scope: ArtifactScope) -> Result<ArtifactsListResponse, tonic::Status> {
        self.list_with(scope, ContentArtifactsListOptions::default()).await
    }
    pub async fn list_with(
        &self, scope: ArtifactScope, options: ContentArtifactsListOptions,
    ) -> Result<ArtifactsListResponse, tonic::Status> {
        let value = ArtifactsListRequest {
            scope: Some(scope),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ArtifactsListRequest, ArtifactsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/List"),
                request,
                ArtifactsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, scope: ArtifactScope) -> Result<Watching<ArtifactsWatchResponse>, tonic::Status> {
        self.watch_with(scope, ContentArtifactsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: ArtifactScope, options: ContentArtifactsWatchOptions,
    ) -> Result<Watching<ArtifactsWatchResponse>, tonic::Status> {
        let value = ArtifactsWatchRequest {
            scope: Some(scope),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ArtifactsWatchRequest, ArtifactsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/Watch"),
                request,
                ArtifactsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.watch", error))?;
        let response = response.with_action("content.artifacts.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ContentExecutionFiles {
    client: ClientTransport,
}
impl ContentExecutionFiles {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, parent: impl Into<String>, name: impl Into<String>) -> Result<ExecutionFilesGetResponse, tonic::Status> {
        self.get_with(parent, name, ContentExecutionFilesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, parent: impl Into<String>, name: impl Into<String>, options: ContentExecutionFilesGetOptions,
    ) -> Result<ExecutionFilesGetResponse, tonic::Status> {
        let value = ExecutionFilesGetRequest {
            parent: parent.into(),
            name: name.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionFilesGetRequest, ExecutionFilesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.ExecutionFiles/Get"),
                request,
                ExecutionFilesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.execution_files.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<ExecutionFilesListResponse, tonic::Status> {
        self.list_with(parent, ContentExecutionFilesListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: ContentExecutionFilesListOptions,
    ) -> Result<ExecutionFilesListResponse, tonic::Status> {
        let value = ExecutionFilesListRequest {
            parent: parent.into(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionFilesListRequest, ExecutionFilesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.ExecutionFiles/List"),
                request,
                ExecutionFilesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.execution_files.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, parent: impl Into<String>) -> Result<Watching<ExecutionFilesWatchResponse>, tonic::Status> {
        self.watch_with(parent, ContentExecutionFilesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, options: ContentExecutionFilesWatchOptions,
    ) -> Result<Watching<ExecutionFilesWatchResponse>, tonic::Status> {
        let value = ExecutionFilesWatchRequest {
            parent: parent.into(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ExecutionFilesWatchRequest, ExecutionFilesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.ExecutionFiles/Watch"),
                request,
                ExecutionFilesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.execution_files.watch", error))?;
        let response = response.with_action("content.execution_files.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Cycles {
    client: ClientTransport,
}
impl Cycles {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<CyclesGetResponse, tonic::Status> {
        self.get_with(name, CyclesGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: CyclesGetOptions) -> Result<CyclesGetResponse, tonic::Status> {
        let value = CyclesGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesGetRequest, CyclesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Get"),
                request,
                CyclesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: CycleWritable, expected_revision: Vec<u8>,
    ) -> Result<CyclesSetResponse, tonic::Status> {
        self.set_with(name, value, expected_revision, CyclesSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: CycleWritable, expected_revision: Vec<u8>, options: CyclesSetOptions,
    ) -> Result<CyclesSetResponse, tonic::Status> {
        let value = CyclesSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesSetRequest, CyclesSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Set"),
                request,
                CyclesSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder,
    ) -> Result<CyclesListResponse, tonic::Status> {
        self.list_with(selection, filter, order, CyclesListOptions::default()).await
    }
    pub async fn list_with(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder, options: CyclesListOptions,
    ) -> Result<CyclesListResponse, tonic::Status> {
        let value = CyclesListRequest {
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesListRequest, CyclesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/List"),
                request,
                CyclesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder,
    ) -> Result<Watching<CyclesWatchResponse>, tonic::Status> {
        self.watch_with(selection, filter, order, CyclesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder, options: CyclesWatchOptions,
    ) -> Result<Watching<CyclesWatchResponse>, tonic::Status> {
        let value = CyclesWatchRequest {
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<CyclesWatchRequest, CyclesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Watch"),
                request,
                CyclesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.watch", error))?;
        let response = response.with_action("cycles.watch");
        Ok(response)
    }
    pub async fn create(
        &self, origins: Vec<Origin>, metadata: Metadata, continue_automatically: bool, source_cycle: impl Into<String>,
    ) -> Result<CyclesCreateResponse, tonic::Status> {
        self.create_with(origins, metadata, continue_automatically, source_cycle, CyclesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, origins: Vec<Origin>, metadata: Metadata, continue_automatically: bool, source_cycle: impl Into<String>,
        options: CyclesCreateOptions,
    ) -> Result<CyclesCreateResponse, tonic::Status> {
        let value = CyclesCreateRequest {
            origins,
            metadata: Some(metadata),
            continue_automatically,
            source_cycle: source_cycle.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesCreateRequest, CyclesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Create"),
                request,
                CyclesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn seal(&self, name: impl Into<String>) -> Result<CyclesSealResponse, tonic::Status> {
        self.seal_with(name, CyclesSealOptions::default()).await
    }
    pub async fn seal_with(&self, name: impl Into<String>, options: CyclesSealOptions) -> Result<CyclesSealResponse, tonic::Status> {
        let value = CyclesSealRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesSealRequest, CyclesSealResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Seal"),
                request,
                CyclesSealResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.seal", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Documents {
    client: ClientTransport,
}
impl Documents {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, scope: DocumentScope, document_id: impl Into<String>) -> Result<DocumentsGetResponse, tonic::Status> {
        self.get_with(scope, document_id, DocumentsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, options: DocumentsGetOptions,
    ) -> Result<DocumentsGetResponse, tonic::Status> {
        let value = DocumentsGetRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            structure_only: options.structure_only.unwrap_or_default(),
            card_ids: options.card_ids.unwrap_or_default(),
            page_token: options.page_token.unwrap_or_default(),
            max_bytes: options.max_bytes.unwrap_or_default(),
            max_frames: options.max_frames.unwrap_or_default(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<DocumentsGetRequest, DocumentsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Documents/Get"),
                request,
                DocumentsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("documents.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, scope: DocumentScope, document_id: impl Into<String>,
    ) -> Result<Watching<DocumentsWatchResponse>, tonic::Status> {
        self.watch_with(scope, document_id, DocumentsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, options: DocumentsWatchOptions,
    ) -> Result<Watching<DocumentsWatchResponse>, tonic::Status> {
        let value = DocumentsWatchRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            structure_only: options.structure_only.unwrap_or_default(),
            card_ids: options.card_ids.unwrap_or_default(),
            page_token: options.page_token.unwrap_or_default(),
            max_bytes: options.max_bytes.unwrap_or_default(),
            max_frames: options.max_frames.unwrap_or_default(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<DocumentsWatchRequest, DocumentsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Documents/Watch"),
                request,
                DocumentsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("documents.watch", error))?;
        let response = response.with_action("documents.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Evidence {
    client: ClientTransport,
    pub health_events: EvidenceHealthEvents,
    pub notes: EvidenceNotes,
    pub records: EvidenceRecords,
    pub samples: EvidenceSamples,
    pub traceability: EvidenceTraceability,
}
impl Evidence {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            health_events: EvidenceHealthEvents::new(client.clone()),
            notes: EvidenceNotes::new(client.clone()),
            records: EvidenceRecords::new(client.clone()),
            samples: EvidenceSamples::new(client.clone()),
            traceability: EvidenceTraceability::new(client.clone()),
            client,
        }
    }
    pub async fn statistics(
        &self, stream: impl Into<String>, field: impl Into<String>,
    ) -> Result<EvidenceStatisticsResponse, tonic::Status> {
        self.statistics_with(stream, field, EvidenceStatisticsOptions::default()).await
    }
    pub async fn statistics_with(
        &self, stream: impl Into<String>, field: impl Into<String>, options: EvidenceStatisticsOptions,
    ) -> Result<EvidenceStatisticsResponse, tonic::Status> {
        let value = EvidenceStatisticsRequest {
            scope: options.scope,
            stream: stream.into(),
            versions: options.versions.unwrap_or_default(),
            between: options.between,
            outputs: options.outputs.unwrap_or_default(),
            field: field.into(),
            step: options.step.unwrap_or_default(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceStatisticsRequest, EvidenceStatisticsResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Evidence/Statistics"),
                request,
                EvidenceStatisticsResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.statistics", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn archive(&self, subject: EvidenceSubject) -> Result<EvidenceArchiveResponse, tonic::Status> {
        self.archive_with(subject, EvidenceArchiveOptions::default()).await
    }
    pub async fn archive_with(
        &self, subject: EvidenceSubject, options: EvidenceArchiveOptions,
    ) -> Result<EvidenceArchiveResponse, tonic::Status> {
        let value = EvidenceArchiveRequest { subject: Some(subject), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceArchiveRequest, EvidenceArchiveResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Evidence/Archive"),
                request,
                EvidenceArchiveResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.archive", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceHealthEvents {
    client: ClientTransport,
}
impl EvidenceHealthEvents {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, selection: AnnotationSelection) -> Result<HealthEventsListResponse, tonic::Status> {
        self.list_with(selection, EvidenceHealthEventsListOptions::default()).await
    }
    pub async fn list_with(
        &self, selection: AnnotationSelection, options: EvidenceHealthEventsListOptions,
    ) -> Result<HealthEventsListResponse, tonic::Status> {
        let value = HealthEventsListRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<HealthEventsListRequest, HealthEventsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.HealthEvents/List"),
                request,
                HealthEventsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.health_events.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, selection: AnnotationSelection) -> Result<Watching<HealthEventsWatchResponse>, tonic::Status> {
        self.watch_with(selection, EvidenceHealthEventsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, selection: AnnotationSelection, options: EvidenceHealthEventsWatchOptions,
    ) -> Result<Watching<HealthEventsWatchResponse>, tonic::Status> {
        let value = HealthEventsWatchRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<HealthEventsWatchRequest, HealthEventsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.HealthEvents/Watch"),
                request,
                HealthEventsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.health_events.watch", error))?;
        let response = response.with_action("evidence.health_events.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceNotes {
    client: ClientTransport,
}
impl EvidenceNotes {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<NotesGetResponse, tonic::Status> {
        self.get_with(name, EvidenceNotesGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: EvidenceNotesGetOptions) -> Result<NotesGetResponse, tonic::Status> {
        let value = NotesGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<NotesGetRequest, NotesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Get"),
                request,
                NotesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: NoteWritable, expected_revision: Vec<u8>,
    ) -> Result<NotesSetResponse, tonic::Status> {
        self.set_with(name, value, expected_revision, EvidenceNotesSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: NoteWritable, expected_revision: Vec<u8>, options: EvidenceNotesSetOptions,
    ) -> Result<NotesSetResponse, tonic::Status> {
        let value = NotesSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<NotesSetRequest, NotesSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Set"),
                request,
                NotesSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, selection: AnnotationSelection) -> Result<NotesListResponse, tonic::Status> {
        self.list_with(selection, EvidenceNotesListOptions::default()).await
    }
    pub async fn list_with(
        &self, selection: AnnotationSelection, options: EvidenceNotesListOptions,
    ) -> Result<NotesListResponse, tonic::Status> {
        let value = NotesListRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<NotesListRequest, NotesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/List"),
                request,
                NotesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, selection: AnnotationSelection) -> Result<Watching<NotesWatchResponse>, tonic::Status> {
        self.watch_with(selection, EvidenceNotesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, selection: AnnotationSelection, options: EvidenceNotesWatchOptions,
    ) -> Result<Watching<NotesWatchResponse>, tonic::Status> {
        let value = NotesWatchRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<NotesWatchRequest, NotesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Watch"),
                request,
                NotesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.watch", error))?;
        let response = response.with_action("evidence.notes.watch");
        Ok(response)
    }
    pub async fn create(&self, owner: AnnotationOwner, value: NoteWritable) -> Result<NotesCreateResponse, tonic::Status> {
        self.create_with(owner, value, EvidenceNotesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, owner: AnnotationOwner, value: NoteWritable, options: EvidenceNotesCreateOptions,
    ) -> Result<NotesCreateResponse, tonic::Status> {
        let value = NotesCreateRequest {
            owner: Some(owner),
            value: Some(value),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<NotesCreateRequest, NotesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Create"),
                request,
                NotesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceRecords {
    client: ClientTransport,
}
impl EvidenceRecords {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, record_type: impl Into<String>) -> Result<RecordsListResponse, tonic::Status> {
        self.list_with(record_type, EvidenceRecordsListOptions::default()).await
    }
    pub async fn list_with(
        &self, record_type: impl Into<String>, options: EvidenceRecordsListOptions,
    ) -> Result<RecordsListResponse, tonic::Status> {
        let value = RecordsListRequest {
            scope: options.scope,
            record_type: record_type.into(),
            versions: options.versions.unwrap_or_default(),
            between: options.between,
            step: options.step.unwrap_or_default(),
            selection: options.selection,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<RecordsListRequest, RecordsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Records/List"),
                request,
                RecordsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.records.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, scope: RecordScope, record_type: impl Into<String>) -> Result<Watching<RecordsWatchResponse>, tonic::Status> {
        self.watch_with(scope, record_type, EvidenceRecordsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: RecordScope, record_type: impl Into<String>, options: EvidenceRecordsWatchOptions,
    ) -> Result<Watching<RecordsWatchResponse>, tonic::Status> {
        let value = RecordsWatchRequest {
            scope: Some(scope),
            record_type: record_type.into(),
            versions: options.versions.unwrap_or_default(),
            between: options.between,
            step: options.step.unwrap_or_default(),
            selection: options.selection,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<RecordsWatchRequest, RecordsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Records/Watch"),
                request,
                RecordsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.records.watch", error))?;
        let response = response.with_action("evidence.records.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceSamples {
    client: ClientTransport,
}
impl EvidenceSamples {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, scope: EvidenceScope, streams: Vec<String>) -> Result<SamplesListResponse, tonic::Status> {
        self.list_with(scope, streams, EvidenceSamplesListOptions::default()).await
    }
    pub async fn list_with(
        &self, scope: EvidenceScope, streams: Vec<String>, options: EvidenceSamplesListOptions,
    ) -> Result<SamplesListResponse, tonic::Status> {
        let value = SamplesListRequest {
            scope: Some(scope),
            streams,
            versions: options.versions.unwrap_or_default(),
            fields: options.fields.unwrap_or_default(),
            sources: options.sources.unwrap_or_default(),
            signals: options.signals.unwrap_or_default(),
            between: options.between,
            bucket: options.bucket,
            latest: options.latest.unwrap_or_default(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SamplesListRequest, SamplesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Samples/List"),
                request,
                SamplesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.samples.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, scope: EvidenceScope, streams: Vec<String>) -> Result<Watching<SamplesWatchResponse>, tonic::Status> {
        self.watch_with(scope, streams, EvidenceSamplesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: EvidenceScope, streams: Vec<String>, options: EvidenceSamplesWatchOptions,
    ) -> Result<Watching<SamplesWatchResponse>, tonic::Status> {
        let value = SamplesWatchRequest {
            scope: Some(scope),
            streams,
            versions: options.versions.unwrap_or_default(),
            fields: options.fields.unwrap_or_default(),
            sources: options.sources.unwrap_or_default(),
            signals: options.signals.unwrap_or_default(),
            between: options.between,
            bucket: options.bucket,
            latest: options.latest.unwrap_or_default(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<SamplesWatchRequest, SamplesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Samples/Watch"),
                request,
                SamplesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.samples.watch", error))?;
        let response = response.with_action("evidence.samples.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceTraceability {
    client: ClientTransport,
}
impl EvidenceTraceability {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<TraceabilityGetResponse, tonic::Status> {
        self.get_with(EvidenceTraceabilityGetOptions::default()).await
    }
    pub async fn get_with(&self, options: EvidenceTraceabilityGetOptions) -> Result<TraceabilityGetResponse, tonic::Status> {
        let value = TraceabilityGetRequest {
            scope: options.scope,
            page: options.page,
            at: options.at,
            entry: options.entry,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<TraceabilityGetRequest, TraceabilityGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Traceability/Get"),
                request,
                TraceabilityGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.traceability.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, scope: EvidenceScope) -> Result<Watching<TraceabilityWatchResponse>, tonic::Status> {
        self.watch_with(scope, EvidenceTraceabilityWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: EvidenceScope, options: EvidenceTraceabilityWatchOptions,
    ) -> Result<Watching<TraceabilityWatchResponse>, tonic::Status> {
        let value = TraceabilityWatchRequest {
            scope: Some(scope),
            page: options.page,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<TraceabilityWatchRequest, TraceabilityWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Traceability/Watch"),
                request,
                TraceabilityWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.traceability.watch", error))?;
        let response = response.with_action("evidence.traceability.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Executions {
    client: ClientTransport,
    pub commands: ExecutionsCommands,
    pub progress: ExecutionsProgress,
    pub schemas: ExecutionsSchemas,
}
impl Executions {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            commands: ExecutionsCommands::new(client.clone()),
            progress: ExecutionsProgress::new(client.clone()),
            schemas: ExecutionsSchemas::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<ExecutionsGetResponse, tonic::Status> {
        self.get_with(name, ExecutionsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: ExecutionsGetOptions) -> Result<ExecutionsGetResponse, tonic::Status> {
        let value = ExecutionsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsGetRequest, ExecutionsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Get"),
                request,
                ExecutionsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: ExecutionWritable, expected_revision: Vec<u8>,
    ) -> Result<ExecutionsSetResponse, tonic::Status> {
        self.set_with(name, value, expected_revision, ExecutionsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: ExecutionWritable, expected_revision: Vec<u8>, options: ExecutionsSetOptions,
    ) -> Result<ExecutionsSetResponse, tonic::Status> {
        let value = ExecutionsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsSetRequest, ExecutionsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Set"),
                request,
                ExecutionsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
    ) -> Result<ExecutionsListResponse, tonic::Status> {
        self.list_with(parent, selection, filter, order, ExecutionsListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
        options: ExecutionsListOptions,
    ) -> Result<ExecutionsListResponse, tonic::Status> {
        let value = ExecutionsListRequest {
            parent: parent.into(),
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsListRequest, ExecutionsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/List"),
                request,
                ExecutionsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
    ) -> Result<Watching<ExecutionsWatchResponse>, tonic::Status> {
        self.watch_with(parent, selection, filter, order, ExecutionsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
        options: ExecutionsWatchOptions,
    ) -> Result<Watching<ExecutionsWatchResponse>, tonic::Status> {
        let value = ExecutionsWatchRequest {
            parent: parent.into(),
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ExecutionsWatchRequest, ExecutionsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Watch"),
                request,
                ExecutionsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.watch", error))?;
        let response = response.with_action("executions.watch");
        Ok(response)
    }
    pub async fn create(
        &self, parent: impl Into<String>, executions: Vec<ExecutionBinding>,
    ) -> Result<ExecutionsCreateResponse, tonic::Status> {
        self.create_with(parent, executions, ExecutionsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, parent: impl Into<String>, executions: Vec<ExecutionBinding>, options: ExecutionsCreateOptions,
    ) -> Result<ExecutionsCreateResponse, tonic::Status> {
        let value = ExecutionsCreateRequest {
            parent: parent.into(),
            executions,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsCreateRequest, ExecutionsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Create"),
                request,
                ExecutionsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn start(&self, name: impl Into<String>, mode: ExecutionMode) -> Result<ExecutionsStartResponse, tonic::Status> {
        self.start_with(name, mode, ExecutionsStartOptions::default()).await
    }
    pub async fn start_with(
        &self, name: impl Into<String>, mode: ExecutionMode, options: ExecutionsStartOptions,
    ) -> Result<ExecutionsStartResponse, tonic::Status> {
        let value = ExecutionsStartRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            mode: mode as i32,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsStartRequest, ExecutionsStartResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Start"),
                request,
                ExecutionsStartResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.start", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn stop(&self, name: impl Into<String>) -> Result<ExecutionsStopResponse, tonic::Status> {
        self.stop_with(name, ExecutionsStopOptions::default()).await
    }
    pub async fn stop_with(
        &self, name: impl Into<String>, options: ExecutionsStopOptions,
    ) -> Result<ExecutionsStopResponse, tonic::Status> {
        let value = ExecutionsStopRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsStopRequest, ExecutionsStopResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Stop"),
                request,
                ExecutionsStopResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.stop", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn cancel(&self, name: impl Into<String>) -> Result<ExecutionsCancelResponse, tonic::Status> {
        self.cancel_with(name, ExecutionsCancelOptions::default()).await
    }
    pub async fn cancel_with(
        &self, name: impl Into<String>, options: ExecutionsCancelOptions,
    ) -> Result<ExecutionsCancelResponse, tonic::Status> {
        let value = ExecutionsCancelRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsCancelRequest, ExecutionsCancelResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Cancel"),
                request,
                ExecutionsCancelResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.cancel", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn seal(&self, name: impl Into<String>) -> Result<ExecutionsSealResponse, tonic::Status> {
        self.seal_with(name, ExecutionsSealOptions::default()).await
    }
    pub async fn seal_with(
        &self, name: impl Into<String>, options: ExecutionsSealOptions,
    ) -> Result<ExecutionsSealResponse, tonic::Status> {
        let value = ExecutionsSealRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsSealRequest, ExecutionsSealResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Seal"),
                request,
                ExecutionsSealResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.seal", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>) -> Result<ExecutionsDeleteResponse, tonic::Status> {
        self.delete_with(name, ExecutionsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, options: ExecutionsDeleteOptions,
    ) -> Result<ExecutionsDeleteResponse, tonic::Status> {
        let value = ExecutionsDeleteRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsDeleteRequest, ExecutionsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Delete"),
                request,
                ExecutionsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn parameters(
        &self, name: impl Into<String>, set: ExecutionParameterSet,
    ) -> Result<ExecutionsParametersResponse, tonic::Status> {
        self.parameters_with(name, set, ExecutionsParametersOptions::default()).await
    }
    pub async fn parameters_with(
        &self, name: impl Into<String>, set: ExecutionParameterSet, options: ExecutionsParametersOptions,
    ) -> Result<ExecutionsParametersResponse, tonic::Status> {
        let value = ExecutionsParametersRequest {
            name: name.into(),
            set: set as i32,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsParametersRequest, ExecutionsParametersResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Parameters"),
                request,
                ExecutionsParametersResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.parameters", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn steps(&self, name: impl Into<String>) -> Result<ExecutionsStepsResponse, tonic::Status> {
        self.steps_with(name, ExecutionsStepsOptions::default()).await
    }
    pub async fn steps_with(
        &self, name: impl Into<String>, options: ExecutionsStepsOptions,
    ) -> Result<ExecutionsStepsResponse, tonic::Status> {
        let value = ExecutionsStepsRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsStepsRequest, ExecutionsStepsResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Steps"),
                request,
                ExecutionsStepsResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.steps", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ExecutionsCommands {
    client: ClientTransport,
}
impl ExecutionsCommands {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<CommandsGetResponse, tonic::Status> {
        self.get_with(name, parent, ExecutionsCommandsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: ExecutionsCommandsGetOptions,
    ) -> Result<CommandsGetResponse, tonic::Status> {
        let value = CommandsGetRequest {
            name: name.into(),
            parent: parent.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CommandsGetRequest, CommandsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Commands/Get"),
                request,
                CommandsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.commands.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn create(
        &self, parent: impl Into<String>, command_id: impl Into<String>, action: ExecutionCommandAction,
    ) -> Result<CommandsCreateResponse, tonic::Status> {
        self.create_with(parent, command_id, action, ExecutionsCommandsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, parent: impl Into<String>, command_id: impl Into<String>, action: ExecutionCommandAction,
        options: ExecutionsCommandsCreateOptions,
    ) -> Result<CommandsCreateResponse, tonic::Status> {
        let value = CommandsCreateRequest {
            parent: parent.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            command_id: command_id.into(),
            action: action as i32,
            step_id: options.step_id,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CommandsCreateRequest, CommandsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Commands/Create"),
                request,
                CommandsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.commands.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ExecutionsProgress {
    client: ClientTransport,
}
impl ExecutionsProgress {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<ProgressGetResponse, tonic::Status> {
        self.get_with(ExecutionsProgressGetOptions::default()).await
    }
    pub async fn get_with(&self, options: ExecutionsProgressGetOptions) -> Result<ProgressGetResponse, tonic::Status> {
        let value = ProgressGetRequest { name: options.name, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ProgressGetRequest, ProgressGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Progress/Get"),
                request,
                ProgressGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.progress.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, name: impl Into<String>) -> Result<Watching<ProgressWatchResponse>, tonic::Status> {
        self.watch_with(name, ExecutionsProgressWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, name: impl Into<String>, options: ExecutionsProgressWatchOptions,
    ) -> Result<Watching<ProgressWatchResponse>, tonic::Status> {
        let value = ProgressWatchRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ProgressWatchRequest, ProgressWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Progress/Watch"),
                request,
                ProgressWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.progress.watch", error))?;
        let response = response.with_action("executions.progress.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ExecutionsSchemas {
    client: ClientTransport,
}
impl ExecutionsSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<SchemasGetResponse, tonic::Status> {
        self.get_with(name, ExecutionsSchemasGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, options: ExecutionsSchemasGetOptions,
    ) -> Result<SchemasGetResponse, tonic::Status> {
        let value = SchemasGetRequest {
            name: name.into(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SchemasGetRequest, SchemasGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Schemas/Get"),
                request,
                SchemasGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.schemas.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Files {
    client: ClientTransport,
    pub uploads: FilesUploads,
}
impl Files {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            uploads: FilesUploads::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, file: FileReference) -> Result<FilesGetResponse, tonic::Status> {
        self.get_with(file, FilesGetOptions::default()).await
    }
    pub async fn get_with(&self, file: FileReference, options: FilesGetOptions) -> Result<FilesGetResponse, tonic::Status> {
        let value = FilesGetRequest { file: Some(file), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesGetRequest, FilesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Get"),
                request,
                FilesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn read(&self, file: FileReference, offset: u64, max_bytes: u64) -> Result<FilesReadResponse, tonic::Status> {
        self.read_with(file, offset, max_bytes, FilesReadOptions::default()).await
    }
    pub async fn read_with(
        &self, file: FileReference, offset: u64, max_bytes: u64, options: FilesReadOptions,
    ) -> Result<FilesReadResponse, tonic::Status> {
        let value = FilesReadRequest {
            file: Some(file),
            offset,
            max_bytes,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesReadRequest, FilesReadResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Read"),
                request,
                FilesReadResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.read", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, offset: u64) -> Result<Watching<FilesWatchResponse>, tonic::Status> {
        self.watch_with(offset, FilesWatchOptions::default()).await
    }
    pub async fn watch_with(&self, offset: u64, options: FilesWatchOptions) -> Result<Watching<FilesWatchResponse>, tonic::Status> {
        let value = FilesWatchRequest {
            target: match (options.file, options.upload) {
                (Some(value), None) => Some(dokime_protocol::host::files_watch_request::Target::File(value)),
                (None, Some(value)) => Some(dokime_protocol::host::files_watch_request::Target::Upload(value)),
                (None, None) => None,
                _ => return Err(tonic::Status::invalid_argument("Choose At Most One target Variant")),
            },
            offset,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<FilesWatchRequest, FilesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Watch"),
                request,
                FilesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.watch", error))?;
        let response = response.with_action("files.watch");
        Ok(response)
    }
    pub async fn delete(&self, file: FileReference) -> Result<Receipt, tonic::Status> {
        self.delete_with(file, FilesDeleteOptions::default()).await
    }
    pub async fn delete_with(&self, file: FileReference, options: FilesDeleteOptions) -> Result<Receipt, tonic::Status> {
        let value = FilesDeleteRequest {
            file: Some(file),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesDeleteRequest, FilesDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Delete"),
                request,
                FilesDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| tonic::Status::data_loss("Read Files.Delete: Missing Payload"))
    }
}
#[derive(Clone)]
pub struct FilesUploads {
    client: ClientTransport,
}
impl FilesUploads {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, metadata: FileMetadata) -> Result<UploadsCreateResponse, tonic::Status> {
        self.create_with(metadata, FilesUploadsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, metadata: FileMetadata, options: FilesUploadsCreateOptions,
    ) -> Result<UploadsCreateResponse, tonic::Status> {
        let value = UploadsCreateRequest {
            metadata: Some(metadata),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<UploadsCreateRequest, UploadsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Create"),
                request,
                UploadsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn get(&self, upload: Upload) -> Result<UploadsGetResponse, tonic::Status> {
        self.get_with(upload, FilesUploadsGetOptions::default()).await
    }
    pub async fn get_with(&self, upload: Upload, options: FilesUploadsGetOptions) -> Result<UploadsGetResponse, tonic::Status> {
        let value = UploadsGetRequest { upload: Some(upload), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<UploadsGetRequest, UploadsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Get"),
                request,
                UploadsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn resume(&self, upload: Upload) -> Result<UploadsResumeResponse, tonic::Status> {
        self.resume_with(upload, FilesUploadsResumeOptions::default()).await
    }
    pub async fn resume_with(&self, upload: Upload, options: FilesUploadsResumeOptions) -> Result<UploadsResumeResponse, tonic::Status> {
        let value = UploadsResumeRequest {
            upload: Some(upload),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<UploadsResumeRequest, UploadsResumeResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Resume"),
                request,
                UploadsResumeResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.resume", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn append(&self, upload: Upload, offset: u64, data: Vec<u8>) -> Result<UploadsAppendResponse, tonic::Status> {
        self.append_with(upload, offset, data, FilesUploadsAppendOptions::default()).await
    }
    pub async fn append_with(
        &self, upload: Upload, offset: u64, data: Vec<u8>, options: FilesUploadsAppendOptions,
    ) -> Result<UploadsAppendResponse, tonic::Status> {
        let value = UploadsAppendRequest {
            upload: Some(upload),
            offset,
            data,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<UploadsAppendRequest, UploadsAppendResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Append"),
                request,
                UploadsAppendResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.append", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn publish(&self, upload: Upload, length: u64, sha256: Vec<u8>) -> Result<UploadsPublishResponse, tonic::Status> {
        self.publish_with(upload, length, sha256, FilesUploadsPublishOptions::default()).await
    }
    pub async fn publish_with(
        &self, upload: Upload, length: u64, sha256: Vec<u8>, options: FilesUploadsPublishOptions,
    ) -> Result<UploadsPublishResponse, tonic::Status> {
        let value = UploadsPublishRequest {
            upload: Some(upload),
            length,
            sha256,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<UploadsPublishRequest, UploadsPublishResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Publish"),
                request,
                UploadsPublishResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.publish", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, upload: Upload) -> Result<UploadsDeleteResponse, tonic::Status> {
        self.delete_with(upload, FilesUploadsDeleteOptions::default()).await
    }
    pub async fn delete_with(&self, upload: Upload, options: FilesUploadsDeleteOptions) -> Result<UploadsDeleteResponse, tonic::Status> {
        let value = UploadsDeleteRequest {
            upload: Some(upload),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<UploadsDeleteRequest, UploadsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Delete"),
                request,
                UploadsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Invocations {
    client: ClientTransport,
}
impl Invocations {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, operation: impl Into<String>, fingerprint: Vec<u8>, payload: Vec<u8>) -> Result<Vec<u8>, tonic::Status> {
        self.create_with(operation, fingerprint, payload, InvocationsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, operation: impl Into<String>, fingerprint: Vec<u8>, payload: Vec<u8>, options: InvocationsCreateOptions,
    ) -> Result<Vec<u8>, tonic::Status> {
        let value = InvocationsCreateRequest {
            operation: operation.into(),
            fingerprint,
            payload,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<InvocationsCreateRequest, InvocationsCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Create"),
                request,
                InvocationsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.create", error))?;
        let response = response.into_inner();
        Ok(response.payload)
    }
    pub async fn get(&self, instance: u64) -> Result<InstanceState, tonic::Status> {
        self.get_with(instance, InvocationsGetOptions::default()).await
    }
    pub async fn get_with(&self, instance: u64, options: InvocationsGetOptions) -> Result<InstanceState, tonic::Status> {
        let value = InvocationsGetRequest { instance };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<InvocationsGetRequest, InvocationsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Get"),
                request,
                InvocationsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.get", error))?;
        let response = response.into_inner();
        response.invocation.ok_or_else(|| tonic::Status::data_loss("Read Invocations.Get: Missing Payload"))
    }
    pub async fn cancel(&self, instance: u64, reason: impl Into<String>) -> Result<InstanceState, tonic::Status> {
        self.cancel_with(instance, reason, InvocationsCancelOptions::default()).await
    }
    pub async fn cancel_with(
        &self, instance: u64, reason: impl Into<String>, options: InvocationsCancelOptions,
    ) -> Result<InstanceState, tonic::Status> {
        let value = InvocationsCancelRequest { instance, reason: reason.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<InvocationsCancelRequest, InvocationsCancelResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Cancel"),
                request,
                InvocationsCancelResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.cancel", error))?;
        let response = response.into_inner();
        response.invocation.ok_or_else(|| tonic::Status::data_loss("Read Invocations.Cancel: Missing Payload"))
    }
    pub async fn watch(&self, instance: u64) -> Result<Watching<InstanceState>, tonic::Status> {
        self.watch_with(instance, InvocationsWatchOptions::default()).await
    }
    pub async fn watch_with(&self, instance: u64, options: InvocationsWatchOptions) -> Result<Watching<InstanceState>, tonic::Status> {
        let value = InvocationsWatchRequest { instance };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<InvocationsWatchRequest, InvocationsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Watch"),
                request,
                InvocationsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.watch", error))?;
        let response = response.with_action("invocations.watch");
        Ok(response.map(|response| response.invocation.ok_or_else(|| tonic::Status::data_loss("Read Invocations.Watch: Missing Payload"))))
    }
}
#[derive(Clone)]
pub struct Media {
    pub captures: MediaCaptures,
}
impl Media {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            captures: MediaCaptures::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct MediaCaptures {
    client: ClientTransport,
}
impl MediaCaptures {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(
        &self, scope: ArtifactScope, kind: CaptureKind, audio: RecordingAudio,
    ) -> Result<CapturesCreateResponse, tonic::Status> {
        self.create_with(scope, kind, audio, MediaCapturesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, scope: ArtifactScope, kind: CaptureKind, audio: RecordingAudio, options: MediaCapturesCreateOptions,
    ) -> Result<CapturesCreateResponse, tonic::Status> {
        let value = CapturesCreateRequest {
            scope: Some(scope),
            kind: kind as i32,
            input: options.input,
            camera: options.camera,
            display: options.display,
            audio: Some(audio),
            maximum_seconds: options.maximum_seconds,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CapturesCreateRequest, CapturesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Create"),
                request,
                CapturesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<CapturesGetResponse, tonic::Status> {
        self.get_with(name, MediaCapturesGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: MediaCapturesGetOptions) -> Result<CapturesGetResponse, tonic::Status> {
        let value = CapturesGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CapturesGetRequest, CapturesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Get"),
                request,
                CapturesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, name: impl Into<String>) -> Result<Watching<CapturesWatchResponse>, tonic::Status> {
        self.watch_with(name, MediaCapturesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, name: impl Into<String>, options: MediaCapturesWatchOptions,
    ) -> Result<Watching<CapturesWatchResponse>, tonic::Status> {
        let value = CapturesWatchRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<CapturesWatchRequest, CapturesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Watch"),
                request,
                CapturesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.watch", error))?;
        let response = response.with_action("media.captures.watch");
        Ok(response)
    }
    pub async fn stop(&self, name: impl Into<String>) -> Result<CapturesStopResponse, tonic::Status> {
        self.stop_with(name, MediaCapturesStopOptions::default()).await
    }
    pub async fn stop_with(
        &self, name: impl Into<String>, options: MediaCapturesStopOptions,
    ) -> Result<CapturesStopResponse, tonic::Status> {
        let value = CapturesStopRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CapturesStopRequest, CapturesStopResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Stop"),
                request,
                CapturesStopResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.stop", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Operations {
    client: ClientTransport,
}
impl Operations {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<DiscoveredOperation>, tonic::Status> {
        self.list_with(OperationsListOptions::default()).await
    }
    pub async fn list_with(&self, options: OperationsListOptions) -> Result<Vec<DiscoveredOperation>, tonic::Status> {
        let value = OperationsListRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<OperationsListRequest, OperationsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Operations/List"),
                request,
                OperationsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("operations.list", error))?;
        let response = response.into_inner();
        Ok(response.operations)
    }
    pub async fn get(&self, id: impl Into<String>) -> Result<DiscoveredOperation, tonic::Status> {
        self.get_with(id, OperationsGetOptions::default()).await
    }
    pub async fn get_with(&self, id: impl Into<String>, options: OperationsGetOptions) -> Result<DiscoveredOperation, tonic::Status> {
        let value = OperationsGetRequest { id: id.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<OperationsGetRequest, OperationsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Operations/Get"),
                request,
                OperationsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("operations.get", error))?;
        let response = response.into_inner();
        response.operation.ok_or_else(|| tonic::Status::data_loss("Read Operations.Get: Missing Payload"))
    }
}
#[derive(Clone)]
pub struct Plots {
    pub settings: PlotsSettings,
    pub snapshots: PlotsSnapshots,
}
impl Plots {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            settings: PlotsSettings::new(client.clone()),
            snapshots: PlotsSnapshots::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct PlotsSettings {
    client: ClientTransport,
}
impl PlotsSettings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>,
    ) -> Result<dokime_protocol::host::plots::SettingsGetResponse, tonic::Status> {
        self.get_with(scope, document_id, plot_id, PlotsSettingsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>, options: PlotsSettingsGetOptions,
    ) -> Result<dokime_protocol::host::plots::SettingsGetResponse, tonic::Status> {
        let value = dokime_protocol::host::plots::SettingsGetRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            plot_id: plot_id.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<dokime_protocol::host::plots::SettingsGetRequest, dokime_protocol::host::plots::SettingsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Settings/Get"),
                request,
                dokime_protocol::host::plots::SettingsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.settings.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>, value: PlotSettingsValue,
        expected_revision: Vec<u8>,
    ) -> Result<dokime_protocol::host::plots::SettingsSetResponse, tonic::Status> {
        self.set_with(scope, document_id, plot_id, value, expected_revision, PlotsSettingsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>, value: PlotSettingsValue,
        expected_revision: Vec<u8>, options: PlotsSettingsSetOptions,
    ) -> Result<dokime_protocol::host::plots::SettingsSetResponse, tonic::Status> {
        let value = dokime_protocol::host::plots::SettingsSetRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            plot_id: plot_id.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<dokime_protocol::host::plots::SettingsSetRequest, dokime_protocol::host::plots::SettingsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Settings/Set"),
                request,
                dokime_protocol::host::plots::SettingsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.settings.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PlotsSnapshots {
    client: ClientTransport,
}
impl PlotsSnapshots {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, requests: Vec<PlotRequest>) -> Result<SnapshotsListResponse, tonic::Status> {
        self.list_with(requests, PlotsSnapshotsListOptions::default()).await
    }
    pub async fn list_with(
        &self, requests: Vec<PlotRequest>, options: PlotsSnapshotsListOptions,
    ) -> Result<SnapshotsListResponse, tonic::Status> {
        let value = SnapshotsListRequest {
            requests,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SnapshotsListRequest, SnapshotsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Snapshots/List"),
                request,
                SnapshotsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.snapshots.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, requests: Vec<PlotRequest>) -> Result<Watching<SnapshotsWatchResponse>, tonic::Status> {
        self.watch_with(requests, PlotsSnapshotsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, requests: Vec<PlotRequest>, options: PlotsSnapshotsWatchOptions,
    ) -> Result<Watching<SnapshotsWatchResponse>, tonic::Status> {
        let value = SnapshotsWatchRequest {
            requests,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<SnapshotsWatchRequest, SnapshotsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Snapshots/Watch"),
                request,
                SnapshotsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.snapshots.watch", error))?;
        let response = response.with_action("plots.snapshots.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Plugins {
    client: ClientTransport,
    pub secrets: PluginsSecrets,
    pub settings: PluginsSettings,
}
impl Plugins {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            secrets: PluginsSecrets::new(client.clone()),
            settings: PluginsSettings::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<PluginsGetResponse, tonic::Status> {
        self.get_with(name, PluginsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: PluginsGetOptions) -> Result<PluginsGetResponse, tonic::Status> {
        let value = PluginsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsGetRequest, PluginsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Get"),
                request,
                PluginsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self) -> Result<PluginsListResponse, tonic::Status> {
        self.list_with(PluginsListOptions::default()).await
    }
    pub async fn list_with(&self, options: PluginsListOptions) -> Result<PluginsListResponse, tonic::Status> {
        let value = PluginsListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsListRequest, PluginsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/List"),
                request,
                PluginsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<PluginsWatchResponse>, tonic::Status> {
        self.watch_with(PluginsWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: PluginsWatchOptions) -> Result<Watching<PluginsWatchResponse>, tonic::Status> {
        let value = PluginsWatchRequest { name: options.name, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<PluginsWatchRequest, PluginsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Watch"),
                request,
                PluginsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.watch", error))?;
        let response = response.with_action("plugins.watch");
        Ok(response)
    }
    pub async fn create(
        &self, name: impl Into<String>, placement: PluginPlacement, value: PluginValue,
    ) -> Result<PluginsCreateResponse, tonic::Status> {
        self.create_with(name, placement, value, PluginsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, name: impl Into<String>, placement: PluginPlacement, value: PluginValue, options: PluginsCreateOptions,
    ) -> Result<PluginsCreateResponse, tonic::Status> {
        let value = PluginsCreateRequest {
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            name: name.into(),
            placement: Some(placement),
            value: Some(value),
            origin: options.origin,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsCreateRequest, PluginsCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Create"),
                request,
                PluginsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: PluginValue, expected_revision: Vec<u8>,
    ) -> Result<PluginsSetResponse, tonic::Status> {
        self.set_with(name, value, expected_revision, PluginsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: PluginValue, expected_revision: Vec<u8>, options: PluginsSetOptions,
    ) -> Result<PluginsSetResponse, tonic::Status> {
        let value = PluginsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSetRequest, PluginsSetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Set"),
                request,
                PluginsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>, expected_revision: Vec<u8>) -> Result<Receipt, tonic::Status> {
        self.delete_with(name, expected_revision, PluginsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, expected_revision: Vec<u8>, options: PluginsDeleteOptions,
    ) -> Result<Receipt, tonic::Status> {
        let value = PluginsDeleteRequest { name: name.into(), expected_revision };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsDeleteRequest, PluginsDeleteResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Delete"),
                request,
                PluginsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| tonic::Status::data_loss("Read Plugins.Delete: Missing Payload"))
    }
    pub async fn restart(&self, name: impl Into<String>, expected_revision: Vec<u8>) -> Result<PluginEntry, tonic::Status> {
        self.restart_with(name, expected_revision, PluginsRestartOptions::default()).await
    }
    pub async fn restart_with(
        &self, name: impl Into<String>, expected_revision: Vec<u8>, options: PluginsRestartOptions,
    ) -> Result<PluginEntry, tonic::Status> {
        let value = PluginsRestartRequest { name: name.into(), expected_revision };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsRestartRequest, PluginsRestartResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Restart"),
                request,
                PluginsRestartResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.restart", error))?;
        let response = response.into_inner();
        response.entry.ok_or_else(|| tonic::Status::data_loss("Read Plugins.Restart: Missing Payload"))
    }
    pub async fn replace(
        &self, name: impl Into<String>, placement: PluginPlacement, expected_revision: Vec<u8>,
    ) -> Result<PluginsReplaceResponse, tonic::Status> {
        self.replace_with(name, placement, expected_revision, PluginsReplaceOptions::default()).await
    }
    pub async fn replace_with(
        &self, name: impl Into<String>, placement: PluginPlacement, expected_revision: Vec<u8>, options: PluginsReplaceOptions,
    ) -> Result<PluginsReplaceResponse, tonic::Status> {
        let value = PluginsReplaceRequest {
            name: name.into(),
            placement: Some(placement),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsReplaceRequest, PluginsReplaceResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Replace"),
                request,
                PluginsReplaceResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.replace", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PluginsSecrets {
    client: ClientTransport,
}
impl PluginsSecrets {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<dokime_protocol::host::plugins::SecretsListResponse, tonic::Status> {
        self.list_with(PluginsSecretsListOptions::default()).await
    }
    pub async fn list_with(
        &self, options: PluginsSecretsListOptions,
    ) -> Result<dokime_protocol::host::plugins::SecretsListResponse, tonic::Status> {
        let value = dokime_protocol::host::plugins::SecretsListRequest { name: options.name, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<dokime_protocol::host::plugins::SecretsListRequest, dokime_protocol::host::plugins::SecretsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plugins.Secrets/List"),
                request,
                dokime_protocol::host::plugins::SecretsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.secrets.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PluginsSettings {
    client: ClientTransport,
}
impl PluginsSettings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, name: impl Into<String>) -> Result<SettingsListResponse, tonic::Status> {
        self.list_with(name, PluginsSettingsListOptions::default()).await
    }
    pub async fn list_with(
        &self, name: impl Into<String>, options: PluginsSettingsListOptions,
    ) -> Result<SettingsListResponse, tonic::Status> {
        let value = SettingsListRequest { name: Some(name.into()), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SettingsListRequest, SettingsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.plugins.Settings/List"),
                request,
                SettingsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.settings.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: SettingValues, expected_revision: Vec<u8>,
    ) -> Result<harmos_protocol::host::plugins::SettingsSetResponse, tonic::Status> {
        self.set_with(name, value, expected_revision, PluginsSettingsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: SettingValues, expected_revision: Vec<u8>, options: PluginsSettingsSetOptions,
    ) -> Result<harmos_protocol::host::plugins::SettingsSetResponse, tonic::Status> {
        let value = harmos_protocol::host::plugins::SettingsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<harmos_protocol::host::plugins::SettingsSetRequest, harmos_protocol::host::plugins::SettingsSetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.plugins.Settings/Set"),
                request,
                harmos_protocol::host::plugins::SettingsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.settings.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PromptDefinitions {
    client: ClientTransport,
}
impl PromptDefinitions {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<PromptDefinitionsGetResponse, tonic::Status> {
        self.get_with(name, parent, schema_version, PromptDefinitionsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: PromptDefinitionsGetOptions,
    ) -> Result<PromptDefinitionsGetResponse, tonic::Status> {
        let value = PromptDefinitionsGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptDefinitionsGetRequest, PromptDefinitionsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.PromptDefinitions/Get"),
                request,
                PromptDefinitionsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompt_definitions.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<PromptDefinitionsListResponse, tonic::Status> {
        self.list_with(parent, PromptDefinitionsListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: PromptDefinitionsListOptions,
    ) -> Result<PromptDefinitionsListResponse, tonic::Status> {
        let value = PromptDefinitionsListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptDefinitionsListRequest, PromptDefinitionsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.PromptDefinitions/List"),
                request,
                PromptDefinitionsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompt_definitions.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Prompts {
    client: ClientTransport,
    pub answers: PromptsAnswers,
}
impl Prompts {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            answers: PromptsAnswers::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<PromptsGetResponse, tonic::Status> {
        self.get_with(name, PromptsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: PromptsGetOptions) -> Result<PromptsGetResponse, tonic::Status> {
        let value = PromptsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptsGetRequest, PromptsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Prompts/Get"),
                request,
                PromptsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>, pending_only: bool) -> Result<PromptsListResponse, tonic::Status> {
        self.list_with(parent, pending_only, PromptsListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, pending_only: bool, options: PromptsListOptions,
    ) -> Result<PromptsListResponse, tonic::Status> {
        let value = PromptsListRequest {
            parent: parent.into(),
            cycle_id: options.cycle_id,
            execution_id: options.execution_id,
            pending_only,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptsListRequest, PromptsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Prompts/List"),
                request,
                PromptsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, parent: impl Into<String>, pending_only: bool) -> Result<Watching<PromptsWatchResponse>, tonic::Status> {
        self.watch_with(parent, pending_only, PromptsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, pending_only: bool, options: PromptsWatchOptions,
    ) -> Result<Watching<PromptsWatchResponse>, tonic::Status> {
        let value = PromptsWatchRequest {
            parent: parent.into(),
            cycle_id: options.cycle_id,
            execution_id: options.execution_id,
            pending_only,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<PromptsWatchRequest, PromptsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Prompts/Watch"),
                request,
                PromptsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.watch", error))?;
        let response = response.with_action("prompts.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PromptsAnswers {
    client: ClientTransport,
}
impl PromptsAnswers {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(
        &self, parent: impl Into<String>, action: impl Into<String>, values: MapValue, accepted: bool,
    ) -> Result<AnswersCreateResponse, tonic::Status> {
        self.create_with(parent, action, values, accepted, PromptsAnswersCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, parent: impl Into<String>, action: impl Into<String>, values: MapValue, accepted: bool, options: PromptsAnswersCreateOptions,
    ) -> Result<AnswersCreateResponse, tonic::Status> {
        let value = AnswersCreateRequest {
            parent: parent.into(),
            action: action.into(),
            values: Some(values),
            accepted,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AnswersCreateRequest, AnswersCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.prompts.Answers/Create"),
                request,
                AnswersCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.answers.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct RecordSchemas {
    client: ClientTransport,
}
impl RecordSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<RecordSchemasGetResponse, tonic::Status> {
        self.get_with(name, parent, schema_version, RecordSchemasGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: RecordSchemasGetOptions,
    ) -> Result<RecordSchemasGetResponse, tonic::Status> {
        let value = RecordSchemasGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<RecordSchemasGetRequest, RecordSchemasGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.RecordSchemas/Get"),
                request,
                RecordSchemasGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("record_schemas.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<RecordSchemasListResponse, tonic::Status> {
        self.list_with(parent, RecordSchemasListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: RecordSchemasListOptions,
    ) -> Result<RecordSchemasListResponse, tonic::Status> {
        let value = RecordSchemasListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<RecordSchemasListRequest, RecordSchemasListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.RecordSchemas/List"),
                request,
                RecordSchemasListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("record_schemas.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Reports {
    client: ClientTransport,
}
impl Reports {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, subject: ReportSubject, name: impl Into<String>) -> Result<ReportsGetResponse, tonic::Status> {
        self.get_with(subject, name, ReportsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, subject: ReportSubject, name: impl Into<String>, options: ReportsGetOptions,
    ) -> Result<ReportsGetResponse, tonic::Status> {
        let value = ReportsGetRequest {
            subject: Some(subject),
            name: name.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ReportsGetRequest, ReportsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Reports/Get"),
                request,
                ReportsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("reports.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn create(&self, subject: ReportSubject, document_id: impl Into<String>) -> Result<ReportsCreateResponse, tonic::Status> {
        self.create_with(subject, document_id, ReportsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, subject: ReportSubject, document_id: impl Into<String>, options: ReportsCreateOptions,
    ) -> Result<ReportsCreateResponse, tonic::Status> {
        let value = ReportsCreateRequest {
            subject: Some(subject),
            document_id: document_id.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ReportsCreateRequest, ReportsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Reports/Create"),
                request,
                ReportsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("reports.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn publish(
        &self, name: impl Into<String>, subject: ReportSubject, expected_revision: Vec<u8>,
    ) -> Result<ReportsPublishResponse, tonic::Status> {
        self.publish_with(name, subject, expected_revision, ReportsPublishOptions::default()).await
    }
    pub async fn publish_with(
        &self, name: impl Into<String>, subject: ReportSubject, expected_revision: Vec<u8>, options: ReportsPublishOptions,
    ) -> Result<ReportsPublishResponse, tonic::Status> {
        let value = ReportsPublishRequest {
            name: name.into(),
            subject: Some(subject),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ReportsPublishRequest, ReportsPublishResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Reports/Publish"),
                request,
                ReportsPublishResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("reports.publish", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Schemas {
    client: ClientTransport,
}
impl Schemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<SchemaDescription>, tonic::Status> {
        self.list_with(SchemasListOptions::default()).await
    }
    pub async fn list_with(&self, options: SchemasListOptions) -> Result<Vec<SchemaDescription>, tonic::Status> {
        let value = SchemasListRequest { schema: options.schema };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SchemasListRequest, SchemasListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Schemas/List"),
                request,
                SchemasListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("schemas.list", error))?;
        let response = response.into_inner();
        Ok(response.schemas)
    }
}
#[derive(Clone)]
pub struct Secrets {
    client: ClientTransport,
}
impl Secrets {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<SecretsGetResponse, tonic::Status> {
        self.get_with(name, SecretsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: SecretsGetOptions) -> Result<SecretsGetResponse, tonic::Status> {
        let value = SecretsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsGetRequest, SecretsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Get"),
                request,
                SecretsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: SecretWritable, expected_revision: Vec<u8>,
    ) -> Result<SecretsSetResponse, tonic::Status> {
        self.set_with(name, value, expected_revision, SecretsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: SecretWritable, expected_revision: Vec<u8>, options: SecretsSetOptions,
    ) -> Result<SecretsSetResponse, tonic::Status> {
        let value = SecretsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsSetRequest, SecretsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Set"),
                request,
                SecretsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self) -> Result<dokime_protocol::host::SecretsListResponse, tonic::Status> {
        self.list_with(SecretsListOptions::default()).await
    }
    pub async fn list_with(&self, options: SecretsListOptions) -> Result<dokime_protocol::host::SecretsListResponse, tonic::Status> {
        let value = dokime_protocol::host::SecretsListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<dokime_protocol::host::SecretsListRequest, dokime_protocol::host::SecretsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/List"),
                request,
                dokime_protocol::host::SecretsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<SecretsWatchResponse>, tonic::Status> {
        self.watch_with(SecretsWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: SecretsWatchOptions) -> Result<Watching<SecretsWatchResponse>, tonic::Status> {
        let value = SecretsWatchRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<SecretsWatchRequest, SecretsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Watch"),
                request,
                SecretsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.watch", error))?;
        let response = response.with_action("secrets.watch");
        Ok(response)
    }
    pub async fn delete(&self, names: Vec<String>, expected_revision: Vec<u8>) -> Result<SecretsDeleteResponse, tonic::Status> {
        self.delete_with(names, expected_revision, SecretsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, names: Vec<String>, expected_revision: Vec<u8>, options: SecretsDeleteOptions,
    ) -> Result<SecretsDeleteResponse, tonic::Status> {
        let value = SecretsDeleteRequest {
            names,
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsDeleteRequest, SecretsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Delete"),
                request,
                SecretsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn export(&self, names: Vec<String>, generation: u64) -> Result<SecretsExportResponse, tonic::Status> {
        self.export_with(names, generation, SecretsExportOptions::default()).await
    }
    pub async fn export_with(
        &self, names: Vec<String>, generation: u64, options: SecretsExportOptions,
    ) -> Result<SecretsExportResponse, tonic::Status> {
        let value = SecretsExportRequest { names, generation, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsExportRequest, SecretsExportResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Export"),
                request,
                SecretsExportResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.export", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn import(
        &self, data: Vec<u8>, choices: Vec<SecretImportChoice>, generation: u64, validate_only: bool,
    ) -> Result<SecretsImportResponse, tonic::Status> {
        self.import_with(data, choices, generation, validate_only, SecretsImportOptions::default()).await
    }
    pub async fn import_with(
        &self, data: Vec<u8>, choices: Vec<SecretImportChoice>, generation: u64, validate_only: bool, options: SecretsImportOptions,
    ) -> Result<SecretsImportResponse, tonic::Status> {
        let value = SecretsImportRequest {
            data,
            choices,
            generation,
            validate_only,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsImportRequest, SecretsImportResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Import"),
                request,
                SecretsImportResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.import", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Settings {
    client: ClientTransport,
}
impl Settings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<dokime_protocol::host::SettingsGetResponse, tonic::Status> {
        self.get_with(SettingsGetOptions::default()).await
    }
    pub async fn get_with(&self, options: SettingsGetOptions) -> Result<dokime_protocol::host::SettingsGetResponse, tonic::Status> {
        let value = dokime_protocol::host::SettingsGetRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<dokime_protocol::host::SettingsGetRequest, dokime_protocol::host::SettingsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Settings/Get"),
                request,
                dokime_protocol::host::SettingsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("settings.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, value: ApplicationSettings, expected_revision: Vec<u8>,
    ) -> Result<dokime_protocol::host::SettingsSetResponse, tonic::Status> {
        self.set_with(value, expected_revision, SettingsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: ApplicationSettings, expected_revision: Vec<u8>, options: SettingsSetOptions,
    ) -> Result<dokime_protocol::host::SettingsSetResponse, tonic::Status> {
        let value = dokime_protocol::host::SettingsSetRequest {
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<dokime_protocol::host::SettingsSetRequest, dokime_protocol::host::SettingsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Settings/Set"),
                request,
                dokime_protocol::host::SettingsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("settings.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn schema(&self) -> Result<SettingsSchemaResponse, tonic::Status> {
        self.schema_with(SettingsSchemaOptions::default()).await
    }
    pub async fn schema_with(&self, options: SettingsSchemaOptions) -> Result<SettingsSchemaResponse, tonic::Status> {
        let value = SettingsSchemaRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SettingsSchemaRequest, SettingsSchemaResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Settings/Schema"),
                request,
                SettingsSchemaResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("settings.schema", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Status {
    client: ClientTransport,
    pub diagnostics: StatusDiagnostics,
    pub logs: StatusLogs,
    pub trace_events: StatusTraceEvents,
}
impl Status {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            diagnostics: StatusDiagnostics::new(client.clone()),
            logs: StatusLogs::new(client.clone()),
            trace_events: StatusTraceEvents::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self) -> Result<StatusGetResponse, tonic::Status> {
        self.get_with(StatusGetOptions::default()).await
    }
    pub async fn get_with(&self, options: StatusGetOptions) -> Result<StatusGetResponse, tonic::Status> {
        let value = StatusGetRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StatusGetRequest, StatusGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Status/Get"),
                request,
                StatusGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<StatusWatchResponse>, tonic::Status> {
        self.watch_with(StatusWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: StatusWatchOptions) -> Result<Watching<StatusWatchResponse>, tonic::Status> {
        let value = StatusWatchRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<StatusWatchRequest, StatusWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Status/Watch"),
                request,
                StatusWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.watch", error))?;
        let response = response.with_action("status.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StatusDiagnostics {
    client: ClientTransport,
}
impl StatusDiagnostics {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>, offset: u64, max_bytes: u64) -> Result<DiagnosticsGetResponse, tonic::Status> {
        self.get_with(name, offset, max_bytes, StatusDiagnosticsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, offset: u64, max_bytes: u64, options: StatusDiagnosticsGetOptions,
    ) -> Result<DiagnosticsGetResponse, tonic::Status> {
        let value = DiagnosticsGetRequest { name: name.into(), offset, max_bytes };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<DiagnosticsGetRequest, DiagnosticsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.Diagnostics/Get"),
                request,
                DiagnosticsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.diagnostics.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StatusLogs {
    client: ClientTransport,
}
impl StatusLogs {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<LogsListResponse, tonic::Status> {
        self.list_with(StatusLogsListOptions::default()).await
    }
    pub async fn list_with(&self, options: StatusLogsListOptions) -> Result<LogsListResponse, tonic::Status> {
        let value = LogsListRequest { at: options.at, page: options.page };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<LogsListRequest, LogsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.Logs/List"),
                request,
                LogsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.logs.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, max_bytes: u64, max_events: u32) -> Result<Watching<LogsWatchResponse>, tonic::Status> {
        self.watch_with(max_bytes, max_events, StatusLogsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, max_bytes: u64, max_events: u32, options: StatusLogsWatchOptions,
    ) -> Result<Watching<LogsWatchResponse>, tonic::Status> {
        let value = LogsWatchRequest { at: options.at, max_bytes, max_events };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<LogsWatchRequest, LogsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.Logs/Watch"),
                request,
                LogsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.logs.watch", error))?;
        let response = response.with_action("status.logs.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StatusTraceEvents {
    client: ClientTransport,
}
impl StatusTraceEvents {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<TraceEventsListResponse, tonic::Status> {
        self.list_with(StatusTraceEventsListOptions::default()).await
    }
    pub async fn list_with(&self, options: StatusTraceEventsListOptions) -> Result<TraceEventsListResponse, tonic::Status> {
        let value = TraceEventsListRequest { at: options.at, page: options.page };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<TraceEventsListRequest, TraceEventsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.TraceEvents/List"),
                request,
                TraceEventsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.trace_events.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, max_bytes: u64, max_events: u32) -> Result<Watching<TraceEventsWatchResponse>, tonic::Status> {
        self.watch_with(max_bytes, max_events, StatusTraceEventsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, max_bytes: u64, max_events: u32, options: StatusTraceEventsWatchOptions,
    ) -> Result<Watching<TraceEventsWatchResponse>, tonic::Status> {
        let value = TraceEventsWatchRequest { at: options.at, max_bytes, max_events };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<TraceEventsWatchRequest, TraceEventsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.TraceEvents/Watch"),
                request,
                TraceEventsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.trace_events.watch", error))?;
        let response = response.with_action("status.trace_events.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Storage {
    pub sources: StorageSources,
}
impl Storage {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            sources: StorageSources::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct StorageSources {
    client: ClientTransport,
}
impl StorageSources {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<SourcesGetResponse, tonic::Status> {
        self.get_with(StorageSourcesGetOptions::default()).await
    }
    pub async fn get_with(&self, options: StorageSourcesGetOptions) -> Result<SourcesGetResponse, tonic::Status> {
        let value = SourcesGetRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SourcesGetRequest, SourcesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.storage.Sources/Get"),
                request,
                SourcesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("storage.sources.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, value: dokime_protocol::host::storage::StorageSources, expected_revision: Vec<u8>,
    ) -> Result<SourcesSetResponse, tonic::Status> {
        self.set_with(value, expected_revision, StorageSourcesSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: dokime_protocol::host::storage::StorageSources, expected_revision: Vec<u8>, options: StorageSourcesSetOptions,
    ) -> Result<SourcesSetResponse, tonic::Status> {
        let value = SourcesSetRequest {
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SourcesSetRequest, SourcesSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.storage.Sources/Set"),
                request,
                SourcesSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("storage.sources.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StreamSchemas {
    client: ClientTransport,
}
impl StreamSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<StreamSchemasGetResponse, tonic::Status> {
        self.get_with(name, parent, schema_version, StreamSchemasGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: StreamSchemasGetOptions,
    ) -> Result<StreamSchemasGetResponse, tonic::Status> {
        let value = StreamSchemasGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StreamSchemasGetRequest, StreamSchemasGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.StreamSchemas/Get"),
                request,
                StreamSchemasGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("stream_schemas.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<StreamSchemasListResponse, tonic::Status> {
        self.list_with(parent, StreamSchemasListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: StreamSchemasListOptions,
    ) -> Result<StreamSchemasListResponse, tonic::Status> {
        let value = StreamSchemasListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StreamSchemasListRequest, StreamSchemasListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.StreamSchemas/List"),
                request,
                StreamSchemasListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("stream_schemas.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Streams {
    client: ClientTransport,
}
impl Streams {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<StreamIdentity>, tonic::Status> {
        self.list_with(StreamsListOptions::default()).await
    }
    pub async fn list_with(&self, options: StreamsListOptions) -> Result<Vec<StreamIdentity>, tonic::Status> {
        let value = StreamsListRequest { schema: options.schema };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StreamsListRequest, StreamsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Streams/List"),
                request,
                StreamsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("streams.list", error))?;
        let response = response.into_inner();
        Ok(response.streams)
    }
}
#[derive(Clone)]
pub struct Suites {
    client: ClientTransport,
}
impl Suites {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<SuitesGetResponse, tonic::Status> {
        self.get_with(name, parent, schema_version, SuitesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: SuitesGetOptions,
    ) -> Result<SuitesGetResponse, tonic::Status> {
        let value = SuitesGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SuitesGetRequest, SuitesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Suites/Get"),
                request,
                SuitesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("suites.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<SuitesListResponse, tonic::Status> {
        self.list_with(parent, SuitesListOptions::default()).await
    }
    pub async fn list_with(&self, parent: impl Into<String>, options: SuitesListOptions) -> Result<SuitesListResponse, tonic::Status> {
        let value = SuitesListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SuitesListRequest, SuitesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Suites/List"),
                request,
                SuitesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("suites.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Time {
    client: ClientTransport,
}
impl Time {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<Timestamp, tonic::Status> {
        self.get_with(TimeGetOptions::default()).await
    }
    pub async fn get_with(&self, options: TimeGetOptions) -> Result<Timestamp, tonic::Status> {
        let value = TimeGetRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<TimeGetRequest, TimeGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Time/Get"),
                request,
                TimeGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("time.get", error))?;
        let response = response.into_inner();
        response.now.ok_or_else(|| tonic::Status::data_loss("Read Time.Get: Missing Payload"))
    }
    pub async fn sleep(&self, duration: Duration) -> Result<TimeSleepResponse, tonic::Status> {
        self.sleep_with(duration, TimeSleepOptions::default()).await
    }
    pub async fn sleep_with(&self, duration: Duration, options: TimeSleepOptions) -> Result<TimeSleepResponse, tonic::Status> {
        let value = TimeSleepRequest { duration: Some(duration) };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<TimeSleepRequest, TimeSleepResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Time/Sleep"),
                request,
                TimeSleepResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("time.sleep", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
