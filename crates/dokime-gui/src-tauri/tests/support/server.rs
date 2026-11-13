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
use std::path::Path;

// Local Imports
use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
use dokime_gui::native_session::{BackendSelection, DesktopSession, backend::Backend};
use dokime_protocol::{client, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Fixture {
    pub session: ConnectionSession,
    pub owner: DesktopSession,
    _root: dokime_gui::test_support::Root,
}

pub struct Published {
    owner: DesktopSession,
    endpoint: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn new() -> Self {
        // Seed an Imported Evidence Container Through the Current Storage Owner
        let root = dokime_gui::test_support::tempdir().unwrap();
        let storage = dokime_storage::Store::open(root.path()).unwrap();
        storage
            .create(&dokime_storage::Execution {
                identifier: "desktop-content".into(),
                cycle: "desktop-cycle".into(),
                origin: Some(v1::Origin {
                    kind: Some(v1::origin::Kind::Import(v1::Import::default())),
                }),
                ..Default::default()
            })
            .unwrap();
        drop(storage);

        // Start the Same Embedded Host and SDK Connection as the Running Desktop
        let session = ConnectionSession::default();
        let owner = DesktopSession::with_local_backend(session.clone(), Backend::new(root.path().to_owned()));
        owner.select(BackendSelection::OwnedLocal).await.unwrap();
        Self { session, owner, _root: root }
    }

    pub fn generation(&self) -> u64 {
        self.session.status().generation.parse().unwrap()
    }

    pub async fn client(&self) -> dokime_gui::connection::host::Session {
        self.owner.local_client().await.unwrap()
    }

    pub async fn thermal(&self) -> dokime_runtime_next::PluginProcess {
        // Use the Shared SDK Fixture Produced Before This Story
        let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| dokime_gui::test_support::fixture("dokime-rust-thermal"));
        let placement = dokime_runtime_next::PlacementSettings {
            connect_timeout: std::time::Duration::from_secs(30),
            reconnect_delay: std::time::Duration::from_millis(25),
            stop_grace: std::time::Duration::from_secs(30),
            stream_window: std::num::NonZeroUsize::new(8).unwrap(),
            message_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let plugin = dokime_runtime_next::PluginProcess::start(&executable, true, &placement).await.unwrap();
        let _: client::Plugin = self
            .client()
            .await
            .unary("/dokime.client.v1.Plugins/Add", client::AddRemotePlugin { url: plugin.endpoint().into() })
            .await
            .unwrap();
        plugin
    }

    pub async fn close(self) {
        self.owner.shutdown().await.unwrap();
    }
}

impl Published {
    pub async fn start(root: &Path) -> Self {
        let owner = DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(root.to_owned()));
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
