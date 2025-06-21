//! Definitions for [`Member`]s, the main performers in a [`Battle`](crate::battle::Battle).

use crate::equipment::Equipment;

/// Fighting entity of a [`Team`](crate::team::Team).
pub trait Member: core::fmt::Debug + Clone + PartialEq + Eq {
    type Statistics: Statistics;
    type Properties: Properties;
    type Equipment: Equipment<Properties = Self::Properties>;

    /// Returns this member's name.
    fn name(&self) -> &str;

    /// Returns a reference to this member's statistics.
    fn statistics(&self) -> &Self::Statistics;

    /// Returns a reference to this member's current properties.
    fn member_properties(&self) -> &Self::Properties;

    /// Returns a mutable reference to this member's current properties.
    fn member_properties_mut(&mut self) -> &mut Self::Properties;

    /// Returns a reference to this member's equipment.
    fn equipment(&self) -> &Self::Equipment;

    /// Returns the final properties after applying equipment bonuses.
    ///
    /// This includes the sum of base properties and equipment-provided bonuses.
    /// This function should not be reimplemented under normal circumstances.
    fn final_properties(&self) -> Self::Properties {
        self.member_properties()
            .sum_properties(&self.equipment().associated_properties())
    }

    /// Returns this member's current health.
    fn health(&self) -> u64 {
        self.member_properties().health()
    }

    /// Inflicts direct damage to this member's health.
    ///
    /// This method automatically logs the damage dealt and remaining health.
    fn damage(&mut self, damage: u64) {
        self.member_properties_mut().damage(damage);
        log::info!(
            "Member {} takes {} damage! Health: {}/{}",
            self.name(),
            damage,
            self.member_properties().health(),
            self.statistics().reference_health(),
        );
    }
}

/// Immutable statistics associated with a specific [`Member`].
///
/// A member's intrinsic characteristics should be defined here and never modified.
/// Use [`Properties`] to track modifiers and current values based on these statistics.
pub trait Statistics: core::fmt::Debug + Clone + PartialEq + Eq {
    /// Returns the reference health value (typically maximum health).
    ///
    /// This is useful for UIs, game logic, and general information display.
    fn reference_health(&self) -> u64;

    /// Returns the base attack value associated with this member.
    ///
    /// This should be the fundamental attack value before any equipment
    /// or temporary modifiers are applied.
    fn base_attack(&self) -> u64;
}

/// Properties of a [`Member`] that can change during a match.
///
/// Most commonly tracks current health, temporary modifiers, and calculated values.
pub trait Properties: core::fmt::Debug + Clone + PartialEq + Eq {
    /// Applies changes from another property object to this one.
    ///
    /// The default implementation uses [`sum_properties`](Self::sum_properties) to calculate the result.
    /// In most cases, override [`sum_properties`](Self::sum_properties) instead of this method.
    fn apply_properties(&mut self, rhs: &Self) {
        *self = self.sum_properties(rhs);
    }

    /// Returns the "sum" of property values with another [`Properties`] object.
    ///
    /// How the "sum" is calculated depends on the property types:
    /// - Scalar values might be added or subtracted
    /// - Multipliers might be multiplied or divided
    /// - Some properties might be ignored or replaced
    ///
    /// The default implementation returns an unmodified clone.
    #[allow(unused_variables)]
    fn sum_properties(&self, rhs: &Self) -> Self {
        self.clone()
    }

    /// Returns the current health value.
    fn health(&self) -> u64;

    /// Returns a mutable reference to the current health value.
    fn health_mut(&mut self) -> &mut u64;

    /// Returns the current attack value after all modifiers.
    ///
    /// This should be the final attack value used for damage calculations,
    /// not the base attack from statistics.
    fn attack(&self) -> u64;

    /// Subtracts damage from current health, saturating at 0.
    fn damage(&mut self, damage: u64) {
        *self.health_mut() = self.health().saturating_sub(damage);
    }
}

/// Identifier of a member using team index and member index within that team.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberIdentifier {
    /// Index of the team this member belongs to.
    pub team_id: usize,
    /// Index of the member within their team.
    pub member_id: usize,
}

impl MemberIdentifier {
    /// Creates a new member identifier.
    pub fn new(team_id: usize, member_id: usize) -> Self {
        Self { team_id, member_id }
    }

    /// Creates a member identifier pointing to the first member of the first team.
    pub fn zeroed() -> Self {
        Self::default()
    }
}
