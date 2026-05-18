use std::ops::ControlFlow;

use super::common::visibility_graphs;
use crate::board::SudokuBoard;
use crate::graph::TwoColorized;
use crate::highlight::Highlight;
use crate::index::CellIndex;
use itertools::Itertools;

pub fn apply_remote_pairs(board: &mut SudokuBoard) -> ControlFlow<Vec<Highlight>> {
    let bv_cells = CellIndex::iter().filter(|idx| board[idx].is_bivalue());

    let bv_cells_groups: Vec<_> = bv_cells
        .into_group_map_by(|idx| board[idx])
        .into_iter()
        .filter(|(_, v)| v.len() >= 3)
        .collect();
    for (cell_value, cells) in bv_cells_groups {
        debug_assert!(cells.iter().all(|idx| board[idx] == cell_value));

        let mask = !cell_value.digit_flags();
        for graph in visibility_graphs(&cells) {
            if let TwoColorized::Consistent(group_a, group_b) = graph.two_colorize() {
                for (&i, j) in group_a.iter().cartesian_product(group_b) {
                    let c0 = graph[i];
                    let c1 = graph[j];

                    let mut changed = false;
                    for c in c0.cells_visible_with(&c1) {
                        changed |= board[c].apply_mask(&mask);
                    }
                    if changed {
                        let mut highlights = Vec::new();
                        for digit in cell_value.digits() {
                            highlights.push((c0, digit).into());
                            highlights.push((c1, digit).into());
                        }

                        for idx in graph.shortest_chain(i, j).unwrap() {
                            highlights.push(graph[idx].into());
                        }

                        return ControlFlow::Break(highlights);
                    }
                }
            }
        }
    }

    ControlFlow::Continue(())
}
