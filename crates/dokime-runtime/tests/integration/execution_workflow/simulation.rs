/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(feature = "simulation")]

use dokime_runtime::SessionId;
use dokime_runtime::app;
use dokime_runtime::state::RuntimeState;

pub(crate) fn declaration_boots_runtime() {
    let origin = RuntimeState::new(dokime_runtime::model::identity::new_id::<SessionId>());

    app::simulate(origin, 42).expect("Dokime's catalogs are simulable");
}
