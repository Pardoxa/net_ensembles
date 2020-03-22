//! # Create Erdős-Rényi networks
//!
//! # Example:
//! ```
//! use net_ensembles;
//! use net_ensembles::Node;
//! use rand_pcg::Pcg64;
//! use rand_core::SeedableRng;
//!
//! // Define your own node
//! struct ExampleNode { }
//!
//! // it has to implement the Node trait
//! impl Node for ExampleNode {
//!     fn new_empty() -> Self {
//!         ExampleNode { }
//!     }
//! }
//!
//! let rng = Pcg64::seed_from_u64(76);
//! let e = net_ensembles::ER::<ExampleNode, Pcg64>::new(20, 0.3, rng);
//! assert_eq!(20, e.get_graph().vertex_count());
//! ```
use crate::node::Node;
use rand_core::SeedableRng;
use crate::graph::Graph;

#[allow(dead_code)]
pub struct ER<T: Node, R: rand::Rng + SeedableRng> {
    graph: Graph<T>,
    prob: f64,
    c: f64,
    rng: R,
}
/// This is going to become an Erdős-Rényi graph
#[allow(dead_code)]
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

    /// Add or remove edge according to ER-prob
    pub fn random_step(&mut self) {
        let (e1, e2) = draw_two_from_range(&mut self.rng, self.graph.vertex_count());

        // Try to add edge. else: remove edge
        if self.rng.gen::<f64>() <= self.prob {
            let _ = self.graph.add_edge(e1, e2);
        } else {
            let _ = self.graph.remove_edge(e1, e2);
        }
    }

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

    pub fn get_graph(&self) -> &Graph<T> {
        &self.graph
    }

    fn get_mut_graph(&mut self) -> &mut Graph<T> {
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
        (first, second +1)
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
        let ec = e.get_graph().edge_count();
        assert_eq!(0, ec);
        // empty graph should not be connected:
        assert!(
            !e.get_graph()
                .is_connected()
        );

        // add edge
        e.get_mut_graph()
            .add_edge(0,1)
            .unwrap();
        let ec_1 = e.get_graph().edge_count();
        assert_eq!(1, ec_1);

        let mut res = e.get_mut_graph()
            .add_edge(0,1);
        assert!(res.is_err());

        // remove edge
        e.get_mut_graph()
            .remove_edge(0, 1)
            .unwrap();
        let ec_0 = e.get_graph().edge_count();
        assert_eq!(0, ec_0);

        res = e.get_mut_graph()
            .remove_edge(0, 1);
        assert!(res.is_err());
    }

    #[test]
    fn test_graph_construction() {
        let rng = Pcg64::seed_from_u64(76);
        let e = ER::<TestNode, Pcg64>::new(20, 2.7, rng);
        assert_eq!(e.get_graph().edge_count(), 28);
        assert_eq!(20, e.get_graph().vertex_count());
    }

    #[test]
    fn test_complete_graph() {
        let rng = Pcg64::seed_from_u64(76);
        let e = ER::<TestNode, Pcg64>::new(20, 19.0, rng);
        assert_eq!(20, e.get_graph().vertex_count());
        assert_eq!(190, e.get_graph().edge_count());
        assert!(e.get_graph().is_connected());
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
