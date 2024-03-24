use crate::{
    action::{Action, ChoiceCallback, Context, MemberIdentifier, Target, TargetIter},
    team::{self, Member, Team},
};

use std::{cell::RefCell, rc::Rc};

/// Instance of a unique fight between multiple [`Team`]s.
pub struct Battle {
    /// List of all teams involved in the battle.
    team_list: Rc<Vec<Team>>,
    startup: Option<StartupInfo>,
    /// Turn system in charge of handling turns and actions of the battle.
    ///
    /// When [`None`], it means the battle has yet to start.
    turn_system: Option<TurnSystem>,
    /// Current battle state.
    state: State,
    action_choice_callback: Box<ChoiceCallback>,
}

pub struct BattleBuilder {
    inner: Battle,
}

/// Current state of a [`Battle`].
pub enum State {
    /// The battle has yet to start.
    Preparating,
    InProgress,
    Finished,
}

impl BattleBuilder {
    pub fn new(
        team_list: Vec<Team>,
        startup: Option<StartupInfo>,
        action_choice_callback: Box<ChoiceCallback>,
    ) -> Self {
        Self {
            inner: Battle {
                team_list: Rc::new(team_list),
                startup,
                turn_system: None,
                state: State::Preparating,
                action_choice_callback: action_choice_callback,
            },
        }
    }

    pub fn build(mut self) -> Battle {
        self.inner.turn_system = Some(TurnSystem::default());
        self.inner
    }
}

impl Battle {
    /// Runs a [`Battle`] to completion, returning the final state of the battling teams.
    ///
    /// The winner will be declared by the end of this function.
    pub fn run(mut self) -> Vec<Team> {
        let turn_system = self
            .turn_system
            .as_mut()
            .expect("no turn system was initialized");

        loop {
            self.state =
                turn_system.play_turn(self.team_list.clone(), &self.action_choice_callback);

            if let State::Finished = self.state {
                break;
            }
        }

        // Return ending state of the battling teams.
        Rc::into_inner(self.team_list)
            .expect("found multiple references to what should be the unique final result")
    }
}

/// Information needed to start a new [`Battle`].
///
/// Here can be stored all sorts of specific infos, like the first team/player that has to play etc.
pub struct StartupInfo {}

/// Handler of the turn-based combat.
///
/// Stores information about the turn cycle and the current playing member.
pub struct TurnSystem {
    turn_number: u64,
    playing_member: MemberIdentifier,
}

impl TurnSystem {
    pub fn new(starting_team: usize, starting_member: usize) -> Self {
        Self {
            turn_number: 0,
            playing_member: MemberIdentifier {
                team_id: starting_team,
                member_id: starting_member,
            },
        }
    }

    pub fn play_turn(
        &mut self,
        mut team_list: Rc<Vec<Team>>,
        action_choice_callback: &Box<ChoiceCallback>,
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

        let (action, performers, targets) = action_choice_callback();

        // TODO: Make an action to substitute the autodamange functionality
        // playing_member.autodamage(15);

        // Setup the chosen action
        let context = Context::new(team_list.clone(), performers, targets);
        action.act(context);

        match self.find_next_player(team_list.clone()) {
            Some(m) => {
                self.playing_member = m;

                State::InProgress
            }
            None => State::Finished,
        }
    }

    fn find_next_player(&mut self, team_list: Rc<Vec<Team>>) -> Option<MemberIdentifier> {
        for (team_id, team) in
            cycle_from_point_enumerated(team_list.as_slice(), self.playing_member.team_id)
        {
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

/// Defaults to using the first given team and its fist given member as starters of the Battle.
impl Default for TurnSystem {
    fn default() -> Self {
        Self::new(0, 0)
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
