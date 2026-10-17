/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! In-Process Backends That Stories Connect The Desktop Session To.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{path::Path, sync::Arc};
// Third Party Imports

// Local Imports
use super::artifact;
use dokime_app::{
    Runtime, RuntimeConfig,
    core::{Identifier, Metadata},
};
use dokime_gui::connection::{
    ConnectRequest, ConnectionCredentials, ConnectionSession, ConnectionTarget,
};
use dokime_gui::native_session::BackendSelection;
use dokime_server::grpc::{Config, Server, access::Role};

/* === Constants === */
/// The Bearer Token The Fixture Server Grants The Desktop.
pub const TOKEN: &str = "rpc-test-token-012345678901234567890123456789";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// A Runtime With An Execution Fixture Served Over gRPC And A Connected Session.
pub struct Fixture {
    pub session: ConnectionSession,
    pub server: Server,
    pub runtime: Runtime,
    pub artifacts: Arc<artifact::Provider>,
    _root: tempfile::TempDir,
}

/// A Local Runtime Served Over gRPC That A Desktop Session Attaches To.
pub struct Published {
    runtime: Runtime,
    server: Server,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Fixture {
    pub async fn new() -> Self {
        Self::with_role(Role::Control).await
    }

    pub async fn with_role(role: Role) -> Self {
        // Runtime With A Calibration Record
        let root = tempfile::tempdir().unwrap();
        let artifacts = Arc::new(artifact::Provider::default());
        let runtime = Runtime::with_execution_fixture(
            RuntimeConfig::new(
                root.path(),
                Metadata::new(Identifier::new("rpc.fixture").unwrap(), "RPC fixture", None),
            ),
            "fixture",
            artifacts.clone(),
        )
        .await
        .unwrap();
        let calibration = dokime_app::records::ProductCalibrationDetails::new(
            "rpc-device",
            dokime_app::records::ProductCalibrationKind::Field,
            dokime_app::records::ProductCalibrationPhase::Completed,
        );
        runtime
            .handle()
            .publish_product_record(calibration.into_draft("rpc.fixture"))
            .await
            .unwrap();

        // Authenticated Server And Session
        let grant = dokime_server::grpc::access::BearerGrant::new("native-gui".into(), role, TOKEN)
            .unwrap();
        let config = Config::local().with_bearer_auth(vec![grant]).unwrap();
        let server = Server::start(runtime.handle(), config).await.unwrap();
        let session = ConnectionSession::default();
        session
            .connect(ConnectRequest {
                target: ConnectionTarget::Local {
                    endpoint: server.local_endpoint(),
                },
                credentials: ConnectionCredentials::Bearer {
                    token: TOKEN.into(),
                    remember: false,
                },
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
        let mut config = dokime_app::config::Config::local(root);
        config.seed_plugins = false;
        config.credential_backend =
            serde_json::from_value(serde_json::json!({"backend":"memory"})).unwrap();
        let runtime = dokime_app::app::launch(config).await.unwrap();
        let server = Server::start(runtime.handle(), Config::local())
            .await
            .unwrap();
        Self { runtime, server }
    }

    pub fn endpoint(&self) -> String {
        self.server.local_endpoint()
    }

    pub async fn shutdown(mut self) {
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

/* === Public Functions === */
/// Selects An Attachment To An Unauthenticated Local Endpoint.
pub fn attach(endpoint: String) -> BackendSelection {
    BackendSelection::Attach {
        request: ConnectRequest {
            target: ConnectionTarget::Local { endpoint },
            credentials: ConnectionCredentials::None,
            expected_backend_id: None,
        },
    }
}

/* === Private Functions === */

/* === Tests === */
