use std::collections::VecDeque;

use itertools::Itertools;
use strum::{EnumIter, EnumCount};
use strum::IntoEnumIterator;
use ena::unify::{UnifyKey, InPlaceUnificationTable};

use crate::board::{SudokuFlags, SudokuBoard, DigitMask, DigitMaskFromIter};
use crate::index::{BlockIndex, RowIndex, ColumnIndex, CellIndex, DigitIndex, HouseIndex, HouseIndexer};
use crate::error::SudokuError;
use crate::display::Highlight;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum RedBlack {
    Red,
    Black,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct IntKey(u32);

impl UnifyKey for IntKey {
    type Value = ();
    fn index(&self) -> u32 {
        self.0
    }
    fn from_index(u: u32) -> IntKey {
        IntKey(u)
    }
    fn tag() -> &'static str {
        "IntKey"
    }
}

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

    NakedTriple,
    HiddenTriple,
    NakedQuad,
    HiddenQuad,

    // a candidate appears in two different columns or rows twice and
    // in the same row/column respecectively (forming a square),
    // as such we can eliminate all the other appearance of the candidate
    // in the rest of the column/row as they are locked to that row/column
    XWing,

    // Cells with exactly same two candidates may form a link of locked/complementary pairs.
    // In particular, this chain link will make pairs in an odd distance to
    // be locked pairs themselves, and so any cell that sees both cannot have
    // either candidate.
    RemotePair,


    // ChuteRemotePair,
    // ColoringType1,
    // ColoringType2,
    // 

}

fn naked_group(n: usize, board: &mut SudokuBoard, highlights: &mut Vec<Highlight>) {
    assert!(1 < n && n < 9);
    for &house_idx in HouseIndex::domain() {
        let candidate_cells: Vec::<_> = board.indexed_house(house_idx).filter_map(|(idx, cell)| {
            let digits = cell.num_digits();
            (digits > 1 && digits <= n).then_some(idx)
        }).collect();

        let candidate_groups: Vec<_> = candidate_cells.into_iter().combinations(n)
            .filter_map(|indices|{
                let digits = indices.iter().map(|idx| board[*idx])
                    .fold(SudokuFlags::ZERO, |acc, d| acc | d.digits());
                (digits.count_ones() == n).then_some((digits, indices))
            }).collect();

        for (digits, indices) in candidate_groups {
            let mut changed: bool = false;
            let mask = DigitMask::new(!digits);
            for (idx, cell) in board.indexed_house_mut(house_idx) {
                if indices.contains(&idx) { continue }
                changed |= cell.apply_mask(&mask);
            }

            if changed {
                highlights.push(house_idx.into());
                for d in digits.iter_ones() {
                    let d = DigitIndex::new(d);
                    for idx in indices.iter() {
                        if board[idx].contains(d) {
                            highlights.push((*idx, d).into());
                        }
                    }
                }
                return // do not process more than one naked group
            }
        }
    }
}

fn hidden_group(n: usize, board: &mut SudokuBoard, highlights: &mut Vec<Highlight>) {
    assert!(n < 9);
    for &house_idx in HouseIndex::domain() {
        let candidates = {
            
            // for each digit, get which cells in the house contain them
            let digit_cells = {
                let mut digit_cells = [SudokuFlags::default(); DigitIndex::COUNT];
                for (idx, cell) in board.house(house_idx).enumerate() {
                    for &d in DigitIndex::domain() {
                        digit_cells[d.value()].set(idx, cell.contains(d))
                    }
                }
                digit_cells
            };

            // Filter out digits that are not possible as they appear in more than n cells or they belong
            // to an already set cell (i.e. cell with a single digit).
            // Neither condition is strictly necessary, but they will reduce the search space.
            let digit_cells: Vec<_> = digit_cells.into_iter().enumerate()
                .filter(|(_, which_cells)| {
                    (which_cells.count_ones() <= n) && which_cells.iter_ones().all(|i| !board[house_idx.cell_index(i)].is_digit())
                })
                .map(|(d, which_cells)| {
                    let mut flags = SudokuFlags::ZERO;
                    flags.set(d, true);
                    (flags, which_cells)
                }).collect();

            if digit_cells.len() < n { continue }

            // merge cells into cell groups (n-1) times, so that we get all the hidden groups
            let mut candidates = vec![ (SudokuFlags::ZERO, SudokuFlags::ZERO) ];
            for _ in 0..n {
                candidates = candidates.into_iter().flat_map(|(digits, cells)|{
                    digit_cells.iter().filter_map(move |(d, c)|{
                        let new_digit = (digits & d) == SudokuFlags::ZERO;
                        let compatible = (cells | c).count_ones() <= n;
                        (new_digit && compatible).then_some((digits | d, cells | c))
                    })
                }).collect();
            }
            candidates
        };

        for (digits, cells) in candidates {
            let mask = DigitMask::new(digits);
            let mut changed = false;
            for cell_idx in cells.iter_ones() {
                let cell_idx = house_idx.cell_index(cell_idx);
                changed |= board[cell_idx].apply_mask(&mask);
            }

            if changed {
                highlights.push(house_idx.into());
                for d in digits.iter_ones() {
                    let d = DigitIndex::new(d);
                    for cell_idx in cells.iter_ones() {
                        let cell_idx = house_idx.cell_index(cell_idx);
                        if board[cell_idx].contains(d) {
                            highlights.push((cell_idx, d).into());
                        }
                    }
                }

                return // do not process more than one hidden group
            }
        }
    }
}

fn apply_strategy(s: Strategy, mut board: SudokuBoard) -> Result<(SudokuBoard, Vec<Highlight>), SudokuError> {
    let mut highlights = Vec::<Highlight>::new();
    match s {
        Strategy::Primaries => {
            let primary_cells: Vec<_> = board.indexed_iter().filter_map(|(cell_idx, cell)| {
                cell.digit_value().map(move |d| (cell_idx, d))
            }).collect();

            let mut rows_highlight = SudokuFlags::ZERO;
            let mut columns_highlight = SudokuFlags::ZERO;
            let mut blocks_highlight = SudokuFlags::ZERO;
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
                                HouseIndex::Block(b) => blocks_highlight.set(b.value(), true),
                                HouseIndex::Column(c) => columns_highlight.set(c.value(), true),
                                HouseIndex::Row(r) => rows_highlight.set(r.value(), true),
                            }
                        }
                    }
                }
                if relevant_digit {
                    highlights.push(primary.into());
                }
            }

            highlights.extend(
                rows_highlight.iter_ones()
                    .map(|i| { Highlight::from(HouseIndex::from(RowIndex::new(i))) })
            );
            highlights.extend(
                columns_highlight.iter_ones()
                    .map(|i| { Highlight::from(HouseIndex::from(ColumnIndex::new(i))) })
            );
            highlights.extend(
                blocks_highlight.iter_ones()
                    .map(|i| { Highlight::from(HouseIndex::from(BlockIndex::new(i))) })
            );
        }
        Strategy::HiddenSingle => {
            hidden_group(1, &mut board, &mut highlights)
        }
        Strategy::NakedPair => { 
            naked_group(2, &mut board, &mut highlights);
        },
        Strategy::HiddenPair => {
            hidden_group(2, &mut board, &mut highlights);
        },
        Strategy::LockedCandidatePointing => {
            'outer: for b in BlockIndex::domain() {
                for d in DigitIndex::domain() {
                    let mut rows: Vec<_> = Vec::new();
                    let mut columns: Vec<_> = Vec::new();

                    for cell_idx in b.cell_indices() {
                        if board[cell_idx].contains(*d) {
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
                        for idx in claiming_house.cell_indices() {
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
                    let cells: Vec<_> = h.cell_indices().iter().cloned().filter(|idx|{
                        board[idx].contains(*d)
                    }).collect();
                    let same_block = cells.iter().map(CellIndex::block).all_equal();
                    if cells.is_empty() || !same_block { continue }
                    let block = cells.first().unwrap().block();
                    
                    let mask = DigitMask::all_but(*d);
                    let mut changed = false;
                    for idx in block.cell_indices() {
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
        Strategy::NakedTriple => {
            naked_group(3, &mut board, &mut highlights);
        },
        Strategy::HiddenTriple => {
            hidden_group(3, &mut board, &mut highlights);
        },
        Strategy::NakedQuad => {
            naked_group(4, &mut board, &mut highlights);
        },
        Strategy::HiddenQuad => {
            hidden_group(4, &mut board, &mut highlights);
        },
        Strategy::XWing => {
            'strategy: for &d in DigitIndex::domain() {
                for search_houses in [
                            (*ColumnIndex::domain()).map(HouseIndex::from), 
                            (*RowIndex::domain()).map(HouseIndex::from)
                        ] {
                    let appear_in: Vec<_> = search_houses.iter().map(|&idx|{
                            let appear = board.house(idx).enumerate().fold(SudokuFlags::ZERO, |mut acc, (flat_idx, cell_value)|{
                                acc.set(flat_idx, cell_value.contains(d));
                                acc
                            });
                            (idx, appear)
                        })
                        .filter(|(_, appear)| appear.count_ones() == 2)
                        .collect();
                    let candidate_pairs = appear_in.into_iter().combinations(2).filter(|pair| pair[0].1 == pair[1].1);
                    for candidate in candidate_pairs {
                        let h0 = candidate[0].0;
                        let h1 = candidate[1].0;
                        let appear = candidate[0].1;

                        let mut changed = false;
                        let mask: DigitMask = DigitMask::all_but(d);
                        for i in appear.iter_ones() {
                            for h in [h0.crossed_by(i).unwrap(), h1.crossed_by(i).unwrap()] {
                                for (cell_idx, cell) in board.indexed_house_mut(h) {
                                    if h0.contains(cell_idx) || h1.contains(cell_idx) { continue }
                                    changed |= cell.apply_mask(&mask);
                                }
                            }
                        }

                        if changed {
                            highlights.push(h0.into());
                            highlights.push(h1.into());
                            for i in appear.iter_ones() {
                                highlights.push(h0.crossed_by(i).unwrap().into());
                                highlights.push(h1.crossed_by(i).unwrap().into());
                                highlights.push((h0.cell_index(i), d).into());
                                highlights.push((h1.cell_index(i), d).into());
                            }
                            break 'strategy
                        }
                    }
                }
            }
        },
        Strategy::RemotePair => {
            let bv_cells: Vec<_> = CellIndex::domain().filter(|idx| board[idx].num_digits() == 2)
                .into_group_map_by(|idx| board[idx].digits())
                .into_iter()
                .filter(|(_, v)| v.len() >= 3)
                .collect()
                ;
            'strategy: for (digits, cells) in bv_cells {
                let mut graph: Vec<Vec<usize>> = vec![Vec::new(); cells.len()];
                let mut ut = InPlaceUnificationTable::<IntKey>::new();
                let keys: Vec<_> = cells.iter().map(|_| ut.new_key(())).collect();
                for i in 0..cells.len() {
                    for j in (i+1)..cells.len() {
                        if cells[i].share_house(&cells[j]) {
                            ut.union(keys[i], keys[j]);
                            graph[i].push(j);
                            graph[j].push(i);
                        }
                    }
                }

                let mut processed_groups = Vec::new();
                for key in keys {
                    let root = ut.find(key);
                    if processed_groups.contains(&root) { continue }
                    processed_groups.push(root);

                    let root = root.index() as usize;

                    // coloring the cells starting from root
                    let mut red_group = Vec::new();
                    let mut black_group = Vec::new();
                    
                    let mut visited = vec![false; cells.len()];
                    let mut stack = vec![(0usize, root)];
                    while let Some((dist, idx)) = stack.pop() {
                        if visited[idx] { continue }
                        visited[idx] = true;

                        if dist.is_multiple_of(2) {
                            red_group.push(idx);
                        } else {
                            black_group.push(idx);
                        }
                        for &other in graph[idx].iter() {
                            stack.push((dist + 1, other));
                        }
                    }

                    // all cells in red_group are locked pairs
                    // all cells in black_group are locked pairs
                    // check for each pair in red_group and pair in black_group, whether
                    // there is a cell which is visible to both of them and contain
                    // the digits
                    let mask = DigitMask::new(!digits);
                    for pairs in red_group.iter().combinations(2).chain(black_group.iter().combinations(2)) {
                        let c0 = cells[*pairs[0]];
                        let c1 = cells[*pairs[1]];
                        
                        let mut changed_cells = Vec::new();
                        for c in c0.visible_with(&c1) {
                            if board[c].would_change(&mask) {
                                changed_cells.push(c);
                            }
                        }

                        if !changed_cells.is_empty() {
                            for c in changed_cells.iter() {
                                board[c].apply_mask(&mask);
                            //     highlights.extend(
                            //         c.shared_houses(&c0).into_iter().map(Highlight::from)
                            //     );
                            //     highlights.extend(
                            //         c.shared_houses(&c1).into_iter().map(Highlight::from)
                            //     );
                            }
                            for d in digits.iter_ones() {
                                let d = DigitIndex::new(d);
                                highlights.push((c0, d).into());
                                highlights.push((c1, d).into());
                            }

                            // find shortest chain between c0 and c1
                            let mut visited = vec![false; cells.len()];
                            let mut queue = VecDeque::new();
                            queue.push_back(vec![*pairs[0]]);
                            let shortest_chain = loop {
                                let path = queue.pop_front().unwrap();
                                let idx = *path.last().unwrap();
                                if idx == *pairs[1] {
                                    break path
                                }

                                if visited[idx] { continue }
                                visited[idx] = true;

                                for &other in &graph[idx] {
                                    let mut path = path.clone();
                                    path.push(other);
                                    queue.push_back(path);
                                }
                            };

                            for idx in shortest_chain.into_iter().map(|i| cells[i]) {
                                highlights.push(idx.into());
                            }
                            
                            break 'strategy
                        }
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