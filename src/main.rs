mod board;
mod io;
mod error;
mod strategy;
mod display;
mod index;
mod graph;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Parser};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::{DefaultTerminal, Frame};
use ratatui::style::{Style, Color, Modifier, Stylize};
use ratatui::widgets::{Block, Gauge, HighlightSpacing, List, ListState};
use ratatui::text::Span;
use crossterm::event::{KeyCode};
use rayon::prelude::*;
use strum::{EnumCount, IntoEnumIterator};

use crate::board::{SudokuBoard};
use crate::display::render_sudoku_board;
use crate::strategy::{solve, SolvedGame, Strategy};
use crate::error::SudokuError;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name="FILE", required=true, num_args=1..)]
    games: Vec<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    sequential: bool,

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ExitRequest {
    Continue,
    Quit,
}

#[derive(Debug)]
struct App {
    screen: AppScreen,
    games: Vec<SolvedGame>,

    games_titles: Vec<(String, Style)>,
    filtered_games_indices: Vec<usize>,
    filtered_games_list_state: ListState,
    filtered_strategies: Vec<(Strategy, usize, FilterStatus)>,
    filtered_strategies_list_state: ListState,
}

impl App {
    fn new(games: Vec<SolvedGame>) -> Self {
        let games_titles: Vec<_> = games.iter().enumerate().map(|(idx, game)| {
            let solved = game.is_solved();
            let strats: Vec<_> = game.strategies.iter()
                .collect();

            if solved {
                (format!("Game {} - solved {:?}", idx+1, strats), Style::default().blue())
            } else {
                (format!("Game {} - unsolved {:?}", idx+1, strats), Style::default().red())
            }
        }).collect();

        // only consider for filtering strategies that occur and that
        // do not appear in every game
        let filtered_strategies: Vec<_> = games.iter().fold(vec![0; Strategy::COUNT], |mut freq, game| {
                for s in &game.strategies {
                    freq[*s as usize] += 1;
                }
                freq
            }).into_iter()
            .zip(Strategy::iter()).filter_map(|(f, s)| {
                if f != games.len() && f > 0 { Some((s, f, FilterStatus::Neutral)) } else { None }
            }).collect();

        let n = games_titles.len();
        App {
            screen: Default::default(),
            games,
            games_titles,
            filtered_games_indices: Vec::from_iter(0..n),
            filtered_games_list_state: ListState::default(),

            filtered_strategies,
            filtered_strategies_list_state: ListState::default().with_selected(Some(0)),
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if self.handle_input()? == ExitRequest::Quit { break }
        }
        Ok(())
    }

    fn update_filtered_list(&mut self) {
        let include_strategies: Vec<_> = self.filtered_strategies.iter().filter_map(|(strat, _, status)|{
            if *status == FilterStatus::Include { Some(*strat) } else { None }
        }).collect();

        let exclude_strategies: Vec<_> = self.filtered_strategies.iter().filter_map(|(strat, _, status)|{
            if *status == FilterStatus::Exclude { Some(*strat) } else { None }
        }).collect();

        self.filtered_games_indices = self.games.iter().enumerate()
            .filter_map(|(idx, game)|{
                if game.strategies.iter().any(|s| exclude_strategies.contains(s))
                    || include_strategies.iter().any(|s| game.strategies.iter().all(|gs| gs != s))
                {
                    None
                } else {
                    Some(idx)
                }
            }).collect();
    }

    fn handle_input(&mut self) -> color_eyre::Result<ExitRequest> {
        if let Some(key) = crossterm::event::read()?.as_key_press_event() {
            match &mut self.screen {
                AppScreen::GameView(view_state) => {
                    let game = &self.games[view_state.game_idx];
                    let n = game.boards.len();

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.screen = AppScreen::default()
                        },
                        KeyCode::Char('j') | KeyCode::Down if view_state.step + 1 < 2*n - 1 => {
                            view_state.step += 1;
                        },
                        KeyCode::Char('k') | KeyCode::Up if view_state.step > 0 => {
                            view_state.step -= 1;
                        },
                        _ => {},
                    }
                },
                AppScreen::GameSelectionView(GameSelectionViewState::Selection) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(ExitRequest::Quit),
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('f') => self.screen = AppScreen::GameSelectionView(GameSelectionViewState::Filter),
                    KeyCode::Char('j') | KeyCode::Down => self.filtered_games_list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.filtered_games_list_state.select_previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(idx) = self.filtered_games_list_state.selected() {
                            self.screen = AppScreen::GameView(GameViewState { game_idx: self.filtered_games_indices[idx], step: 0 })
                        }
                    },
                    _ => {},
                },
                AppScreen::GameSelectionView(GameSelectionViewState::Filter) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(ExitRequest::Quit),
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('f') => self.screen = AppScreen::GameSelectionView(GameSelectionViewState::Selection),
                    KeyCode::Char('j') | KeyCode::Down => self.filtered_strategies_list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.filtered_strategies_list_state.select_previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(idx) = self.filtered_strategies_list_state.selected() {
                            self.filtered_strategies[idx].2.advance();
                            self.filtered_games_list_state.select(None);
                            self.update_filtered_list();
                        }
                    },
                    _ => {},
                },
            }
        }

        Ok(ExitRequest::Continue)
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

        let filter_size = Strategy::iter().fold(0, |acc, s| acc.max(format!("|X {:?} ({})|", s, self.games.len()).len() as u16) );

        let [filter_area, games_area] = Layout::horizontal([
            Constraint::Max(filter_size),
            Constraint::Fill(1),
        ]).areas(top_area);

        let filter_list = List::new(self.filtered_strategies.iter()
            .map(|(strategy, freq, status)|{
                match status {
                    FilterStatus::Neutral => Span::from(format!("  {:?} ({})", strategy, freq)),
                    FilterStatus::Include => Span::styled(format!("✓ {:?} ({})", strategy, freq), Style::default().green()),
                    FilterStatus::Exclude => Span::styled(format!("X {:?} ({})", strategy, freq), Style::default().red()),
                }
            }))
            .block(Block::bordered().title("Filter"))
            ;

        let game_list_title = if self.filtered_games_indices.len() == self.games.len() {
            format!("Games ({})", self.games.len())
        } else {
            format!("Games ({}/{})", self.filtered_games_indices.len(), self.games.len())
        };

        let games_list = List::new(self.filtered_games_indices.iter()
                .map(|idx| {
                    let (text, style) = &self.games_titles[*idx];
                    Span::styled(text, *style)
                })
            )
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::bordered().title(game_list_title))
            ;

        match state {
            GameSelectionViewState::Filter => {
                let filter_list = filter_list.style(Color::White).highlight_style(Modifier::REVERSED);
                frame.render_stateful_widget(filter_list, filter_area, &mut self.filtered_strategies_list_state); 

                let games_list = games_list.dim();
                frame.render_stateful_widget(games_list, games_area, &mut self.filtered_games_list_state); 
            },
            GameSelectionViewState::Selection => {
                let filter_list = filter_list.dim();
                frame.render_stateful_widget(filter_list, filter_area, &mut self.filtered_strategies_list_state); 

                let games_list = games_list.style(Color::White).highlight_style(Modifier::REVERSED);
                frame.render_stateful_widget(games_list, games_area, &mut self.filtered_games_list_state); 
            },
        }

        frame.render_widget("[q], [ESC]: exit | [↑], [k]: move up | [↓], [j]: move down | [Space], [Enter]: select | [f], [Tab]: Filter ⇄ Games", shortcut_area);
    }

    fn render_game(&self, frame: &mut Frame, state: &GameViewState) {
        let game = &self.games[state.game_idx];
        let n = 2*game.boards.len() - 1;
        assert!(n > 0);

        let area = Rect::new((frame.area().width - 73)/2, 0, 73, frame.area().height);
        let [header_area, strat_area, board_area, shortcut_area] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])  
            .areas(area);

        let gauge = Gauge::default()
            .block(Block::bordered().title(format!("Game {} - Step {}/{}", state.game_idx + 1, state.step + 1, n)))
            .gauge_style(Style::new().white().on_black().italic())
            .ratio(state.step as f64 / (n - 1) as f64)
        ;

        frame.render_widget(gauge, header_area);

        let board_idx = state.step / 2;
        let board = &game.boards[board_idx];
            
        if state.step.is_multiple_of(2) {
            let message = if board_idx + 1 == game.boards.len() {
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
            let (strat, highlights) = &game.steps[board_idx];
            let message = format!("Apply {:?}", strat);
            frame.render_widget(message, strat_area);

            let next_board = &game.boards[board_idx + 1];
            let diff = next_board.diff(board);
            render_sudoku_board(frame, board_area, next_board, highlights, &diff);
        }

        frame.render_widget("[q], [ESC]: back | [↑], [k]: previous move | [↓], [j]: next move", shortcut_area);
    }

}

fn render_load_games_progress(terminal: &mut DefaultTerminal, progress: usize, total: usize) -> color_eyre::Result<ExitRequest> {
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
            .block(Block::bordered().title(format!("Solving games {}/{}", progress, total)))
            .gauge_style(Style::new().white().on_black().italic())
            .ratio(progress as f64 / total as f64);
        frame.render_widget(gauge, gauge_area);
    })?;

    if crossterm::event::poll(std::time::Duration::from_secs(0))?
        && let Some(key) = crossterm::event::read()?.as_key_press_event()
        && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) 
    {
        Ok(ExitRequest::Quit)
    } else {
        Ok(ExitRequest::Continue)
    }
}

fn load_games(terminal: &mut DefaultTerminal, file_paths: &[PathBuf], sequential: bool) -> color_eyre::Result<Option<Vec<SolvedGame>>> {
    let games = {
        let mut games = Vec::<SudokuBoard>::new();
        for fpath in file_paths {
            games.extend(crate::io::load_games(fpath))
        }
        // games = vec![games.into_iter().nth(4).unwrap()];
        games
    };

    if games.is_empty() { return Err(SudokuError::NoBoardFound.into()); }

    let total = games.len();
    let games = 
        if sequential {
            let mut solved_games = Vec::new();

            for (i, g) in games.into_iter().enumerate() {
                if let Ok(g) = solve(g) {
                    solved_games.push(g)
                }
                if render_load_games_progress(terminal, i, total)? == ExitRequest::Quit {
                    return Ok(None)
                }

            }

            solved_games
        } else {
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

            loop {
                // check if job is done
                if let Ok(v) = result.try_lock() && let Some(games) = &*v {
                    break games.clone()
                }

                if render_load_games_progress(terminal, progress.get(), total)? == ExitRequest::Quit {
                    return Ok(None)
                }
            }
        };

    if games.is_empty() {
        return Err(SudokuError::NoBoardFound.into())
    }
    Ok(Some(games))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(|term| {
        let cli = Cli::parse();
        if let Some(games) = load_games(term, &cli.games, cli.sequential)? {
            App::new(games).run(term)?
        }
        Ok(())
    })
}
