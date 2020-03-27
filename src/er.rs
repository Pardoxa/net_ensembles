//! # Create Erdős-Rényi networks
//!
//! # Minimal example:
//! ```
//! use net_ensembles;
//! use net_ensembles::Node;
//! use rand_pcg::Pcg64;
//! use rand_core::SeedableRng;
//!
//! // Define your own node
//! #[derive(Clone)]
//! struct ExampleNode { }
//!
//! // it has to implement the Node trait
//! impl Node for ExampleNode {
//!     fn new_from_index(index: u32) -> Self {
//!         ExampleNode { }
//!     }
//! }
//!
//! // now choose your random number generator
//! let rng = Pcg64::seed_from_u64(76);
//! // the following creates an ER graph with 20 vertices and a connectivity of 0.3
//! // and uses thre random number generator rng
//! let e = net_ensembles::ER::<ExampleNode, Pcg64>::new(20, 0.3, rng);
//! assert_eq!(20, e.graph().vertex_count());
//! ```
//! Take a look at the struct `ER` for details
use crate::node::Node;
use rand_core::SeedableRng;
use crate::graph::Graph;
use crate::graph::GraphErrors;

/// # Implements Erdős-Rényi graph
pub struct ER<T: Node, R: rand::Rng + SeedableRng> {
    graph: Graph<T>,
    prob: f64,
    c: f64,
    rng: R,
}

impl<T: Node, R: rand::Rng + SeedableRng> ER<T, R> {
    fn random(&mut self) {
        if self.graph.edge_count() > 0 {
            panic!("cant randomize graph which already has edges in it -> not implemented yet")
        }
        // iterate over all possible edges once
        for i in 0..self.graph.vertex_count() {
            for j in i+1..self.graph.vertex_count() {
                if self.rng.gen::<f64>() <= self.prob {
                    self.graph.add_edge(i, j).unwrap();
                }
            }
        }
    }

    /// returns target connectivity
    /// # Explanation
    /// The target connectivity is the connectivity used to
    /// calculate the probability, that any two vertices a and b (where a != b)
    /// are connected.
    ///
    /// $p_{ij} = \frac{c_{target}}{N - 1}$
    /// where $N$ is the number of vertices in the graph
    pub fn get_target_connectivity(&self) -> f64 {
        self.c
    }

    /// Add or remove edge according to ER-prob
    ///
    /// will be changed shortly
    pub fn random_step(&mut self) -> ((u32, u32), Result<(), GraphErrors>) {
        let edge = draw_two_from_range(&mut self.rng, self.graph.vertex_count());

        // Try to add edge. else: remove edge
        if self.rng.gen::<f64>() <= self.prob {
            (
                edge,
                self.graph.add_edge(edge.0, edge.1)
            )
        } else {
            (
                edge,
                self.graph.remove_edge(edge.0, edge.1)
            )
        }
    }

    /// create new graph with `size` nodes and connectivity `c`.
    /// `rng` is consumed and used as rondom number generator in the following
    pub fn new(size: u32, c: f64, rng: R) -> Self {
        let prob = c / (size - 1) as f64;
        let graph: Graph<T> = Graph::new(size);
        let mut e = ER {
            graph,
            c,
            prob,
            rng,
        };
        e.random();
        e
    }

    /// returns reference to the underlying topology aka, the `Graph<T>`
    ///
    /// Use this to call functions regarding the topology
    pub fn graph(&self) -> &Graph<T> {
        &self.graph
    }

    #[allow(dead_code)]
    fn graph_mut(&mut self) -> &mut Graph<T> {
        &mut self.graph
    }
}

/// high is exclusive
fn draw_two_from_range<T: rand::Rng>(rng: &mut T, high: u32) -> (u32, u32){
    let first = rng.gen_range(0, high);
    let second = rng.gen_range(0, high - 1);
    return if second < first {
        (first, second)
    } else {
        (first, second + 1)
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::node::TestNode;

    fn test_graph(seed: u64, size: u32, c: f64) -> ER::<TestNode, Pcg64> {
        let rng = Pcg64::seed_from_u64(seed);
        ER::<TestNode, Pcg64>::new(size, c, rng)
    }

    #[test]
    fn test_edge_count() {
        // create empty graph
        let mut e = test_graph(12, 100, 0.0);
        let ec = e.graph().edge_count();
        assert_eq!(0, ec);
        // empty graph should not be connected:
        assert!(
            !e.graph()
                .is_connected()
                .expect("test_edge_count error 1")
        );

        // add edge
        e.graph_mut()
            .add_edge(0,1)
            .unwrap();
        let ec_1 = e.graph().edge_count();
        assert_eq!(1, ec_1);

        let mut res = e.graph_mut()
            .add_edge(0,1);
        assert!(res.is_err());

        // remove edge
        e.graph_mut()
            .remove_edge(0, 1)
            .unwrap();
        let ec_0 = e.graph().edge_count();
        assert_eq!(0, ec_0);

        res = e.graph_mut()
            .remove_edge(0, 1);
        assert!(res.is_err());
    }

    #[test]
    fn test_graph_construction() {
        let rng = Pcg64::seed_from_u64(76);
        let e = ER::<TestNode, Pcg64>::new(20, 2.7, rng);
        assert_eq!(e.graph().edge_count(), 28);
        assert_eq!(20, e.graph().vertex_count());
    }

    #[test]
    fn test_complete_graph() {
        let rng = Pcg64::seed_from_u64(76);
        let e = ER::<TestNode, Pcg64>::new(20, 19.0, rng);
        assert_eq!(20, e.graph().vertex_count());
        assert_eq!(190, e.graph().edge_count());
        assert!(e.graph().is_connected().expect("test_complete_graph error"));
    }

    #[test]
    fn draw_2(){
        let mut rng = Pcg64::seed_from_u64(762132);
        for _i in 0..100 {
            let (first, second) = draw_two_from_range(&mut rng, 2);
            assert!(first != second);
        }
        for i in 2..100 {
            let (first, second) = draw_two_from_range(&mut rng, i);
            assert!(first != second);
        }
    }
}
