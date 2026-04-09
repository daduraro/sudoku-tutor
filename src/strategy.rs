use ndarray::{ArrayViewMut};

use crate::{board::{SudokuBoard, SudokuBoardTrait, SudokuCell, SudokuCellTrait}, error::SudokuError};

#[derive(Clone, Copy, Debug)]
pub enum Strategy {
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

pub fn simplify(board: SudokuBoard) -> Result<SudokuBoard, SudokuError> {
    if !board.is_valid() { return Err(SudokuError::UnsolvableSudoku) }

    fn simplify_region<D: ndarray::Dimension>(region: ArrayViewMut<SudokuCell, D>) -> Result<bool, SudokuError> {
        let mut region = region;

        // collect single digits
        let mut allowed_digits = !SudokuCell::ZERO;
        for c in region.iter_mut() {
            if let Some(d) = c.digit_value() {
                let d = d as usize;
                if !allowed_digits[d] { return Err(SudokuError::UnsolvableSudoku) } // invalid sudoku
                allowed_digits.set(d, false);
            }
        }

        // modify the whole region so that we remove already existing digits
        let mut changed = false;
        for c in region {
            if c.is_digit() { continue }

            let new_c = *c & allowed_digits;
            if *c != new_c {
                *c = new_c;
                changed = true;
            }
        }
        Ok(changed)
    }

    let mut board = board;
    while !board.is_solved() {
        let mut changed = false;
        for i in 0..9 {
            changed |= simplify_region(board.row_mut(i))?;
            changed |= simplify_region(board.column_mut(i))?;
            changed |= simplify_region(board.block_mut(i))?;
        }
        if !changed { break }
    }

    Ok(board)
}

fn apply_strategy(s: Strategy, board: SudokuBoard) -> Result<SudokuBoard, SudokuError> {
    let mut board = board;
    match s {
        Strategy::Primary => {
            fn find_primary<D: ndarray::Dimension>(region: ArrayViewMut<SudokuCell, D>) -> bool {
                let mut region: Vec<&mut SudokuCell> = region.into_iter().collect();
                let primary_indices: Vec<_> = region.iter().enumerate()
                    .filter_map(|(idx, c)| if c.is_digit() { Some(idx) } else { None } )
                    .collect();

                for primary_idx in primary_indices {
                    let allowed_digits =  !*region[primary_idx];
                    for i in 0..9 {
                        if i == primary_idx { continue }
                        let new_value = *region[i] & allowed_digits;
                        if new_value != *region[i] {
                            *region[i] = new_value;
                            return true
                        }
                    }
                }
                false
            }
            for i in 0..9 {
                if find_primary(board.row_mut(i))
                    || find_primary(board.column_mut(i))
                    || find_primary(board.block_mut(i))
                { break; }
            }
        }
        Strategy::HiddenSingle => { 
            fn find_hidden<D: ndarray::Dimension>(region: ArrayViewMut<SudokuCell, D>) -> bool {
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

                for c in region {
                    if c.is_digit() { continue }
                    if (*c & single_digits).any() {
                        *c &= single_digits;
                        return true
                    }
                }
                false
            }
            for i in 0..9 {
                if find_hidden(board.row_mut(i))
                    || find_hidden(board.column_mut(i))
                    || find_hidden(board.block_mut(i))
                { break; }
            }
        }
        Strategy::NakedPair => { todo!() }
    };

    Ok(board)
}

pub fn solve(board: SudokuBoard) -> Result<(Vec<SudokuBoard>, Vec<Strategy>), SudokuError>
{
    let mut boards = Vec::<SudokuBoard>::new();
    let mut steps = Vec::<Strategy>::new();

    let mut current_board = board;
    loop {
        let mut has_advanced = false;
        for s in [Strategy::Primary, Strategy::HiddenSingle] {
            let next_board = apply_strategy(s, current_board.clone())?;
            if next_board != current_board {
                has_advanced = true;
                boards.push(current_board);
                current_board = next_board;
                steps.push(s);
                break
            }
        }
        if !has_advanced || current_board.is_solved() {
            boards.push(current_board);
            break
        }
    }

    Ok((boards, steps))
}