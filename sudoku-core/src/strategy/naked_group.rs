use std::ops::ControlFlow;

use itertools::Itertools;

use crate::board::{SudokuBoard, SudokuBoardIter};
use crate::index::HouseIndex;
use crate::highlight::Highlight;

pub(super) fn apply_naked_group(board: &mut SudokuBoard, n: usize) -> ControlFlow<Vec<Highlight>> {
    assert!(1 < n && n < 9);
    for house_idx in HouseIndex::iter() {
        let candidate_cells: Vec::<_> = board.indexed_region(&house_idx).filter_map(|(idx, cell)| {
            let digits = cell.num_digits();
            (digits > 1 && digits <= n).then_some(idx)
        }).collect();

        let candidate_groups: Vec<_> = candidate_cells.into_iter().combinations(n)
            .filter_map(|indices|{
                let digits = indices.iter().cloned().digits(board);
                (digits.count() == n).then_some((digits, indices))
            }).collect();

        for (digits, indices) in candidate_groups {
            let mut changed: bool = false;
            let mask = !digits;
            for (idx, cell) in board.indexed_region_mut(&house_idx) {
                if indices.contains(&idx) { continue }
                changed |= cell.apply_mask(&mask);
            }

            if changed {
                let mut highlights = Vec::new();
                highlights.push(house_idx.into());
                for digit in digits.iter() {
                    for idx in indices.iter() {
                        if board[idx].contains(digit) {
                            highlights.push((*idx, digit).into());
                        }
                    }
                }
                return ControlFlow::Break(highlights)
            }
        }
    }
    ControlFlow::Continue(())
}
