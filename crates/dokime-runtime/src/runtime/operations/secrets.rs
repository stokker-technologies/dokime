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
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, Response};

// Local Imports
use crate::secrets::{Change, SecretStore, SecretStoreError, SecretValue};
use crate::{RuntimeHandle, app::Lanes};
use dokime_domain::{
    model::secrets::catalog::{Snapshot, Value},
    operations::secrets as wire,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Lookup {
    store: Arc<SecretStore>,
}
struct List {
    store: Arc<SecretStore>,
}
struct Put {
    store: Arc<SecretStore>,
}
struct Remove {
    store: Arc<SecretStore>,
}
struct Adopt {
    store: Arc<SecretStore>,
}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Lookup(Lookup),
    List(List),
    Put(Put),
    Remove(Remove),
    Adopt(Adopt),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Lookup {
    type Declaration = wire::Lookup;
    type Resources = SecretStore;
    type Error = SecretStoreError;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Lookup, _: &mut ()) -> Result<Response<Value>, Self::Error> {
        let store = Arc::clone(&self.store);
        let name = std::mem::take(&mut request.name);
        let value = tokio::task::spawn_blocking(move || store.get_secret(&name)).await.map_err(|_| SecretStoreError::Unavailable)??;
        let value = value.ok_or(SecretStoreError::Invalid("The named secret is missing or was deleted"))?;
        Ok(Value { value: Some(value.expose().to_owned()) }.into())
    }
}

impl Operation for List {
    type Declaration = wire::List;
    type Resources = SecretStore;
    type Error = SecretStoreError;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, _: &mut wire::List, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let snapshot = tokio::task::spawn_blocking(move || store.snapshot(false)).await.map_err(|_| SecretStoreError::Unavailable)?;
        Ok(snapshot.into())
    }
}

impl Operation for Put {
    type Declaration = wire::Put;
    type Resources = SecretStore;
    type Error = SecretStoreError;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Put, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let change = Change { metadata: request.metadata.clone(), value: request.value.take().map(SecretValue::new) };
        let generation = request.generation;
        let snapshot = tokio::task::spawn_blocking(move || {
            let cleanup = store.apply(vec![change], &[], generation)?;
            Ok::<_, SecretStoreError>(store.snapshot(cleanup))
        })
        .await
        .map_err(|_| SecretStoreError::Unavailable)??;
        Ok(snapshot.into())
    }
}

impl Operation for Remove {
    type Declaration = wire::Remove;
    type Resources = SecretStore;
    type Error = SecretStoreError;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Remove, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let names = std::mem::take(&mut request.names);
        let generation = request.generation;
        let snapshot = tokio::task::spawn_blocking(move || {
            let cleanup = store.apply(Vec::new(), &names, generation)?;
            Ok::<_, SecretStoreError>(store.snapshot(cleanup))
        })
        .await
        .map_err(|_| SecretStoreError::Unavailable)??;
        Ok(snapshot.into())
    }
}

impl Operation for Adopt {
    type Declaration = wire::Adopt;
    type Resources = SecretStore;
    type Error = SecretStoreError;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Adopt, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let metadata = request.metadata.clone();
        let id = std::mem::take(&mut request.legacy_id);
        let generation = request.generation;
        let snapshot = tokio::task::spawn_blocking(move || {
            let cleanup = store.adopt(metadata, &id, generation)?;
            Ok::<_, SecretStoreError>(store.snapshot(cleanup))
        })
        .await
        .map_err(|_| SecretStoreError::Unavailable)??;
        Ok(snapshot.into())
    }
}

/* === Public Functions === */

pub(crate) fn register(runtime: &RuntimeHandle) -> crate::runtime::Result<()> {
    runtime.operations.register_wire_operations::<Operations, SecretStore>("dokime", Lanes::Open, runtime.secret_store())?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
