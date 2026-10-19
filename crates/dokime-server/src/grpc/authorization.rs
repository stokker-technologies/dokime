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
use harmos::operations::authorization::{Authorizer, Provider, Required};

// Local Imports
use super::access::{Principal, Role};
use super::{Config, Error};
use dokime_domain::operations::Endpoints;
use dokime_domain::operations::Operations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Authorization {
    local_only: bool,
    requirements: Vec<Requirement>,
}

struct Requirement {
    role: Role,
    local_only: bool,
    required: Required,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Authorization {
    fn new(local_only: bool) -> Result<Self, Error> {
        let mut requirements = Vec::new();
        <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
            requirements.push(Requirement {
                role: entry.access,
                local_only: entry.local_only,
                required: Required {
                    contract: entry.wire.contract()?,
                    // Role Permission Does Not Confer A Request-Dependent Scoped Grant.
                    grants: Vec::new(),
                },
            });
            Ok::<_, Error>(())
        })?;
        requirements.sort_unstable_by(|left, right| left.required.contract.operation().id.cmp(&right.required.contract.operation().id));
        if requirements.windows(2).any(|pair| pair[0].required.contract.operation().id == pair[1].required.contract.operation().id) {
            return Err(Error::Configuration("duplicate declared authorization identity"));
        }
        Ok(Self { local_only, requirements })
    }
}

impl Authorizer for Authorization {
    fn required(&self, context: &CallContext, provider: Provider<'_>) -> harmos::Result<Option<Required>> {
        let Some(principal) = context.get::<Principal>() else {
            return Ok(None);
        };
        // Canonical Product Providers Belong To The Original Dokime Runtime Owner.
        // A Plugin Reusing A Public Operation ID Does Not Inherit Its Role Policy.
        if provider.owner.as_str() != "dokime" {
            let Some(permission) = principal.operation(provider.owner.as_str(), provider.definition.id.as_str()) else {
                return Ok(None);
            };
            return Ok(Some(Required {
                // Discovery And Invocation Share The Original Live Contract And Caller Grants.
                contract: provider.contract.clone(),
                grants: permission.grants.clone(),
            }));
        }
        let Ok(index) =
            self.requirements.binary_search_by(|requirement| requirement.required.contract.operation().id.cmp(&provider.definition.id))
        else {
            // Unmapped Product Operations Remain Refused.
            return Ok(None);
        };
        let requirement = &self.requirements[index];
        if principal.role() < requirement.role
            || (requirement.local_only && !self.local_only)
            || !provider.contract.compatible(&requirement.required.contract)
        {
            return Ok(None);
        }
        Ok(Some(requirement.required.clone()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &dokime_runtime::RuntimeHandle, config: &Config) -> Result<(), Error> {
    config.validate()?;
    let authorizer = Arc::new(Authorization::new(config.local_only())?);
    runtime.operations().register_authorizer(authorizer)?;
    runtime.grpc().register_middleware(Arc::new(harmos::grpc::authentication::Authenticate::new(Arc::new(config.policy.clone()))))?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
