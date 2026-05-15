use std::ops::ControlFlow;

use crate::board::SudokuBoard;
use crate::flags::{DigitFlags, SudokuFlags};
use crate::index::{DigitIndex, HouseIndex, HouseRegion};
use crate::highlight::Highlight;

pub(super) fn apply_hidden_group(board: &mut SudokuBoard, n: usize) -> ControlFlow<Vec<Highlight>> {
    assert!(n < 9);
    for house_idx in HouseIndex::iter() {
        let candidates = {

            // for each digit, get which cells in the house contain them
            let digit_cells: Vec<_> = DigitIndex::iter()
                .map(|digit|{ board.cells_with(&house_idx, digit) })
                .collect();

            // Filter out digits that are not possible as they appear in more than n cells or they belong
            // to an already set cell (i.e. cell with a single digit).
            // Neither condition is strictly necessary, but they will reduce the search space.
            let digit_cells: Vec<_> = digit_cells.into_iter().enumerate()
                .filter(|(_, which_cells)| {
                    (which_cells.count() <= n) && which_cells.iter().all(|i| !board[house_idx.get(i)].is_digit())
                })
                .map(|(d, which_cells)| {
                    (DigitFlags::only(DigitIndex::new(d).unwrap()), which_cells)
                }).collect();

            if digit_cells.len() < n { continue }

            // merge cells into cell groups (n-1) times, so that we get all the hidden groups
            let mut candidates = vec![ (DigitFlags::ZERO, SudokuFlags::ZERO) ];
            for _ in 0..n {
                candidates = candidates.into_iter().flat_map(|(digits, cells)|{
                    digit_cells.iter().filter_map(move |(d, c)|{
                        let new_digit = (digits & *d) == DigitFlags::ZERO;
                        let compatible = (cells | *c).count() <= n;
                        (new_digit && compatible).then_some((digits | *d, cells | *c))
                    })
                }).collect();
            }
            candidates
        };

        for (digits, cells) in candidates {
            let mask = digits;
            let mut changed = false;
            for cell_idx in cells.iter() {
                let cell_idx = house_idx.get(cell_idx);
                changed |= board[cell_idx].apply_mask(&mask);
            }

            if changed {
                let mut highlights = Vec::new();
                highlights.push(house_idx.into());
                for digit in digits.iter() {
                    for cell_idx in cells.iter() {
                        let cell_idx = house_idx.get(cell_idx);
                        if board[cell_idx].contains(digit) {
                            highlights.push((cell_idx, digit).into());
                        }
                    }
                }

                return ControlFlow::Break(highlights)
            }
        }
    }
    ControlFlow::Continue(())
}
