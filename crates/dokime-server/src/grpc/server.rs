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
use std::{net::SocketAddr, sync::Arc};

// Third Party Imports
use dokime::harmos::grpc::Attached;

// Local Imports
use super::access::Listeners;
use super::{Config, Error, appearance, authorization, catalog, content, execution, history, observation, planning, reports};
use dokime::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Service {
    runtime: RuntimeHandle,
    config: Config,
    listeners: Arc<Listeners>,
    remote: Option<Arc<super::remote::Controller>>,
}

pub struct Server {
    address: SocketAddr,
    tls: bool,
    attachment: Option<Attached>,
    listeners: Arc<Listeners>,
    remote: Option<Arc<super::remote::Controller>>,
    detach_failure: Option<Arc<dokime::harmos::Error>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Service {
    pub async fn new(runtime: RuntimeHandle, config: Config) -> Result<Self, Error> {
        config.validate()?;
        appearance::register(&runtime, config.max_message_bytes()).await?;
        catalog::register(&runtime, config.max_message_bytes()).await?;
        planning::register(&runtime, config.max_message_bytes()).await?;
        execution::register(&runtime, config.max_message_bytes()).await?;
        observation::register(&runtime, config.max_message_bytes()).await?;
        history::register(&runtime, config.max_message_bytes()).await?;
        reports::register(&runtime, config.max_message_bytes()).await?;
        content::register(&runtime, config.max_message_bytes()).await?;
        super::files::register(&runtime).await?;
        super::media::register(&runtime, config.max_message_bytes()).await?;
        super::notes::register(&runtime, config.max_message_bytes()).await?;
        super::prompts::register(&runtime, config.max_message_bytes()).await?;
        super::telemetry::register(&runtime, config.max_message_bytes()).await?;
        super::query::register(&runtime, config.max_message_bytes()).await?;
        super::plugins::register(&runtime, config.max_message_bytes()).await?;
        super::evidence::register(&runtime, config.max_message_bytes()).await?;
        super::plots::register(&runtime, config.max_message_bytes()).await?;
        super::documents::register(&runtime, config.max_message_bytes()).await?;
        super::settings::register(&runtime, config.max_message_bytes()).await?;
        super::backend::register(&runtime, config.max_message_bytes()).await?;
        let journal = config.trace_journal.clone().unwrap_or_else(|| crate::trace::Journal::new(crate::trace::Limits::default()));
        super::trace::register(&runtime, config.max_message_bytes(), journal).await?;
        let listeners = authorization::register(&runtime)?;
        let mut service = Self { runtime: runtime.clone(), config, listeners, remote: None };
        service.remote = Some(super::remote::register(&runtime, service.clone()).await?);
        Ok(service)
    }

    pub(super) fn sessions(&self) -> Arc<super::remote::sessions::SessionLog> {
        self.listeners.sessions()
    }

    pub(super) fn message_limit(&self) -> usize {
        self.config.max_message_bytes()
    }

    pub async fn listen(&self) -> Result<Server, Error> {
        let mut server = self.listen_with(self.config.clone()).await?;
        if let Some(remote) = &self.remote {
            remote.start().await;
            server.remote = Some(remote.clone());
        }
        Ok(server)
    }

    pub async fn listen_with(&self, config: Config) -> Result<Server, Error> {
        config.validate()?;
        if config.max_message_bytes() != self.config.max_message_bytes() {
            return Err(Error::Configuration("all listeners must use the registered service message limit"));
        }
        let tls = config.tls.is_some();
        let policy = config.policy.clone();
        let local = config.local_only();
        let settings = config.into_listener_settings()?;
        let attachment = self.runtime.grpc().attach(settings).await?;
        let address = attachment.local_addr();
        self.listeners.insert(address, policy, local);
        Ok(Server { address, tls, attachment: Some(attachment), listeners: self.listeners.clone(), remote: None, detach_failure: None })
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
            address.set_ip(if address.is_ipv4() { std::net::Ipv4Addr::LOCALHOST.into() } else { std::net::Ipv6Addr::LOCALHOST.into() });
        }
        format!("{}://{address}", if self.tls { "https" } else { "http" })
    }

    pub async fn shutdown(mut self) -> Result<(), Error> {
        self.drain().await
    }

    pub(crate) async fn drain(&mut self) -> Result<(), Error> {
        if let Some(remote) = self.remote.as_ref() {
            remote.stop().await?;
        }
        self.detach_listener().await
    }

    pub(super) async fn detach(mut self) -> Result<(), Error> {
        self.detach_listener().await
    }

    pub(super) async fn detach_listener(&mut self) -> Result<(), Error> {
        if let Some(error) = &self.detach_failure {
            return Err(Error::ListenerDetach(error.clone()));
        }
        self.listeners.remove(self.address);
        if let Some(attachment) = self.attachment.take()
            && let Err(error) = attachment.detach().await
        {
            let error = Arc::new(error);
            self.detach_failure = Some(error.clone());
            return Err(Error::ListenerDetach(error));
        }
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.listeners.remove(self.address);
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
