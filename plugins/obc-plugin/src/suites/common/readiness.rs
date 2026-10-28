/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) async fn ready<T, O, I>(ctx: &dokime::execution::InitializationContext<'_, T, O, I>) -> Result<(), crate::error::Error>
where
    crate::domain::operations::AskReadiness: dokime::execution::Called<T>,
{
    match ctx.call::<crate::domain::operations::AskReadiness>(crate::domain::operations::AskReadiness {}).await?.stopped {
        true => Err(crate::error::Error::Device(obc_runtime::Error::Stopped)),
        false => Ok(()),
    }
}
