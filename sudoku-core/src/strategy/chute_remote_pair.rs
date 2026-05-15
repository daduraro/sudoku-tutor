use std::ops::ControlFlow;

use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::board::SudokuBoard;
use crate::flags::DigitFlags;
use crate::index::{ChuteIndex, RegionIntersection, SudokuRegion};
use crate::highlight::Highlight;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CRPType {
    Single,
    Double,
}

pub fn apply_chute_remote_pair(board: &mut SudokuBoard, crp_type: CRPType) -> ControlFlow<Vec<Highlight>> {
    for chute in ChuteIndex::iter() {
        let direction = chute.direction();

        let bv_cells: Vec<_> = board.indexed_region(&chute).filter(|(_, cell)| cell.is_bivalue()).map(|(idx,_)| idx).collect();
        for indices in bv_cells.iter().combinations(2) {
            let cell_0 = *indices[0];
            let cell_1 = *indices[1];

            if board[cell_0] != board[cell_1] { continue }
            if cell_0.visible(&cell_1) { continue }

            let cell = board[cell_0];

            let line_0 = cell_0.line(direction);
            let line_1 = cell_1.line(direction);
            let line_other = chute.lines().into_iter().find(|i| i != &line_0 && i != &line_1).unwrap();

            let block_0 = cell_0.block();
            let block_1 = cell_1.block();
            let block_other = chute.blocks().into_iter().find(|i| i != &block_0 && i != &block_1).unwrap();

            let digits_in_other = line_other.intersect(&block_other).into_iter()
                    .map(|cell_idx| board[cell_idx].digit_flags())
                    .fold(DigitFlags::ZERO, |acc, flags| acc | flags)
                & cell.digit_flags();

            let mut changed = false;
            if digits_in_other.count() == 1 && crp_type == CRPType::Single {
                let mask = !digits_in_other;
                let roi = block_0.intersect(&line_1).into_iter().chain(block_1.intersect(&line_0));
                for cell_idx in roi {
                    changed |= board[cell_idx].apply_mask(&mask);
                }
            } else if digits_in_other.count() == 0 && crp_type == CRPType::Double {
                let mask = !cell.digit_flags();
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
                for digit in digits_in_other.iter() {
                    for cell in line_other.intersect(&block_other) {
                        highlights.push((cell, digit).into());
                    }
                }
                highlights.extend(line_other.intersect(&block_other).into_iter().map(Highlight::from));
                return ControlFlow::Break(highlights)
            }
        }
    }
    ControlFlow::Continue(())
}
