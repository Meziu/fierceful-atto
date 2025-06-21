//! Pre-made actions for common battle scenarios.

use crate::action::{Action, Context};
use crate::member::{Member, Properties, Statistics};

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

        let target_count = context.targets().fold(0usize, |count, target| {
            target.damage(total_damage);
            count + 1
        });

        log::info!(
            "Direct attack hits {} target(s) for {} damage each",
            target_count,
            total_damage
        );
    }
}

/// Healing action that restores health to targets.
///
/// Each target receives the specified healing amount.
pub struct Heal {
    pub amount: u64,
}

impl<M: Member> Action<M> for Heal {
    fn act(&mut self, mut context: Context<M>) {
        context.targets().for_each(|target| {
            let max_health = target.statistics().reference_health();
            let current_health = target.health();
            let new_health = current_health.saturating_add(self.amount).min(max_health);
            *target.member_properties_mut().health_mut() = new_health;

            log::info!(
                "Member {} healed for {} points! Health: {}/{}",
                target.name(),
                new_health - current_health,
                new_health,
                max_health
            );
        });
    }
}

/// Skip turn action that does nothing.
///
/// Useful for passing turns or when no action is desired.
pub struct Skip;

impl<M: Member> Action<M> for Skip {
    fn act(&mut self, mut context: Context<M>) {
        for performer in context.performers() {
            log::info!("Member {} skips their turn", performer.name());
        }
    }
}

/// Sacrifice action where performers damage themselves to heal targets.
///
/// Performers lose the specified amount of health.
/// Targets gain health equal to the total sacrifice amount.
pub struct Sacrifice {
    pub amount: u64,
}

impl<M: Member> Action<M> for Sacrifice {
    fn act(&mut self, mut context: Context<M>) {
        let total_sacrifice = context
            .performers()
            .map(|performer| {
                performer.damage(self.amount);
                log::info!(
                    "Member {} sacrifices {} health!",
                    performer.name(),
                    self.amount
                );
                self.amount
            })
            .fold(0u64, |acc, amount| acc.saturating_add(amount));

        context.targets().for_each(|target| {
            let max_health = target.statistics().reference_health();
            let current_health = target.health();
            let new_health = current_health
                .saturating_add(total_sacrifice)
                .min(max_health);
            *target.member_properties_mut().health_mut() = new_health;

            log::info!(
                "Member {} receives {} healing from sacrifice! Health: {}/{}",
                target.name(),
                new_health - current_health,
                new_health,
                max_health
            );
        });
    }
}

/// Drain action that steals health from targets and gives it to performers.
///
/// Targets lose the specified amount of health.
/// Performers gain the health that was drained.
pub struct Drain {
    pub amount: u64,
}

impl<M: Member> Action<M> for Drain {
    fn act(&mut self, mut context: Context<M>) {
        let total_drained = context
            .targets()
            .map(|target| {
                let initial_health = target.health();
                target.damage(self.amount);
                initial_health - target.health()
            })
            .fold(0u64, |acc, drained| acc.saturating_add(drained));

        context.performers().for_each(|performer| {
            let max_health = performer.statistics().reference_health();
            let current_health = performer.health();
            let new_health = current_health.saturating_add(total_drained).min(max_health);
            *performer.member_properties_mut().health_mut() = new_health;

            log::info!(
                "Member {} drains {} health! Health: {}/{}",
                performer.name(),
                new_health - current_health,
                new_health,
                max_health
            );
        });
    }
}

/// Self-destruct action that damages all targets while knocking out the performer.
///
/// Performer's health is set to 0.
/// Targets receive damage equal to performer's current health.
pub struct SelfDestruct;

impl<M: Member> Action<M> for SelfDestruct {
    fn act(&mut self, mut context: Context<M>) {
        let total_damage = context
            .performers()
            .map(|performer| {
                let current_health = performer.health();
                *performer.member_properties_mut().health_mut() = 0;
                log::info!(
                    "Member {} self-destructs for {} damage!",
                    performer.name(),
                    current_health
                );
                current_health
            })
            .fold(0u64, |acc, damage| acc.saturating_add(damage));

        for target in context.targets() {
            target.damage(total_damage);
        }
    }
}

/// Area attack that distributes damage among multiple targets.
///
/// Total damage is split evenly among all targets.
/// More efficient against groups but weaker against single targets.
pub struct AreaAttack;

impl<M: Member> Action<M> for AreaAttack {
    fn act(&mut self, mut context: Context<M>) {
        let total_attack = context
            .performers()
            .map(|performer| performer.final_properties().attack())
            .fold(0u64, |acc, attack| acc.saturating_add(attack));

        let targets: Vec<_> = context.targets().collect();
        let target_count = targets.len() as u64;

        if target_count > 0 {
            let damage_per_target = total_attack / target_count;
            for target in targets {
                target.damage(damage_per_target);
            }

            log::info!(
                "Area attack hits {} targets for {} damage each",
                target_count,
                damage_per_target
            );
        }
    }
}
