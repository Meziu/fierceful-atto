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

impl<M: Member> SuggestedPerformerCriteria<M> {
    pub fn search(
        &self,
        current_playing_member: Option<MemberIdentifier>,
        team_list: &[Team<M>],
    ) -> Option<MemberIdentifier> {
        match self {
            Self::None => None,
            Self::Constant(member) => Some(*member),
            Self::CycleAlive => {
                self.cycle_with_condition(current_playing_member, team_list, |_, member| {
                    member.health() > 0
                })
            }
            Self::CycleWith(condition) => {
                self.cycle_with_condition(current_playing_member, team_list, condition)
            }
        }
    }

    fn cycle_with_condition<F>(
        &self,
        current_playing_member: Option<MemberIdentifier>,
        team_list: &[Team<M>],
        condition: F,
    ) -> Option<MemberIdentifier>
    where
        F: Fn(MemberIdentifier, &M) -> bool,
    {
        let current = current_playing_member.unwrap_or_default();

        for (team_id, team) in cycle_from_point_enumerated(team_list, current.team_id) {
            let start_member = if current.team_id == team_id {
                current.member_id + 1
            } else {
                0
            };

            for (member_id, member) in team.member_list().iter().enumerate().skip(start_member) {
                let id = MemberIdentifier::new(team_id, member_id);
                if condition(id, member) {
                    return Some(id);
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
