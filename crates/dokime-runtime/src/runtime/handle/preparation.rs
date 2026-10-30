/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prepare Native Runtime Resources

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use super::BootParts;
use crate::app::{Dokime, Lanes, LiveRuntime};
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Prepared {
    pub(super) provider_invocations: Arc<crate::runtime::operations::provider::ProviderInvocations>,
    pub(super) execution_sessions: Arc<crate::domains::execution::run::Sessions>,
    pub(super) step_builtins: Arc<crate::domains::files::StepBuiltins>,
    pub(super) record_ingress: Arc<crate::runtime::sidecars::core::RecordIngress>,
}

/* === Enums === */

#[harmos::catalog]
enum Catalog {
    Catalog(harmos::builtins::host::Catalog),
}

/* === Traits === */

/* === Implementations === */

impl Prepared {
    pub(crate) fn configure(setup: &mut harmos::Setup<Dokime>, parts: &BootParts, live: &LiveRuntime) -> Result<Self> {
        let engine = std::ops::Deref::deref(&parts.storage).erased();
        setup.resource::<Option<Arc<_>>>(Some(Arc::new(engine)));
        setup.operations::<harmos::builtins::Storage>(Lanes::Open);
        setup.operations::<Catalog>(Lanes::Open);
        let provider_invocations = Arc::new(crate::runtime::operations::provider::ProviderInvocations::default());
        crate::runtime::operations::provider::prepare(setup, provider_invocations.clone())?;
        let execution_sessions = Arc::new(crate::domains::execution::run::Sessions::default());
        crate::domains::execution::run::host::prepare(setup, &execution_sessions)?;
        crate::plot::builtins::prepare(setup);
        let step_builtins = crate::domains::files::prepare(setup)?;
        crate::runtime::sidecars::core::prepare_publications(setup)?;
        let record_ingress = crate::runtime::sidecars::core::RecordIngress::prepare(setup)?;
        crate::runtime::operations::runtime::prepare(setup, live);
        crate::runtime::operations::secrets::prepare(setup, Arc::new(parts.secrets.clone()), live)?;
        crate::runtime::operations::reports::prepare(setup, live);
        Ok(Self {
            provider_invocations,
            execution_sessions,
            step_builtins,
            record_ingress,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
