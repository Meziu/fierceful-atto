use fierceful_atto::action::{Action, Context, Target};
use fierceful_atto::battle::{Battle, Builder, ChoiceReturn, EndCondition};
use fierceful_atto::team::{Member, MemberIdentifier, Properties, Statistics, Team};

// Example of a possible action
struct BasicAttack;

impl Action for BasicAttack {
    fn act(&mut self, mut context: Context) {
        let mut damage_sum: u64 = 0;

        for p in context.performers() {
            // Calculate the sum of all performers' attacks
            damage_sum = damage_sum.saturating_add(p.statistics().attack);
        }

        for t in context.targets() {
            // Unleash hell on a poor target
            let curr_props = t.mut_properties();
            curr_props.health = curr_props.health.saturating_sub(damage_sum);

            println!("Member {} takes {} damage!", t.name(), damage_sum);
            println!("Member only has {} health points!", t.health());
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
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
    (
        Box::new(BasicAttack),
        Target::Single(MemberIdentifier::new(0, 0)),
        Target::Single(MemberIdentifier::new(1, 0)),
    )
}
