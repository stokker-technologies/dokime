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
pub(crate) async fn ready<T, O, I>(ctx: &dokime_runtime::execution::InitializationContext<'_, T, O, I>) -> Result<(), crate::error::Error>
where
    crate::domain::operations::AskReadiness: dokime_runtime::execution::Called<T>,
{
    match ctx.call::<crate::domain::operations::AskReadiness>(crate::domain::operations::AskReadiness {}).await?.stopped {
        true => Err(crate::error::Error::Device(obc_runtime::Error::Stopped)),
        false => Ok(()),
    }
}

/* === Private Functions === */

/* === Tests === */
