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
/// Refuses to start a template once the embedded bench stopped taking work.
pub(crate) async fn ready<T, O, I>(ctx: &dokime_runtime::execution::InitializationContext<'_, T, O, I>) -> Result<(), crate::error::Error>
where
    T: dokime_runtime::execution::Calls<crate::operations::readiness::AskReadiness>,
{
    match ctx.call::<crate::operations::readiness::AskReadiness>(()).await?.stopped {
        true => Err(crate::error::Error::Device(obc_runtime::Error::Stopped)),
        false => Ok(()),
    }
}

/* === Private Functions === */

/* === Tests === */
