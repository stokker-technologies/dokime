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
use std::{path::Path, sync::Arc};
// Third Party Imports

// Local Imports
use super::artifact;
use dokime::domain::model;
use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
use dokime_gui::native_session::BackendSelection;
use dokime_server::grpc::{Config, Server};
use {dokime::Identifier, dokime::Metadata, dokime::Runtime, dokime::RuntimeConfig};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Fixture {
    pub session: ConnectionSession,
    pub server: Server,
    pub runtime: Runtime,
    pub artifacts: Arc<artifact::Provider>,
    _root: tempfile::TempDir,
}

pub struct Published {
    owner: dokime_gui::native_session::DesktopSession,
    endpoint: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn new() -> Self {
        // Runtime with a Calibration Record
        let root = tempfile::tempdir().unwrap();
        let artifacts = Arc::new(artifact::Provider::default());
        let mut config = RuntimeConfig::new(root.path(), Metadata::new(Identifier::new("rpc.fixture").unwrap(), "RPC fixture", None));
        super::settings::configure(&mut config);
        config.storage.response.batch_bytes = std::num::NonZeroUsize::new(512 * 1024).unwrap();
        let server_config = Config::local().with_max_message_bytes(1024 * 1024);
        let prepared_runtime = dokime_server::grpc::Service::prepare(server_config).unwrap();
        let runtime = Runtime::with_execution_fixture(prepared_runtime.configuration(config), "fixture", artifacts.clone()).await.unwrap();
        let calibration = model::records::product::ProductCalibrationDetails::new(
            "rpc-device",
            model::records::product::ProductCalibrationKind::Field,
            model::records::product::ProductCalibrationPhase::Completed,
        );
        runtime.handle().publish_product_record(calibration.into_draft("rpc.fixture")).await.unwrap();

        // Native Server and Session
        // Keep The 1.2 MB Artifact In Multiple Frames While Admitting 256 KiB Upload Chunks.

        let server = prepared_runtime.ready().await.unwrap().listen().await.unwrap();
        let session = ConnectionSession::default();
        session
            .connect(ConnectRequest {
                target: ConnectionTarget::Local { endpoint: server.local_endpoint() },
                expected_backend_id: None,
            })
            .await
            .unwrap();

        Self {
            session,
            server,
            runtime,
            artifacts,
            _root: root,
        }
    }

    pub fn generation(&self) -> u64 {
        self.session.status().generation.parse().unwrap()
    }

    pub async fn close(mut self) {
        self.session.disconnect().unwrap();
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

impl Published {
    pub async fn start(root: &Path) -> Self {
        let owner = dokime_gui::native_session::DesktopSession::with_local_backend(
            ConnectionSession::default(),
            dokime_gui::native_session::backend::Backend::new(root.to_owned()),
        );
        let endpoint = owner.select(BackendSelection::OwnedLocal).await.unwrap().connection.endpoint.unwrap();
        Self { owner, endpoint }
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub async fn shutdown(self) {
        self.owner.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

pub fn attach(endpoint: String) -> BackendSelection {
    BackendSelection::Attach {
        request: ConnectRequest {
            target: ConnectionTarget::Local { endpoint },
            expected_backend_id: None,
        },
    }
}

/* === Private Functions === */

/* === Tests === */
