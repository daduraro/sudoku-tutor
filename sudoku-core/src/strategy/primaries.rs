use std::ops::ControlFlow;

use crate::board::SudokuBoard;
use crate::flags::{ColumnFlags, RowFlags, BlockFlags, DigitFlags};
use crate::index::HouseIndex;
use crate::highlight::Highlight;

pub fn apply_primaries(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
    let primary_cells: Vec<_> = board.primaries().collect();

    let mut rows_highlight = RowFlags::ZERO;
    let mut columns_highlight = ColumnFlags::ZERO;
    let mut blocks_highlight = BlockFlags::ZERO;

    let mut highlights = Vec::new();
    for (primary_cell_idx, digit) in primary_cells {
        let mut relevant_digit = false;

        let mask = DigitFlags::all_but(digit);
        for house in primary_cell_idx.houses() {
            for (cell_idx, cell) in board.indexed_region_mut(&house) {
                if cell_idx == primary_cell_idx { continue }
                if cell.apply_mask(&mask) {
                    relevant_digit = true;
                    match house {
                        HouseIndex::Block(b) => blocks_highlight += b,
                        HouseIndex::Column(c) => columns_highlight += c,
                        HouseIndex::Row(r) => rows_highlight += r,
                    }
                }
            }
        }
        if relevant_digit {
            highlights.push((primary_cell_idx, digit).into());
        }
    }

    highlights.extend(
        rows_highlight.iter()
            .map(|i| { Highlight::from(HouseIndex::from(i)) })
    );
    highlights.extend(
        columns_highlight.iter()
            .map(|i| { Highlight::from(HouseIndex::from(i)) })
    );
    highlights.extend(
        blocks_highlight.iter()
            .map(|i| { Highlight::from(HouseIndex::from(i)) })
    );
    if !highlights.is_empty() {
        ControlFlow::Break(highlights)
    } else {
        ControlFlow::Continue(())
    }
}
