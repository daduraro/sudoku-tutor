use ndarray::{ArrayView2, ArrayViewMut2};
use itertools::Itertools;

use crate::{board::{SudokuBoard, SudokuBoardTrait, SudokuCell, SudokuCellTrait, SudokuSubCellIndex}, error::SudokuError};
use crate::display::Highlight;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Strategy {
    // apply primary strategy of all current primaries at once
    AllPrimaries,

    // remove candidates when other cells in their houses have them
    Primary,

    // remove other candidates from a cell when a house
    // is the sole owner of a digit
    HiddenSingle,

    // two cells in a single house have only two candidates
    // that are the same, remove those from all other cells in the
    // house
    NakedPair,

    // in a house, two digits appear in just two cells, removing
    // the rest of the candidates from those two cells
    HiddenPair,
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
        Strategy::NakedPair => { 
            // two cells in same region have same two numbers
            let two_digits: Vec::<_> = board.indexed_iter().filter_map(|(idx, cell)| {
                if cell.count_ones() == 2 { Some(idx) } else { None }
            }).collect();

            let naked_pairs: Vec<_> = two_digits.iter().enumerate().flat_map(|(i, idx_a)|{
                two_digits.iter().skip(i+1).filter_map(|idx_b| {
                    if board[*idx_a] == board[*idx_b] && (
                        idx_a.0 == idx_b.0 || idx_a.1 == idx_b.1 ||
                        SudokuBoard::block_index(idx_a.0, idx_a.1) == SudokuBoard::block_index(idx_b.0, idx_b.1)
                    ) {
                        Some((*idx_a, *idx_b))
                    } else {
                        None
                    }
                })
            }).collect();
            for (idx_a, idx_b) in naked_pairs {
                let mut changed = false;

                let mask = !board[idx_a];
                if idx_a.0 == idx_b.0 { // they share row
                    let r = idx_a.0;
                    for c in 0..9 {
                        if c == idx_a.1 || c == idx_b.1 { continue }
                        if board[(r,c)] != board[(r,c)] & mask {
                            board[(r, c)] &= mask;
                            changed = true;
                            highlights.push(Highlight::Row(r as u8));
                        }
                    }
                }
                if idx_a.1 == idx_b.1 { // same column
                    let c = idx_a.1;
                    for r in 0..9 {
                        if r == idx_a.0 || r == idx_b.0 { continue }
                        if board[(r,c)] != board[(r,c)] & mask {
                            board[(r, c)] &= mask;
                            changed = true;
                            highlights.push(Highlight::Column(c as u8));
                        }
                    }
                }
                let block_idx = SudokuBoard::block_index(idx_a.0, idx_a.1);
                if block_idx == SudokuBoard::block_index(idx_b.0, idx_b.1) {
                    for i in 0..9 {
                        let idx = SudokuBoard::block_row_col(block_idx, i);
                        if idx == idx_a || idx == idx_b { continue }
                        if board[idx] != board[idx] & mask {
                            board[idx] &= mask;
                            changed = true;
                            highlights.push(Highlight::Block(block_idx as u8));
                        }
                    }
                }

                if changed {
                    let [d0, d1] = board[idx_a].iter_ones().collect_array().unwrap();
                    highlights.push(Highlight::Digit((idx_a.0, idx_a.1, d0 as u8)));
                    highlights.push(Highlight::Digit((idx_a.0, idx_a.1, d1 as u8)));
                    highlights.push(Highlight::Digit((idx_b.0, idx_b.1, d0 as u8)));
                    highlights.push(Highlight::Digit((idx_b.0, idx_b.1, d1 as u8)));
                    break
                }
            }
        }
        Strategy::HiddenPair => {
            fn find_hidden_pairs(region: ArrayViewMut2<SudokuCell>) -> Option<(usize, usize, u8, u8)> {
                // `digit_cells` tells for each digit, which cells (0..9) contain them.
                // If two digits have the exact same two cells, then they are a hidden pair.
                let digit_to_cell_flat_index = {
                    let mut digit_to_cell_flat_index = [SudokuCell::default(); 9];
                    for (idx, cell) in region.iter().enumerate() {
                        for d in 0..9 {
                            digit_to_cell_flat_index[d].set(idx, cell[d])
                        }
                    }
                    digit_to_cell_flat_index
                };

                let digits_in_two_cells: Vec<_> = digit_to_cell_flat_index.iter().enumerate().filter_map(|(digit, cell)|{
                    if cell.count_ones() == 2 { Some(digit) } else { None }
                }).collect();

                let mut region = region;
                for digits in digits_in_two_cells.iter().combinations(2) {
                    if digit_to_cell_flat_index[*digits[0]] == digit_to_cell_flat_index[*digits[1]] {
                        let flat_indices: [usize; 2] = digit_to_cell_flat_index[*digits[0]].iter_ones().collect_array().unwrap();
                        let mask = {
                            let mut mask = SudokuCell::default();
                            mask.set(*digits[0], true);
                            mask.set(*digits[1], true);
                            mask
                        };

                        let mut changed = false;
                        for flat_index in flat_indices.iter() {
                            let cell = region.iter_mut().nth(*flat_index).unwrap();
                            if *cell != *cell & mask {
                                changed = true;
                                *cell &= mask;
                            }
                        }

                        if changed {
                            return Some((flat_indices[0], flat_indices[1], *digits[0] as u8, *digits[1] as u8)) 
                        }
                    }
                }

                None
            }
            for i in 0..9 {
                if let Some((flatidx_a, flatidx_b, d0, d1)) = find_hidden_pairs(board.row_collapse_mut(i)) {
                    highlights.push(Highlight::Row(i as u8));
                    highlights.push(Highlight::Digit((i, flatidx_a, d0)));
                    highlights.push(Highlight::Digit((i, flatidx_a, d1)));
                    highlights.push(Highlight::Digit((i, flatidx_b, d0)));
                    highlights.push(Highlight::Digit((i, flatidx_b, d1)));
                    break
                }
                if let Some((flatidx_a, flatidx_b, d0, d1)) = find_hidden_pairs(board.column_collapse_mut(i)) {
                    highlights.push(Highlight::Column(i as u8));
                    highlights.push(Highlight::Digit((flatidx_a, i, d0)));
                    highlights.push(Highlight::Digit((flatidx_a, i, d1)));
                    highlights.push(Highlight::Digit((flatidx_b, i, d0)));
                    highlights.push(Highlight::Digit((flatidx_b, i, d1)));
                    break
                }
                if let Some((flatidx_a, flatidx_b, d0, d1)) = find_hidden_pairs(board.block_mut(i)) {
                    highlights.push(Highlight::Block(i as u8));
                    {
                        let (r, c) = SudokuBoard::block_row_col(i, flatidx_a);
                        highlights.push(Highlight::Digit((r, c, d0)));
                        highlights.push(Highlight::Digit((r, c, d1)));
                    }
                    {
                        let (r, c) = SudokuBoard::block_row_col(i, flatidx_b);
                        highlights.push(Highlight::Digit((r, c, d0)));
                        highlights.push(Highlight::Digit((r, c, d1)));
                    }
                    break
                }
            }
        }
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
        for s in [Strategy::Primary, Strategy::HiddenSingle, Strategy::NakedPair, Strategy::HiddenPair] {
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