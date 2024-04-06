use crate::{
    action::{Action, Context, Target},
    team::{MemberIdentifier, Team},
};

pub type ChoiceReturn = (Box<dyn Action>, Target, Target);
pub type ChoiceCallback = dyn Fn() -> ChoiceReturn;

/// Instance of a unique fight between multiple [`Team`]s.
pub struct Battle {
    /// List of all teams involved in the battle.
    team_list: Vec<Team>,
    #[allow(dead_code)]
    startup: Option<StartupInfo>,
    /// Turn system in charge of handling turns and actions of the battle.
    turn_system: TurnSystem,
    /// Current battle state.
    state: State,
    action_choice_callback: Box<ChoiceCallback>,
}

pub struct Builder {
    inner: Battle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCondition {
    /// End the battle if only one member is "alive" in the whole battle.
    ///
    /// # Notes
    ///
    /// It is up to the developer to ensure a way to resolve stalemates if more members of the same team remain alive.
    LastMemberStanding,
    /// End the battle if only one battling team has any "alive" members.
    ///
    /// This is the most common end condition for team-to-team fighting.
    LastTeamStanding,
}

/// Current state of a [`Battle`].
pub enum State {
    /// The battle has yet to start.
    Preparating,
    InProgress,
    Finished,
}

impl Builder {
    pub fn new(
        team_list: Vec<Team>,
        startup: Option<StartupInfo>,
        action_choice_callback: Box<ChoiceCallback>,
        end_condition: EndCondition,
    ) -> Self {
        Self {
            inner: Battle {
                team_list,
                startup,
                turn_system: TurnSystem::new(MemberIdentifier::zeroed(), end_condition),
                state: State::Preparating,
                action_choice_callback,
            },
        }
    }

    pub fn build(self) -> Battle {
        self.inner
    }
}

impl Battle {
    /// Runs a [`Battle`] to completion, returning the final state of the battling teams.
    ///
    /// The winner will be declared by the end of this function.
    ///
    /// # Panics
    ///
    /// The function will panic if no turn system was internally initialized, or if any conditions for which [`TurnSystem::play_turn()`] would panic occour.
    pub fn run(mut self) -> Vec<Team> {
        loop {
            self.state = self
                .turn_system
                .play_turn(&mut self.team_list, &self.action_choice_callback);

            if let State::Finished = self.state {
                break;
            }
        }

        // Return ending state of the battling teams.
        self.team_list
    }
}

/// Information needed to start a new [`Battle`].
///
/// Here can be stored all sorts of specific infos, like the first team/player that has to play etc.
#[non_exhaustive]
pub struct StartupInfo {}

/// Handler of the turn-based combat.
///
/// Stores information about the turn cycle and the current playing member.
pub struct TurnSystem {
    turn_number: u64,
    playing_member: MemberIdentifier,
    end_condition: EndCondition,
}

impl TurnSystem {
    pub fn new(starting_member: MemberIdentifier, end_condition: EndCondition) -> Self {
        Self {
            turn_number: 0,
            playing_member: starting_member,
            end_condition,
        }
    }

    /// Simulate one turn of the battle.
    ///
    /// # Panics
    ///
    /// The function will panic if the turn counter overflows `u64::MAX` or if teams/members are not found when specified.
    pub fn play_turn(
        &mut self,
        team_list: &mut Vec<Team>,
        action_choice_callback: &ChoiceCallback,
    ) -> State {
        // Count the new turn
        self.turn_number = self
            .turn_number
            .checked_add(1)
            .expect("turn counter overflowed");

        println!("Playing turn number {}.", self.turn_number);

        // Get the playing team.
        let playing_team = team_list
            .get(self.playing_member.team_id)
            .expect("playing team was not found");

        println!("Plays the team \"{}\"", playing_team.name());

        // Get the "active" player of this turn.
        let playing_member = playing_team
            .member(self.playing_member.member_id)
            .expect("playing member was not found");

        println!("It's the turn of {}", playing_member.name());

        let (mut action, performers, targets) = action_choice_callback();

        // TODO: Make an action to substitute the autodamage functionality
        // playing_member.autodamage(15);

        // Setup the chosen action
        let context = Context::new(team_list, performers, targets);
        action.act(context);

        // TODO: Run an "end of turn" custom hook.

        // Check whether the battle should continue or whether it's finished.
        if self.check_end_condition(team_list) {
            return State::Finished;
        }

        // TODO: custom performer finder (does it even make sense with the "everyone" can perform model? maybe just as default behaviour for a more modular system)
        match self.find_next_player(team_list) {
            Some(m) => {
                self.playing_member = m;

                State::InProgress
            }
            None => State::Finished,
        }
    }

    /// Returns whether or not the battle should continue.
    fn check_end_condition(&self, team_list: &[Team]) -> bool {
        match self.end_condition {
            EndCondition::LastMemberStanding => {
                let mut members_alive: u8 = 0;

                for t in team_list {
                    for m in t.member_list() {
                        if m.health() > 0 {
                            members_alive = members_alive.saturating_add(1);

                            // We don't need to check every member. Once we find 2 alive, we know the battle should continue.
                            if members_alive >= 2 {
                                return false;
                            }
                        }
                    }
                }

                true
            }
            EndCondition::LastTeamStanding => {
                let mut teams_alive: u8 = 0;

                for t in team_list {
                    for m in t.member_list() {
                        if m.health() > 0 {
                            teams_alive = teams_alive.saturating_add(1);

                            // We don't need to check every team. Once we find 2 alive, we know the battle should continue.
                            if teams_alive >= 2 {
                                return false;
                            }

                            // If even one member is alive, we know the state of this team (and can go check the next one).
                            break;
                        }
                    }
                }

                true
            }
        }
    }

    fn find_next_player(&mut self, team_list: &[Team]) -> Option<MemberIdentifier> {
        for (team_id, team) in cycle_from_point_enumerated(team_list, self.playing_member.team_id) {
            for (member_id, member) in
                cycle_from_point_enumerated(team.member_list(), team.member_list().len())
            {
                if member.health() != 0 {
                    return Some(MemberIdentifier { team_id, member_id });
                }
            }
        }

        None
    }
}

/// Defaults to using the first given team and its fist given member as starters of the [`Battle`]`, with a [`LastTeamStanding`](EndCondition::LastTeamStanding) end condition.
impl Default for TurnSystem {
    fn default() -> Self {
        Self::new(MemberIdentifier::zeroed(), EndCondition::LastTeamStanding)
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
