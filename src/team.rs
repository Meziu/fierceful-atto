/// Coalition made up of multiple fighting [`Member`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    name: String,
    member_list: Vec<Member>,
}

impl Team {
    pub fn new(name: String, member_list: Vec<Member>) -> Self {
        Self { name, member_list }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn member_list(&self) -> &[Member] {
        &self.member_list
    }

    pub fn member_list_mut(&mut self) -> &mut [Member] {
        &mut self.member_list
    }

    pub fn member(&self, member_id: usize) -> Option<&Member> {
        self.member_list.get(member_id)
    }

    pub fn member_mut(&mut self, member_id: usize) -> Option<&mut Member> {
        self.member_list.get_mut(member_id)
    }
}

/// Fighting entity of a [`Team`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    name: String,
    properties: Properties,
    statistics: Statistics,
}

impl Member {
    pub fn new(name: String, properties: Properties, statistics: Statistics) -> Self {
        Self {
            name,
            properties,
            statistics,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn health(&self) -> u64 {
        self.properties.health
    }

    pub fn statistics(&self) -> &Statistics {
        &self.statistics
    }

    pub fn mut_statistics(&mut self) -> &mut Statistics {
        &mut self.statistics
    }

    pub fn mut_properties(&mut self) -> &mut Properties {
        &mut self.properties
    }

    /// Testing only!
    pub fn autodamage(&mut self, damage: u64) {
        self.properties.health = self.properties.health.saturating_sub(damage);
    }
}

/// Properties of a [`Member`] that can change during a match.
///
/// Most commonly, here must be implemented the current health points and additional multipliers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Properties {
    pub health: u64,
}

impl Properties {
    pub fn from_stats(statistics: &Statistics) -> Self {
        Self {
            health: statistics.max_health,
        }
    }
}

/// Unmutable statistics related to a specific member.
///
/// Here can go stuff like "max health points" or "attack".
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Statistics {
    pub max_health: u64,
    pub attack: u64,
}

impl Statistics {
    pub fn new(max_health: u64, attack: u64) -> Self {
        Self { max_health, attack }
    }
}
