// @generated from the protocol descriptor table; do not edit.
use harmos::client::{ClientTransport, Watching};
use prost_reflect::ReflectMessage;
#[derive(Clone)]
pub struct Root {
    pub access: Access,
    pub aggregates: Aggregates,
    pub backend: Backend,
    pub cases: Cases,
    pub catalog: Catalog,
    pub catalog_entries: CatalogEntries,
    pub content: Content,
    pub cycles: Cycles,
    pub documents: Documents,
    pub evidence: Evidence,
    pub executions: Executions,
    pub invocations: Invocations,
    pub media: Media,
    pub plots: Plots,
    pub plugins: Plugins,
    pub prompts: Prompts,
    pub reports: Reports,
    pub secrets: Secrets,
    pub settings: Settings,
    pub status: Status,
    pub storage: Storage,
    pub time: Time,
}
impl Root {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            access: Access::new(client.clone()),
            aggregates: Aggregates::new(client.clone()),
            backend: Backend::new(client.clone()),
            cases: Cases::new(client.clone()),
            catalog: Catalog::new(client.clone()),
            catalog_entries: CatalogEntries::new(client.clone()),
            content: Content::new(client.clone()),
            cycles: Cycles::new(client.clone()),
            documents: Documents::new(client.clone()),
            evidence: Evidence::new(client.clone()),
            executions: Executions::new(client.clone()),
            invocations: Invocations::new(client.clone()),
            media: Media::new(client.clone()),
            plots: Plots::new(client.clone()),
            plugins: Plugins::new(client.clone()),
            prompts: Prompts::new(client.clone()),
            reports: Reports::new(client.clone()),
            secrets: Secrets::new(client.clone()),
            settings: Settings::new(client.clone()),
            status: Status::new(client.clone()),
            storage: Storage::new(client.clone()),
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
        Self {
            client,
        }
    }
    pub async fn create(&self, request: tonic::Request<harmos_protocol::v1::access::CertificatesCreateRequest>) -> Result<tonic::Response<harmos_protocol::v1::access::CertificatesCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.access.Certificates/Create"), request, harmos_protocol::v1::access::CertificatesCreateResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct AccessListener {
    client: ClientTransport,
}
impl AccessListener {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<harmos_protocol::v1::access::ListenerGetRequest>) -> Result<tonic::Response<harmos_protocol::v1::access::ListenerGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.access.Listener/Get"), request, harmos_protocol::v1::access::ListenerGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<harmos_protocol::v1::access::ListenerSetRequest>) -> Result<tonic::Response<harmos_protocol::v1::access::ListenerSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.access.Listener/Set"), request, harmos_protocol::v1::access::ListenerSetResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct AccessPairings {
    client: ClientTransport,
}
impl AccessPairings {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn create(&self, request: tonic::Request<harmos_protocol::v1::access::PairingsCreateRequest>) -> Result<tonic::Response<harmos_protocol::v1::access::PairingsCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.access.Pairings/Create"), request, harmos_protocol::v1::access::PairingsCreateResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<harmos_protocol::v1::access::PairingsListRequest>) -> Result<tonic::Response<harmos_protocol::v1::access::PairingsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.access.Pairings/List"), request, harmos_protocol::v1::access::PairingsListResponse::default().descriptor()).await
    }
    pub async fn delete(&self, request: tonic::Request<harmos_protocol::v1::access::PairingsDeleteRequest>) -> Result<tonic::Response<harmos_protocol::v1::access::PairingsDeleteResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.access.Pairings/Delete"), request, harmos_protocol::v1::access::PairingsDeleteResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Aggregates {
    client: ClientTransport,
}
impl Aggregates {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::AggregatesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::AggregatesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Aggregates/Get"), request, dokime_protocol::v1::AggregatesGetResponse::default().descriptor()).await
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
    pub async fn appearance(&self, request: tonic::Request<dokime_protocol::v1::BackendAppearanceRequest>) -> Result<tonic::Response<dokime_protocol::v1::BackendAppearanceResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Backend/Appearance"), request, dokime_protocol::v1::BackendAppearanceResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct BackendOverview {
    client: ClientTransport,
}
impl BackendOverview {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::backend::OverviewGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::backend::OverviewGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.backend.Overview/Get"), request, dokime_protocol::v1::backend::OverviewGetResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::backend::OverviewWatchRequest>) -> Result<Watching<dokime_protocol::v1::backend::OverviewWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.backend.Overview/Watch"), request, dokime_protocol::v1::backend::OverviewWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Cases {
    client: ClientTransport,
}
impl Cases {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::CasesCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::CasesCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cases/Create"), request, dokime_protocol::v1::CasesCreateResponse::default().descriptor()).await
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::CasesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::CasesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cases/Get"), request, dokime_protocol::v1::CasesGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::CasesListRequest>) -> Result<tonic::Response<dokime_protocol::v1::CasesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cases/List"), request, dokime_protocol::v1::CasesListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::CasesWatchRequest>) -> Result<Watching<dokime_protocol::v1::CasesWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cases/Watch"), request, dokime_protocol::v1::CasesWatchResponse::default().descriptor()).await
    }
    pub async fn delete(&self, request: tonic::Request<dokime_protocol::v1::CasesDeleteRequest>) -> Result<tonic::Response<dokime_protocol::v1::CasesDeleteResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cases/Delete"), request, dokime_protocol::v1::CasesDeleteResponse::default().descriptor()).await
    }
    pub async fn parameters(&self, request: tonic::Request<dokime_protocol::v1::CasesParametersRequest>) -> Result<tonic::Response<dokime_protocol::v1::CasesParametersResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cases/Parameters"), request, dokime_protocol::v1::CasesParametersResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Catalog {
    client: ClientTransport,
    pub prompt_definitions: CatalogPromptDefinitions,
    pub record_schemas: CatalogRecordSchemas,
    pub stream_schemas: CatalogStreamSchemas,
    pub suites: CatalogSuites,
}
impl Catalog {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            prompt_definitions: CatalogPromptDefinitions::new(client.clone()),
            record_schemas: CatalogRecordSchemas::new(client.clone()),
            stream_schemas: CatalogStreamSchemas::new(client.clone()),
            suites: CatalogSuites::new(client.clone()),
            client,
        }
    }
    pub async fn analyses(&self, request: tonic::Request<dokime_protocol::v1::CatalogAnalysesRequest>) -> Result<tonic::Response<dokime_protocol::v1::CatalogAnalysesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Catalog/Analyses"), request, dokime_protocol::v1::CatalogAnalysesResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct CatalogPromptDefinitions {
    client: ClientTransport,
}
impl CatalogPromptDefinitions {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::catalog::PromptDefinitionsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::PromptDefinitionsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.PromptDefinitions/Get"), request, dokime_protocol::v1::catalog::PromptDefinitionsGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::catalog::PromptDefinitionsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::PromptDefinitionsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.PromptDefinitions/List"), request, dokime_protocol::v1::catalog::PromptDefinitionsListResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct CatalogRecordSchemas {
    client: ClientTransport,
}
impl CatalogRecordSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::catalog::RecordSchemasGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::RecordSchemasGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.RecordSchemas/Get"), request, dokime_protocol::v1::catalog::RecordSchemasGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::catalog::RecordSchemasListRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::RecordSchemasListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.RecordSchemas/List"), request, dokime_protocol::v1::catalog::RecordSchemasListResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct CatalogStreamSchemas {
    client: ClientTransport,
}
impl CatalogStreamSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::catalog::StreamSchemasGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::StreamSchemasGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.StreamSchemas/Get"), request, dokime_protocol::v1::catalog::StreamSchemasGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::catalog::StreamSchemasListRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::StreamSchemasListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.StreamSchemas/List"), request, dokime_protocol::v1::catalog::StreamSchemasListResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct CatalogSuites {
    client: ClientTransport,
}
impl CatalogSuites {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::catalog::SuitesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::SuitesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.Suites/Get"), request, dokime_protocol::v1::catalog::SuitesGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::catalog::SuitesListRequest>) -> Result<tonic::Response<dokime_protocol::v1::catalog::SuitesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.catalog.Suites/List"), request, dokime_protocol::v1::catalog::SuitesListResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct CatalogEntries {
    client: ClientTransport,
}
impl CatalogEntries {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<harmos_protocol::v1::CatalogEntriesListRequest>) -> Result<tonic::Response<harmos_protocol::v1::CatalogEntriesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.CatalogEntries/List"), request, harmos_protocol::v1::CatalogEntriesListResponse::default().descriptor()).await
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
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::content::ArtifactsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::content::ArtifactsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.Artifacts/Get"), request, dokime_protocol::v1::content::ArtifactsGetResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::content::ArtifactsCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::content::ArtifactsCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.Artifacts/Create"), request, dokime_protocol::v1::content::ArtifactsCreateResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::content::ArtifactsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::content::ArtifactsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.Artifacts/List"), request, dokime_protocol::v1::content::ArtifactsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::content::ArtifactsWatchRequest>) -> Result<Watching<dokime_protocol::v1::content::ArtifactsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.Artifacts/Watch"), request, dokime_protocol::v1::content::ArtifactsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct ContentExecutionFiles {
    client: ClientTransport,
}
impl ContentExecutionFiles {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::content::ExecutionFilesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::content::ExecutionFilesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.ExecutionFiles/Get"), request, dokime_protocol::v1::content::ExecutionFilesGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::content::ExecutionFilesListRequest>) -> Result<tonic::Response<dokime_protocol::v1::content::ExecutionFilesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.ExecutionFiles/List"), request, dokime_protocol::v1::content::ExecutionFilesListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::content::ExecutionFilesWatchRequest>) -> Result<Watching<dokime_protocol::v1::content::ExecutionFilesWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.content.ExecutionFiles/Watch"), request, dokime_protocol::v1::content::ExecutionFilesWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Cycles {
    client: ClientTransport,
}
impl Cycles {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::CyclesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::CyclesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cycles/Get"), request, dokime_protocol::v1::CyclesGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::CyclesSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::CyclesSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cycles/Set"), request, dokime_protocol::v1::CyclesSetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::CyclesListRequest>) -> Result<tonic::Response<dokime_protocol::v1::CyclesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cycles/List"), request, dokime_protocol::v1::CyclesListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::CyclesWatchRequest>) -> Result<Watching<dokime_protocol::v1::CyclesWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cycles/Watch"), request, dokime_protocol::v1::CyclesWatchResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::CyclesCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::CyclesCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cycles/Create"), request, dokime_protocol::v1::CyclesCreateResponse::default().descriptor()).await
    }
    pub async fn seal(&self, request: tonic::Request<dokime_protocol::v1::CyclesSealRequest>) -> Result<tonic::Response<dokime_protocol::v1::CyclesSealResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Cycles/Seal"), request, dokime_protocol::v1::CyclesSealResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Documents {
    client: ClientTransport,
}
impl Documents {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::DocumentsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::DocumentsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Documents/Get"), request, dokime_protocol::v1::DocumentsGetResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::DocumentsWatchRequest>) -> Result<Watching<dokime_protocol::v1::DocumentsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Documents/Watch"), request, dokime_protocol::v1::DocumentsWatchResponse::default().descriptor()).await
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
    pub async fn statistics(&self, request: tonic::Request<dokime_protocol::v1::EvidenceStatisticsRequest>) -> Result<tonic::Response<dokime_protocol::v1::EvidenceStatisticsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Evidence/Statistics"), request, dokime_protocol::v1::EvidenceStatisticsResponse::default().descriptor()).await
    }
    pub async fn archive(&self, request: tonic::Request<dokime_protocol::v1::EvidenceArchiveRequest>) -> Result<tonic::Response<dokime_protocol::v1::EvidenceArchiveResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Evidence/Archive"), request, dokime_protocol::v1::EvidenceArchiveResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct EvidenceHealthEvents {
    client: ClientTransport,
}
impl EvidenceHealthEvents {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::evidence::HealthEventsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::HealthEventsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.HealthEvents/List"), request, dokime_protocol::v1::evidence::HealthEventsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::evidence::HealthEventsWatchRequest>) -> Result<Watching<dokime_protocol::v1::evidence::HealthEventsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.HealthEvents/Watch"), request, dokime_protocol::v1::evidence::HealthEventsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct EvidenceNotes {
    client: ClientTransport,
}
impl EvidenceNotes {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::evidence::NotesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::NotesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Notes/Get"), request, dokime_protocol::v1::evidence::NotesGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::evidence::NotesSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::NotesSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Notes/Set"), request, dokime_protocol::v1::evidence::NotesSetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::evidence::NotesListRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::NotesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Notes/List"), request, dokime_protocol::v1::evidence::NotesListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::evidence::NotesWatchRequest>) -> Result<Watching<dokime_protocol::v1::evidence::NotesWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Notes/Watch"), request, dokime_protocol::v1::evidence::NotesWatchResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::evidence::NotesCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::NotesCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Notes/Create"), request, dokime_protocol::v1::evidence::NotesCreateResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct EvidenceRecords {
    client: ClientTransport,
}
impl EvidenceRecords {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::evidence::RecordsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::RecordsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Records/List"), request, dokime_protocol::v1::evidence::RecordsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::evidence::RecordsWatchRequest>) -> Result<Watching<dokime_protocol::v1::evidence::RecordsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Records/Watch"), request, dokime_protocol::v1::evidence::RecordsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct EvidenceSamples {
    client: ClientTransport,
}
impl EvidenceSamples {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::evidence::SamplesListRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::SamplesListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Samples/List"), request, dokime_protocol::v1::evidence::SamplesListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::evidence::SamplesWatchRequest>) -> Result<Watching<dokime_protocol::v1::evidence::SamplesWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Samples/Watch"), request, dokime_protocol::v1::evidence::SamplesWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct EvidenceTraceability {
    client: ClientTransport,
}
impl EvidenceTraceability {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::evidence::TraceabilityGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::evidence::TraceabilityGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Traceability/Get"), request, dokime_protocol::v1::evidence::TraceabilityGetResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::evidence::TraceabilityWatchRequest>) -> Result<Watching<dokime_protocol::v1::evidence::TraceabilityWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.evidence.Traceability/Watch"), request, dokime_protocol::v1::evidence::TraceabilityWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Executions {
    client: ClientTransport,
    pub commands: ExecutionsCommands,
    pub progress: ExecutionsProgress,
}
impl Executions {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            commands: ExecutionsCommands::new(client.clone()),
            progress: ExecutionsProgress::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Get"), request, dokime_protocol::v1::ExecutionsGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Set"), request, dokime_protocol::v1::ExecutionsSetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/List"), request, dokime_protocol::v1::ExecutionsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsWatchRequest>) -> Result<Watching<dokime_protocol::v1::ExecutionsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Watch"), request, dokime_protocol::v1::ExecutionsWatchResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Create"), request, dokime_protocol::v1::ExecutionsCreateResponse::default().descriptor()).await
    }
    pub async fn start(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsStartRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsStartResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Start"), request, dokime_protocol::v1::ExecutionsStartResponse::default().descriptor()).await
    }
    pub async fn stop(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsStopRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsStopResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Stop"), request, dokime_protocol::v1::ExecutionsStopResponse::default().descriptor()).await
    }
    pub async fn cancel(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsCancelRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsCancelResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Cancel"), request, dokime_protocol::v1::ExecutionsCancelResponse::default().descriptor()).await
    }
    pub async fn delete(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsDeleteRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsDeleteResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Delete"), request, dokime_protocol::v1::ExecutionsDeleteResponse::default().descriptor()).await
    }
    pub async fn parameters(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsParametersRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsParametersResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Parameters"), request, dokime_protocol::v1::ExecutionsParametersResponse::default().descriptor()).await
    }
    pub async fn steps(&self, request: tonic::Request<dokime_protocol::v1::ExecutionsStepsRequest>) -> Result<tonic::Response<dokime_protocol::v1::ExecutionsStepsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Executions/Steps"), request, dokime_protocol::v1::ExecutionsStepsResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct ExecutionsCommands {
    client: ClientTransport,
}
impl ExecutionsCommands {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::executions::CommandsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::executions::CommandsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.executions.Commands/Get"), request, dokime_protocol::v1::executions::CommandsGetResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::executions::CommandsCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::executions::CommandsCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.executions.Commands/Create"), request, dokime_protocol::v1::executions::CommandsCreateResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct ExecutionsProgress {
    client: ClientTransport,
}
impl ExecutionsProgress {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::executions::ProgressGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::executions::ProgressGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.executions.Progress/Get"), request, dokime_protocol::v1::executions::ProgressGetResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::executions::ProgressWatchRequest>) -> Result<Watching<dokime_protocol::v1::executions::ProgressWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.executions.Progress/Watch"), request, dokime_protocol::v1::executions::ProgressWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Invocations {
    client: ClientTransport,
}
impl Invocations {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn invoke(&self, request: tonic::Request<harmos_protocol::v1::InvocationsInvokeRequest>) -> Result<tonic::Response<harmos_protocol::v1::InvocationsInvokeResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Invocations/Invoke"), request, harmos_protocol::v1::InvocationsInvokeResponse::default().descriptor()).await
    }
    pub async fn get(&self, request: tonic::Request<harmos_protocol::v1::InvocationsGetRequest>) -> Result<tonic::Response<harmos_protocol::v1::InvocationsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Invocations/Get"), request, harmos_protocol::v1::InvocationsGetResponse::default().descriptor()).await
    }
    pub async fn cancel(&self, request: tonic::Request<harmos_protocol::v1::InvocationsCancelRequest>) -> Result<tonic::Response<harmos_protocol::v1::InvocationsCancelResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Invocations/Cancel"), request, harmos_protocol::v1::InvocationsCancelResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<harmos_protocol::v1::InvocationsWatchRequest>) -> Result<Watching<harmos_protocol::v1::InvocationsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Invocations/Watch"), request, harmos_protocol::v1::InvocationsWatchResponse::default().descriptor()).await
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
        Self {
            client,
        }
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::media::CapturesCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::media::CapturesCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.media.Captures/Create"), request, dokime_protocol::v1::media::CapturesCreateResponse::default().descriptor()).await
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::media::CapturesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::media::CapturesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.media.Captures/Get"), request, dokime_protocol::v1::media::CapturesGetResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::media::CapturesWatchRequest>) -> Result<Watching<dokime_protocol::v1::media::CapturesWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.media.Captures/Watch"), request, dokime_protocol::v1::media::CapturesWatchResponse::default().descriptor()).await
    }
    pub async fn stop(&self, request: tonic::Request<dokime_protocol::v1::media::CapturesStopRequest>) -> Result<tonic::Response<dokime_protocol::v1::media::CapturesStopResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.media.Captures/Stop"), request, dokime_protocol::v1::media::CapturesStopResponse::default().descriptor()).await
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
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::plots::SettingsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::plots::SettingsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.plots.Settings/Get"), request, dokime_protocol::v1::plots::SettingsGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::plots::SettingsSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::plots::SettingsSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.plots.Settings/Set"), request, dokime_protocol::v1::plots::SettingsSetResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct PlotsSnapshots {
    client: ClientTransport,
}
impl PlotsSnapshots {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::plots::SnapshotsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::plots::SnapshotsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.plots.Snapshots/List"), request, dokime_protocol::v1::plots::SnapshotsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::plots::SnapshotsWatchRequest>) -> Result<Watching<dokime_protocol::v1::plots::SnapshotsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.plots.Snapshots/Watch"), request, dokime_protocol::v1::plots::SnapshotsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Plugins {
    client: ClientTransport,
    pub settings: PluginsSettings,
}
impl Plugins {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            settings: PluginsSettings::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<harmos_protocol::v1::PluginsGetRequest>) -> Result<tonic::Response<harmos_protocol::v1::PluginsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/Get"), request, harmos_protocol::v1::PluginsGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<harmos_protocol::v1::PluginsListRequest>) -> Result<tonic::Response<harmos_protocol::v1::PluginsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/List"), request, harmos_protocol::v1::PluginsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<harmos_protocol::v1::PluginsWatchRequest>) -> Result<Watching<harmos_protocol::v1::PluginsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/Watch"), request, harmos_protocol::v1::PluginsWatchResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<harmos_protocol::v1::PluginsCreateRequest>) -> Result<tonic::Response<harmos_protocol::v1::PluginsCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/Create"), request, harmos_protocol::v1::PluginsCreateResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<harmos_protocol::v1::PluginsSetRequest>) -> Result<tonic::Response<harmos_protocol::v1::PluginsSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/Set"), request, harmos_protocol::v1::PluginsSetResponse::default().descriptor()).await
    }
    pub async fn delete(&self, request: tonic::Request<harmos_protocol::v1::PluginsDeleteRequest>) -> Result<tonic::Response<harmos_protocol::v1::PluginsDeleteResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/Delete"), request, harmos_protocol::v1::PluginsDeleteResponse::default().descriptor()).await
    }
    pub async fn restart(&self, request: tonic::Request<harmos_protocol::v1::PluginsRestartRequest>) -> Result<tonic::Response<harmos_protocol::v1::PluginsRestartResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Plugins/Restart"), request, harmos_protocol::v1::PluginsRestartResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct PluginsSettings {
    client: ClientTransport,
}
impl PluginsSettings {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::plugins::SettingsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::plugins::SettingsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.plugins.Settings/Get"), request, dokime_protocol::v1::plugins::SettingsGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::plugins::SettingsSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::plugins::SettingsSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.plugins.Settings/Set"), request, dokime_protocol::v1::plugins::SettingsSetResponse::default().descriptor()).await
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
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::PromptsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::PromptsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Prompts/Get"), request, dokime_protocol::v1::PromptsGetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::PromptsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::PromptsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Prompts/List"), request, dokime_protocol::v1::PromptsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::PromptsWatchRequest>) -> Result<Watching<dokime_protocol::v1::PromptsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Prompts/Watch"), request, dokime_protocol::v1::PromptsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct PromptsAnswers {
    client: ClientTransport,
}
impl PromptsAnswers {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::prompts::AnswersCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::prompts::AnswersCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.prompts.Answers/Create"), request, dokime_protocol::v1::prompts::AnswersCreateResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Reports {
    client: ClientTransport,
}
impl Reports {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::ReportsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::ReportsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Reports/Get"), request, dokime_protocol::v1::ReportsGetResponse::default().descriptor()).await
    }
    pub async fn create(&self, request: tonic::Request<dokime_protocol::v1::ReportsCreateRequest>) -> Result<tonic::Response<dokime_protocol::v1::ReportsCreateResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Reports/Create"), request, dokime_protocol::v1::ReportsCreateResponse::default().descriptor()).await
    }
    pub async fn publish(&self, request: tonic::Request<dokime_protocol::v1::ReportsPublishRequest>) -> Result<tonic::Response<dokime_protocol::v1::ReportsPublishResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Reports/Publish"), request, dokime_protocol::v1::ReportsPublishResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Secrets {
    client: ClientTransport,
}
impl Secrets {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::SecretsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::SecretsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/Get"), request, dokime_protocol::v1::SecretsGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::SecretsSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::SecretsSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/Set"), request, dokime_protocol::v1::SecretsSetResponse::default().descriptor()).await
    }
    pub async fn list(&self, request: tonic::Request<dokime_protocol::v1::SecretsListRequest>) -> Result<tonic::Response<dokime_protocol::v1::SecretsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/List"), request, dokime_protocol::v1::SecretsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<dokime_protocol::v1::SecretsWatchRequest>) -> Result<Watching<dokime_protocol::v1::SecretsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/Watch"), request, dokime_protocol::v1::SecretsWatchResponse::default().descriptor()).await
    }
    pub async fn delete(&self, request: tonic::Request<dokime_protocol::v1::SecretsDeleteRequest>) -> Result<tonic::Response<dokime_protocol::v1::SecretsDeleteResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/Delete"), request, dokime_protocol::v1::SecretsDeleteResponse::default().descriptor()).await
    }
    pub async fn export(&self, request: tonic::Request<dokime_protocol::v1::SecretsExportRequest>) -> Result<tonic::Response<dokime_protocol::v1::SecretsExportResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/Export"), request, dokime_protocol::v1::SecretsExportResponse::default().descriptor()).await
    }
    pub async fn import(&self, request: tonic::Request<dokime_protocol::v1::SecretsImportRequest>) -> Result<tonic::Response<dokime_protocol::v1::SecretsImportResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Secrets/Import"), request, dokime_protocol::v1::SecretsImportResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Settings {
    client: ClientTransport,
}
impl Settings {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::SettingsGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::SettingsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Settings/Get"), request, dokime_protocol::v1::SettingsGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::SettingsSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::SettingsSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Settings/Set"), request, dokime_protocol::v1::SettingsSetResponse::default().descriptor()).await
    }
    pub async fn schema(&self, request: tonic::Request<dokime_protocol::v1::SettingsSchemaRequest>) -> Result<tonic::Response<dokime_protocol::v1::SettingsSchemaResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.Settings/Schema"), request, dokime_protocol::v1::SettingsSchemaResponse::default().descriptor()).await
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
    pub async fn get(&self, request: tonic::Request<harmos_protocol::v1::StatusGetRequest>) -> Result<tonic::Response<harmos_protocol::v1::StatusGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Status/Get"), request, harmos_protocol::v1::StatusGetResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<harmos_protocol::v1::StatusWatchRequest>) -> Result<Watching<harmos_protocol::v1::StatusWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Status/Watch"), request, harmos_protocol::v1::StatusWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct StatusDiagnostics {
    client: ClientTransport,
}
impl StatusDiagnostics {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<harmos_protocol::v1::status::DiagnosticsGetRequest>) -> Result<tonic::Response<harmos_protocol::v1::status::DiagnosticsGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.status.Diagnostics/Get"), request, harmos_protocol::v1::status::DiagnosticsGetResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct StatusLogs {
    client: ClientTransport,
}
impl StatusLogs {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<harmos_protocol::v1::status::LogsListRequest>) -> Result<tonic::Response<harmos_protocol::v1::status::LogsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.status.Logs/List"), request, harmos_protocol::v1::status::LogsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<harmos_protocol::v1::status::LogsWatchRequest>) -> Result<Watching<harmos_protocol::v1::status::LogsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.status.Logs/Watch"), request, harmos_protocol::v1::status::LogsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct StatusTraceEvents {
    client: ClientTransport,
}
impl StatusTraceEvents {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn list(&self, request: tonic::Request<harmos_protocol::v1::status::TraceEventsListRequest>) -> Result<tonic::Response<harmos_protocol::v1::status::TraceEventsListResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.status.TraceEvents/List"), request, harmos_protocol::v1::status::TraceEventsListResponse::default().descriptor()).await
    }
    pub async fn watch(&self, request: tonic::Request<harmos_protocol::v1::status::TraceEventsWatchRequest>) -> Result<Watching<harmos_protocol::v1::status::TraceEventsWatchResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.status.TraceEvents/Watch"), request, harmos_protocol::v1::status::TraceEventsWatchResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Storage {
    client: ClientTransport,
    pub sources: StorageSources,
}
impl Storage {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            sources: StorageSources::new(client.clone()),
            client,
        }
    }
    pub async fn create_file_production(&self, request: tonic::Request<harmos_protocol::v1::StorageCreateFileProductionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageCreateFileProductionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/CreateFileProduction"), request, harmos_protocol::v1::StorageCreateFileProductionResponse::default().descriptor()).await
    }
    pub async fn get_file_production(&self, request: tonic::Request<harmos_protocol::v1::StorageGetFileProductionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetFileProductionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetFileProduction"), request, harmos_protocol::v1::StorageGetFileProductionResponse::default().descriptor()).await
    }
    pub async fn append_file_bytes(&self, request: tonic::Request<harmos_protocol::v1::StorageAppendFileBytesRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageAppendFileBytesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/AppendFileBytes"), request, harmos_protocol::v1::StorageAppendFileBytesResponse::default().descriptor()).await
    }
    pub async fn create_file_writer(&self, request: tonic::Request<harmos_protocol::v1::StorageCreateFileWriterRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageCreateFileWriterResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/CreateFileWriter"), request, harmos_protocol::v1::StorageCreateFileWriterResponse::default().descriptor()).await
    }
    pub async fn publish_file_production(&self, request: tonic::Request<harmos_protocol::v1::StoragePublishFileProductionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StoragePublishFileProductionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/PublishFileProduction"), request, harmos_protocol::v1::StoragePublishFileProductionResponse::default().descriptor()).await
    }
    pub async fn delete_file_production(&self, request: tonic::Request<harmos_protocol::v1::StorageDeleteFileProductionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageDeleteFileProductionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/DeleteFileProduction"), request, harmos_protocol::v1::StorageDeleteFileProductionResponse::default().descriptor()).await
    }
    pub async fn get_file_bytes(&self, request: tonic::Request<harmos_protocol::v1::StorageGetFileBytesRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetFileBytesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetFileBytes"), request, harmos_protocol::v1::StorageGetFileBytesResponse::default().descriptor()).await
    }
    pub async fn capabilities(&self, request: tonic::Request<harmos_protocol::v1::StorageCapabilitiesRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageCapabilitiesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/Capabilities"), request, harmos_protocol::v1::StorageCapabilitiesResponse::default().descriptor()).await
    }
    pub async fn create_snapshot(&self, request: tonic::Request<harmos_protocol::v1::StorageCreateSnapshotRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageCreateSnapshotResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/CreateSnapshot"), request, harmos_protocol::v1::StorageCreateSnapshotResponse::default().descriptor()).await
    }
    pub async fn delete_snapshot(&self, request: tonic::Request<harmos_protocol::v1::StorageDeleteSnapshotRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageDeleteSnapshotResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/DeleteSnapshot"), request, harmos_protocol::v1::StorageDeleteSnapshotResponse::default().descriptor()).await
    }
    pub async fn list_column_rows(&self, request: tonic::Request<harmos_protocol::v1::StorageListColumnRowsRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageListColumnRowsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/ListColumnRows"), request, harmos_protocol::v1::StorageListColumnRowsResponse::default().descriptor()).await
    }
    pub async fn list_index_entries(&self, request: tonic::Request<harmos_protocol::v1::StorageListIndexEntriesRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageListIndexEntriesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/ListIndexEntries"), request, harmos_protocol::v1::StorageListIndexEntriesResponse::default().descriptor()).await
    }
    pub async fn set_index_definition(&self, request: tonic::Request<harmos_protocol::v1::StorageSetIndexDefinitionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageSetIndexDefinitionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/SetIndexDefinition"), request, harmos_protocol::v1::StorageSetIndexDefinitionResponse::default().descriptor()).await
    }
    pub async fn set_projection_definition(&self, request: tonic::Request<harmos_protocol::v1::StorageSetProjectionDefinitionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageSetProjectionDefinitionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/SetProjectionDefinition"), request, harmos_protocol::v1::StorageSetProjectionDefinitionResponse::default().descriptor()).await
    }
    pub async fn advance_projection(&self, request: tonic::Request<harmos_protocol::v1::StorageAdvanceProjectionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageAdvanceProjectionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/AdvanceProjection"), request, harmos_protocol::v1::StorageAdvanceProjectionResponse::default().descriptor()).await
    }
    pub async fn list_projection_rows(&self, request: tonic::Request<harmos_protocol::v1::StorageListProjectionRowsRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageListProjectionRowsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/ListProjectionRows"), request, harmos_protocol::v1::StorageListProjectionRowsResponse::default().descriptor()).await
    }
    pub async fn get_projection_status(&self, request: tonic::Request<harmos_protocol::v1::StorageGetProjectionStatusRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetProjectionStatusResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetProjectionStatus"), request, harmos_protocol::v1::StorageGetProjectionStatusResponse::default().descriptor()).await
    }
    pub async fn set_object(&self, request: tonic::Request<harmos_protocol::v1::StorageSetObjectRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageSetObjectResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/SetObject"), request, harmos_protocol::v1::StorageSetObjectResponse::default().descriptor()).await
    }
    pub async fn list_rows(&self, request: tonic::Request<harmos_protocol::v1::StorageListRowsRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageListRowsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/ListRows"), request, harmos_protocol::v1::StorageListRowsResponse::default().descriptor()).await
    }
    pub async fn watch_rows(&self, request: tonic::Request<harmos_protocol::v1::StorageWatchRowsRequest>) -> Result<Watching<harmos_protocol::v1::StorageWatchRowsResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/WatchRows"), request, harmos_protocol::v1::StorageWatchRowsResponse::default().descriptor()).await
    }
    pub async fn append_records(&self, request: tonic::Request<harmos_protocol::v1::StorageAppendRecordsRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageAppendRecordsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/AppendRecords"), request, harmos_protocol::v1::StorageAppendRecordsResponse::default().descriptor()).await
    }
    pub async fn append_samples(&self, request: tonic::Request<harmos_protocol::v1::StorageAppendSamplesRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageAppendSamplesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/AppendSamples"), request, harmos_protocol::v1::StorageAppendSamplesResponse::default().descriptor()).await
    }
    pub async fn append_object_revisions(&self, request: tonic::Request<harmos_protocol::v1::StorageAppendObjectRevisionsRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageAppendObjectRevisionsResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/AppendObjectRevisions"), request, harmos_protocol::v1::StorageAppendObjectRevisionsResponse::default().descriptor()).await
    }
    pub async fn get_file_hash(&self, request: tonic::Request<harmos_protocol::v1::StorageGetFileHashRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetFileHashResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetFileHash"), request, harmos_protocol::v1::StorageGetFileHashResponse::default().descriptor()).await
    }
    pub async fn capture_file(&self, request: tonic::Request<harmos_protocol::v1::StorageCaptureFileRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageCaptureFileResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/CaptureFile"), request, harmos_protocol::v1::StorageCaptureFileResponse::default().descriptor()).await
    }
    pub async fn watch_file_bytes(&self, request: tonic::Request<harmos_protocol::v1::StorageWatchFileBytesRequest>) -> Result<Watching<harmos_protocol::v1::StorageWatchFileBytesResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/WatchFileBytes"), request, harmos_protocol::v1::StorageWatchFileBytesResponse::default().descriptor()).await
    }
    pub async fn watch_file_production(&self, request: tonic::Request<harmos_protocol::v1::StorageWatchFileProductionRequest>) -> Result<Watching<harmos_protocol::v1::StorageWatchFileProductionResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/WatchFileProduction"), request, harmos_protocol::v1::StorageWatchFileProductionResponse::default().descriptor()).await
    }
    pub async fn list_directory_entries(&self, request: tonic::Request<harmos_protocol::v1::StorageListDirectoryEntriesRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageListDirectoryEntriesResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/ListDirectoryEntries"), request, harmos_protocol::v1::StorageListDirectoryEntriesResponse::default().descriptor()).await
    }
    pub async fn watch_directory_entries(&self, request: tonic::Request<harmos_protocol::v1::StorageWatchDirectoryEntriesRequest>) -> Result<Watching<harmos_protocol::v1::StorageWatchDirectoryEntriesResponse>, tonic::Status> {
        self.client.watch_known(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/WatchDirectoryEntries"), request, harmos_protocol::v1::StorageWatchDirectoryEntriesResponse::default().descriptor()).await
    }
    pub async fn get_index_definition(&self, request: tonic::Request<harmos_protocol::v1::StorageGetIndexDefinitionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetIndexDefinitionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetIndexDefinition"), request, harmos_protocol::v1::StorageGetIndexDefinitionResponse::default().descriptor()).await
    }
    pub async fn get_projection_definition(&self, request: tonic::Request<harmos_protocol::v1::StorageGetProjectionDefinitionRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetProjectionDefinitionResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetProjectionDefinition"), request, harmos_protocol::v1::StorageGetProjectionDefinitionResponse::default().descriptor()).await
    }
    pub async fn get_object(&self, request: tonic::Request<harmos_protocol::v1::StorageGetObjectRequest>) -> Result<tonic::Response<harmos_protocol::v1::StorageGetObjectResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Storage/GetObject"), request, harmos_protocol::v1::StorageGetObjectResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct StorageSources {
    client: ClientTransport,
}
impl StorageSources {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<dokime_protocol::v1::storage::SourcesGetRequest>) -> Result<tonic::Response<dokime_protocol::v1::storage::SourcesGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.storage.Sources/Get"), request, dokime_protocol::v1::storage::SourcesGetResponse::default().descriptor()).await
    }
    pub async fn set(&self, request: tonic::Request<dokime_protocol::v1::storage::SourcesSetRequest>) -> Result<tonic::Response<dokime_protocol::v1::storage::SourcesSetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/dokime.v1.storage.Sources/Set"), request, dokime_protocol::v1::storage::SourcesSetResponse::default().descriptor()).await
    }
}
#[derive(Clone)]
pub struct Time {
    client: ClientTransport,
}
impl Time {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            client,
        }
    }
    pub async fn get(&self, request: tonic::Request<harmos_protocol::v1::TimeGetRequest>) -> Result<tonic::Response<harmos_protocol::v1::TimeGetResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Time/Get"), request, harmos_protocol::v1::TimeGetResponse::default().descriptor()).await
    }
    pub async fn sleep(&self, request: tonic::Request<harmos_protocol::v1::TimeSleepRequest>) -> Result<tonic::Response<harmos_protocol::v1::TimeSleepResponse>, tonic::Status> {
        self.client.unary(tonic::codegen::http::uri::PathAndQuery::from_static("/harmos.v1.Time/Sleep"), request, harmos_protocol::v1::TimeSleepResponse::default().descriptor()).await
    }
}
