pub trait Action {}

/// Simple representation of the team index + member index of a specific member.
pub struct MemberIdentifier {
    pub team_id: usize,
    pub member_id: usize,
}

pub enum Target {
    /// A single member is affected by the action.
    Single(MemberIdentifier),
    /// A specific choice of members is affected by the action.
    DiscreteMultiple(Vec<MemberIdentifier>),
    /// A complete team is affected by the action.
    FullTeam { team_id: usize },
    /// All members of all teams are affected by the action.
    All,
}
