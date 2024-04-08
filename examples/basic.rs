use fierceful_atto::action::{Action, Context, Target};
use fierceful_atto::battle::{Battle, Builder, ChoiceReturn, EndCondition};
use fierceful_atto::team::{Member, MemberIdentifier, Properties, Statistics, Team};

// Example of a simple action that inficts direct damage on targets.
struct BasicAttack;

impl Action for BasicAttack {
    fn act(&mut self, mut context: Context) {
        let mut damage_sum: u64 = 0;

        for p in context.performers() {
            // Calculate the sum of all performers' attacks.
            damage_sum = damage_sum.saturating_add(p.statistics().attack);
        }

        for t in context.targets() {
            // Unleash the combined damage on a single target.
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

fn main() {
    let picco_stats = Statistics::new(100, 15);
    let bacco_stats = Statistics::new(150, 12);

    let player_1 = Member::new(
        String::from("Picco"),
        Properties::from_stats(&picco_stats),
        picco_stats,
    );
    let player_2 = Member::new(
        String::from("Bacco"),
        Properties::from_stats(&bacco_stats),
        bacco_stats,
    );

    let teams = vec![
        Team::new(String::from("Strong Ones"), vec![player_1]),
        Team::new(String::from("Weak Ones"), vec![player_2]),
    ];

    // Output the starting configuration of the battling teams.
    println!("Before battle: {teams:#?}");

    // The battle must be mutable to make incremental steps (it's currently fully consumed by the system)
    let battle: Battle = Builder::new(
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

fn action_choice() -> ChoiceReturn {
    // TODO: Make this an actual choice (or maybe based on the turn?).
    (
        Box::new(BasicAttack),
        Target::Single(MemberIdentifier::new(0, 0)),
        Target::Single(MemberIdentifier::new(1, 0)),
    )
}
