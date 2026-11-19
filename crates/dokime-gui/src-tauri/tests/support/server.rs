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

use dokime_client_internal::PluginsCreateOptions;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginState;
use dokime_client_internal::host::PluginValue;
use dokime_client_internal::host::plugin_placement;
use dokime_client_internal::{PluginsGetOptions, PluginsWatchOptions};

use dokime_gui::connection::{ConnectRequest, ConnectionSession, ConnectionTarget};
use dokime_gui::native_session::{BackendSelection, DesktopSession, backend::Backend};
use dokime_sdk::Import;
use dokime_sdk::Origin;
use dokime_sdk::origin;

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
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
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

    pub async fn thermal(&self) -> dokime_runtime::PluginProcess {
        // Use the Shared SDK Fixture Produced Before This Story
        let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| dokime_gui::test_support::fixture("dokime-rust-thermal"));
        let placement = dokime_runtime::PlacementSettings {
            reconnect_delay: std::time::Duration::from_millis(25),
            stream_window: std::num::NonZeroUsize::new(8).unwrap(),
            interval_window: dokime_runtime::INTERVAL_WINDOW,
            interval_lag: dokime_runtime::INTERVAL_LAG,
            message_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let plugin = dokime_runtime::PluginProcess::start(&executable, true, &placement).await.unwrap();
        let session = self.client().await;
        let options = PluginsCreateOptions {
            context: session.call_options().unwrap(),
            ..Default::default()
        };
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
        };
        let admitted = session
            .execute(session.client().plugins.create_with("desktop_thermal", placement, PluginValue { enabled: true }, options))
            .await
            .unwrap();
        let mut entry = admitted.entry.unwrap();
        let options = PluginsWatchOptions {
            name: Some(entry.name.clone()),
            context: session.call_options().unwrap(),
            ..Default::default()
        };
        let mut watching = session.execute(session.client().plugins.watch_with(options)).await.unwrap();
        while entry.state != PluginState::Ready as i32 {
            assert!(entry.failure.is_none(), "Admit Desktop Fixture: {:?}", entry.failure);
            assert_ne!(entry.state, PluginState::Stopped as i32);
            session.execute(watching.next()).await.unwrap().expect("Admit Desktop Fixture: Watch Closed");
            let options = PluginsGetOptions {
                context: session.call_options().unwrap(),
                ..Default::default()
            };
            entry = session.execute(session.client().plugins.get_with(entry.name, options)).await.unwrap().entry.unwrap();
        }
        drop(watching);
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
