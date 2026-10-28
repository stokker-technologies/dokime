/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use harmos::{Context, RegistrationExt};
use tonic::Status;

use crate::operations::hosting::{Host, Serve, Served};
use crate::operations::secrets::{Export, Import, List, Lookup, Preview, Put, Remove};
use crate::secrets::{Change, SecretStore, SecretStoreError, SecretValue};
use crate::{RuntimeHandle, app::Lanes};
use dokime_domain::model::secrets::catalog::{Snapshot, Value};
use dokime_domain::model::secrets::transfer::{Archive, Preview as ImportPreview};

impl Serve<Lookup> for SecretStore {
    fn serve<'a>(&'a self, request: Lookup, _: Context<'a, Lookup>) -> Served<'a, Value> {
        Box::pin(async move {
            let value = self.get_secret(&request.identifier).await?;
            let value = value.ok_or(SecretStoreError::Invalid("The named secret is missing or was deleted"))?;
            Ok(Value { value: Some(value.expose().to_owned()) }.into())
        })
    }
}

impl Serve<List> for SecretStore {
    fn serve<'a>(&'a self, _: List, _: Context<'a, List>) -> Served<'a, Snapshot> {
        Box::pin(async move { Ok(self.snapshot(false).await?.into()) })
    }
}

impl Serve<Put> for SecretStore {
    fn serve<'a>(&'a self, request: Put, _: Context<'a, Put>) -> Served<'a, Snapshot> {
        Box::pin(async move {
            let change = Change {
                metadata: request.metadata,
                value: request.value.map(SecretValue::new),
            };
            let cleanup = self.apply(vec![change], &[], request.generation).await?;
            Ok(self.snapshot(cleanup).await?.into())
        })
    }
}

impl Serve<Remove> for SecretStore {
    fn serve<'a>(&'a self, request: Remove, _: Context<'a, Remove>) -> Served<'a, Snapshot> {
        Box::pin(async move {
            let cleanup = self.apply(Vec::new(), &request.identifiers, request.generation).await?;
            Ok(self.snapshot(cleanup).await?.into())
        })
    }
}

impl Serve<Export> for SecretStore {
    fn serve<'a>(&'a self, request: Export, _: Context<'a, Export>) -> Served<'a, Archive> {
        Box::pin(async move { Ok(self.export(&request.identifiers, request.generation).await?.into()) })
    }
}

impl Serve<Preview> for SecretStore {
    fn serve<'a>(&'a self, request: Preview, _: Context<'a, Preview>) -> Served<'a, ImportPreview> {
        Box::pin(async move { Ok(self.preview(&request.data).await?.into()) })
    }
}

impl Serve<Import> for SecretStore {
    fn serve<'a>(&'a self, request: Import, _: Context<'a, Import>) -> Served<'a, Snapshot> {
        Box::pin(async move {
            let cleanup = self.import(&request.data, &request.choices, request.generation).await?;
            Ok(self.snapshot(cleanup).await?.into())
        })
    }
}

// Store errors contain only fixed operator guidance, never input or keyring diagnostics.
impl From<SecretStoreError> for Status {
    fn from(error: SecretStoreError) -> Self {
        match error {
            SecretStoreError::Invalid(message) => Self::invalid_argument(message),
            SecretStoreError::Conflict => Self::aborted(error.to_string()),
            SecretStoreError::Unavailable => Self::unavailable(error.to_string()),
            SecretStoreError::Storage(_) => Self::unavailable("Secret catalog storage is unavailable"),
            SecretStoreError::Persistence => Self::failed_precondition(error.to_string()),
        }
    }
}

pub(crate) async fn register(runtime: &RuntimeHandle) -> crate::runtime::Result<()> {
    let store: Arc<SecretStore> = runtime.secret_store();
    let mut dependencies = harmos::Dependencies::new();
    dependencies.insert(Host::<Lookup>::new(store.clone()))?;
    dependencies.insert(Host::<List>::new(store.clone()))?;
    dependencies.insert(Host::<Put>::new(store.clone()))?;
    dependencies.insert(Host::<Remove>::new(store.clone()))?;
    dependencies.insert(Host::<Export>::new(store.clone()))?;
    dependencies.insert(Host::<Preview>::new(store.clone()))?;
    dependencies.insert(Host::<Import>::new(store))?;
    let batch = harmos::RegistrationBatch::new(dependencies)
        .operation::<Lookup>()
        .operation::<List>()
        .operation::<Put>()
        .operation::<Remove>()
        .operation::<Export>()
        .operation::<Preview>()
        .operation::<Import>();
    runtime.operations.register(batch.lane(Lanes::Open)).await?;
    Ok(())
}
