//! Pre-made actions for common battle scenarios.

use crate::action::{Action, Context};
use crate::member::{Member, Properties};

/// Simple direct damage attack that ignores defense and status effects.
///
/// Multiple performers have their attacks summed together.
/// Each target receives the full combined damage.
pub struct DirectAttack;

impl<M: Member> Action<M> for DirectAttack {
    fn act(&mut self, mut context: Context<M>) {
        let total_damage = context
            .performers()
            .map(|performer| performer.final_properties().attack())
            .fold(0u64, |acc, attack| acc.saturating_add(attack));

        for target in context.targets() {
            target.damage(total_damage);
        }
    }
}
