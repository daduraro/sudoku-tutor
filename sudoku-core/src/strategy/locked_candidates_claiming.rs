use std::ops::ControlFlow;

use itertools::Itertools;
use crate::board::SudokuBoard;
use crate::flags::DigitFlags;
use crate::index::{CellIndex, DigitIndex, HouseIndex, SudokuRegion};
use crate::highlight::Highlight;

pub fn apply_locked_candidates_claiming(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
    for house_idx in HouseIndex::rows_and_columns() {
        for digit in DigitIndex::iter() {
            let cells: Vec<_> = house_idx.cell_indices().filter(|idx|{
                board[idx].contains(digit)
            }).collect();
            let same_block = cells.iter().map(CellIndex::block).all_equal();
            if cells.is_empty() || !same_block { continue }
            let block = cells.first().unwrap().block();

            let mask = DigitFlags::all_but(digit);
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
                return ControlFlow::Break(highlights)
            }
        }
    }
    ControlFlow::Continue(())
}
