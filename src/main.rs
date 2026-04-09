mod board;
mod io;
mod error;
mod strategy;
mod display;

use std::iter::zip;
use std::path::PathBuf;

use clap::{Parser};
use colored::Colorize;

use crate::board::{SudokuBoard, SudokuBoardTrait};
use crate::display::show_sudoku_board;
use crate::io::load_games;
use crate::strategy::{solve, simplify};

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

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let mut boards = Vec::<SudokuBoard>::new();
    for fpath in &cli.games {
        let reader = std::fs::File::open(fpath)?;
        let reader = std::io::BufReader::new(reader);
        boards.extend(load_games(Box::new(reader)));
    }

    if let Some(board) = boards.last() {
        println!("Step-by-step resolution");

        let board = simplify(board.clone())?;

        let (states, steps) = solve(board)?;
        for ((prev_board, curr_board), step) in zip(zip(&states, &states[1..]), &steps) {
            println!();
            println!("Applying {:?}", step);
            println!("{}", show_sudoku_board(curr_board, Vec::new(), find_diff(curr_board, prev_board)));
        }
    }

    Ok(())
}
