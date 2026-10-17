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

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) async fn ready<T, O, I>(
    ctx: &dokime::execution::InitializationContext<'_, T, O, I>,
) -> Result<(), crate::error::Error>
where
    T: dokime::execution::Calls<crate::operations::readiness::AskReadiness>,
{
    match ctx
        .call::<crate::operations::readiness::AskReadiness>(())
        .await?
        .stopped
    {
        true => Err(crate::error::Error::Device(obc_runtime::Error::Stopped)),
        false => Ok(()),
    }
}

pub(super) fn decided<R: dokime::records::DeclaredRecord>(
    records: dokime::Result<Vec<R>>,
    accepted: impl Fn(&R) -> bool,
) -> Result<bool, crate::error::Error> {
    records?
        .last()
        .map(accepted)
        .ok_or_else(|| crate::error::Error::MissingEvidence(R::RECORD_TYPE.to_owned()))
}

/* === Private Functions === */

/* === Tests === */
