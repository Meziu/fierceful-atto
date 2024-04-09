//! Definitions for [`Member`]s, the main actors in a [`Battle`](crate::battle::Battle).

/// Fighting entity of a [`Team`](crate::team::Team).
pub trait Member<S: Statistics, P: Properties>: core::fmt::Debug + Clone + PartialEq + Eq {
    /// Returns this [`Member`]'s name.
    fn name(&self) -> &str;

    /// Returns a reference to this [`Member`]'s statistics.
    fn statistics(&self) -> &S;

    /// Returns a reference to this [`Member`]'s properties.
    fn properties(&self) -> &P;

    /// Returns a mutable reference to this [`Member`]'s properties.
    fn properties_mut(&mut self) -> &mut P;

    // `Properties` and `Statistics` function escalation (to access them directly via `Member`).

    /// Returns this [`Member`]'s current health.
    fn health(&self) -> u64 {
        self.properties().health()
    }

    /// Returns this [`Member`]'s current health.
    fn damage(&mut self, damage: u64) {
        self.properties_mut().damage(damage);
    }
}

/// Unmutable statistics associated with a specific [`Member`].
///
/// A member's intrinsic characteristics should be defined here and never modified.
///
/// # Notes
///
/// Use [`Properties`] to keep track and calculate modifiers on these statistics.
pub trait Statistics: core::fmt::Debug + Clone + PartialEq + Eq {}

/// Properties of a [`Member`] that can change during a match.
///
/// Most commonly, a struct that implements this trait should keep track the current health points and additional multipliers.
pub trait Properties: core::fmt::Debug + Clone + PartialEq + Eq {
    fn health(&self) -> u64;
    fn health_mut(&mut self) -> &mut u64;

    /// Auto-generate a new set of [`Properties`] from some [`Statistics`].
    // TODO: Require From<Statistics>
    /*fn from_stats(statistics: &Statistics) -> Self {
        Self {
            health: statistics.max_health,
        }
    }*/

    /// Subtract the exact amount of health points as the damage from these properties.
    ///
    /// # Notes
    ///
    /// The health subtraction saturates to 0 if the damage exceeds the current health.
    ///
    /// This function should not be reimplemented.
    fn damage(&mut self, damage: u64) {
        *self.health_mut() = self.health().saturating_sub(damage);
    }
}

/// Identifier of a member using the team index and a "relative" member index.
#[non_exhaustive]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberIdentifier {
    pub team_id: usize,
    pub member_id: usize,
}

impl MemberIdentifier {
    /// Create a new [`MemberIdentifier`] using the member's team index and relative index.
    pub fn new(team_id: usize, member_id: usize) -> Self {
        Self { team_id, member_id }
    }

    /// Create a new [`MemberIdentifier`] that reference's to the first team's first member.
    pub fn zeroed() -> Self {
        Self {
            team_id: 0,
            member_id: 0,
        }
    }
}
