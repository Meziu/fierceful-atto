// Interactive battling example with random encounters that uses a TUI made with Ratatui.

use fierceful_atto::action::{ChoiceReturn, Target};
use fierceful_atto::battle::{self, EndCondition};
use fierceful_atto::catalogue::actions::DirectAttack;
use fierceful_atto::equipment::Equipment;
use fierceful_atto::member::{Member, MemberIdentifier, Properties, Statistics};
use fierceful_atto::team::Team;

use ratatui::layout::Constraint;
use ratatui::style::Stylize;
// Ratatui imports to make the TUI
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Layout,
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Terminal,
};

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
        let mut sum = self.clone();

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
    // Setup a panic hook to correctly exit the application when using Ratatui.
    init_panic_hook();

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
        //.chain(std::io::stdout())
        ;

    if logger_dispatch.apply().is_err() {
        eprintln!("could not setup logger. log information will not be retrieved")
    }

    // Setup TUI using Ratatui.
    // Enter an alternate screen session and enable raw mode for full control over the term window.
    let mut terminal = init_tui().expect("could not create custom terminal session");

    let picco_stats = Stats::new(100, 15);
    let bacco_stats = Stats::new(150, 12);

    let player_1 = Player::new(String::from("Picco"), picco_stats, Props::from(picco_stats));
    let player_2 = Player::new(String::from("Bacco"), bacco_stats, Props::from(bacco_stats));

    let teams = vec![
        Team::new(String::from("Strong Ones"), vec![player_1]),
        Team::new(String::from("Weak Ones"), vec![player_2]),
    ];

    // The battle must be mutable to make incremental steps (it's currently fully consumed by the system)
    let mut battle = battle::Builder::new(
        teams,
        None,
        Box::new(action_choice),
        EndCondition::LastTeamStanding,
    )
    .build();

    while !battle.is_finished() {
        battle.play_turn();

        terminal
            .draw(|frame| {
                let area = frame.size();

                // Split the screen in two main zones, the battle "view", with the playing members, and the battle's menu for the player to use.
                let screen_split =
                    Layout::vertical([Constraint::Percentage(75), Constraint::Min(8)]);
                let [battle_scene_area, battle_menu_area] = screen_split.areas(area);

                frame.render_widget(
                    Paragraph::new("I'm the battle screen lmao").on_green(),
                    battle_scene_area,
                );

                let menu_split =
                    Layout::horizontal([Constraint::Percentage(40), Constraint::Min(30)]);
                let [characters_list_area, characters_stats_area] =
                    menu_split.areas(battle_menu_area);

                let characters_list_block = Block::bordered()
                    .border_set(ratatui::symbols::border::THICK)
                    .border_style(Style::default().fg(Color::White))
                    .style(Style::default().bg(Color::Rgb(13, 61, 86))) // A nice indigo background for enhanced readability.
                    .title("Statistics");

                let characters_stats_list_area = characters_list_block.inner(characters_stats_area);

                frame.render_widget(
                    Paragraph::new("We should put a list in here")
                        .style(Style::default().bg(Color::Rgb(13, 61, 86))),
                    characters_list_area,
                );

                frame.render_widget(characters_list_block, characters_stats_area);
                frame.render_widget(
                    Paragraph::new("How much health left?"),
                    characters_stats_list_area,
                );
            })
            .expect("could not draw ratatui interface");

        if event::poll(std::time::Duration::from_secs(16)).expect("could not poll terminal events")
        {
            if let event::Event::Key(key) = event::read().expect("could not read terminal events") {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    let _resulting_teams = battle.take_teams();

    // Ratatui exit routine.
    restore_tui().unwrap();
}

fn action_choice(
    team_list: &[Team<Player>],
    hint_performer: Option<MemberIdentifier>,
) -> ChoiceReturn<Player> {
    // It should never be `None` in our example, but lets avoid panicking nontheless.
    let hint_performer = hint_performer.unwrap_or_default();

    let mut target = None;

    for (t_id, t) in team_list.iter().enumerate() {
        if t_id != hint_performer.team_id {
            for (m_id, _) in t.member_list().iter().enumerate() {
                target = Some(MemberIdentifier::new(t_id, m_id));
            }
        }
    }

    let target = match target {
        Some(m_id) => Target::Single(m_id),
        None => Target::None,
    };

    (
        Box::new(DirectAttack),
        Target::Single(hint_performer),
        target,
    )
}

pub fn init_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn init_tui() -> std::io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(std::io::stdout()))
}

pub fn restore_tui() -> std::io::Result<()> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}