/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation policies bound each operation by what it does, not by one default.

use std::time::Duration;

use harmos::operations::OperationDeclaration;

use dokime_operations::plugins::{Install, Remove};

// A slow machine may take minutes to install and reload a debug plugin, so the
// host bounds installation as one transfer and removal,
// which deletes the bundle's files, has the heavy-mutation deadline.
#[test]
fn slow_installs_complete() {
    assert!(Install::POLICY.timeout >= Duration::from_secs(3_600));
    assert!(Install::POLICY.attempt_timeout >= Duration::from_secs(3_600));
    assert_eq!(Remove::POLICY.timeout, Duration::from_secs(120));
}
