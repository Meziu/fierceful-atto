use fierceful_atto::action::{ChoiceReturn, Target};
use fierceful_atto::battle::{self, EndCondition};
use fierceful_atto::equipment::Equipment;
use fierceful_atto::member::{Member, MemberIdentifier, Properties, Statistics};
use fierceful_atto::team::Team;

// We will use the `DirectAttack` type from the prefab catalogue to inflict direct damage on our foes.
// We will also use the `Heal` type for sporadic healing.
use fierceful_atto::catalogue::actions::{DirectAttack, Heal};
use rand::Rng;

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
    pub base_attack: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Props {
    pub health: u64,
    pub attack: u64,
}

pub struct Gear;

impl Stats {
    pub fn new(max_health: u64, base_attack: u64) -> Self {
        Self {
            max_health,
            base_attack,
        }
    }
}

impl Member for Player {
    type Statistics = Stats;
    type Properties = Props;
    type Equipment = Gear;

    fn name(&self) -> &str {
        &self.name
    }

    fn member_properties(&self) -> &Props {
        &self.properties
    }

    fn member_properties_mut(&mut self) -> &mut Props {
        &mut self.properties
    }

    fn statistics(&self) -> &Stats {
        &self.statistics
    }

    fn equipment(&self) -> &Self::Equipment {
        &Gear
    }
}

impl From<Stats> for Props {
    /// Auto-generate a new set of [`Props`] from some [`Stats`].
    fn from(statistics: Stats) -> Self {
        Self {
            health: statistics.max_health,
            attack: statistics.base_attack,
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

    fn attack(&self) -> u64 {
        self.attack
    }

    fn sum_properties(&self, rhs: &Self) -> Self {
        let mut sum = *self; // Props implements `Copy` in this example.

        sum.health = sum.health.saturating_add(rhs.attack);
        sum.attack = sum.attack.saturating_add(rhs.attack);

        sum
    }
}

impl Equipment for Gear {
    type Properties = Props;

    fn associated_properties(&self) -> Self::Properties {
        Props {
            health: 0,
            attack: 0,
        }
    }
}

impl Statistics for Stats {
    fn reference_health(&self) -> u64 {
        self.max_health
    }

    fn base_attack(&self) -> u64 {
        self.base_attack
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
    team_list: &[Team<Player>],
    hint_performer: Option<MemberIdentifier>,
) -> ChoiceReturn<Player> {
    // It should never be `None` in our example, but lets avoid panicking nontheless.
    let hint_performer = hint_performer.unwrap_or_default();

    let mut rng = rand::rng();

    // 30% chance to heal, 70% chance to attack
    if rng.random_bool(0.3) {
        // Healing action - target a teammate or self
        let mut heal_target = None;

        // Find a teammate (including self) who needs healing
        for (t_id, t) in team_list.iter().enumerate() {
            if t_id == hint_performer.team_id {
                for (m_id, member) in t.member_list().iter().enumerate() {
                    if member.health() < member.statistics().reference_health() {
                        heal_target = Some(MemberIdentifier::new(t_id, m_id));
                        break;
                    }
                }
            }
        }

        let target = match heal_target {
            Some(m_id) => Target::Single(m_id),
            // There should always be an available target (the performer), but we use this as a failsafe
            None => Target::Single(hint_performer),
        };

        return (
            Box::new(Heal { amount: 25 }),
            Target::Single(hint_performer),
            target,
        );
    }

    // Attack action- find an enemy
    let mut attack_target = None;

    for (t_id, t) in team_list.iter().enumerate() {
        if t_id != hint_performer.team_id {
            for (m_id, _) in t.member_list().iter().enumerate() {
                attack_target = Some(MemberIdentifier::new(t_id, m_id));
            }
        }
    }

    let target = match attack_target {
        Some(m_id) => Target::Single(m_id),
        None => Target::None,
    };

    (
        Box::new(DirectAttack),
        Target::Single(hint_performer),
        target,
    )
}
