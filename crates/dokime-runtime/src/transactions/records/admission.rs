/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::state::PluginOwner;
use crate::state::RuntimeState;
use dokime_domain::model::identity::Identifier;

pub(crate) fn host(state: &RuntimeState, kind: &Identifier) -> Result<(), String> {
    if builtin(kind)
        || state.plugin_owners.iter().any(|(source, owner)| matches!(owner, PluginOwner::Host) && declares(state, source, kind))
    {
        Ok(())
    } else {
        Err(format!("record {kind} requires an explicit host declaration or its supervised plugin publisher"))
    }
}

pub(crate) fn provider(state: &RuntimeState, owner: &str, kind: &Identifier) -> Result<bool, String> {
    if !builtin(kind) && !declares(state, owner, kind) {
        return Err(format!("record {kind} is not declared by {owner}"));
    }
    match state.plugin_owners.get(owner) {
        Some(PluginOwner::Host) => Ok(false),
        Some(PluginOwner::Bundle { .. } | PluginOwner::Embedded { .. }) => Ok(true),
        None => Err(format!("record publisher {owner} has no admitted ownership; re-register its catalog")),
    }
}

pub(crate) fn sidecar(state: &RuntimeState, owner: &str, kind: &Identifier) -> Result<(), String> {
    if provider(state, owner, kind)? { Ok(()) } else { Err(format!("record publisher {owner} is not owned by a plugin bundle")) }
}

fn builtin(kind: &Identifier) -> bool {
    crate::records::builtin_record_types().contains(kind)
}

fn declares(state: &RuntimeState, owner: &str, kind: &Identifier) -> bool {
    state
        .installed_plugins
        .get(owner)
        .into_iter()
        .flat_map(|installation| &installation.record_contracts)
        .chain(state.suites.suites().filter(|suite| suite.source == owner).flat_map(|suite| &suite.record_contracts))
        .any(|contract| &contract.record_type == kind)
}
