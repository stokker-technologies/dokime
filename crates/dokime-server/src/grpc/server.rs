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
use dokime_runtime::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Configured operations and immutable listener policy for one runtime.
pub struct Service {
    runtime: RuntimeHandle,
    settings: harmos::grpc::Settings,
    tls: bool,
}

pub struct Server {
    address: SocketAddr,
    tls: bool,
    attachment: Option<Attached>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Service {
    pub async fn new(runtime: RuntimeHandle, config: Config) -> Result<Self, Error> {
        config.validate()?;
        let tls = config.tls.is_some();
        appearance::register(&runtime, config.max_message_bytes())?;
        catalog::register(&runtime, config.max_message_bytes())?;
        planning::register(&runtime)?;
        execution::register(&runtime, config.max_message_bytes())?;
        observation::register(&runtime, config.max_message_bytes())?;
        history::register(&runtime, config.max_message_bytes())?;
        reports::register(&runtime, config.max_message_bytes())?;
        content::register(&runtime, config.max_message_bytes())?;
        super::files::register(&runtime)?;
        super::media::register(&runtime, config.max_message_bytes())?;
        super::notes::register(&runtime, config.max_message_bytes())?;
        super::prompts::register(&runtime, config.max_message_bytes())?;
        super::telemetry::register(&runtime, config.max_message_bytes())?;
        super::query::register(&runtime, config.max_message_bytes())?;
        super::plugins::register(&runtime, config.max_message_bytes(), config.local_only())?;
        super::evidence::register(&runtime, config.max_message_bytes())?;
        super::plots::register(&runtime, config.max_message_bytes())?;
        super::documents::register(&runtime, config.max_message_bytes())?;
        super::settings::register(&runtime, config.max_message_bytes())?;
        super::backend::register(&runtime, config.max_message_bytes()).await?;
        super::trace::register(&runtime, config.max_message_bytes())?;
        authorization::register(&runtime, &config)?;
        let settings = config.into_listener_settings()?;
        Ok(Self {
            runtime,
            settings,
            tls,
        })
    }

    /// Attach or reattach this configured service without registering it again.
    pub async fn listen(&self) -> Result<Server, Error> {
        let attachment = self.runtime.grpc().attach(self.settings.clone()).await?;
        Ok(Server {
            address: attachment.local_addr(),
            tls: self.tls,
            attachment: Some(attachment),
        })
    }
}

impl Server {
    pub async fn start(runtime: RuntimeHandle, config: Config) -> Result<Self, Error> {
        Service::new(runtime, config).await?.listen().await
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
