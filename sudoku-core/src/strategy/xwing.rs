use std::ops::{Add, ControlFlow};

use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::board::SudokuBoard;
use crate::flags::{DigitFlags, SudokuFlags};
use crate::index::{DigitIndex, HouseRegion, LineDirection, SudokuIndex, SudokuRegion};
use crate::highlight::Highlight;

pub(super) fn apply_xwing(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
    for digit in DigitIndex::iter() {
        for search_direction in LineDirection::iter() {
            let search_houses = search_direction.lines();
            let perpendicular_direction = search_direction.other();
            let appear_in: Vec<_> = search_houses.into_iter().map(|house_idx|{
                    let appear = board.region(&house_idx).enumerate()
                        .filter(|(_, cell_value)| cell_value.contains(digit))
                        .map(|(idx, _)| SudokuIndex::new(idx).unwrap())
                        .fold(SudokuFlags::ZERO, SudokuFlags::add);
                    (house_idx, appear)
                })
                .filter(|(_, appear)| appear.count() == 2)
                .collect();
            let candidate_pairs = appear_in.into_iter().combinations(2).filter(|pair| pair[0].1 == pair[1].1);
            for candidate in candidate_pairs {
                let h0 = candidate[0].0;
                let h1 = candidate[1].0;
                let appear = candidate[0].1;

                let mut changed = false;
                let mask = DigitFlags::all_but(digit);
                for i in appear.iter() {
                    for h in [
                                h0.get(i).line(perpendicular_direction),
                                h1.get(i).line(perpendicular_direction),
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
                    for i in appear.iter() {
                        highlights.push(h0.get(i).line(perpendicular_direction).into());
                        highlights.push(h1.get(i).line(perpendicular_direction).into());
                        highlights.push((h0.get(i), digit).into());
                        highlights.push((h1.get(i), digit).into());
                    }
                    return ControlFlow::Break(highlights)
                }
            }
        }
    }
    ControlFlow::Continue(())
}
