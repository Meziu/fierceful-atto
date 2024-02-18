use crate::team::Team;

/// Instance of a unique fight between multiple [`Team`]s.
pub struct Battle {
    team_list: Vec<Team>,
    startup: Option<StartupInfo>,
}

impl Battle {
    pub fn new(team_list: Vec<Team>, startup: Option<StartupInfo>) -> Self {
        Self {
            team_list,
            startup,
        }
    }

    /// Runs a [`Battle`] to completion.
    /// 
    /// The winner will be declared by the end of this function.
    pub fn run(&mut self) {
        let mut turn_system = TurnSystem::default();

        loop {
            turn_system.play_turn(&mut self.team_list)
        }
    }
}

/// Information needed to start a new [`Battle`].
/// 
/// Here can be stored all sorts of specific infos, like the first team/player that has to play etc.
pub struct StartupInfo {

}

/// Handler of the turn-based combat.
/// 
/// Stores information about the turn cycle and the current playing member.
pub struct TurnSystem {
    turn_number: u64,
    playing_team_id: usize,
    playing_member_id: usize,
}

impl TurnSystem {
    pub fn new(starting_team: usize, starting_member: usize) -> Self {
        Self {
            turn_number: 0,
            playing_team_id: starting_team,
            playing_member_id: starting_member,
        }
    }

    pub fn play_turn(&mut self, team_list: &mut [Team]) {
        // Count the new turn
        self.turn_number = self.turn_number.checked_add(1).expect("turn counter overflowed");

        println!("Playing turn number {}.", self.turn_number);

        // Get the playing team.
        let playing_team = team_list.get_mut(self.playing_team_id).expect("playing team was not found");

        println!("Plays the team \"{}\"", playing_team.name());

        // Get the "active" player of this turn.
        let playing_member = playing_team.member_mut(self.playing_member_id).expect("playing member was not found");

        println!("It's the turn of {}", playing_member.name());

        self.find_next_player(team_list);
    }

    fn find_next_player(&mut self, team_list: &[Team]) {
        for (t_id, team) in cycle_from_point_enumerated(team_list, self.playing_team_id) {
            for (m_id, member) in cycle_from_point_enumerated(team.member_list(), team.member_list().len()) {
                if member.health() != 0 {
                    self.playing_team_id = t_id;
                    self.playing_member_id = m_id;
                }
            }
        }
    }
}

/// Defaults to using the first given team and its fist given member as starters of the Battle.
impl Default for TurnSystem {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Create a cyclic operator over a slice starting from a point and ending at the one before it.
fn cycle_from_point_enumerated<T>(slice: &[T], start_pos: usize) -> impl Iterator<Item = (usize, &T)> {
    slice.iter().enumerate().cycle().skip(start_pos).take(slice.len())
}
