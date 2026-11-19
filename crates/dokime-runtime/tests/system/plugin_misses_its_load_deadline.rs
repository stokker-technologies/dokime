/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Plugin That Never Loads Within the Plugins Operation Timeout Is Reported Unavailable, Naming the Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;

// Third Party Imports
use harmos_protocol::host::PluginState;
use tokio::net::TcpListener;

// Local Imports
use super::support;
use dokime_runtime::{Host, PLUGINS_TIMEOUT};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn plugin_misses_its_load_deadline() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // A Remote Endpoint Accepts Connections and Never Answers, So the Plugin Can Never Describe Itself.
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let held = tokio::spawn(async move {
            let mut connections = Vec::new();
            while let Ok((connection, _)) = listener.accept().await {
                connections.push(connection);
            }
        });
        let root = support::tempdir()?;
        let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;

        // Adding It Fails Once the Plugins Timeout Bounds Its Load, and the Plugin Is Reported Unavailable.
        let refused = host.add_remote("silent", &endpoint).await.expect_err("Refuse a Plugin That Never Loads");
        let reason = refused.to_string();
        assert!(reason.contains(&format!("{PLUGINS_TIMEOUT:?}")), "Name the Plugins Timeout That Bounded the Load: {reason}");
        assert!(
            host.plugin_entries()?.contains(&("silent".to_owned(), PluginState::Unavailable)),
            "Report the Plugin Unavailable: {:?}",
            host.plugin_entries()?
        );
        host.shutdown().await?;
        held.abort();
        Ok(())
    }))
}
