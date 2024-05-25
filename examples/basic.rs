use fierceful_atto::action::{Action, ChoiceReturn, Context, Target};
use fierceful_atto::battle::{self, EndCondition};
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

impl From<Stats> for Props {
    /// Auto-generate a new set of [`Props`] from some [`Stats`].
    fn from(statistics: Stats) -> Self {
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

impl Statistics for Stats {
    fn reference_health(&self) -> u64 {
        self.max_health
    }
}

fn main() {
    // Setup logging using fern.
    let logger_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());

    if logger_dispatch.apply().is_err() {
        eprintln!("could not setup logger. log information will not be retrieved")
    }

    let picco_stats = Stats::new(100, 15);
    let bacco_stats = Stats::new(150, 12);

    let player_1 = Player::new(String::from("Picco"), picco_stats, Props::from(picco_stats));
    let player_2 = Player::new(String::from("Bacco"), bacco_stats, Props::from(bacco_stats));

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

fn action_choice(
    team_list: &[Team<Player, Stats, Props>],
    hint_performer: Option<MemberIdentifier>,
) -> ChoiceReturn<Player, Stats, Props> {
    // It should never be `None` in our example, but lets avoid panicking nontheless.
    let hint_performer = hint_performer.unwrap_or_default();

    let mut target = MemberIdentifier::zeroed();

    for (t_id, t) in team_list.iter().enumerate() {
        if t_id != hint_performer.team_id {
            for (m_id, _) in t.member_list().iter().enumerate() {
                target = MemberIdentifier::new(t_id, m_id);
            }
        }
    }

    (
        Box::new(BasicAttack),
        Target::Single(hint_performer),
        Target::Single(target),
    )
}
