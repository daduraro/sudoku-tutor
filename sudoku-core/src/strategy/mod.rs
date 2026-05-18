use std::ops::ControlFlow;

use strum::IntoEnumIterator;
use strum::{EnumCount, EnumIter};

use crate::board::SudokuBoard;
use crate::error::SudokuError;
use crate::highlight::Highlight;

mod backtrack;
mod chute_remote_pair;
mod common;
mod hidden_group;
mod locked_candidates_claiming;
mod locked_candidates_pointing;
mod naked_group;
mod primaries;
// mod remote_pairs;
mod simple_coloring;
mod xwing;

use backtrack::solve_backtrack;
use chute_remote_pair::{CRPType, apply_chute_remote_pair};
use hidden_group::apply_hidden_group;
use locked_candidates_claiming::apply_locked_candidates_claiming;
use locked_candidates_pointing::apply_locked_candidates_pointing;
use naked_group::apply_naked_group;
use primaries::apply_primaries;
use xwing::apply_xwing;

fn apply_strategies(
    board: &mut SudokuBoard,
    strategies: &[Strategy],
) -> ControlFlow<(Strategy, Vec<Highlight>)> {
    strategies
        .iter()
        .try_for_each(|s| s.apply(board).map_break(|h| (*s, h)))
}

fn solve_with_strategies(board: &mut SudokuBoard, strategies: &[Strategy]) {
    while !board.is_finished() && apply_strategies(board, strategies).is_break() {}
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
    // RemotePair, // deprecated in favor of SimpleColoring
    SimpleColoring,

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

    // solve using backtracking
    Backtrack,
}

impl Strategy {
    fn iter_no_backtrack() -> impl Iterator<Item = Self> {
        Self::iter().filter(|s| *s != Strategy::Backtrack)
    }

    fn safe_strategies() -> Vec<Strategy> {
        Strategy::iter().filter(Strategy::is_safe).collect()
    }

    pub const fn is_safe(&self) -> bool {
        match self {
            Strategy::Primaries
            | Strategy::HiddenSingle
            | Strategy::NakedPair
            | Strategy::HiddenPair
            | Strategy::LockedCandidatePointing
            | Strategy::LockedCandidateClaiming
            | Strategy::XWing
            | Strategy::ChuteRemotePairDouble
            | Strategy::ChuteRemotePairSingle
            | Strategy::NakedTriple
            | Strategy::HiddenTriple
            | Strategy::NakedQuad
            | Strategy::HiddenQuad
            // | Strategy::RemotePair
            | Strategy::SimpleColoring
            | Strategy::Backtrack => true,
        }
    }

    pub fn apply(&self, board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
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
            // Strategy::RemotePair => apply_remote_pairs(board),
            Strategy::ChuteRemotePairDouble => apply_chute_remote_pair(board, CRPType::Double),
            Strategy::ChuteRemotePairSingle => apply_chute_remote_pair(board, CRPType::Single),
            Strategy::SimpleColoring => simple_coloring::apply(board),
            Strategy::Backtrack => solve_backtrack(board),
        }
    }
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

pub fn solve(mut board: SudokuBoard) -> Result<SolvedGame, SudokuError> {
    let mut boards = Vec::<SudokuBoard>::new();
    let mut steps = Vec::<(Strategy, Vec<Highlight>)>::new();

    let strategies: Vec<_> = Strategy::iter_no_backtrack().collect();
    while !board.is_finished() {
        if !board.is_valid() {
            return Err(SudokuError::UnsolvableSudoku);
        }
        let mut next = board.clone();
        if let ControlFlow::Break(step) = apply_strategies(&mut next, &strategies) {
            steps.push(step);
            boards.push(board);
            board = next;
        } else {
            // we did not advance
            debug_assert_eq!(board, next);
            break;
        }
    }
    // assert!(solve_backtrack(board.clone()).is_some());

    boards.push(board);

    let strategies: Vec<_> = steps
        .iter()
        .fold(vec![false; Strategy::COUNT], |mut acc, (strat, _)| {
            acc[*strat as usize] = true;
            acc
        })
        .into_iter()
        .zip(Strategy::iter())
        .filter_map(|(b, strat)| if b { Some(strat) } else { None })
        .collect();

    Ok(SolvedGame {
        boards,
        steps,
        strategies,
    })
}
