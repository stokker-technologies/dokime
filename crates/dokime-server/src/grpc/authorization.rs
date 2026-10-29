/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authorize External Calls From Canonical Operation Declarations

use std::sync::Arc;

use dokime::harmos::catalog::{Catalog, Projection};
use dokime::harmos::operations::authorization::{Authorization, Authorizer, Provider};
use dokime::harmos::operations::{CallContext, OperationDeclaration};

use super::Error;
use super::access::{Listeners, Principal};
use dokime::operations::Endpoints;
use dokime::operations::Operations;

const HOST_OWNER: &str = "host";
const PRODUCT_NAMESPACE: &str = "dokime.";

pub(super) struct Policy {
    requirements: Vec<Requirement>,
    schema_settings: dokime::harmos::config::SchemaSettings,
}

struct Requirement {
    local_only: bool,
    contract: dokime::harmos::operations::Contract,
}

impl Policy {
    fn new(settings: &dokime::harmos::config::SchemaSettings) -> Result<Self, Error> {
        let mut requirements = Vec::new();
        <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
            requirements.push(Requirement {
                local_only: entry.local_only,
                contract: entry.wire.contract_with_settings(settings)?,
            });
            Ok::<_, Error>(())
        })?;
        requirements.sort_unstable_by(|left, right| left.contract.operation().id.cmp(&right.contract.operation().id));
        if requirements.windows(2).any(|pair| pair[0].contract.operation().id == pair[1].contract.operation().id) {
            return Err(Error::Configuration("duplicate declared authorization identity"));
        }
        Ok(Self { requirements, schema_settings: *settings })
    }
}

impl Authorizer for Policy {
    fn authorize(&self, context: &CallContext, provider: Provider<'_>) -> dokime::harmos::Result<Option<Authorization>> {
        let Some(principal) = context.get::<Principal>() else {
            return Ok(None);
        };
        // Canonical Product Providers Include The Published Host Catalog; Harmos Owns
        // Every Host Entry As `host`. A Plugin Reusing A Public Operation ID Cannot Inherit Native Trust.
        let operation = provider.definition.id.as_str();
        let product = !provider.plugin
            && provider.owner.as_str() == HOST_OWNER
            && (operation.starts_with(PRODUCT_NAMESPACE) || operation == dokime::harmos::builtins::host::Catalog::ID);
        if !product {
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
        if (requirement.local_only && !principal.is_local())
            || !provider.contract.compatible_with_settings(&requirement.contract, &self.schema_settings)
        {
            return Ok(None);
        }
        Ok(Some(Authorization::Full(provider.contract.clone())))
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, settings: &dokime::harmos::config::SchemaSettings,
) -> Result<(Arc<Listeners>, Arc<Policy>), Error> {
    let listeners = Arc::new(Listeners::new());
    let authorizer = Arc::new(Policy::new(settings)?);
    setup.resource::<Listeners>(listeners.clone());
    setup.resource::<Policy>(authorizer.clone());
    Ok((listeners, authorizer))
}

pub(super) fn activate(runtime: &dokime::RuntimeHandle, listeners: Arc<Listeners>, authorizer: Arc<Policy>) -> Result<(), Error> {
    runtime.operations().register_authorizer(authorizer)?;
    runtime.grpc().register_middleware(Arc::new(dokime::harmos::grpc::authentication::Authenticate::new(listeners)))?;
    Ok(())
}
