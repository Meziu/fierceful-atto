//! Module including definitions for [`Member`]s, the main actors in a [`Battle`](crate::battle::Battle).

/// Generic iterator over mutable [`Member`] references.
///
/// This is mainly used in [`Action`](crate::action::Action)s when iterating over action targets and performers.
pub type MemberIter<'a, 't> = Box<dyn Iterator<Item = &'t mut Member> + 'a>;

/// Fighting entity of a [`Team`](crate::team::Team).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    name: String,
    properties: Properties,
    statistics: Statistics,
}

/// Unmutable statistics related to a specific [`Member`].
///
/// A member's intrinsic characteristics should be defined here and never modified.
///
/// # Notes
///
/// Use [`Properties`] to keep track and calculate modifiers on these statistics.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Statistics {
    pub max_health: u64,
    pub attack: u64,
}

/// Properties of a [`Member`] that can change during a match.
///
/// Most commonly, here must be implemented the current health points and additional multipliers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Properties {
    pub health: u64,
}

/// Identifier of a member using the team index and a "relative" member index.
#[non_exhaustive]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberIdentifier {
    pub team_id: usize,
    pub member_id: usize,
}

impl Member {
    /// Create a new Member using a set of associated [`Statistics`] and [`Properties`].
    /// 
    /// # Notes
    /// 
    /// It's suggested to autogenerate the [`Properties`] from the [`Statistics`] if possible.
    pub fn new(name: String, statistics: Statistics, properties: Properties) -> Self {
        Self {
            name,
            properties,
            statistics,
        }
    }

    /// Returns this member's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this member's current health.
    pub fn health(&self) -> u64 {
        self.properties.health
    }

    /// Returns a reference to this member's statistics.
    pub fn statistics(&self) -> &Statistics {
        &self.statistics
    }

    /// Returns a mutable reference to this member's statistics.
    pub fn mut_statistics(&mut self) -> &mut Statistics {
        &mut self.statistics
    }

    /// Returns a reference to this member's properties.
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    /// Returns a mutable reference to this member's properties.
    pub fn mut_properties(&mut self) -> &mut Properties {
        &mut self.properties
    }

    /// Damages the [`Member`]'s health, subtracting the exact amount of health points passed.
    ///
    /// # Notes
    ///
    /// The health subtraction saturates to 0 if the damage exceeds the current health.
    pub fn damage(&mut self, damage: u64) {
        self.properties.health = self.properties.health.saturating_sub(damage);
    }
}

impl Statistics {
    /// Create a new set of [`Statistics`] to associate to some [`Member`].
    pub fn new(max_health: u64, attack: u64) -> Self {
        Self { max_health, attack }
    }
}

impl Properties {
    /// Auto-generate a new set of [`Properties`] from some [`Statistics`].
    pub fn from_stats(statistics: &Statistics) -> Self {
        Self {
            health: statistics.max_health,
        }
    }
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
