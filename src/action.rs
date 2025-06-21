use crate::member::{Member, MemberIdentifier};
use crate::team::Team;

pub type ChoiceReturn<M> = (Box<dyn Action<M>>, Target, Target);
/// Function type to dynamically decide the next [`Action`] to perform.
pub type ChoiceCallback<M> = Box<dyn Fn(&[Team<M>], Option<MemberIdentifier>) -> ChoiceReturn<M>>;

/// Action that can be performed by team members that affects a specified target.
///
/// # Notes
///
/// More than one member may be appointed as "action performers".
/// Even members of different teams or whole teams can perform the same action together!
pub trait Action<M> {
    /// Action logic performer.
    ///
    /// # Notes
    ///
    /// Depending on the action, you may need to damage the interested targets or modify their status.
    /// You may want to iterate over all performers and targets to retrieve the
    /// necessary data by using [`Context::performers()`] or [`Context::targets()`].
    fn act(&mut self, context: Context<'_, M>);
}

/// Single or multiple targets being affected by an action.
///
/// It may also refer to the action's performer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// No target is affected by the action.
    None,
    /// A single member is affected by the action.
    Single(MemberIdentifier),
    /// A specific choice of members is affected by the action.
    ///
    /// # Notes
    ///
    /// Any duplicate [`MemberIdentifier`] will be considered only once.
    DiscreteMultiple(Vec<MemberIdentifier>),
    /// A whole team is affected by the action.
    FullTeam { team_id: usize },
    /// All members of all teams are affected by the action.
    All,
}

pub struct Context<'team, M> {
    team_list: &'team mut Vec<Team<M>>,
    performers: Target,
    targets: Target,
}

impl<'team, M: Member> Context<'team, M> {
    pub fn new(team_list: &'team mut Vec<Team<M>>, performers: Target, targets: Target) -> Self {
        Self {
            team_list,
            performers,
            targets,
        }
    }

    /// Returns a mutable iterator over all [`Member`](crate::team::Member)s that are flagged as action performers.
    ///
    /// # Notes
    ///
    /// It must not be expected for this iterator to return references in any particular order.
    ///
    /// The result of this function depends on the [`Target`]s passed as input in the [`Context`] struct.
    /// If members are not placed where the [`MemberIdentifier`]s are pointing to, either the wrong member
    /// is going to be returned, or no reference will be returned. Beware of the [`Team`]'s ordering.
    pub fn performers(&mut self) -> Box<dyn Iterator<Item = &mut M> + '_> {
        self.target_iter(self.performers.clone())
    }

    /// Returns a mutable iterator over all [`Member`](crate::team::Member)s that are flagged as action targets.
    ///
    /// # Notes
    ///
    /// It must not be expected for this iterator to return references in any particular order.
    ///
    /// The result of this function depends on the [`Target`]s passed as input in the [`Context`] struct.
    /// If members are not placed where the [`MemberIdentifier`]s are pointing to, either the wrong member
    /// is going to be returned, or no reference will be returned. Beware of the [`Team`]'s ordering.
    pub fn targets(&mut self) -> Box<dyn Iterator<Item = &mut M> + '_> {
        self.target_iter(self.targets.clone())
    }

    /// Function that iterates over all members targeted.
    fn target_iter(&mut self, target: Target) -> Box<dyn Iterator<Item = &mut M> + '_> {
        match target {
            Target::None => Box::new(std::iter::empty()),
            Target::Single(id) => self.get_single_member_iter(id),
            Target::DiscreteMultiple(targets) => self.get_discrete_members_iter(targets),
            Target::FullTeam { team_id } => self.get_team_members_iter(team_id),
            Target::All => Box::new(
                self.team_list
                    .iter_mut()
                    .flat_map(|team| team.member_list_mut().iter_mut()),
            ),
        }
    }

    fn get_single_member_iter(
        &mut self,
        id: MemberIdentifier,
    ) -> Box<dyn Iterator<Item = &mut M> + '_> {
        match self
            .team_list
            .get_mut(id.team_id)
            .and_then(|team| team.member_mut(id.member_id))
        {
            Some(member) => Box::new(std::iter::once(member)),
            None => {
                log::warn!(
                    "Could not find requested member at index {:?}. Returning an empty iterator instead",
                    id
                );
                Box::new(std::iter::empty())
            }
        }
    }

    fn get_discrete_members_iter(
        &mut self,
        targets: Vec<MemberIdentifier>,
    ) -> Box<dyn Iterator<Item = &mut M> + '_> {
        Box::new(
            self.team_list
                .iter_mut()
                .enumerate()
                .flat_map(|(team_id, team)| {
                    std::iter::repeat(team_id).zip(team.member_list_mut().iter_mut().enumerate())
                })
                .filter(move |(team_id, (member_id, _))| {
                    targets.contains(&MemberIdentifier {
                        team_id: *team_id,
                        member_id: *member_id,
                    })
                })
                .map(|(_, (_, member))| member),
        )
    }

    fn get_team_members_iter(&mut self, team_id: usize) -> Box<dyn Iterator<Item = &mut M> + '_> {
        match self.team_list.get_mut(team_id) {
            Some(team) => Box::new(team.member_list_mut().iter_mut()),
            None => {
                log::warn!(
                    "Could not find requested team at index {}. Returning an empty iterator instead",
                    team_id
                );
                Box::new(std::iter::empty())
            }
        }
    }
}
