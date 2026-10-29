/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prepare And Activate Server Handler Resources

use std::sync::{Arc, OnceLock};

use super::{Config, Error, Service, authorization};

pub(super) type Owner = tokio::sync::RwLock<Option<Service>>;

pub struct PreparedService {
    composition: dokime::app::Composition,
    owner: Arc<Owner>,
}

impl PreparedService {
    pub fn composition(&self) -> dokime::app::Composition {
        self.composition.clone()
    }

    pub fn configure(&self, config: &mut dokime::config::RuntimeConfig) {
        config.composition.push(self.composition());
    }

    pub fn configuration(&self, mut config: dokime::RuntimeConfig) -> dokime::RuntimeConfig {
        self.configure(&mut config);
        config
    }

    pub async fn ready(&self) -> Result<Service, Error> {
        self.owner.read().await.clone().ok_or(Error::Configuration("server lifecycle has not initialized"))
    }
}

pub(super) fn prepare(config: Config) -> Result<PreparedService, Error> {
    config.validate()?;
    let owner = Arc::new(Owner::default());
    let authorization = Arc::new(OnceLock::new());
    let prepared_config = config.clone();
    let prepared_owner = owner.clone();
    let prepared_authorization = authorization.clone();
    let initialized_owner = owner.clone();
    let cleanup_owner = owner.clone();
    let composition = dokime::app::Composition {
        prepare: Arc::new(move |setup, live, runtime_config| {
            Service::prepare_handlers(setup, live, &prepared_config, runtime_config).map_err(hosting)?;
            super::remote::operations::prepare(setup, prepared_owner.clone()).map_err(hosting)?;
            setup.resource::<Owner>(prepared_owner.clone());
            let access = authorization::prepare(setup, &runtime_config.schema).map_err(hosting)?;
            prepared_authorization.set(access).map_err(|_| hosting(Error::Configuration("server lifecycle prepared twice")))?;
            Ok(())
        }),
        initialize: Arc::new(move |runtime| {
            let config = config.clone();
            let owner = initialized_owner.clone();
            let access = authorization.get().cloned();
            Box::pin(async move {
                let (listeners, policy) = access.ok_or_else(|| hosting(Error::Configuration("server authorization was not prepared")))?;
                authorization::activate(&runtime, listeners.clone(), policy).map_err(hosting)?;
                let service = Service::activate(runtime, config, listeners).await.map_err(hosting)?;
                *owner.write().await = Some(service);
                Ok(())
            })
        }),
        cleanup: Arc::new(move || {
            let owner = cleanup_owner.clone();
            Box::pin(async move {
                let service = owner.read().await.clone();
                if let Some(service) = service {
                    service.deactivate().await.map_err(hosting)?;
                }
                owner.write().await.take();
                Ok(())
            })
        }),
    };
    Ok(PreparedService { composition, owner })
}

pub(super) async fn controller(owner: &Owner) -> Result<Arc<super::remote::Controller>, tonic::Status> {
    owner
        .read()
        .await
        .as_ref()
        .and_then(Service::controller)
        .ok_or_else(|| tonic::Status::unavailable("remote listener controller is not active"))
}

fn hosting(source: Error) -> dokime::runtime::Error {
    dokime::runtime::Error::Hosting { source: Box::new(source) }
}
