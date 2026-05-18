use std::ops::ControlFlow;

use itertools::Itertools;

use crate::board::SudokuBoard;
use crate::flags::DigitFlags;
use crate::graph::TwoColorized;
use crate::highlight::Highlight;
use crate::index::DigitIndex;
use crate::strategy::common::bilocation_graphs;

pub fn apply(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
    for digit in DigitIndex::iter() {
        let mask = DigitFlags::all_but(digit);
        for graph in bilocation_graphs(board, digit) {
            match graph.two_colorize() {
                TwoColorized::Inconsistent {
                    inconsistent,
                    other,
                } => {
                    for idx in inconsistent.into_iter().map(|v| graph[v]) {
                        board[idx].apply_mask(&mask);
                    }
                    return ControlFlow::Break(
                        other
                            .into_iter()
                            .map(|i| Highlight::Digit((graph[i], digit)))
                            .collect(),
                    );
                }
                TwoColorized::Consistent(group_a, group_b) => {
                    for (&i, j) in group_a.iter().cartesian_product(group_b) {
                        let c0 = graph[i];
                        let c1 = graph[j];
                        let mut changed = false;
                        for c in c0.cells_visible_with(&c1) {
                            changed |= board[c].apply_mask(&mask);
                        }
                        if changed {
                            let mut highlights = vec![(c0, digit).into(), (c1, digit).into()];
                            for idx in graph.shortest_chain(i, j).unwrap() {
                                highlights.push(graph[idx].into());
                            }
                            return ControlFlow::Break(highlights);
                        }
                    }
                }
            }
        }
    }

    ControlFlow::Continue(())
}
