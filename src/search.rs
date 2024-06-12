//! Helper module to search for special conditions in battles and teams.

use crate::member::{Member, MemberIdentifier};
use crate::team::Team;

pub type FilterCriteria<M> = dyn Fn(MemberIdentifier, &M) -> bool;

#[non_exhaustive]
pub enum SuggestedPerformerCriteria<M> {
    /// Suggests no performer every time.
    None,
    /// Suggests the given member ID every time.
    Constant(MemberIdentifier),
    /// If possible, chooses the next "alive" member (`health > 0`) of the currently acting member's team.
    ///
    /// Otherwise, cycles through the teams going forward choosing the first member that is found alive.
    CycleAlive,
    /// If possible, chooses the next member that satisfies the precondition of the currently acting member's team.
    ///
    /// Otherwise, cycles through the teams going forward choosing the first member that is found to satisfy the precondition.
    ///
    /// # Notes
    ///
    /// Use [`CycleAlive`] if all you need is to check whether a member is currently alive.
    CycleWith(Box<FilterCriteria<M>>),
}

// TODO: remove yucky code duplication
impl<M: Member> SuggestedPerformerCriteria<M> {
    pub fn search(
        &self,
        current_playing_member: Option<MemberIdentifier>,
        team_list: &[Team<M>],
    ) -> Option<MemberIdentifier> {
        match self {
            Self::None => return None,
            Self::Constant(member) => return Some(*member),
            Self::CycleAlive => {
                let current_playing_member = current_playing_member.unwrap_or_default();

                for (team_id, team) in
                    cycle_from_point_enumerated(team_list, current_playing_member.team_id)
                {
                    let skip = if current_playing_member.team_id == team_id {
                        current_playing_member.member_id + 1
                    } else {
                        0
                    };

                    for (member_id, member) in team.member_list().iter().enumerate().skip(skip) {
                        if member.health() != 0 {
                            return Some(MemberIdentifier { team_id, member_id });
                        }
                    }
                }
            }
            Self::CycleWith(condition) => {
                let current_playing_member = current_playing_member.unwrap_or_default();

                for (team_id, team) in
                    cycle_from_point_enumerated(team_list, current_playing_member.team_id)
                {
                    let skip = if current_playing_member.team_id == team_id {
                        current_playing_member.member_id + 1
                    } else {
                        0
                    };

                    for (member_id, member) in team.member_list().iter().enumerate().skip(skip) {
                        let id = MemberIdentifier::new(team_id, member_id);

                        if condition(id, member) {
                            return Some(MemberIdentifier { team_id, member_id });
                        }
                    }
                }
            }
        }

        None
    }
}

/// Create a cyclic operator over a slice starting from a point and ending at the one before it.
fn cycle_from_point_enumerated<T>(
    slice: &[T],
    start_pos: usize,
) -> impl Iterator<Item = (usize, &T)> {
    slice
        .iter()
        .enumerate()
        .cycle()
        .skip(start_pos)
        .take(slice.len())
}
