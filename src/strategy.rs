use itertools::Itertools;

use crate::board::{ALL_DIGITS_FLAG, DigitFlag, SudokuBoard};
use crate::index::{DigitIndex, HouseIndex, HouseIndexer};
use crate::error::SudokuError;
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

impl Strategy {
    pub const fn domain() -> &'static [Strategy] {
        &[
            Strategy::Primary,
            Strategy::HiddenSingle,
            Strategy::NakedPair,
            Strategy::HiddenPair,
        ]
    }
}

fn apply_strategy(s: Strategy, board: SudokuBoard) -> Result<(SudokuBoard, Vec<Highlight>), SudokuError> {
    let mut board = board;
    let mut highlights = Vec::<Highlight>::new();
    match s {
        Strategy::AllPrimaries | Strategy::Primary => {
            let primary_cells: Vec<_> = board.indexed_iter().filter_map(|(cell_idx, cell)| {
                cell.digit_value().map(move |d| (cell_idx, d))
            }).collect();

            for primary in primary_cells {
                let (primary_cell_idx, d) = primary;
                let mut changed = false;

                let mask = {
                    let mut mask = ALL_DIGITS_FLAG;
                    mask.set(*d, false);
                    mask
                };

                for h in primary_cell_idx.houses() {
                    for (cell_idx, cell) in board.indexed_house_mut(h) {
                        if cell_idx == primary_cell_idx { continue }
                        changed |= cell.apply_mask(&mask);
                    }
                }

                if matches!(s, Strategy::Primary) {
                    if changed {
                        highlights.push(primary.into());
                        highlights.push(primary_cell_idx.row().into());
                        highlights.push(primary_cell_idx.column().into());
                        highlights.push(primary_cell_idx.block().into());
                        break
                    }
                } else {
                    highlights.push(Highlight::Digit(primary));
                }
            }
        }
        Strategy::HiddenSingle => {
            'houses: for &house_idx in HouseIndex::domain() {
                let freq: Vec<u8> = board.house(house_idx).fold(vec![0; DigitIndex::domain().len()], |acc, c| {
                    let mut acc = acc;
                    for &d in DigitIndex::domain() {
                        if c[d] { acc[*d] += 1; }
                    }
                    acc
                });

                let single_digits = {
                    let mut single_digits = DigitFlag::default();
                    for (d, f) in freq.into_iter().enumerate() {
                        single_digits.set(d, f == 1);
                    }
                    single_digits
                };
                for (cell_index, mut cell) in board.indexed_house_mut(house_idx) {
                    if cell.is_digit() { continue }
                    if let Some(d) = (**cell & single_digits).first_one() {
                        cell &= single_digits;
                        highlights.push((cell_index, DigitIndex::new(d)).into());
                        highlights.push(house_idx.into());
                        break 'houses
                    }
                }
            }
        }
        Strategy::NakedPair => { 
            // two cells in same region have same two numbers
            let two_digits: Vec::<_> = board.indexed_iter().filter_map(|(idx, cell)| {
                if cell.count_ones() == 2 { Some(idx) } else { None }
            }).collect();

            let naked_pairs: Vec<_> = two_digits.iter().combinations(2)
                .filter_map(|indices|{
                    let idx_a = indices[0];
                    let idx_b = indices[1];
                    if board[idx_a] == board[idx_b] && idx_a.share_house(idx_b) {
                        Some((*idx_a, *idx_b))
                    } else {
                        None
                    }
                }).collect();
            
            for (idx_a, idx_b) in naked_pairs {
                let mut changed = false;

                let mask = !*board[idx_a];
                for house_idx in idx_a.houses() {
                    if !house_idx.contains(idx_b) { continue }
                    for (idx, cell) in board.indexed_house_mut(house_idx) {
                        if idx == idx_a || idx == idx_b { continue }
                        if cell.apply_mask(&mask) {
                            changed = true;
                            highlights.push(house_idx.into());
                        }
                    }
                }

                if changed {
                    for d in board[idx_a].iter_ones() {
                        let d = DigitIndex::from(d);
                        highlights.push((idx_a, d).into());
                        highlights.push((idx_b, d).into());
                    }
                    break // do not process more than one naked_pair
                }
            }
        }
        Strategy::HiddenPair => {
            'outer: for &house_idx in HouseIndex::domain() {
                // `hidden_pairs_candidates` will yield all the pair of cells within the house
                // that are the only ones to contain two particular digits.
                // These will include naked pairs, which we will filter out later.
                let hidden_pairs_candidates = {
                    // for each digit, get which cells in the house contain them
                    let digit_cells = {
                        let mut digit_cells = [DigitFlag::default(); 9];
                        for (idx, cell) in board.house(house_idx).enumerate() {
                            for &d in DigitIndex::domain() {
                                digit_cells[*d].set(idx, cell[d])
                            }
                        }
                        digit_cells
                    };

                    // get the digits that appear in exactly two cells
                    let digit_two_cells = digit_cells.into_iter().enumerate().filter_map(|(d, which_cells)| {
                        if which_cells.count_ones() == 2 { Some(DigitIndex::new(d)) } else { None }
                    });

                    // for each digit that contain only two cells, check if there is
                    // another digit with the same two cells
                    digit_two_cells.combinations(2).filter_map(move |digits|{
                        let digit_a = digits[0];
                        let digit_b = digits[1];

                        if digit_cells[*digit_a] == digit_cells[*digit_b] {
                            let indices: [usize; 2] = digit_cells[*digit_a].iter_ones().collect_array().unwrap();
                            Some((digit_a, digit_b, house_idx.index(indices[0]), house_idx.index(indices[1])))
                        } else {
                            None
                        }
                    })
                };

                for (d0, d1, cell_idx_a, cell_idx_b) in hidden_pairs_candidates {
                    // check if they are a naked pair, instead of a hidden one
                    if board[cell_idx_a].count_ones() == 2 && board[cell_idx_b].count_ones() == 2 {
                        continue
                    }

                    let mask = {
                        let mut mask = DigitFlag::default();
                        mask.set(*d0, true);
                        mask.set(*d1, true);
                        mask
                    };

                    board[cell_idx_a].apply_mask(&mask);
                    board[cell_idx_b].apply_mask(&mask);

                    highlights.push(house_idx.into());
                    highlights.push((cell_idx_a, d0).into());
                    highlights.push((cell_idx_a, d1).into());
                    highlights.push((cell_idx_b, d0).into());
                    highlights.push((cell_idx_b, d1).into());

                    break 'outer // do not process more than one hidden pair
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
        for &s in Strategy::domain() {
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