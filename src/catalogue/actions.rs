//! Pre-made actions using generic implementation for all needs.

use crate::action::{Action, Context};
use crate::member::{Member, Properties};

/// Simple action that inflicts direct damage on targets.
///
/// # Notes
///
/// Defense and status ailments are NOT taken into consideration when calculating the inflicted damage.
///
/// If multiple members are appointed as performers, their attack will be summed up together.
/// If multiple members are appointed as targets, each will be damaged by the *total* of the summed attack.
pub struct DirectAttack;

impl<M: Member> Action<M> for DirectAttack {
    fn act(&mut self, mut context: Context<M>) {
        let mut damage_sum: u64 = 0;

        for p in context.performers() {
            // Calculate the sum of all performers' attacks.
            // In this case, we use the "final" calculated properties based on member equipped gear and other variables.
            damage_sum = damage_sum.saturating_add(p.final_properties().attack());
        }

        for t in context.targets() {
            // Unleash the combined damage on all targets.
            t.damage(damage_sum);
        }
    }
}
