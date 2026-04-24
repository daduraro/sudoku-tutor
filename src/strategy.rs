use itertools::Itertools;
use strum::{EnumIter, EnumCount};
use strum::IntoEnumIterator;

use crate::board::{SudokuFlags, SudokuBoard, DigitMask};
use crate::index::{BlockIndex, CellIndex, ChuteIndex, ColumnIndex, DigitIndex, HouseIndex, SudokuRegion, RowIndex, RegionIntersection, LineDirection};
use crate::error::SudokuError;
use crate::display::Highlight;
use crate::graph::Graph;

fn solve_with_strategies(board: &mut SudokuBoard, strategies: &[Strategy]) {
    'step: loop {
        for strategy in strategies {
            if matches!(strategy.apply(board), StrategyResult::Advanced(_)) { continue 'step }
        }
        break
    }
}

fn solve_backtrack(board: SudokuBoard) -> Option<SudokuBoard> {
    const SAFE_STRATEGIES: &[Strategy] = &[
        Strategy::Primaries,
        Strategy::HiddenSingle,
        Strategy::NakedPair,
        Strategy::HiddenPair,
        Strategy::LockedCandidatePointing,
        Strategy::LockedCandidateClaiming,
    ];

    let mut stack = vec![board];
    while let Some(mut board) = stack.pop() {
        solve_with_strategies(&mut board, SAFE_STRATEGIES);
        if board.is_solved() { return Some(board) }
        if !board.is_valid() { continue }
        for (cell_idx, cell) in board.indexed_iter() {
            if cell.is_digit() { continue }
            for digit in cell.digits() {
                let mut next = board.clone();
                next[cell_idx].apply_mask(&DigitMask::only(digit));
                stack.push(next);
            }
        }
    }
    None
}

#[derive(Clone, Debug)]
pub enum StrategyResult {
    Unaffected,
    Advanced(Vec<Highlight>),
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

    ChuteRemotePairDouble,
    ChuteRemotePairSingle,

    // Cells with exactly same two candidates may form a link of locked/complementary pairs.
    // In particular, this chain link will make pairs in an odd distance to
    // be locked pairs themselves, and so any cell that sees both cannot have
    // either candidate.
    RemotePair,

    // ColoringType1,
    // ColoringType2,
    // TurbotSkyscraper,
    // Turbot2StringKate,
    // TurbotCrane,
    // EmptyRectangle,
    // Swordfish,
    // XYWing,
    // XYZWing,
    // XChain,
    // XChainLoop,
    // XChainOneEndpoint,
    // XYChain,
    // XYChainLoop,
    // BUG,
    // UniqueRectangleType1,
    // UniqueRectangleType2,
    // UniqueRectangleType3,
    // UniqueRectangleType4,
    // UniqueRectangleType5,
    // UniqueRectangleType6,
    // UniqueRectangleType7,
    // Medusa,
    // WXYZWing,
    // Starfish,
    // Balena,
    // Leviathan,
}

impl Strategy {
    pub fn apply(&self, board: &mut SudokuBoard) -> StrategyResult {
        match self {
            Strategy::Primaries => apply_primaries(board),
            Strategy::HiddenSingle => apply_hidden_group(board, 1),
            Strategy::NakedPair => apply_naked_group(board, 2),
            Strategy::HiddenPair => apply_hidden_group(board, 2),
            Strategy::LockedCandidatePointing => apply_locked_candidates_pointing(board),
            Strategy::LockedCandidateClaiming => apply_locked_candidates_claiming(board),
            Strategy::NakedTriple => apply_naked_group(board, 3),
            Strategy::HiddenTriple => apply_hidden_group(board, 3),
            Strategy::NakedQuad => apply_naked_group(board, 4),
            Strategy::HiddenQuad => apply_hidden_group(board, 4),
            Strategy::XWing => apply_xwing(board),
            Strategy::RemotePair => apply_remote_pairs(board),
            Strategy::ChuteRemotePairDouble => apply_chute_remote_pair(board, CRPType::Double),
            Strategy::ChuteRemotePairSingle => apply_chute_remote_pair(board, CRPType::Single),
        }
    }
}

fn apply_primaries(board: &mut SudokuBoard) -> StrategyResult {
    let primary_cells: Vec<_> = board.indexed_iter().filter_map(|(cell_idx, cell)| {
        cell.digit_value().map(move |d| (cell_idx, d))
    }).collect();

    let mut rows_highlight = SudokuFlags::ZERO;
    let mut columns_highlight = SudokuFlags::ZERO;
    let mut blocks_highlight = SudokuFlags::ZERO;

    let mut highlights = Vec::new();
    for primary in primary_cells {
        let (primary_cell_idx, d) = primary;
        let mut relevant_digit = false;

        let mask = DigitMask::all_but(d);
        for house in primary_cell_idx.houses() {
            for (cell_idx, cell) in board.indexed_region_mut(&house) {
                if cell_idx == primary_cell_idx { continue }
                if cell.apply_mask(&mask) {
                    relevant_digit = true;
                    match house {
                        HouseIndex::Block(b) => blocks_highlight.set(b.flat_index(), true),
                        HouseIndex::Column(c) => columns_highlight.set(c.index(), true),
                        HouseIndex::Row(r) => rows_highlight.set(r.index(), true),
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
            .map(|i| { Highlight::from(HouseIndex::from(BlockIndex::from_flat_index(i))) })
    );
    if !highlights.is_empty() {
        StrategyResult::Advanced(highlights)
    } else {
        StrategyResult::Unaffected
    }
}

fn apply_naked_group(board: &mut SudokuBoard, n: usize) -> StrategyResult {
    assert!(1 < n && n < 9);
    for house_idx in HouseIndex::iter() {
        let candidate_cells: Vec::<_> = board.indexed_region(&house_idx).filter_map(|(idx, cell)| {
            let digits = cell.num_digits();
            (digits > 1 && digits <= n).then_some(idx)
        }).collect();

        let candidate_groups: Vec<_> = candidate_cells.into_iter().combinations(n)
            .filter_map(|indices|{
                let digits = indices.iter().map(|idx| board[*idx])
                    .fold(SudokuFlags::ZERO, |acc, d| acc | d.digit_flags());
                (digits.count_ones() == n).then_some((digits, indices))
            }).collect();

        for (digits, indices) in candidate_groups {
            let mut changed: bool = false;
            let mask = DigitMask::new(!digits);
            for (idx, cell) in board.indexed_region_mut(&house_idx) {
                if indices.contains(&idx) { continue }
                changed |= cell.apply_mask(&mask);
            }

            if changed {
                let mut highlights = Vec::new();
                highlights.push(house_idx.into());
                for d in digits.iter_ones() {
                    let d = DigitIndex::new(d);
                    for idx in indices.iter() {
                        if board[idx].contains(d) {
                            highlights.push((*idx, d).into());
                        }
                    }
                }
                return StrategyResult::Advanced(highlights) // do not process more than one naked group
            }
        }
    }
    StrategyResult::Unaffected
}

fn apply_hidden_group(board: &mut SudokuBoard, n: usize) -> StrategyResult {
    assert!(n < 9);
    for house_idx in HouseIndex::iter() {
        let candidates = {

            // for each digit, get which cells in the house contain them
            let digit_cells = {
                let mut digit_cells = [SudokuFlags::default(); DigitIndex::COUNT];
                for (idx, cell) in board.region(&house_idx).enumerate() {
                    for digit in DigitIndex::iter() {
                        digit_cells[digit.index()].set(idx, cell.contains(digit))
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
                let mut highlights = Vec::new();
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

                return StrategyResult::Advanced(highlights) // do not process more than one hidden group
            }
        }
    }
    StrategyResult::Unaffected
}

fn apply_locked_candidates_pointing(board: &mut SudokuBoard) -> StrategyResult {
    for block in BlockIndex::iter() {
        for digit in DigitIndex::iter() {
            let mut rows: Vec<_> = Vec::new();
            let mut columns: Vec<_> = Vec::new();

            for cell_idx in block.cell_indices() {
                if board[cell_idx].contains(digit) {
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
                let mask = DigitMask::all_but(digit);
                let mut changed = false;
                for idx in claiming_house.cell_indices() {
                    if idx.block() == block { continue }
                    changed |= board[idx].apply_mask(&mask)
                }

                if changed {
                    let mut highlights = Vec::new();
                    highlights.push(claiming_house.into());
                    highlights.push(block.into());
                    highlights.extend(cells.into_iter().map(|c| Highlight::Digit((c, digit))));
                    return StrategyResult::Advanced(highlights)
                }
            }
        }
    }
    StrategyResult::Unaffected
}

fn apply_locked_candidates_claiming(board: &mut SudokuBoard) -> StrategyResult {
    for house_idx in HouseIndex::rows_and_columns() {
        for digit in DigitIndex::iter() {
            let cells: Vec<_> = house_idx.cell_indices().filter(|idx|{
                board[idx].contains(digit)
            }).collect();
            let same_block = cells.iter().map(CellIndex::block).all_equal();
            if cells.is_empty() || !same_block { continue }
            let block = cells.first().unwrap().block();

            let mask = DigitMask::all_but(digit);
            let mut changed = false;
            for idx in block.cell_indices() {
                if house_idx.contains(idx) { continue }
                changed |= board[idx].apply_mask(&mask);
            }

            if changed {
                let mut highlights = Vec::new();
                highlights.push(house_idx.into());
                highlights.push(block.into());
                highlights.extend(cells.into_iter().map(|idx| Highlight::Digit((idx, digit))));
                return StrategyResult::Advanced(highlights)
            }
        }
    }
    StrategyResult::Unaffected
}

fn apply_xwing(board: &mut SudokuBoard) -> StrategyResult {
    for d in DigitIndex::iter() {
        for search_direction in LineDirection::iter() {
            let search_houses = search_direction.lines();
            let perpendicular_direction = search_direction.other();
            let appear_in: Vec<_> = search_houses.into_iter().map(|idx|{
                    let appear = board.region(&idx).enumerate().fold(SudokuFlags::ZERO, |mut acc, (flat_idx, cell_value)|{
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
                    for h in [
                                h0.cell_index(i).line(perpendicular_direction),
                                h1.cell_index(i).line(perpendicular_direction),
                            ] {
                        for (cell_idx, cell) in board.indexed_region_mut(&h) {
                            if h0.contains(cell_idx) || h1.contains(cell_idx) { continue }
                            changed |= cell.apply_mask(&mask);
                        }
                    }
                }

                if changed {
                    let mut highlights = Vec::new();
                    highlights.push(h0.into());
                    highlights.push(h1.into());
                    for i in appear.iter_ones() {
                        highlights.push(h0.cell_index(i).line(perpendicular_direction).into());
                        highlights.push(h1.cell_index(i).line(perpendicular_direction).into());
                        highlights.push((h0.cell_index(i), d).into());
                        highlights.push((h1.cell_index(i), d).into());
                    }
                    return StrategyResult::Advanced(highlights)
                }
            }
        }
    }
    StrategyResult::Unaffected
}

fn visibility_graphs(indices: &[CellIndex]) -> Vec<Graph<CellIndex>> {
    let mut graph = Graph::new(indices.to_vec(), Vec::new());
    for i in 0..graph.len() {
        for j in (i+1)..graph.len() {
            if graph[i].visible(&graph[j]) {
                graph.add_edge(i, j);
            }
        }
    }

    graph.split_connected_components()
}

fn apply_remote_pairs(board: &mut SudokuBoard) -> StrategyResult {
    let bv_cells = CellIndex::iter().filter(|idx| board[idx].is_bivalue());

    let bv_cells_groups: Vec<_> = bv_cells
        .into_group_map_by(|idx| board[idx])
        .into_iter()
        .filter(|(_, v)| v.len() >= 3)
        .collect()
        ;
    for (cell_value, cells) in bv_cells_groups {
        debug_assert!(cells.iter().all(|idx| board[idx] == cell_value));

        let mask = DigitMask::new(!cell_value.digit_flags());
        for graph in visibility_graphs(&cells) {
            if let Some((group_a, group_b)) = graph.two_colorize() {
                for (&i, j) in group_a.iter().cartesian_product(group_b) {
                    let c0 = graph[i];
                    let c1 = graph[j];

                    let mut changed = false;
                    for c in c0.cells_visible_with(&c1) {
                        changed |= board[c].apply_mask(&mask);
                    }
                    if changed {
                        let mut highlights = Vec::new();
                        for digit in cell_value.digits() {
                            highlights.push((c0, digit).into());
                            highlights.push((c1, digit).into());
                        }

                        for idx in graph.shortest_chain(i, j).unwrap() {
                            highlights.push(graph[idx].into());
                        }

                        return StrategyResult::Advanced(highlights)
                    }
                }
            }
        }
    }

    StrategyResult::Unaffected
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum CRPType {
    Single,
    Double,
}

fn apply_chute_remote_pair(board: &mut SudokuBoard, crp_type: CRPType) -> StrategyResult {
    for chute in ChuteIndex::iter() {
        let direction = chute.direction();

        let bv_cells: Vec<_> = board.indexed_region(&chute).filter(|(_, cell)| cell.is_bivalue()).map(|(idx,_)| idx).collect();
        for indices in bv_cells.iter().combinations(2) {
            let cell_0 = *indices[0];
            let cell_1 = *indices[1];

            if board[cell_0] != board[cell_1] { continue } // they do not share bivalue
            if cell_0.visible(&cell_1) { continue } // naked pairs, ignore them

            let cell = board[cell_0];

            let line_0 = cell_0.line(direction);
            let line_1 = cell_1.line(direction);
            let line_other = chute.lines().into_iter().find(|i| i != &line_0 && i != &line_1).unwrap();

            let block_0 = cell_0.block();
            let block_1 = cell_1.block();
            let block_other = chute.blocks().into_iter().find(|i| i != &block_0 && i != &block_1).unwrap();

            let digits_in_other = line_other.intersect(&block_other).into_iter()
                    .map(|cell_idx| board[cell_idx].digit_flags())
                    .fold(SudokuFlags::ZERO, |acc, flags| acc | flags)
                & cell.digit_flags();

            let mut changed = false;
            if digits_in_other.count_ones() == 1 && crp_type == CRPType::Single {
                // chute remote pair (single)
                let mask = DigitMask::new(!digits_in_other);
                let roi = block_0.intersect(&line_1).into_iter().chain(block_1.intersect(&line_0));
                for cell_idx in roi {
                    changed |= board[cell_idx].apply_mask(&mask);
                }
            } else if digits_in_other.count_ones() == 0 && crp_type == CRPType::Double {
                // chute remote pair (double)
                let mask = DigitMask::new(!cell.digit_flags());
                let roi = line_0.cell_indices().filter(|idx| idx != &cell_0 && idx.block() != block_other)
                    .chain(line_1.cell_indices().filter(|idx| idx != &cell_1 && idx.block() != block_other));
                for cell_idx in roi {
                    changed |= board[cell_idx].apply_mask(&mask);
                }
            }

            if changed {
                let mut highlights = Vec::new();
                for digit in cell.digits() {
                    highlights.push((cell_0, digit).into());
                    highlights.push((cell_1, digit).into());
                }
                for digit in digits_in_other.iter_ones().map(DigitIndex::new) {
                    for cell in line_other.intersect(&block_other) {
                        highlights.push((cell, digit).into());
                    }
                }
                highlights.extend(line_other.intersect(&block_other).into_iter().map(Highlight::from));
                return StrategyResult::Advanced(highlights)
            }
        }
    }
    StrategyResult::Unaffected
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

pub fn solve(mut board: SudokuBoard) -> Result<SolvedGame, SudokuError>
{
    let mut boards = Vec::<SudokuBoard>::new();
    let mut steps = Vec::<(Strategy, Vec<Highlight>)>::new();

    'step: while !board.is_solved() {
        if !board.is_valid() { return Err(SudokuError::UnsolvableSudoku) }
        let mut next = board.clone();
        for s in Strategy::iter() {
            if let StrategyResult::Advanced(highlights) = s.apply(&mut next) {
                steps.push((s, highlights));
                boards.push(board);
                board = next;
                continue 'step
            } else {
                debug_assert_eq!(board, next);
            }
        }

        // we did not advance
        break
    }
    // assert!(solve_backtrack(board.clone()).is_some());

    boards.push(board);

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