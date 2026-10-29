/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CLI Story Configuration Uses The Production Bootstrap

use std::path::Path;

pub fn resolve(root: &Path) -> dokime::config::Config {
    dokime::config::ApplicationOptions {
        application_root: Some(root.to_owned()),
        ..Default::default()
    }
    .resolve()
    .unwrap()
}
