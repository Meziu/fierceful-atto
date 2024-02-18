use fierceful_atto::battle::Battle;
use fierceful_atto::team::{Member, Properties, Statistics, Team};

fn main() {
    let global_stats = Statistics::new(100, 15);

    let player_1 = Member::new(String::from("Picco"), Properties::from_stats(&global_stats), global_stats);
    let player_2 = Member::new(String::from("Bacco"), Properties::from_stats(&global_stats), global_stats);

    let teams = vec![Team::new(String::from("Strong Ones"), vec![player_1]), Team::new(String::from("Weak Ones"), vec![player_2])];
    let mut battle = Battle::new(teams, None);

    battle.run();
}
