use crate::team::{Member, Team};

use std::{cell::RefCell, rc::Rc};

pub type ChoiceCallback = dyn Fn() -> (Box<dyn Action>, Target, Target);

/// Action that can be performed by team members that affects a specified target.
///
/// # Notes
///
/// More than one member may be appointed as "action performers".
/// Even members of different teams or whole teams can perform the same action together!
pub trait Action {
    ///
    fn act(&self, context: Context);
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

pub struct Context {
    team_list: Rc<Vec<Team>>,
    performers: Target,
    targets: Target,
}

enum TargetIterCounter {
    /// No targets yielded yet.
    None,
    /// The "single" target has been yielded
    DoneSingle,
    /// Every target (in the original order) was yielded up until this index.
    DoneUntil(usize),
}

pub struct TargetIter<'team> {
    target: Target,
    team_list: &'team [Team],
    iterator_counter: TargetIterCounter,
}

impl Context {
    pub fn new(team_list: Rc<Vec<Team>>, performers: Target, targets: Target) -> Self {
        Self {
            team_list,
            performers,
            targets,
        }
    }

    pub fn performers(&self) -> TargetIter {
        TargetIter::new(self.performers.clone(), &self.team_list)
    }

    pub fn targets(&self) -> TargetIter {
        TargetIter::new(self.targets.clone(), &self.team_list)
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

impl<'team> TargetIter<'team> {
    pub fn new(target: Target, team_list: &'team [Team]) -> Self {
        Self {
            target,
            team_list,
            iterator_counter: TargetIterCounter::None,
        }
    }
}

impl<'team> Iterator for TargetIter<'team> {
    type Item = &'team mut Member;

    fn next(&mut self) -> Option<Self::Item> {
        return match &self.target {
            Target::Single(id) => {
                // If we haven't yielded this single target yet, do so and save the change in the counter.
                if let TargetIterCounter::None = self.iterator_counter {
                    self.iterator_counter = TargetIterCounter::DoneSingle;

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
                // If no targets have been yielded, set to yield the first
                if let TargetIterCounter::None = self.iterator_counter {
                    self.iterator_counter = TargetIterCounter::DoneUntil(0);
                }

                // Get the target to yield from the counter and return it.
                if let TargetIterCounter::DoneUntil(next) = self.iterator_counter {
                    let target = ids.get(next)?;

                    self.iterator_counter = TargetIterCounter::DoneUntil(
                        next.checked_add(1)
                            .expect("usize overflow when fetching next target"),
                    );

                    return Some(
                        self.team_list
                            .get_mut(target.team_id)
                            .expect("could not find specified team")
                            .member_mut(target.member_id)
                            .expect("could not find specified member"),
                    );
                }

                None
            }
            Target::FullTeam { team_id } => {
                // If no targets have been yielded, set to yield the first
                if let TargetIterCounter::None = self.iterator_counter {
                    self.iterator_counter = TargetIterCounter::DoneUntil(0);
                }

                // Get the target to yield from the counter and return it.
                if let TargetIterCounter::DoneUntil(next) = self.iterator_counter {
                    let target = self
                        .team_list
                        .get_mut(*team_id)
                        .expect("could not find specified team")
                        .member_mut(next)
                        .expect("could not find specified member");

                    self.iterator_counter = TargetIterCounter::DoneUntil(
                        next.checked_add(1)
                            .expect("usize overflow when fetching next target"),
                    );

                    return Some(target);
                }

                None
            }
            Target::All => {
                let mut counter: usize = 0;

                for team in self.team_list {
                    for m in team.member_list_mut() {
                        if let TargetIterCounter::DoneUntil(c) = self.iterator_counter {
                            if c == counter {
                                self.iterator_counter = TargetIterCounter::DoneUntil(
                                    counter
                                        .checked_add(1)
                                        .expect("usize overflow when fetching next target"),
                                );
                                return Some(m);
                            }
                        }

                        counter = counter
                            .checked_add(1)
                            .expect("usize overflow when fetching next target");
                    }
                }

                None
            }
        };
    }
}
