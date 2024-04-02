use crate::team::{Member, Team};

pub type ChoiceReturn = (Box<dyn Action>, Target, Target);
pub type ChoiceCallback = dyn Fn() -> ChoiceReturn;
pub type MemberIter<'a, 't> = Box<dyn Iterator<Item = &'t mut Member> + 'a>;

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

/// Simple representation of the team index + member index of a specific member.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberIdentifier {
    pub team_id: usize,
    pub member_id: usize,
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
            Target::DiscreteMultiple(targets) => Box::new(
                self.team_list
                    .iter_mut()
                    .enumerate()
                    .flat_map(|(i, t)| {
                        std::iter::repeat(i).zip(t.member_list_mut().iter_mut().enumerate())
                    })
                    .filter(move |(t_id, (m_id, _))| {
                        return targets.contains(&MemberIdentifier {
                            team_id: *t_id,
                            member_id: *m_id,
                        });
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

impl MemberIdentifier {
    pub fn new(team_id: usize, member_id: usize) -> Self {
        Self { team_id, member_id }
    }

    pub fn zeroed() -> Self {
        Self {
            team_id: 0,
            member_id: 0,
        }
    }
}
/*
impl<'team> TargetIter<'team> {
    pub fn new(target: Target, team_list: &'team mut Vec<Team>) -> Self {
        Self {
            target,
            team_list,
            counter: 0,
        }
    }
}

impl<'team> Iterator for TargetIter<'team> {
    type Item = &'team mut Member;

    fn next(&mut self) -> Option<Self::Item> {
        return match &self.target {
            Target::Single(id) => {
                // If we haven't yielded this single target yet, do so and save the change in the counter.
                if self.counter == 0 {
                    self.counter = 1;

                    return Some(
                        self.team_list
                            .get_mut(id.team_id)
                            .expect("could not find specified team")
                            .member_mut(id.member_id)
                            .expect("could not find specified member"),
                    );
                }

                None
            }
            Target::DiscreteMultiple(ids) => {
                // Get the target to yield from the counter and return it.
                let target = ids.get(self.counter)?;

                self.counter = self
                    .counter
                    .checked_add(1)
                    .expect("usize overflow when fetching next target");

                Some(
                    self.team_list
                        .get_mut(target.team_id)
                        .expect("could not find specified team")
                        .member_mut(target.member_id)
                        .expect("could not find specified member"),
                )
            }
            Target::FullTeam { team_id } => {
                let target = self
                    .team_list
                    .get_mut(*team_id)
                    .expect("could not find specified team")
                    .member_mut(self.counter)
                    .expect("could not find specified member");

                self.counter = self
                    .counter
                    .checked_add(1)
                    .expect("usize overflow when fetching next target");

                Some(target)
            }
            Target::All => {
                let mut counter: usize = 0;

                for team in self.team_list.iter_mut() {
                    for m in team.member_list_mut() {
                        if self.counter == counter {
                            self.counter = counter
                                .checked_add(1)
                                .expect("usize overflow when fetching next target");

                            return Some(m);
                        }
                    }

                    counter = counter
                        .checked_add(1)
                        .expect("usize overflow when fetching next target");
                }
                None
            }
        };
    }

    // Exact size calculation for the ExactSizeIterator trait.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = match &self.target {
            Target::Single(_) => 1,
            Target::DiscreteMultiple(v) => v.len(),
            Target::FullTeam { team_id } => self
                .team_list
                .get(*team_id)
                .expect("could not find specified team")
                .member_list()
                .len(),
            Target::All => {
                let mut counter: usize = 0;

                for t in self.team_list.iter() {
                    counter = counter.saturating_add(t.member_list().len());
                }

                counter
            }
        };

        // Only the "remaining" items must be returned
        let size = size.saturating_sub(self.counter);

        (size, Some(size))
    }
}

impl ExactSizeIterator for TargetIter<'_> {}
*/
