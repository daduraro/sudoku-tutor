use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TwoColor {
    Red,
    Black,
}

impl TwoColor {
    const fn other(self) -> Self {
        match self {
            TwoColor::Black => TwoColor::Red,
            TwoColor::Red => TwoColor::Black,
        }
    }
}

pub struct Graph<Node> {
    nodes: Vec<Node>,
    edges: Vec<Vec<usize>>,
}

impl<Node> std::default::Default for Graph<Node> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl<Node> Graph<Node> {
    pub fn new(nodes: Vec<Node>, mut edges: Vec<Vec<usize>>) -> Self {
        edges.resize(nodes.len(), Vec::new());
        assert!(edges.iter().flatten().all(|i| *i < nodes.len()));

        // check symmetry
        for (i, e) in edges.iter().enumerate() {
            assert!(e.iter().all(|j| edges[*j].contains(&i)));
        }

        // check no 0-loops
        for (i, e) in edges.iter().enumerate() {
            assert!(e.iter().all(|j| *j != i));
        }
        Self {
            nodes,
            edges,
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn edges_of(&self, i: usize) -> &[usize] {
        &self.edges[i]
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
        self.edges.push(Vec::new());
    }

    pub fn add_edge(&mut self, i: usize, j: usize) {
        assert!(i < self.len());
        assert!(j < self.len());
        assert!(i != j);
        self.edges[i].push(j);
        self.edges[j].push(i);
    }

    fn find_connected_components(&self) -> Vec<usize> {
        let mut ccs = vec![0; self.nodes.len()];
        let mut processed = vec![false; self.nodes.len()];
        let mut current_cc = 0;
        for root in 0..self.nodes.len() {
            if processed[root] { continue }
            let mut stack = vec![root];
            while let Some(i) = stack.pop() {
                if processed[i] { continue }
                processed[i] = true;
                ccs[i] = current_cc;
                for &j in self.edges[i].iter() {
                    stack.push(j);
                }
            }
            current_cc += 1;
        }
        ccs
    }

    pub fn split_connected_components(self) -> Vec<Graph<Node>> {
        if self.nodes.is_empty() { return vec![self] }

        let ccs = self.find_connected_components();
        let num_ccs = ccs.iter().max().unwrap() + 1;

        let reindex: Vec<_> = {
            let mut reindex = vec![None; self.nodes.len()];
            let mut current_index = vec![0usize; num_ccs];

            for (i, &cc) in ccs.iter().enumerate() {
                reindex[i] = Some(current_index[cc]);
                current_index[cc] += 1;
            }

            reindex.into_iter().map(Option::unwrap).collect()
        };

        let mut new_graphs = Vec::new();
        for _ in 0..num_ccs { new_graphs.push((Vec::new(), Vec::new())); }

        for (i, (node, edges)) in self.nodes.into_iter().zip(self.edges).enumerate() {
            let graph = &mut new_graphs[ccs[i]];
            let edges: Vec<_> = edges.into_iter().map(|i| reindex[i]).collect();
            graph.0.push(node);
            graph.1.push(edges);
        }

        new_graphs.into_iter().map(|(nodes, edges)| Self::new(nodes, edges)).collect()
    }

    pub fn two_colorize(&self) -> Option<(Vec<usize>, Vec<usize>)> {
        if self.is_empty() { return None }

        let colors: Vec<_> = {
            let mut colors = vec![None; self.len()];

            for root in 0..self.len() {
                if colors[root].is_some() { continue }

                let mut stack = Vec::new();
                stack.push((root, TwoColor::Red));

                while let Some((i, color)) = stack.pop() {
                    if let Some(expected_color) = colors[i] {
                        if expected_color != color { return None }
                        else { continue }
                    }

                    colors[i] = Some(color);

                    for j in self.edges_of(i) {
                        stack.push((*j, color.other()));
                    }
                }
            }

            colors.into_iter().map(Option::unwrap).collect()
        };

        let mut red_nodes = Vec::new();
        let mut black_nodes = Vec::new();
        for (i, color) in colors.iter().enumerate() {
            let nodes = match color {
                TwoColor::Red => &mut red_nodes,
                TwoColor::Black => &mut black_nodes,
            };
            nodes.push(i);
        }

        Some((red_nodes, black_nodes))
    }

    pub fn shortest_chain(&self, origin: usize, dest: usize) -> Option<Vec<usize>> {
        let mut queue = VecDeque::new();
        queue.push_back(vec![origin]);

        while let Some(chain) = queue.pop_front() {
            let curr = *chain.last().unwrap();
            if curr == dest { return Some(chain) }
            for neighbour in self.edges_of(curr) {
                if chain.contains(neighbour) { continue } // graph loop, ignore
                let mut chain = chain.clone();
                chain.push(*neighbour);
                queue.push_back(chain);
            }
        }
        None
    }
}

impl<Node> std::ops::Index<usize> for Graph<Node> {
    type Output = Node;
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}
