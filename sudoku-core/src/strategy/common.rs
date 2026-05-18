use crate::{
    board::SudokuBoard,
    graph::Graph,
    index::{CellIndex, DigitIndex},
};

#[allow(dead_code)]
pub fn visibility_graphs(indices: &[CellIndex]) -> Vec<Graph<CellIndex>> {
    let mut graph = Graph::new(indices.to_vec(), Vec::new());
    for i in 0..graph.len() {
        for j in (i + 1)..graph.len() {
            if graph[i].visible(&graph[j]) {
                graph.add_edge(i, j);
            }
        }
    }

    graph.split_connected_components()
}

pub fn bilocation_graphs(board: &SudokuBoard, digit: DigitIndex) -> Vec<Graph<CellIndex>> {
    let cells: Vec<_> = board
        .indexed_iter()
        .filter_map(|(cell_idx, cell)| cell.contains(digit).then_some(cell_idx))
        .collect();

    let mut graph = Graph::new(cells, Vec::new());
    for i in 0..graph.len() {
        for j in (i + 1)..graph.len() {
            if board.is_bilocation_link(graph[i], graph[j], digit) {
                graph.add_edge(i, j);
            }
        }
    }

    graph
        .split_connected_components()
        .into_iter()
        .filter(|g| g.nodes().len() > 1)
        .collect()
}
