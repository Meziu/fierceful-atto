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

impl<'i, 's: 'i, 'team: 'i, M: Member> Context<'team, M> {
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
    pub fn performers(&'s mut self) -> Box<dyn Iterator<Item = &'s mut M> + 'i> {
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
    pub fn targets(&'s mut self) -> Box<dyn Iterator<Item = &'s mut M> + 'i> {
        self.target_iter(self.targets.clone())
    }

    /// Function that iterates over all members targeted.
    fn target_iter(&'s mut self, target: Target) -> Box<dyn Iterator<Item = &'s mut M> + 'i> {
        match target {
            // Return an empty iterator if no target was found.
            Target::None => Box::new(std::iter::empty()),
            // Return a `Once` iterator to the single member that is targeted.
            Target::Single(id) => {
                let team = self.team_list.get_mut(id.team_id);

                if let Some(t) = team {
                    if let Some(m) = t.member_mut(id.member_id) {
                        return Box::new(std::iter::once(m));
                    }
                }

                log::warn!("Could not find requested member at index {:?}. Returning an empty iterator instead", id);

                // If the member wasn't found, return an empty iterator.
                Box::new(std::iter::empty())
            }
            // Return a filtered iterator over all individual targets.
            Target::DiscreteMultiple(targets) => Box::new(
                self.team_list
                    .iter_mut()
                    // Enumerating helps filter which teams/members we are actually targeting.
                    .enumerate()
                    .flat_map(|(i, t)| {
                        // `Repeat` is used to return the same `team_id` number to each member of a team.
                        // We also re-enumerate over the members to keep track of the `member_id`
                        std::iter::repeat(i).zip(t.member_list_mut().iter_mut().enumerate())
                    })
                    .filter(move |(t_id, (m_id, _))| {
                        targets.contains(&MemberIdentifier {
                            team_id: *t_id,
                            member_id: *m_id,
                        })
                    })
                    .map(|(_, (_, m))| m),
            ),
            // Returns an iterator that iterates over every member of a single team.
            Target::FullTeam { team_id } => match self.team_list.get_mut(team_id) {
                Some(team) => Box::new(team.member_list_mut().iter_mut()),
                None => {
                    log::warn!("Could not find requested team at index {}. Returning an empty iterator instead", team_id);

                    Box::new(std::iter::empty())
                }
            },
            // Returns an iterator that iterates over every member of every team. It's pretty simple with `flat_map()`.
            Target::All => Box::new(
                self.team_list
                    .iter_mut()
                    .flat_map(|t| t.member_list_mut().iter_mut()),
            ),
        }
    }
}
