use std::ops::ControlFlow;

use super::{Strategy, solve_with_strategies};

use crate::flags::DigitFlags;
use crate::board::SudokuBoard;
use crate::highlight::Highlight;

pub fn solve_backtrack(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
    let strategies: Vec<_> = Strategy::safe_strategies().iter().copied()
        .filter(|s| *s != Strategy::Backtrack).collect();

    let mut stack = vec![board.clone()];
    while let Some(mut next) = stack.pop() {
        solve_with_strategies(&mut next, &strategies);
        if next.is_solved() { *board = next; return ControlFlow::Break(Vec::new()) }
        if !next.is_valid() { continue }

        if let Some((cell_idx, cell)) = next.indexed_iter()
            .filter(|(_, cell)| !cell.is_digit())
            .min_by_key(|(_, cell)| cell.num_digits())
        {
            for digit in cell.digits() {
                let mut next = next.clone();
                next[cell_idx].apply_mask(&DigitFlags::only(digit));
                stack.push(next);
            }
        }
    }

    ControlFlow::Continue(())
}