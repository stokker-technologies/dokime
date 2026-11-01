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
use super::{Config, Error, appearance, catalog, content, execution, history, observation, planning, reports};
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
    pub fn prepare(config: Config) -> Result<super::PreparedService, Error> {
        super::preparation::prepare(config)
    }

    pub(super) fn prepare_handlers(
        setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, live: &dokime::app::LiveRuntime, config: &Config,
        runtime_config: &dokime::RuntimeConfig,
    ) -> Result<(), Error> {
        appearance::prepare(setup, live, config.max_message_bytes())?;
        catalog::prepare(setup, live, config.max_message_bytes())?;
        planning::prepare(setup, live, config.max_message_bytes())?;
        execution::prepare(setup, live, config.max_message_bytes())?;
        observation::prepare(setup, live, config.max_message_bytes())?;
        history::prepare(setup, live, config.max_message_bytes())?;
        reports::prepare(setup, live, config.max_message_bytes())?;
        content::prepare(setup, live, config.max_message_bytes())?;
        super::files::prepare(setup, live)?;
        super::media::prepare(setup, live, config.max_message_bytes())?;
        super::notes::prepare(setup, live, config.max_message_bytes())?;
        super::prompts::prepare(setup, live, config.max_message_bytes())?;
        super::telemetry::prepare(setup, live, config.max_message_bytes())?;
        super::query::prepare(setup, live, config.max_message_bytes())?;
        super::plugins::prepare(setup, live, config.max_message_bytes())?;
        super::evidence::prepare(setup, live, config.max_message_bytes())?;
        super::plots::prepare(setup, live, config.max_message_bytes())?;
        super::documents::prepare(setup, live, config.max_message_bytes())?;
        super::settings::prepare(setup, live, config.max_message_bytes())?;
        super::backend::prepare(setup, live, config.max_message_bytes(), runtime_config)?;
        let journal = config.trace_journal.clone().unwrap_or_else(|| crate::trace::Journal::new(crate::trace::Limits::default()));
        super::trace::prepare(setup, live, config.max_message_bytes(), journal)?;
        setup.operations::<dokime::operations::appearance::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::backend::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::catalog::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::content::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::documents::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::evidence::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::execution::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::files::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::history::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::media::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::notes::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::observation::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::planning::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::plots::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::plugins::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::prompts::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::query::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::settings::Operations>(dokime::app::Lanes::Open);
        setup.operations::<dokime::operations::telemetry::Operations>(dokime::app::Lanes::Open);
        Ok(())
    }

    pub(super) async fn activate(runtime: RuntimeHandle, config: Config, listeners: Arc<Listeners>) -> Result<Self, Error> {
        let mut service = Self {
            runtime: runtime.clone(),
            config,
            listeners,
            remote: None,
        };
        service.remote = Some(super::remote::activate(&runtime, service.clone()).await?);
        Ok(service)
    }

    pub(super) fn controller(&self) -> Option<Arc<super::remote::Controller>> {
        self.remote.clone()
    }

    pub(super) async fn deactivate(&self) -> Result<(), Error> {
        if let Some(remote) = &self.remote {
            remote.stop().await?;
        }
        Ok(())
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
        Ok(Server {
            address,
            tls,
            attachment: Some(attachment),
            listeners: self.listeners.clone(),
            remote: None,
            detach_failure: None,
        })
    }
}

impl Server {
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
