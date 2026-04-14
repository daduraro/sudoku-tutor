mod board;
mod io;
mod error;
mod strategy;
mod display;

use std::iter::zip;
use std::path::PathBuf;

use clap::{Parser};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{DefaultTerminal, Frame};
use ratatui::style::{Style, Color, Modifier};
use ratatui::widgets::{Block, Gauge, List, ListState, Paragraph};
use crossterm::event::{KeyCode};

use crate::board::{SudokuBoard, SudokuBoardTrait};
use crate::display::render_sudoku_board;
use crate::io::load_games;
use crate::strategy::{solve, SolvedGame};
use crate::error::SudokuError;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name="FILE", required=true, num_args=1..)]
    games: Vec<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn find_diff(board: &SudokuBoard, prev_board: &SudokuBoard) -> Vec<(usize, usize, u8)>
{
    zip(board.indexed_iter(), prev_board).flat_map(|(((r,c), curr), prev)| {
        (0..9).filter_map(move |d| if curr[d] ^ prev[d] { Some((r, c, d as u8)) } else { None })
    }).collect()
}

struct AppState {
    boards: Vec<SudokuBoard>,
    current: Option<SolvedGame>,
    current_step: usize,
    board_selection: ListState,
}

impl AppState {
    fn new(boards: Vec<SudokuBoard>) -> Self {
        AppState {
            boards,
            current: None,
            current_step: 0,
            board_selection: ListState::default().with_selected(Some(0)),
        }
    }
}

fn app(terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
    let cli = Cli::parse();

    let mut boards = Vec::<SudokuBoard>::new();
    for fpath in &cli.games {
        let reader = std::fs::File::open(fpath)?;
        let reader = std::io::BufReader::new(reader);
        boards.extend(load_games(Box::new(reader)));
    }

    if boards.is_empty() {
        return Err(SudokuError::NoBoardFound.into());
    }


    let mut app_state = AppState::new(boards);
    loop {
        terminal.draw(|frame| render(frame, &mut app_state))?;
        if let Some(key) = crossterm::event::read()?.as_key_press_event() {
            if let Some((states, _)) = &app_state.current {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app_state.current = None;
                    },
                    KeyCode::Char('j') | KeyCode::Down => {
                        let n = states.len();
                        if app_state.current_step + 1 < 2*n - 1 { app_state.current_step += 1; }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app_state.current_step > 0 { app_state.current_step -= 1; }
                    },
                    _ => {},
                }
            }
            else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app_state.board_selection.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.board_selection.select_previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(idx) = app_state.board_selection.selected() {
                            let board = app_state.boards[idx].clone();
                            app_state.current = Some(solve(board)?);
                            app_state.current_step = 0;
                        }
                    },
                    _ => {},
                }
            }
        }
    }
}


fn render(frame: &mut Frame, app_state: &mut AppState) {
    if let Some((states, steps)) = &app_state.current {
        let n = 2*states.len() - 1;
        assert!(n > 0);

        let area = Rect::new((frame.area().width - 73)/2, 0, 73, frame.area().height);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
            ])  
            .split(area);

        let gauge = Gauge::default()
            .block(Block::bordered().title(format!("Step {}/{}", app_state.current_step + 1, n)))
            .gauge_style(Style::new().white().on_black().italic())
            .ratio(app_state.current_step as f64 / (n - 1) as f64)
        ;

        frame.render_widget(gauge, layout[0]);

        let state_idx = app_state.current_step / 2;
        let state = &states[state_idx];
            
        if app_state.current_step.is_multiple_of(2) {
            let message = if state_idx + 1 == states.len() {
                if state.is_solved() {
                    "Solved!"
                } else {
                    "Sudoku is unsolvable with current strategies..."
                }
            } else {
                "Current board"
            };
            frame.render_widget(message, layout[1]);
            render_sudoku_board(frame, layout[2], state, &[], &[]);
        } else {
            let (strat, highlights) = &steps[state_idx];
            let message = format!("Apply {:?}", strat);
            frame.render_widget(message, layout[1]);

            let next_state = &states[state_idx + 1];
            let diff = find_diff(next_state,state);
            render_sudoku_board(frame, layout[2], next_state, highlights, &diff);
        }
    }
    else {
        let list = List::new(app_state.boards.iter().map(SudokuBoardTrait::encode_board))
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");
        frame.render_stateful_widget(list, frame.area(), &mut app_state.board_selection);
    }
}
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}
