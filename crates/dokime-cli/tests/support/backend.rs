/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

// Third Party Imports
use dokime::harmos::Context;
use dokime::harmos::operations::{OperationDeclaration, Response};
use dokime::harmos::storage::{ErasedCatalog, StorageManager};
use dokime::operations::hosting::{Serve, Served};
use sha2::{Digest, Sha256};

// Local Imports
use dokime::client::contracts::{self, backend, plugins as v1, runtime};
use dokime::domain::model;

/* === Constants === */

pub(crate) const ID: &str = "01900000-0000-7000-8000-000000000001";
pub(crate) const CHUNK_LIMIT_BYTES: u32 = 256 * 1024;

/* === Macros === */

macro_rules! operation {
    ($declaration:ty, $host:ident, $message:ident, $input:ident, $body:block) => {
        impl Serve<$declaration> for Host {
            fn serve<'a>(
                &'a self, $message: $declaration, ctx: Context<'a, $declaration>,
            ) -> Served<'a, <$declaration as OperationDeclaration>::Response> {
                Box::pin(async move {
                    let $host = self;
                    let $input = ctx.streams();
                    let served: Result<Response<<$declaration as OperationDeclaration>::Response>, tonic::Status> =
                        async move { $body }.await;
                    served
                })
            }
        }
    };
}

/* === Type Aliases === */

type StorageEngine = <StorageManager<ErasedCatalog> as std::ops::Deref>::Target;

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Host {
    pub(crate) report: Option<Report>,
    pub(crate) entry: Option<model::plugins::wire::PluginInstallation>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) calls: Arc<Mutex<Vec<&'static str>>>,
    pub(crate) install_code: Arc<Mutex<tonic::Code>>,
    pub(crate) descriptor_sha256: Vec<u8>,
}

pub(crate) struct Fixture;

#[derive(Clone)]
pub(crate) struct Report {
    pub(crate) bytes: Arc<Vec<u8>>,
    pub(crate) mode: Arc<Mutex<ReportMode>>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReportMode {
    Current,
    Sealed,
    DigestMismatch,
    TransportFailure,
    OffsetMismatch,
    SizeMismatch,
}

#[dokime::harmos::lanes(harmos_path = dokime::harmos)]
pub(crate) enum Lanes {
    #[default]
    Open,
}

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::Application for Fixture {
    type Lanes = Lanes;
    type State = ();
    type Transaction = dokime::harmos::state::Never;
    type Record = ();
    type Streams = ();
}

operation!(v1::Install, host, _request, input, {
    host.calls.lock().unwrap().push("install");
    let header = input.recv().await.unwrap();
    let Some(model::plugins::wire::InstallPluginFrameFrame::Header(header)) = &header.frame else { panic!("header first") };
    assert_eq!(header.size_bytes, host.bytes.len() as u64);
    assert_eq!(header.sha256, Sha256::digest(&host.bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    let mut bytes = Vec::new();
    while let Some(frame) = input.recv().await {
        let Some(model::plugins::wire::InstallPluginFrameFrame::Chunk(chunk)) = &frame.frame else { panic!("chunk required") };
        assert_eq!(chunk.offset, bytes.len() as u64);
        assert!(chunk.data.len() <= CHUNK_LIMIT_BYTES as usize, "every chunk fits the advertised chunk limit");
        bytes.extend_from_slice(&chunk.data);
    }
    assert_eq!(bytes, host.bytes);
    let code = *host.install_code.lock().unwrap();
    if code != tonic::Code::Ok {
        return Err(tonic::Status::new(code, "installation acknowledgement unavailable"));
    }
    Ok(host.entry.clone().expect("installation fixture").into())
});

operation!(backend::GetIdentity, host, _request, _input, {
    Ok(model::backend::wire::BackendIdentity {
        backend_id: ID.into(),
        epoch: ID.into(),
        api: Some(model::backend::wire::ApiIdentity {
            package: model::backend::wire::API_PACKAGE.into(),
            version: model::backend::wire::API_VERSION.into(),
            descriptor_sha256: host.descriptor_sha256.clone(),
        }),
        product_version: env!("CARGO_PKG_VERSION").into(),
        target: "x86_64-unknown-linux-gnu".into(),
        display_name: "Installation Fixture".into(),
        ..Default::default()
    }
    .into())
});

operation!(backend::GetHealth, _host, _request, _input, {
    Ok(model::backend::wire::BackendHealth {
        backend_id: ID.into(),
        epoch: ID.into(),
        readiness: model::backend::wire::Readiness::Ready,
        observed_at: None,
        message: String::new(),
    }
    .into())
});

operation!(backend::GetAvailability, _host, _request, _input, {
    Ok(model::backend::wire::BackendAvailability {
        backend_id: ID.into(),
        epoch: ID.into(),
        methods: [
            "/harmos.host.v1.Host/Catalog",
            "/dokime.v1.Backend/GetIdentity",
            "/dokime.v1.Backend/GetHealth",
            "/dokime.v1.Backend/GetAvailability",
            "/dokime.v1.Runtime/GetIdentity",
            "/dokime.v1.Runtime/GetHealth",
            "/dokime.v1.Plugins/Install",
            "/dokime.v1.Reports/DownloadExecutionReport",
            "/dokime.v1.Reports/DownloadCycleReport",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        limits: Some(model::backend::wire::BackendLimits {
            max_request_bytes: 16 * 1024 * 1024,
            max_response_bytes: if _host.report.is_some() { 1024 * 1024 } else { 16 * 1024 * 1024 },
            max_content_bytes: Some(272 * 1024 * 1024),
            max_chunk_bytes: Some(if _host.report.is_some() { 850_000 } else { CHUNK_LIMIT_BYTES }),
            ..Default::default()
        }),
    }
    .into())
});

operation!(runtime::GetIdentity, _host, _request, _input, {
    Ok(model::runtime::wire::Identity {
        backend_id: ID.into(),
        epoch: ID.into(),
        display_name: "Installation Fixture".into(),
        product_version: env!("CARGO_PKG_VERSION").into(),
    }
    .into())
});

operation!(runtime::GetHealth, _host, _request, _input, {
    Ok(model::runtime::wire::Health {
        backend_id: ID.into(),
        epoch: ID.into(),
        readiness: model::runtime::wire::Readiness::Ready,
        observed_at_seconds: 0,
        observed_at_nanoseconds: 0,
        message: String::new(),
    }
    .into())
});

#[derive(Default, serde::Deserialize)]
pub(crate) struct FixtureConfig {
    #[serde(skip)]
    pub(crate) host: Option<Arc<Host>>,
}

#[harmos_sdk::catalog(harmos_path = dokime::harmos)]
enum HostOperations {
    Operation0(v1::Install),
    Operation1(backend::GetIdentity),
    Operation2(backend::GetHealth),
    Operation3(backend::GetAvailability),
    Operation4(runtime::GetIdentity),
    Operation5(runtime::GetHealth),
    Operation6(contracts::reports::DownloadExecutionReport),
    Operation7(contracts::reports::DownloadCycleReport),
}

impl dokime::harmos::Lifecycle for Fixture {
    type Config = FixtureConfig;
    type Error = dokime::harmos::Error;
    type Context = dokime::harmos::Startup<Self>;
    async fn prepare(config: Self::Config, setup: &mut dokime::harmos::Setup<Self>) -> dokime::harmos::Result<Self> {
        let host = config.host.expect("fixture hosting resources");
        setup.resource::<Option<Arc<StorageEngine>>>(None);
        setup.resource::<dokime::operations::hosting::Host<v1::Install>>(dokime::operations::hosting::Host::<v1::Install>::new(
            host.clone(),
        ));
        setup.resource::<dokime::operations::hosting::Host<backend::GetIdentity>>(
            dokime::operations::hosting::Host::<backend::GetIdentity>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<backend::GetHealth>>(
            dokime::operations::hosting::Host::<backend::GetHealth>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<backend::GetAvailability>>(dokime::operations::hosting::Host::<
            backend::GetAvailability,
        >::new(host.clone()));
        setup.resource::<dokime::operations::hosting::Host<runtime::GetIdentity>>(
            dokime::operations::hosting::Host::<runtime::GetIdentity>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<runtime::GetHealth>>(
            dokime::operations::hosting::Host::<runtime::GetHealth>::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<contracts::reports::DownloadExecutionReport>>(
            dokime::operations::hosting::Host::new(host.clone()),
        );
        setup.resource::<dokime::operations::hosting::Host<contracts::reports::DownloadCycleReport>>(
            dokime::operations::hosting::Host::new(host.clone()),
        );
        setup.operations::<Discovery>(Lanes::Open);
        setup.operations::<HostOperations>(Lanes::Open);
        Ok(Self)
    }
}

operation!(contracts::reports::DownloadExecutionReport, host, _request, output, { report(host, output, true).await });

operation!(contracts::reports::DownloadCycleReport, host, _request, output, { report(host, output, false).await });

async fn report(
    host: &Host, output: &dokime::harmos::operations::StreamOutput<model::reports::wire::ReportHtmlFrame>, execution: bool,
) -> Result<Response<()>, tonic::Status> {
    use model::reports::wire::*;
    let report = host.report.as_ref().unwrap();
    let mode = *report.mode.lock().unwrap();
    let header = ReportMetadata {
        test_cycle_id: ID.into(),
        test_execution_id: execution.then(|| ID.into()),
        identifier: if execution { "execution.report" } else { "cycle.summary" }.into(),
        metadata: None,
        generated_at: None,
        origin: if mode == ReportMode::Sealed { ReportOrigin::Sealed } else { ReportOrigin::Current },
        warnings: Vec::new(),
    };
    output
        .send_owned(
            ReportHtmlFrame {
                payload: Some(ReportHtmlFramePayload::Report(Box::new(header))),
            }
            .into(),
        )
        .await
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
    // A Single Chunk Fits The Wire Ceiling But Its Base64 JSON Does Not.
    for (index, chunk) in report.bytes.chunks(850_000).enumerate() {
        let offset = (index * 850_000) as u64 + u64::from(mode == ReportMode::OffsetMismatch);
        output
            .send_owned(
                ReportHtmlFrame {
                    payload: Some(ReportHtmlFramePayload::Chunk(ReportHtmlChunk { offset, data: chunk.to_vec() })),
                }
                .into(),
            )
            .await
            .map_err(|error| tonic::Status::internal(error.to_string()))?;
    }
    let sha256 = if mode == ReportMode::DigestMismatch {
        "0".repeat(64)
    } else {
        Sha256::digest(report.bytes.as_slice()).iter().map(|byte| format!("{byte:02x}")).collect()
    };
    let size_bytes = report.bytes.len() as u64 + u64::from(mode == ReportMode::SizeMismatch);
    output
        .send_owned(
            ReportHtmlFrame {
                payload: Some(ReportHtmlFramePayload::Complete(ReportHtmlCompletion { size_bytes, sha256 })),
            }
            .into(),
        )
        .await
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
    if mode == ReportMode::TransportFailure {
        return Err(tonic::Status::unavailable("final transport failed"));
    }
    Ok(().into())
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[harmos_sdk::catalog(harmos_path = dokime::harmos)]
enum Discovery {
    Catalog(dokime::harmos::builtins::host::Catalog),
}
