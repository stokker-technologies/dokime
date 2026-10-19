/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authorize External Calls From Canonical Operation Declarations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::catalog::{Catalog, Projection};
use harmos::operations::CallContext;
use harmos::operations::authorization::{Authorization, Authorizer, Provider};

// Local Imports
use super::Error;
use super::access::{Listeners, Principal};
use dokime_domain::operations::Endpoints;
use dokime_domain::operations::Operations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Policy {
    requirements: Vec<Requirement>,
}

struct Requirement {
    local_only: bool,
    contract: harmos::operations::Contract,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Policy {
    fn new() -> Result<Self, Error> {
        let mut requirements = Vec::new();
        <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
            requirements.push(Requirement { local_only: entry.local_only, contract: entry.wire.contract()? });
            Ok::<_, Error>(())
        })?;
        requirements.sort_unstable_by(|left, right| left.contract.operation().id.cmp(&right.contract.operation().id));
        if requirements.windows(2).any(|pair| pair[0].contract.operation().id == pair[1].contract.operation().id) {
            return Err(Error::Configuration("duplicate declared authorization identity"));
        }
        Ok(Self { requirements })
    }
}

impl Authorizer for Policy {
    fn authorize(&self, context: &CallContext, provider: Provider<'_>) -> harmos::Result<Option<Authorization>> {
        let Some(principal) = context.get::<Principal>() else {
            return Ok(None);
        };
        // Canonical Product Providers Belong To The Original Dokime Runtime Owner.
        // A Plugin Reusing A Public Operation ID Cannot Inherit Native Trust.
        if provider.owner.as_str() != "dokime" {
            if principal.is_local() || provider.plugin {
                return Ok(Some(Authorization::Full(provider.contract.clone())));
            }
            return Ok(None);
        }
        let Ok(index) = self.requirements.binary_search_by(|requirement| requirement.contract.operation().id.cmp(&provider.definition.id))
        else {
            // Unmapped Product Operations Remain Refused.
            return Ok(None);
        };
        let requirement = &self.requirements[index];
        if (requirement.local_only && !principal.is_local()) || !provider.contract.compatible(&requirement.contract) {
            return Ok(None);
        }
        Ok(Some(Authorization::Full(provider.contract.clone())))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &dokime_runtime::RuntimeHandle) -> Result<Arc<Listeners>, Error> {
    let authorizer = Arc::new(Policy::new()?);
    runtime.operations().register_authorizer(authorizer)?;
    let listeners = Arc::new(Listeners::default());
    runtime.grpc().register_middleware(Arc::new(harmos::grpc::authentication::Authenticate::new(listeners.clone())))?;
    Ok(listeners)
}

/* === Private Functions === */

/* === Tests === */
