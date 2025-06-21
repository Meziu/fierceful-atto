use crate::{
    action::{ChoiceCallback, Context},
    member::{Member, MemberIdentifier},
    search::SuggestedPerformerCriteria,
    team::Team,
};

/// Instance of a unique fight between multiple [`Team`]s.
pub struct Battle<M> {
    /// List of all teams involved in the battle.
    team_list: Vec<Team<M>>,
    #[allow(dead_code)]
    startup: Option<StartupInfo>,
    /// Turn system in charge of handling turns and actions of the battle.
    turn_system: TurnSystem,
    /// Current battle state.
    state: State,
    suggested_performer_criteria: SuggestedPerformerCriteria<M>,
    action_choice_callback: ChoiceCallback<M>,
}

pub struct Builder<M> {
    inner: Battle<M>,
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

impl<M: Member> Builder<M> {
    pub fn new(
        team_list: Vec<Team<M>>,
        startup: Option<StartupInfo>,
        action_choice_callback: ChoiceCallback<M>,
        end_condition: EndCondition,
    ) -> Self {
        Self {
            inner: Battle {
                team_list,
                startup,
                turn_system: TurnSystem::new(MemberIdentifier::zeroed(), end_condition),
                state: State::Preparating,
                suggested_performer_criteria: SuggestedPerformerCriteria::CycleAlive,
                action_choice_callback,
            },
        }
    }

    /// Set the criteria used to suggest the performign member.
    ///
    /// # Notes
    ///
    /// By default, [`SuggestedPerformerCriteria::CycleAlive`] is used, as it is the norm for many RPGs.
    pub fn set_suggested_performer_criteria(
        mut self,
        criteria: SuggestedPerformerCriteria<M>,
    ) -> Builder<M> {
        self.inner.suggested_performer_criteria = criteria;

        self
    }

    pub fn build(self) -> Battle<M> {
        self.inner
    }
}

impl<M: Member> Battle<M> {
    /// Runs a [`Battle`] to completion, returning the final state of the battling teams.
    ///
    /// The winner will be declared by the end of this function.
    pub fn run(mut self) -> Vec<Team<M>> {
        log::info!("The battle has started and will run until its conclusion");

        loop {
            self.play_turn();

            if self.is_finished() {
                log::info!(
                    "The battle has concluded after {} turns",
                    self.turn_system.turn_number
                );
                break;
            }
        }

        // Return ending state of the battling teams.
        self.take_teams()
    }

    //TODO: Signal end of battle when returning from `play_turn`.
    /// Runs a [`Battle`] for exactly one turn.
    ///
    /// Nothing will be done if the battle's state indicates it has already completed.
    pub fn play_turn(&mut self) {
        if self.is_finished() {
            return;
        }

        self.state = self.turn_system.play_turn(
            &mut self.team_list,
            &self.action_choice_callback,
            &self.suggested_performer_criteria,
        );
    }

    pub fn teams(&self) -> &[Team<M>] {
        &self.team_list
    }

    /// Unwrap the [`Battle`] instance and return the state of its participants.
    pub fn take_teams(self) -> Vec<Team<M>> {
        self.team_list
    }
}

impl<M> Battle<M> {
    /// Returns whether this [`Battle`] has completed or not.
    pub fn is_finished(&self) -> bool {
        matches!(self.state, State::Finished)
    }

    /// Signal the completion of the [`Battle`] to stop its execution.
    ///
    /// # Notes
    ///
    /// It is necessary to run at least one more turn using [`Battle::play_turn`] for the battle's end to be properly handled.
    pub fn set_completed(&mut self) {
        self.state = State::Finished;
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
    suggested_performer: Option<MemberIdentifier>,
    end_condition: EndCondition,
}

impl TurnSystem {
    pub fn new(starting_member: MemberIdentifier, end_condition: EndCondition) -> Self {
        Self {
            turn_number: 0,
            suggested_performer: Some(starting_member),
            end_condition,
        }
    }
}

// TurnSystem functionality that requires access to teams and members.
impl TurnSystem {
    /// Simulate one turn of the battle.
    ///
    /// # Panics
    ///
    /// The function will panic if the turn counter overflows `u64::MAX` or if teams/members are not found when specified.
    pub fn play_turn<M: Member>(
        &mut self,
        team_list: &mut Vec<Team<M>>,
        action_choice_callback: &ChoiceCallback<M>,
        suggested_performer_criteria: &SuggestedPerformerCriteria<M>,
    ) -> State {
        // Count the new turn
        self.turn_number = match self.turn_number.checked_add(1) {
            Some(t) => t,
            None => {
                log::error!("Turn counter overflowed after {} turns", self.turn_number);

                panic!("turn counter overflowed");
            }
        };

        log::info!("Playing turn number {}.", self.turn_number);

        if let Some(performing_member) = self.suggested_performer {
            // Get the playing team.
            let playing_team = match team_list.get(performing_member.team_id) {
                Some(pt) => pt,
                None => {
                    log::warn!(
                        "Playing team with id {:?} was not found",
                        performing_member.team_id
                    );

                    panic!(
                        "requested team with id {} was not found",
                        performing_member.team_id
                    );
                }
            };

            log::info!("Plays the team \"{}\"", playing_team.name());

            // Get the "active" player of this turn.
            let playing_member = match playing_team.member(performing_member.member_id) {
                Some(pm) => pm,
                None => {
                    log::warn!(
                        "Playing member with id {:?} was not found",
                        performing_member
                    );

                    panic!(
                        "requested member with id {} was not found",
                        performing_member.member_id
                    );
                }
            };

            log::info!("It's the turn of {}", playing_member.name());
        }

        let (mut action, performers, targets) =
            action_choice_callback(team_list, self.suggested_performer);

        // Setup the chosen action
        let context = Context::new(team_list, performers, targets);
        action.act(context);

        // TODO: Programmatically decide when the turn should end (after every player acts? after one player acts?)
        // TODO: Run an "end of turn" custom hook.

        // Check whether the battle should continue or whether it's finished.
        if self.battle_should_end(team_list) {
            return State::Finished;
        }

        // TODO: custom performer finder (does it even make sense with the "everyone can perform" model? maybe just as default behaviour for a more modular system)
        self.suggested_performer =
            self.suggest_next_performer(team_list, suggested_performer_criteria);

        State::InProgress
    }

    /// TODO: Subsitute this with an event based check. Iterating every time is slooooooow.
    /// Returns whether or not the battle should continue.
    fn battle_should_end<M: Member>(&self, team_list: &[Team<M>]) -> bool {
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

    fn suggest_next_performer<M: Member>(
        &mut self,
        team_list: &[Team<M>],
        suggested_performer_criteria: &SuggestedPerformerCriteria<M>,
    ) -> Option<MemberIdentifier> {
        suggested_performer_criteria.search(self.suggested_performer, team_list)
    }
}

/// Defaults to using the first given team and its fist given member as starters of the [`Battle`]`, with a [`LastTeamStanding`](EndCondition::LastTeamStanding) end condition.
impl Default for TurnSystem {
    fn default() -> Self {
        Self::new(MemberIdentifier::zeroed(), EndCondition::LastTeamStanding)
    }
}
