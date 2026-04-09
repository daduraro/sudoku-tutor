mod board;
mod io;
mod error;
mod strategy;

use std::iter::zip;
use std::path::PathBuf;

use clap::{Parser};

use crate::board::{SudokuBoard, SudokuBoardTrait};
use crate::io::load_games;
use crate::strategy::solve;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name="FILE", required=true, num_args=1..)]
    games: Vec<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
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

        if let Ok((states, steps)) = solve(board.clone()) {
            for ((prev_board, curr_board), step) in zip(zip(&states, &states[1..]), &steps) {
                println!();

                let mut lines: Vec<String> = prev_board.pretty_str().split('\n').map(String::from).collect();
                for (line, to_append) in zip(&mut lines,  curr_board.pretty_str().split('\n')) {
                    line.push_str("   ");
                    line.push_str(to_append);
                }

                let step = format!("Applying {:?}", step);
                const LINE_VISUAL_SIZE: usize = (1+9*4)*2 + 3;
                let step_spacing: String = std::iter::repeat_n(' ', (LINE_VISUAL_SIZE - step.len()) / 2).collect();

                println!("{}{}", step_spacing, step);
                for line in lines {
                    println!("{}", line);
                }
            }
        } else {
            println!("Board is invalid!");
        }
    }

    Ok(())
}
