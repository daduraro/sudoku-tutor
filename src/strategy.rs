use ndarray::{ArrayViewMut2};

use crate::{board::{SudokuBoard, SudokuBoardTrait, SudokuCell, SudokuCellTrait, SudokuSubCellIndex}, error::SudokuError};
use crate::display::Highlight;

#[derive(Clone, Copy, Debug)]
pub enum Strategy {
    // apply primary strategy of all current primaries at once
    AllPrimaries,

    // remove candidates when other direct cells have them
    Primary,

    // remove other candidates from a cell when a row/column/block
    // is the sole owner of a digit
    HiddenSingle,

    // two cells in a single row/column/block have only two candidates
    // that are the same, remove those from all other cells in the
    // row/column/block
    NakedPair,
}

fn apply_strategy(s: Strategy, board: SudokuBoard) -> Result<(SudokuBoard, Vec<Highlight>), SudokuError> {
    let mut board = board;
    let mut highlights = Vec::<Highlight>::new();
    match s {
        Strategy::AllPrimaries | Strategy::Primary => {
            let primary_cells: Vec<_> = board.indexed_iter().filter_map(|((i, j), cell)| {
                cell.digit_value().map(move |d| (i, j, d))
            }).collect();

            for (i, j, d) in primary_cells {
                let mut changed = false;

                let mask = {
                    let mut mask = SudokuCell::empty_cell();
                    mask.set(d as usize, false);
                    mask
                };

                for (c, cell) in board.row_mut(i).indexed_iter_mut() {
                    if c == j { continue }
                    let new_cell = *cell & mask;
                    if new_cell != *cell {
                        changed = true;
                        *cell = new_cell;
                    }
                }

                for (r, cell) in board.column_mut(j).indexed_iter_mut() {
                    if r == i { continue }
                    let new_cell = *cell & mask;
                    if new_cell != *cell {
                        changed = true;
                        *cell = new_cell;
                    }
                }

                let block_idx = SudokuBoard::block_index(i, j);
                for ((b_r, b_c), cell) in board.block_mut(block_idx).indexed_iter_mut() {
                    let [r, c] = SudokuBoard::index_from_block(block_idx, b_r, b_c);
                    if r == i && c == j { continue }
                    let new_cell = *cell & mask;
                    if new_cell != *cell {
                        changed = true;
                        *cell = new_cell;
                    }
                }

                if matches!(s, Strategy::Primary) {
                    if changed {
                        highlights.push(Highlight::Digit((i, j, d)));
                        highlights.push(Highlight::Row(i as u8));
                        highlights.push(Highlight::Column(j as u8));
                        highlights.push(Highlight::Block(block_idx as u8));
                        break
                    }
                } else {
                    highlights.push(Highlight::Digit((i, j, d)));
                }
            }
        }
        Strategy::HiddenSingle => { 
            fn find_hidden(region: ArrayViewMut2<SudokuCell>) -> Option<SudokuSubCellIndex> {
                let freq: Vec<u8> = region.iter().fold(vec![0; 9], |acc, c| {
                    let mut acc = acc;
                    for i in 0..9 {
                        if c[i] { acc[i] += 1; }
                    }
                    acc
                });

                let single_digits = {
                    let mut single_digits = SudokuCell::ZERO;
                    for (d, f) in freq.into_iter().enumerate() {
                        single_digits.set(d, f == 1);
                    }
                    single_digits
                };

                for ((i, j), c) in region.into_indexed_iter_mut() {
                    if c.is_digit() { continue }
                    if let Some(d) = (*c & single_digits).first_one() {
                        *c &= single_digits;
                        return Some((i, j, d as u8));
                    }
                }
                None
            }
            for i in 0..9 {
                if let Some((_, c, d)) = find_hidden(board.row_collapse_mut(i)) {
                    highlights.push(Highlight::Digit((i, c, d)));
                    highlights.push(Highlight::Row(i as u8));
                    break;
                }

                if let Some((r, _, d)) = find_hidden(board.column_collapse_mut(i)) {
                    highlights.push(Highlight::Digit((r, i, d)));
                    highlights.push(Highlight::Column(i as u8));
                    break;
                }

                if let Some((b_r, b_c, d)) = find_hidden(board.block_mut(i)) {
                    let [r, c] = SudokuBoard::index_from_block(i, b_r, b_c);
                    highlights.push(Highlight::Digit((r, c, d)));
                    highlights.push(Highlight::Block(i as u8));
                    break;
                }
            }
        }
        Strategy::NakedPair => { todo!() }
    };

    Ok((board, highlights))
}

pub type SolvedGame = (Vec<SudokuBoard>, Vec<(Strategy, Vec<Highlight>)>);

pub fn solve(board: SudokuBoard) -> Result<SolvedGame, SudokuError>
{
    let mut boards = Vec::<SudokuBoard>::new();
    let mut steps = Vec::<(Strategy, Vec<Highlight>)>::new();
    let mut current_board = board;

    // single AllPrimaries step
    let (next_board, highlights) = apply_strategy(Strategy::AllPrimaries, current_board.clone())?;
    if next_board != current_board {
        if !current_board.is_valid() { return Err(SudokuError::UnsolvableSudoku) }
        boards.push(current_board);
        current_board = next_board;
        steps.push((Strategy::AllPrimaries, highlights));
    }

    while !current_board.is_solved() {
        let mut has_advanced = false;
        for s in [Strategy::Primary, Strategy::HiddenSingle] {
            let (next_board, highlights) = apply_strategy(s, current_board.clone())?;
            if next_board != current_board {
                if !current_board.is_valid() { return Err(SudokuError::UnsolvableSudoku) }
                has_advanced = true;
                boards.push(current_board);
                current_board = next_board;
                steps.push((s, highlights));
                break
            }
        }
        if !has_advanced { break }
    }
    boards.push(current_board);

    Ok((boards, steps))
}