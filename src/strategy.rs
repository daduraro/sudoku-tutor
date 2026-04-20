use itertools::Itertools;
use strum::{EnumIter, EnumCount};
use strum::IntoEnumIterator;

use crate::board::{ALL_DIGITS_FLAG, DigitFlag, SudokuBoard, DigitMask, DigitMaskFromIter};
use crate::index::{BlockIndex, RowIndex, ColumnIndex, CellIndex, DigitIndex, HouseIndex, HouseIndexer};
use crate::error::SudokuError;
use crate::display::Highlight;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, EnumCount)]
pub enum Strategy {
    // apply primary strategy of all current primaries at once
    Primaries,

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

    // a candidate appears in a single row/column within a block,
    // thus all other appearence outside the block in the
    // row/column can be eliminated
    LockedCandidatePointing,


    // a candidate in a single row/column appear only in a block,
    // thus this candidate cannot appear in any other cell inside
    // the block
    LockedCandidateClaiming,

    // three cells in a house have the same up to 3 digits
    // 
    // NakedTriple,
    // HiddenTriple,

    // NakedQuad,
    // HiddenQuad,
}

fn apply_strategy(s: Strategy, board: SudokuBoard) -> Result<(SudokuBoard, Vec<Highlight>), SudokuError> {
    let mut board = board;
    let mut highlights = Vec::<Highlight>::new();
    match s {
        Strategy::Primaries => {
            let primary_cells: Vec<_> = board.indexed_iter().filter_map(|(cell_idx, cell)| {
                cell.digit_value().map(move |d| (cell_idx, d))
            }).collect();

            let mut rows_highlight = vec![false; RowIndex::COUNT];
            let mut columns_highlight = vec![false; ColumnIndex::COUNT];
            let mut blocks_highlight = vec![false; BlockIndex::COUNT];
            for primary in primary_cells {
                let (primary_cell_idx, d) = primary;
                let mut relevant_digit = false;

                let mask = DigitMask::all_but(d);
                for h in primary_cell_idx.houses() {
                    for (cell_idx, cell) in board.indexed_house_mut(h) {
                        if cell_idx == primary_cell_idx { continue }
                        if cell.apply_mask(&mask) {
                            relevant_digit = true;
                            match h {
                                HouseIndex::Block(b) => blocks_highlight[*b] = true,
                                HouseIndex::Column(c) => columns_highlight[*c] = true,
                                HouseIndex::Row(r) => rows_highlight[*r] = true,
                            }
                        }
                    }
                }
                if relevant_digit {
                    highlights.push(primary.into());
                }
            }

            highlights.extend(
                rows_highlight.into_iter().enumerate()
                    .filter_map(|(i, include)| {include.then_some(Highlight::from(HouseIndex::from(RowIndex::new(i))))})
            );
            highlights.extend(
                columns_highlight.into_iter().enumerate()
                    .filter_map(|(i, include)| {include.then_some(Highlight::from(HouseIndex::from(ColumnIndex::new(i))))})
            );
            highlights.extend(
                blocks_highlight.into_iter().enumerate()
                    .filter_map(|(i, include)| {include.then_some(Highlight::from(HouseIndex::from(BlockIndex::new(i))))})
            );
        }
        Strategy::HiddenSingle => {
            'houses: for &house_idx in HouseIndex::domain() {
                let freq: Vec<u8> = board.house(house_idx).fold(vec![0; DigitIndex::domain().len()], |mut acc, c| {
                    for &d in DigitIndex::domain() {
                        if c[d] { acc[*d] += 1; }
                    }
                    acc
                });

                let single_digits = freq.into_iter().enumerate().filter_map(|(d, f)|{
                    if f == 1 { Some(DigitIndex::new(d)) } else { None }
                }).only();

                for (cell_index, mut cell) in board.indexed_house_mut(house_idx) {
                    if cell.is_digit() { continue }
                    if let Some(d) = (**cell & *single_digits).first_one() {
                        cell &= *single_digits;
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

                let mask = DigitMask::new(!*board[idx_a]);
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

                    let mask = [d0, d1].into_iter().only();
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
        },
        Strategy::LockedCandidatePointing => {
            'outer: for b in BlockIndex::domain() {
                for d in DigitIndex::domain() {
                    let mut rows: Vec<_> = Vec::new();
                    let mut columns: Vec<_> = Vec::new();

                    for cell_idx in b.indices() {
                        if board[cell_idx].contains(d) {
                            if !rows.contains(&cell_idx.row()) { rows.push(cell_idx.row()); }
                            if !columns.contains(&cell_idx.column()) { columns.push(cell_idx.column()); }
                        }
                    }

                    let claiming_house: Option<(HouseIndex, Vec<CellIndex>)> = 
                        if rows.len() == 1 {
                            let r = rows[0];
                            Some((r.into(), columns.into_iter().map(|c| CellIndex::new(r, c)).collect()))
                        } else if columns.len() == 1 {
                            let c = columns[0];
                            Some((c.into(), rows.into_iter().map(|r| CellIndex::new(r, c)).collect()))
                        } else {
                            None
                        };
                    
                    if let Some((claiming_house, cells)) = claiming_house {
                        let mask = DigitMask::all_but(*d);
                        let mut changed = false;
                        for idx in claiming_house.indices() {
                            if &idx.block() == b { continue }
                            changed |= board[idx].apply_mask(&mask)
                        }

                        if changed {
                            highlights.push(claiming_house.into());
                            highlights.push((*b).into());
                            highlights.extend(cells.into_iter().map(|c| Highlight::Digit((c, *d))));
                            break 'outer;
                        }
                    }
                }
            }
        },
        Strategy::LockedCandidateClaiming => {
            'strategy: for h in HouseIndex::rows_and_columns() {
                for d in DigitIndex::domain() {
                    let cells: Vec<_> = h.indices().iter().cloned().filter(|idx|{
                        board[idx].contains(d)
                    }).collect();
                    let same_block = cells.iter().map(CellIndex::block).all_equal();
                    if cells.is_empty() || !same_block { continue }
                    let block = cells.first().unwrap().block();
                    
                    let mask = DigitMask::all_but(*d);
                    let mut changed = false;
                    for idx in block.indices() {
                        if h.contains(idx) { continue }
                        changed |= board[idx].apply_mask(&mask);
                    }

                    if changed {
                        highlights.push((*h).into());
                        highlights.push(block.into());
                        highlights.extend(cells.into_iter().map(|idx| Highlight::Digit((idx, *d))));
                        break 'strategy;
                    }
                }
            }
        },
    };

    Ok((board, highlights))
}

#[derive(Debug, Clone)]
pub struct SolvedGame {
    pub boards: Vec<SudokuBoard>,
    pub steps: Vec<(Strategy, Vec<Highlight>)>,
    pub strategies: Vec<Strategy>,
}

impl SolvedGame {
    pub fn is_solved(&self) -> bool {
        self.boards.last().map(|b| b.is_solved()).unwrap_or(false)
    }
}

pub fn solve(board: SudokuBoard) -> Result<SolvedGame, SudokuError>
{
    let mut boards = Vec::<SudokuBoard>::new();
    let mut steps = Vec::<(Strategy, Vec<Highlight>)>::new();
    let mut current_board = board;

    // single Primaries step
    let (next_board, highlights) = apply_strategy(Strategy::Primaries, current_board.clone())?;
    if next_board != current_board {
        if !current_board.is_valid() { return Err(SudokuError::UnsolvableSudoku) }
        boards.push(current_board);
        current_board = next_board;
        steps.push((Strategy::Primaries, highlights));
    }

    while !current_board.is_solved() {
        let mut has_advanced = false;
        for s in Strategy::iter() {
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


    let strategies: Vec<_>  = steps.iter()
        .fold(vec![false; Strategy::COUNT], |mut acc, (strat, _)|{
            acc[ *strat as usize ] = true;
            acc
        }).into_iter().zip(Strategy::iter())
        .filter_map(|(b, strat)| {
            if b { Some(strat) } else { None }
        }).collect();

    Ok(SolvedGame { boards, steps, strategies })
}