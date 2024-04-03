use crate::team::{MemberIdentifier, MemberIter, Team};

/// Action that can be performed by team members that affects a specified target.
///
/// # Notes
///
/// More than one member may be appointed as "action performers".
/// Even members of different teams or whole teams can perform the same action together!
pub trait Action {
    ///
    fn act(&mut self, context: Context<'_>);
}

/// Single or multiple targets being affected by an action.
///
/// It may also refer to the action's performer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A single member is affected by the action.
    Single(MemberIdentifier),
    /// A specific choice of members is affected by the action.
    DiscreteMultiple(Vec<MemberIdentifier>),
    /// A whole team is affected by the action.
    FullTeam { team_id: usize },
    /// All members of all teams are affected by the action.
    All,
}

pub struct Context<'team> {
    team_list: &'team mut Vec<Team>,
    performers: Target,
    targets: Target,
}

impl<'i, 's: 'i, 'team: 'i> Context<'team> {
    pub fn new(team_list: &'team mut Vec<Team>, performers: Target, targets: Target) -> Self {
        Self {
            team_list,
            performers,
            targets,
        }
    }

    pub fn performers(&'s mut self) -> MemberIter<'i, 's> {
        self.target_iter(self.performers.clone())
    }

    pub fn targets(&'s mut self) -> MemberIter<'i, 's> {
        self.target_iter(self.targets.clone())
    }

    /// Function that iterates over all members targeted.
    fn target_iter(&'s mut self, target: Target) -> MemberIter<'i, 's> {
        match target {
            // Return a `Once` iterator to the single member that is targeted.
            // TODO: just return None if the member is not found, like the other iterators.
            Target::Single(id) => Box::new(std::iter::once(
                self.team_list
                    .get_mut(id.team_id)
                    .expect("could not find target team")
                    .member_mut(id.member_id)
                    .expect("could not find target member"),
            )),
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
            // Return an iterator that iterates over every member of a single team.
            Target::FullTeam { team_id } => Box::new(
                self.team_list
                    .get_mut(team_id)
                    .expect("could not find target team")
                    .member_list_mut()
                    .iter_mut(),
            ),
            // Return an iterator that iterates over every member of every team. It's pretty simple with `flat_map()`.
            Target::All => Box::new(
                self.team_list
                    .iter_mut()
                    .flat_map(|t| t.member_list_mut().iter_mut()),
            ),
        }
    }
}
