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
use std::net::SocketAddr;

// Third Party Imports
use harmos::grpc::Attached;

// Local Imports
use super::{
    Config, Error, appearance, authorization, catalog, content, execution, history, observation,
    planning, reports,
};
use dokime_app::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Server {
    address: SocketAddr,
    tls: bool,
    attachment: Option<Attached>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Server {
    pub async fn start(runtime: RuntimeHandle, config: Config) -> Result<Self, Error> {
        config.validate()?;
        let tls = config.tls.is_some();
        appearance::register(&runtime, config.max_message_bytes())?;
        catalog::register(&runtime, config.max_message_bytes())?;
        planning::register(&runtime)?;
        execution::register(&runtime, config.max_message_bytes())?;
        observation::register(&runtime, config.max_message_bytes())?;
        history::register(&runtime, config.max_message_bytes())?;
        reports::register(&runtime, config.max_message_bytes())?;
        content::register(&runtime, config.max_message_bytes(), config.local_only())?;
        super::notes::register(&runtime, config.max_message_bytes())?;
        super::prompts::register(&runtime, config.max_message_bytes())?;
        super::telemetry::register(&runtime, config.max_message_bytes())?;
        super::query::register(&runtime, config.max_message_bytes())?;
        super::plugins::register(&runtime, config.max_message_bytes())?;
        super::evidence::register(&runtime, config.max_message_bytes())?;
        super::plots::register(&runtime, config.max_message_bytes())?;
        super::documents::register(&runtime, config.max_message_bytes())?;
        super::settings::register(&runtime, config.max_message_bytes())?;
        super::backend::register(&runtime, config.max_message_bytes()).await?;
        authorization::register(&runtime, &config)?;
        let attachment = runtime
            .grpc()
            .attach(config.into_listener_settings()?)
            .await?;
        Ok(Self {
            address: attachment.local_addr(),
            tls,
            attachment: Some(attachment),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn local_endpoint(&self) -> String {
        let mut address = self.address;
        if address.ip().is_unspecified() {
            address.set_ip(if address.is_ipv4() {
                std::net::Ipv4Addr::LOCALHOST.into()
            } else {
                std::net::Ipv6Addr::LOCALHOST.into()
            });
        }
        format!("{}://{address}", if self.tls { "https" } else { "http" })
    }

    pub async fn shutdown(mut self) -> Result<(), Error> {
        if let Some(attachment) = self.attachment.take() {
            attachment.detach().await?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
