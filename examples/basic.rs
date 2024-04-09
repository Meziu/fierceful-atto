use fierceful_atto::action::{Action, Context, Target};
use fierceful_atto::battle::{self, ChoiceReturn, EndCondition};
use fierceful_atto::member::{Member, MemberIdentifier, Properties, Statistics};
use fierceful_atto::team::Team;

// Example of a simple action that inflicts direct damage on targets.
struct BasicAttack;

impl Action<Player, Stats, Props> for BasicAttack {
    fn act(&mut self, mut context: Context<Player, Stats, Props>) {
        let mut damage_sum: u64 = 0;

        for p in context.performers() {
            // Calculate the sum of all performers' attacks.
            damage_sum = damage_sum.saturating_add(p.statistics().attack);
        }

        for t in context.targets() {
            // Unleash the combined damage on all targets.
            t.damage(damage_sum);

            println!(
                "Member {} takes {} damage! Health: {}/{}",
                t.name(),
                damage_sum,
                t.health(),
                t.statistics().max_health
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    name: String,
    statistics: Stats,
    properties: Props,
}

impl Player {
    pub fn new(name: String, statistics: Stats, properties: Props) -> Self {
        Self {
            name,
            properties,
            statistics,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub max_health: u64,
    pub attack: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Props {
    pub health: u64,
}

impl Stats {
    pub fn new(max_health: u64, attack: u64) -> Self {
        Self { max_health, attack }
    }
}

impl Member<Stats, Props> for Player {
    fn name(&self) -> &str {
        &self.name
    }

    fn properties(&self) -> &Props {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut Props {
        &mut self.properties
    }

    fn statistics(&self) -> &Stats {
        &self.statistics
    }
}

// TODO: Should remove if I want to require `From<S: Statistics>`
impl Props {
    /// Auto-generate a new set of [`Props`] from some [`Stats`].
    pub fn from_stats(statistics: &Stats) -> Self {
        Self {
            health: statistics.max_health,
        }
    }
}

impl Properties for Props {
    fn health(&self) -> u64 {
        self.health
    }

    fn health_mut(&mut self) -> &mut u64 {
        &mut self.health
    }
}

impl Statistics for Stats {}

fn main() {
    let picco_stats = Stats::new(100, 15);
    let bacco_stats = Stats::new(150, 12);

    let player_1 = Player::new(
        String::from("Picco"),
        picco_stats,
        Props::from_stats(&picco_stats),
    );
    let player_2 = Player::new(
        String::from("Bacco"),
        bacco_stats,
        Props::from_stats(&bacco_stats),
    );

    let teams = vec![
        Team::new(String::from("Strong Ones"), vec![player_1]),
        Team::new(String::from("Weak Ones"), vec![player_2]),
    ];

    // Output the starting configuration of the battling teams.
    println!("Before battle: {teams:#?}");

    // The battle must be mutable to make incremental steps (it's currently fully consumed by the system)
    let battle = battle::Builder::new(
        teams,
        None,
        Box::new(action_choice),
        EndCondition::LastTeamStanding,
    )
    .build();

    let resulting_teams = battle.run();

    // Output the starting configuration of the battling teams.
    println!("After battle: {resulting_teams:#?}");
}

fn action_choice() -> ChoiceReturn<Player, Stats, Props> {
    // TODO: Make this an actual choice (or maybe based on the turn?).
    (
        Box::new(BasicAttack),
        Target::Single(MemberIdentifier::new(0, 0)),
        Target::Single(MemberIdentifier::new(1, 0)),
    )
}
