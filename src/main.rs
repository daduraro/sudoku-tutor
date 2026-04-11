mod board;
mod io;
mod error;
mod strategy;
mod display;

use std::iter::zip;
use std::path::PathBuf;

use clap::{Parser};
use ratatui::{DefaultTerminal, Frame};
use ratatui::style::{Color, Modifier};
use ratatui::widgets::{Paragraph, List, ListState};
use crossterm::event::{self, KeyCode};

use crate::board::{SudokuBoard, SudokuBoardTrait};
use crate::display::render_sudoku_board;
use crate::io::load_games;
use crate::strategy::{solve, simplify, SolvedGame};
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

    // if let Some(board) = boards.last() {
    //     println!("Step-by-step resolution");

    //     let board = simplify(board.clone())?;

    //     let (states, steps) = solve(board)?;
    //     for ((prev_board, curr_board), (step, highlighted_cells)) in zip(zip(&states, &states[1..]), &steps) {
    //         println!();
    //         println!("Applying {:?}", step);
    //         println!("{}", show_sudoku_board(curr_board, highlighted_cells, &find_diff(curr_board, prev_board)));
    //     }
    // }

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
                        if app_state.current_step + 1 < n { app_state.current_step += 1; }
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
                            let board = simplify(app_state.boards[idx].clone())?;
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
        let state = &states[app_state.current_step];
        if app_state.current_step > 0 {
            let (strat, highlighted_cells) = &steps[app_state.current_step - 1];
            let prev_state = &states[app_state.current_step - 1];
            let diff = find_diff(state, prev_state);
            let info = Some((
                *strat,
                highlighted_cells.as_ref(),
                diff.as_ref()
            ));
            render_sudoku_board(frame, state, info);
        } else {
            render_sudoku_board(frame, state, None);
        };
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
