mod board;
mod io;
mod error;
mod strategy;
mod display;
mod index;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Parser};
use itertools::Itertools;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::{DefaultTerminal, Frame};
use ratatui::style::{Style, Color, Modifier, Stylize};
use ratatui::widgets::{Block, Gauge, HighlightSpacing, List, ListState};
use ratatui::text::Span;
use crossterm::event::{KeyCode};
use rayon::prelude::*;

use crate::board::{SudokuBoard};
use crate::display::render_sudoku_board;
use crate::io::load_games;
use crate::strategy::{solve, SolvedGame, Strategy};
use crate::error::SudokuError;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name="FILE", required=true, num_args=1..)]
    games: Vec<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(Debug, Default)]
struct GameViewState {
    game_idx: usize,
    step: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum GameSelectionViewState {
    #[default]
    Selection,
    Filter,
}

#[derive(Debug)]
enum AppScreen {
    GameSelectionView(GameSelectionViewState),
    GameView(GameViewState),
}

impl core::default::Default for AppScreen {
    fn default() -> Self {
        AppScreen::GameSelectionView(Default::default())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum FilterStatus {
    #[default]
    Neutral,
    Include,
    Exclude,
}

impl FilterStatus {
    fn next(&self) -> Self {
        match &self {
            FilterStatus::Neutral => FilterStatus::Include,
            FilterStatus::Include => FilterStatus::Exclude,
            FilterStatus::Exclude => FilterStatus::Neutral,
        }
    }

    fn advance(&mut self) {
        *self = self.next()
    }
}

#[derive(Debug)]
struct App {
    screen: AppScreen,

    games: Vec<SolvedGame>,
    game_list: Vec<(String, Style)>,
    game_selection_list_state: ListState,

    filtered_strategies: Vec<FilterStatus>,
    filtered_strategies_list_state: ListState,
}

impl App {
    fn init(terminal: &mut DefaultTerminal) -> color_eyre::Result<Option<Self>> {
        let cli = Cli::parse();
        let games = {
            let mut games = Vec::<SudokuBoard>::new();
            for fpath in &cli.games {
                let reader = std::fs::File::open(fpath)?;
                let reader = std::io::BufReader::new(reader);
                games.extend(load_games(Box::new(reader)))
            }
            games
        };

        if games.is_empty() { return Err(SudokuError::NoBoardFound.into()); }

        // solve the games
        let it = rayon_progress::ProgressAdaptor::new(games);
        let result = Arc::new(Mutex::default());
        let progress = it.items_processed();
        let total = it.len();
        rayon::spawn({
            let result = result.clone();
            move || {
                let games: Vec::<_> = it.filter_map(|b| solve(b).ok() ).collect();
                *result.lock().unwrap() = Some(games);
            }
        });

        let games: Vec<_> = loop {
            // check if job is done
            if let Ok(v) = result.try_lock() && let Some(games) = &*v {
                break games.clone()
            }

            // 
            terminal.draw(|frame| {
                let area = frame.area().centered(
                    Constraint::Max(80),
                    Constraint::Max(6),
                );
                let [gauge_area, text_area] = Layout::vertical([
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .areas(area);

                frame.render_widget("Press 'q' or ESC to exit...", text_area);

                let gauge = Gauge::default()
                    .block(Block::bordered().title(format!("Solving games {}/{}", progress.get(), total)))
                    .gauge_style(Style::new().white().on_black().italic())
                    .ratio(progress.get() as f64 / total as f64);
                frame.render_widget(gauge, gauge_area);
            })?;

            if crossterm::event::poll(std::time::Duration::from_secs(0))?
                && let Some(key) = crossterm::event::read()?.as_key_press_event()
                && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) 
            {
                return Ok(None)
            }
        };

        if games.is_empty() {
            return Err(SudokuError::NoBoardFound.into())
        }

        let game_list: Vec<_> = games.iter().enumerate().map(|(idx, (boards, steps))| {
            let solved = boards.last().map(|b| b.is_solved()).unwrap_or(false);
            let strats: Vec<_> = steps.iter().map(|(strat,_)| strat)
                .unique().sorted()
                .skip(1) // skip AllPrimary strategy
                .collect();

            if solved {
                (format!("Game {} - solved {:?}", idx+1, strats), Style::default().blue())
            } else {
                (format!("Game {} - unsolved {:?}", idx+1, strats), Style::default().red())
            }
        }).collect();

        Ok(Some(App{
            screen: Default::default(),
            games,
            game_list,
            game_selection_list_state: ListState::default().with_selected(Some(0)),

            filtered_strategies: vec![FilterStatus::Neutral; Strategy::domain()[1..].len()],
            filtered_strategies_list_state: ListState::default().with_selected(Some(0)),
        }))
    }


    fn run(mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if !self.handle_input()? { break }
        }
        Ok(())
    }

    fn handle_input(&mut self) -> color_eyre::Result<bool> {
        if let Some(key) = crossterm::event::read()?.as_key_press_event() {
            match &mut self.screen {
                AppScreen::GameView(view_state) => {
                    let (states, _) = &self.games[view_state.game_idx];
                    
                    #[allow(clippy::collapsible_match)]
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.screen = AppScreen::default()
                        },
                        KeyCode::Char('j') | KeyCode::Down => {
                            let n = states.len();
                            if view_state.step + 1 < 2*n - 1 { view_state.step += 1; }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if view_state.step > 0 { view_state.step -= 1; }
                        },
                        _ => {},
                    }
                },
                AppScreen::GameSelectionView(GameSelectionViewState::Selection) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('f') => self.screen = AppScreen::GameSelectionView(GameSelectionViewState::Filter),
                    KeyCode::Char('j') | KeyCode::Down => self.game_selection_list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.game_selection_list_state.select_previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(idx) = self.game_selection_list_state.selected() {
                            self.screen = AppScreen::GameView(GameViewState { game_idx: idx, step: 0 })
                        }
                    },
                    _ => {},
                },
                AppScreen::GameSelectionView(GameSelectionViewState::Filter) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('f') => self.screen = AppScreen::GameSelectionView(GameSelectionViewState::Selection),
                    KeyCode::Char('j') | KeyCode::Down => self.filtered_strategies_list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.filtered_strategies_list_state.select_previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(idx) = self.filtered_strategies_list_state.selected() {
                            self.filtered_strategies[idx].advance();
                            self.game_selection_list_state.select(None);
                        }
                    },
                    _ => {},
                },
            }
        }

        Ok(true)
    }

    fn render(&mut self, frame: &mut Frame) {
        match &self.screen {
            AppScreen::GameSelectionView(state) => self.render_game_list(frame, &state.clone()),
            AppScreen::GameView(state) => self.render_game(frame, state),
        }
    }

    fn render_game_list(&mut self, frame: &mut Frame, state: &GameSelectionViewState) {
        let [top_area, shortcut_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ]).areas(frame.area());
        let [filter_area, games_area] = Layout::horizontal([
            Constraint::Max(20),
            Constraint::Min(30),
        ]).areas(top_area);

        let filter_list = List::new(Strategy::domain()[1..].iter().zip(&self.filtered_strategies)
            .map(|(strategy, status)|{
                match status {
                    FilterStatus::Neutral => Span::from(format!("  {:?}", strategy)),
                    FilterStatus::Include => Span::styled(format!("✓ {:?}", strategy), Style::default().green()),
                    FilterStatus::Exclude => Span::styled(format!("X {:?}", strategy), Style::default().red()),
                }
            }))
            .highlight_symbol("> ")
            .block(Block::bordered().title("Filter"))
            ;

        let include_strategies: Vec<_> = Strategy::domain()[1..].iter().zip(&self.filtered_strategies).filter_map(|(strat, status)|{
            if *status == FilterStatus::Include { Some(*strat) } else { None }
        }).collect();

        let exclude_strategies: Vec<_> = Strategy::domain()[1..].iter().zip(&self.filtered_strategies).filter_map(|(strat, status)|{
            if *status == FilterStatus::Exclude { Some(*strat) } else { None }
        }).collect();

        let games_list = List::new(self.game_list.iter().zip(&self.games)
                .filter_map(|((text, style), (_game, steps))| {
                    if steps.iter().any(|(strat,_)|{ exclude_strategies.contains(strat) })
                       || include_strategies.iter().any(|strat|{ steps.iter().all(|(s, _)| s != strat) })
                    { 
                        None 
                    } else {
                        Some(Span::styled(text, *style))
                    }
                })
            )
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::bordered().title("Games"))
            ;

        match state {
            GameSelectionViewState::Filter => {
                let filter_list = filter_list.style(Color::White).highlight_style(Modifier::REVERSED);
                frame.render_stateful_widget(filter_list, filter_area, &mut self.filtered_strategies_list_state); 

                let games_list = games_list.dim();
                frame.render_stateful_widget(games_list, games_area, &mut self.game_selection_list_state); 
            },
            GameSelectionViewState::Selection => {
                let filter_list = filter_list.dim();
                frame.render_stateful_widget(filter_list, filter_area, &mut self.filtered_strategies_list_state); 

                let games_list = games_list.style(Color::White).highlight_style(Modifier::REVERSED);
                frame.render_stateful_widget(games_list, games_area, &mut self.game_selection_list_state); 
            },
        }

        frame.render_widget("[q], [ESC]: exit | [↑], [k]: move up | [↓], [j]: move down | [Space], [Enter]: select | [f], [Tab]: Filter ⇄ Games", shortcut_area);
    }

    fn render_game(&self, frame: &mut Frame, state: &GameViewState) {
        let (boards, steps) = &self.games[state.game_idx];
        let n = 2*boards.len() - 1;
        assert!(n > 0);

        let area = Rect::new((frame.area().width - 73)/2, 0, 73, frame.area().height);
        let [header_area, strat_area, board_area] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
            ])  
            .areas(area);

        let gauge = Gauge::default()
            .block(Block::bordered().title(format!("Step {}/{}", state.step + 1, n)))
            .gauge_style(Style::new().white().on_black().italic())
            .ratio(state.step as f64 / (n - 1) as f64)
        ;

        frame.render_widget(gauge, header_area);

        let board_idx = state.step / 2;
        let board = &boards[board_idx];
            
        if state.step.is_multiple_of(2) {
            let message = if board_idx + 1 == boards.len() {
                if board.is_solved() {
                    "Solved!"
                } else {
                    "Sudoku is unsolvable with current strategies..."
                }
            } else {
                "Current board"
            };
            frame.render_widget(message, strat_area);
            render_sudoku_board(frame, board_area, board, &[], &[]);
        } else {
            let (strat, highlights) = &steps[board_idx];
            let message = format!("Apply {:?}", strat);
            frame.render_widget(message, strat_area);

            let next_board = &boards[board_idx + 1];
            let diff = next_board.diff(board);
            render_sudoku_board(frame, board_area, next_board, highlights, &diff);
        }
    }

}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(|term| {
        if let Some(app) = App::init(term)? {
            app.run(term)?
        }
        Ok(())
    })
}
