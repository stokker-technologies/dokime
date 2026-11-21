/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Effect Journal: Control Inverses Kept in the Plugin's Durable State Value

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports

// Local Imports
use crate::domain::controls::ControlState;
use dokime::operations::Context;
use dokime::{Error, Message, Result};
use harmos::plugin::StateHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Revision of the Plugin's Durable State Value.
pub struct Saved<R> {
    pub value: Effects,
    pub revision: R,
}

/// The Outstanding Effects, Read From and Saved To One Durable State Value.
pub struct Journal<'a, S> {
    store: &'a S,
}

/// One Outstanding Effect: the Target a Control Mutated and the State That Restores It.
#[derive(Clone, Message)]
#[dokime(package = "host")]
struct Effect {
    target: String,
    inverse: Option<ControlState>,
}

/// The Durable Journal Value, Outstanding Effects in Recording Order.
#[derive(Clone, Message)]
#[dokime(package = "host")]
pub struct Effects {
    effects: Vec<Effect>,
}

/* === Enums === */

/* === Traits === */

/// The Plugin's Single Revisioned Value; a Conflict Returns False and Requires a Fresh Read.
pub trait Store: Sync {
    type Revision: Send;

    fn load(&self) -> impl Future<Output = Result<Saved<Self::Revision>>>;
    fn save(&self, value: Effects, expected: Self::Revision) -> impl Future<Output = Result<bool>>;
}

/* === Implementations === */

impl<'a, S: Store> Journal<'a, S> {
    pub fn new(store: &'a S) -> Self {
        Self { store }
    }

    /// Verify the Durable Value Before Any Native Probe or Mutation.
    pub async fn check(&self) -> Result<()> {
        self.read().await.map(drop)
    }

    async fn read(&self) -> Result<(Effects, S::Revision)> {
        let saved = self.store.load().await?;
        Ok((saved.value, saved.revision))
    }
}

impl<S: Store> crate::controls::Journal for Journal<'_, S> {
    async fn check(&self) -> Result<()> {
        Journal::check(self).await
    }

    async fn record(&self, target: &str, inverse: &ControlState) -> Result<bool> {
        loop {
            let (mut effects, revision) = self.read().await?;
            if effects.effects.iter().any(|effect| effect.target == target) {
                return Ok(false);
            }
            effects.effects.push(Effect {
                target: target.into(),
                inverse: Some(inverse.clone()),
            });
            if self.store.save(effects, revision).await? {
                return Ok(true);
            }
        }
    }

    async fn clear(&self, target: &str) -> Result<()> {
        loop {
            let (mut effects, revision) = self.read().await?;
            let count = effects.effects.len();
            effects.effects.retain(|effect| effect.target != target);
            if effects.effects.len() == count {
                return Ok(());
            }
            if self.store.save(effects, revision).await? {
                return Ok(());
            }
        }
    }

    async fn outstanding(&self) -> Result<Vec<(String, ControlState)>> {
        let (effects, _) = self.read().await?;
        effects
            .effects
            .into_iter()
            .map(|effect| {
                let inverse = effect.inverse.ok_or_else(|| {
                    Error::Invalid(format!("Read Effect Journal: Restore {} by Hand; Its Inverse Is Missing", effect.target))
                })?;
                Ok((effect.target, inverse))
            })
            .collect()
    }
}

/// Keep Control Inverses in the Host's Durable Plugin Value, Using Its Observed Revision.
impl Store for Context {
    type Revision = StateHandle;

    async fn load(&self) -> Result<Saved<StateHandle>> {
        let state = self.state()?;
        let effects = state.get::<Effects>().await?;
        Ok(Saved { value: effects, revision: state })
    }

    async fn save(&self, value: Effects, expected: StateHandle) -> Result<bool> {
        match expected.put(&value).await {
            Ok(_) => Ok(true),
            Err(harmos::Error::Failure(failure)) if failure.code == "state_revision_conflict" => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
