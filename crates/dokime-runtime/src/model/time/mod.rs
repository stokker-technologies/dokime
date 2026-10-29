/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub fn now_utc() -> dokime_domain::model::time::Timestamp {
    dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now())
}
