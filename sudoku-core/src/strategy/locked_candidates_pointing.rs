use std::ops::ControlFlow;

use crate::board::SudokuBoard;
use crate::flags::DigitFlags;
use crate::index::{BlockIndex, CellIndex, DigitIndex, HouseIndex, SudokuRegion};
use crate::highlight::Highlight;

pub fn apply_locked_candidates_pointing(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
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
                let mask = DigitFlags::all_but(digit);
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
                    return ControlFlow::Break(highlights)
                }
            }
        }
    }
    ControlFlow::Continue(())
}
