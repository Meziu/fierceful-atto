use std::ops::SubAssign;

use fierceful_atto::action::Action;
use fierceful_atto::battle::{Battle, BattleBuilder};
use fierceful_atto::team::{Member, Properties, Statistics, Team};

// Example of a possible action
struct BasicAttack;

impl Action for BasicAttack {
    fn act(performers: Vec<&mut Member>, targets: Vec<&mut Member>) {
        let mut damage_sum: u64 = 0;

        for p in performers {
            // Calculate the sum of all performers' attacks
            damage_sum = damage_sum.saturating_add(p.statistics().attack);
        }

        for t in targets {
            // Unleash hell on a poor target
            let curr_props = t.mut_properties();
            curr_props.health = curr_props.health.saturating_sub(damage_sum);
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
    println!("Before battle: {:#?}", teams);

    // The battle must be mutable to make incremental steps (it's currently fully consumed by the system)
    let mut battle: Battle = BattleBuilder::new(teams, None).build();

    let resulting_teams = battle.run();

    // Output the starting configuration of the battling teams.
    println!("After battle: {:#?}", resulting_teams);
}
