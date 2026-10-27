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
use tonic::Status;

// Local Imports
use crate::secrets::{Change, SecretStore, SecretStoreError, SecretValue};
use crate::{RuntimeHandle, app::Lanes};
use dokime_domain::model::secrets::transfer::{Archive, Preview as ImportPreview};
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
struct Export {
    store: Arc<SecretStore>,
}
struct Preview {
    store: Arc<SecretStore>,
}
struct Import {
    store: Arc<SecretStore>,
}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Lookup(Lookup),
    List(List),
    Put(Put),
    Remove(Remove),
    Export(Export),
    Preview(Preview),
    Import(Import),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Lookup {
    type Declaration = wire::Lookup;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Lookup, _: &mut ()) -> Result<Response<Value>, Self::Error> {
        let store = Arc::clone(&self.store);
        let identifier = std::mem::take(&mut request.identifier);
        let value = store.get_secret(&identifier).await?;
        let value = value.ok_or(SecretStoreError::Invalid("The named secret is missing or was deleted"))?;
        Ok(Value { value: Some(value.expose().to_owned()) }.into())
    }
}

impl Operation for List {
    type Declaration = wire::List;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, _: &mut wire::List, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let snapshot = store.snapshot(false).await?;
        Ok(snapshot.into())
    }
}

impl Operation for Put {
    type Declaration = wire::Put;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Put, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let change = Change { metadata: request.metadata.clone(), value: request.value.take().map(SecretValue::new) };
        let generation = request.generation;
        let snapshot = {
            let cleanup = store.apply(vec![change], &[], generation).await?;
            store.snapshot(cleanup).await?
        };
        Ok(snapshot.into())
    }
}

impl Operation for Remove {
    type Declaration = wire::Remove;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Remove, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let identifiers = std::mem::take(&mut request.identifiers);
        let generation = request.generation;
        let snapshot = {
            let cleanup = store.apply(Vec::new(), &identifiers, generation).await?;
            store.snapshot(cleanup).await?
        };
        Ok(snapshot.into())
    }
}

impl Operation for Export {
    type Declaration = wire::Export;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Export, _: &mut ()) -> Result<Response<Archive>, Self::Error> {
        let store = Arc::clone(&self.store);
        let identifiers = std::mem::take(&mut request.identifiers);
        let generation = request.generation;
        let archive = store.export(&identifiers, generation).await?;
        Ok(archive.into())
    }
}

impl Operation for Preview {
    type Declaration = wire::Preview;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::Preview, _: &mut (),
    ) -> Result<Response<ImportPreview>, Self::Error> {
        let store = Arc::clone(&self.store);
        let data = std::mem::take(&mut request.data);
        let preview = store.preview(&data).await?;
        Ok(preview.into())
    }
}

impl Operation for Import {
    type Declaration = wire::Import;
    type Resources = SecretStore;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(store: Arc<SecretStore>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { store })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut wire::Import, _: &mut ()) -> Result<Response<Snapshot>, Self::Error> {
        let store = Arc::clone(&self.store);
        let data = std::mem::take(&mut request.data);
        let choices = std::mem::take(&mut request.choices);
        let generation = request.generation;
        let snapshot = {
            let cleanup = store.import(&data, &choices, generation).await?;
            store.snapshot(cleanup).await?
        };
        Ok(snapshot.into())
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

/* === Public Functions === */

pub(crate) fn register(runtime: &RuntimeHandle) -> crate::runtime::Result<()> {
    runtime.operations.register_wire_operations::<Operations, SecretStore>("dokime", Lanes::Open, runtime.secret_store())?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
