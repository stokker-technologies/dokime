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
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

// Third Party Imports
use harmos::operations::{Context, Operation, OperationManager, streams};

// Local Imports
use crate::{InstalledSidecar, RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type NativePublication = harmos::sidecar::Publish<crate::Dokime, harmos::sidecar::Executable, crate::SidecarNoticeHandler>;

/* === Structs === */

#[derive(Default)]
pub(crate) struct Publications {
    residents: RwLock<BTreeMap<String, InstalledSidecar>>,
}

#[derive(Default)]
pub(crate) struct RecordIngress {
    runtime: RwLock<Option<RuntimeHandle>>,
}

struct Filing {
    ingress: Arc<RecordIngress>,
}

struct Invocation {
    owner: Arc<Publications>,
    native: Option<NativePublication>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Publications {
    pub(crate) fn register(operations: &OperationManager) -> harmos::Result<Arc<Self>> {
        let owner = Arc::new(Self::default());
        operations.register_wire_operation::<Invocation>("dokime.native-publications", crate::app::Lanes::Open, owner.clone())?;
        Ok(owner)
    }

    pub(crate) fn insert(&self, owner: &str, resident: InstalledSidecar) -> harmos::Result<()> {
        let mut residents = self.residents.write().expect("publication registry is healthy");
        if residents.contains_key(owner) {
            return Err(harmos::Error::InvalidInput {
                field: "native publication owner",
                reason: "previous resident has not retired",
            });
        }
        residents.insert(owner.to_owned(), resident);
        Ok(())
    }

    pub(crate) fn remove(&self, owner: &str) {
        self.residents.write().expect("publication registry is healthy").remove(owner);
    }
}

impl harmos::operations::Declares for Invocation {
    type Declaration = streams::Publish;
}

impl Operation for Invocation {
    type Declaration = streams::Publish;
    type Resources = Publications;
    type Endpoints = ();
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;

    async fn initialize<'ctx>(owner: Arc<Publications>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self { owner, native: None })
    }

    async fn run<'ctx>(
        &mut self, context: Self::Context<'ctx>, request: &mut streams::Publish, _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<()>> {
        let scope = context.caller_scope().ok_or(harmos::Error::InvalidInput {
            field: "native publication",
            reason: "original plugin caller is required",
        })?;
        if self.native.is_none() {
            let resident =
                self.owner.residents.read().expect("publication registry is healthy").get(scope.owner()).cloned().ok_or_else(|| {
                    harmos::Error::NotRegistered {
                        domain: "native publication owner",
                        name: scope.owner().to_owned(),
                    }
                })?;
            self.native = Some(NativePublication::initialize(Arc::new(resident), context.clone()).await?);
        }
        // The Original Connection Validates Authority And Retains Receipts.
        self.native.as_mut().expect("native invocation initialized").run(context, request, &mut ()).await
    }

    async fn stop(&mut self) -> harmos::Result<()> {
        if let Some(native) = &mut self.native {
            native.stop().await?;
        }
        Ok(())
    }

    async fn cleanup(&mut self) -> harmos::Result<()> {
        if let Some(native) = &mut self.native {
            native.cleanup().await?;
        }
        Ok(())
    }
}

impl RecordIngress {
    pub(crate) fn register(operations: &OperationManager) -> harmos::Result<Arc<Self>> {
        let ingress = Arc::new(Self::default());
        operations.register_wire_operation::<Filing>("dokime.records", crate::app::Lanes::Open, ingress.clone())?;
        Ok(ingress)
    }

    pub(crate) fn attach(&self, runtime: RuntimeHandle) {
        *self.runtime.write().expect("record ingress is healthy") = Some(runtime);
    }

    pub(crate) fn detach(&self) {
        self.runtime.write().expect("record ingress is healthy").take();
    }

    fn runtime(&self) -> harmos::Result<RuntimeHandle> {
        self.runtime.read().expect("record ingress is healthy").clone().ok_or(harmos::Error::Closing { domain: "record ingress" })
    }
}

impl harmos::operations::Declares for Filing {
    type Declaration = crate::records::RecordPublication;
}

impl Operation for Filing {
    type Declaration = crate::records::RecordPublication;
    type Resources = RecordIngress;
    type Endpoints = ();
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;

    async fn initialize<'ctx>(ingress: Arc<RecordIngress>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self { ingress })
    }

    async fn run<'ctx>(
        &mut self, context: Self::Context<'ctx>, publication: &mut crate::records::RecordPublication, _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<()>> {
        let scope = context.caller_scope().ok_or(harmos::Error::InvalidInput {
            field: "record publication",
            reason: "the original plugin caller is required",
        })?;
        let owner = scope.owner().to_owned();
        let runtime = self.ingress.runtime()?;
        let Some((request, produced_at)) = super::record_publication(&owner, std::mem::take(publication)) else {
            return Err(harmos::Error::InvalidInput {
                field: "record publication",
                reason: "the publication is not a record publish request",
            });
        };
        let record_type = request.record_type.clone();
        runtime.publish_sidecar_record_at(request, &owner, produced_at).await.map(|_| ().into()).map_err(|rejection| {
            tracing::warn!(
                owner,
                record_type = record_type.as_str(),
                detail = ?rejection,
                "refusing a record publication"
            );
            harmos::Error::InvalidInput {
                field: "record publication",
                reason: "the record was refused against the plugin's declared contracts",
            }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
