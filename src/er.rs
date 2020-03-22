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
use graphlib::Graph;
use graphlib::VertexId;
use rand_core::SeedableRng;
use rand::seq::SliceRandom;

#[allow(dead_code)]
pub struct ER<T: Node, R: rand::Rng + SeedableRng> {
    graph: Graph<T>,
    prob: f64,
    ids: Vec<VertexId>,
    rng: R,
}
/// This is going to become an Erdős-Rényi graph
#[allow(dead_code)]
impl<T: Node, R: rand::Rng + SeedableRng> ER<T, R> {
    fn random(&mut self) {
        if self.graph.edge_count() > 0 {
            panic!("cant randomize graph which already has edges in it -> not implemented yet")
        }
        for i in 0..self.ids.len() {
            for j in i+1..self.ids.len() {
                println!("i,j {}, {}", i, j);
                if self.rng.gen::<f64>() <= self.prob {
                    println!("yes!");
                    self.graph.add_edge(&self.ids[i], &self.ids[j]).unwrap();
                }
            }
        }
    }

    pub fn random_step(&mut self) {
        let sample: Vec<_> = self.ids
            .choose_multiple(&mut self.rng, 2)
            .collect();
        println!("{:?}", sample);
        if self.graph.has_edge(&sample[0], &sample[1]) {
            panic!("Has edge!");
        }else {
            let i = self.graph.fetch(&sample[0]).unwrap().get_id();
            let j = self.graph.fetch(&sample[1]).unwrap().get_id();
            println!("{}, {}", i, j);
            panic!("=?")
        }
        //self.graph.add_edge(&sample[0], &sample[1]).unwrap();
    }

    pub fn new(size: usize, prob: f64, rng: R) -> Self {
        let mut graph: Graph<T> = Graph::with_capacity(size);
        let mut ids = Vec::new();
        for i in 0..size {
            ids.push(graph.add_vertex(T::new_empty(i)));
        }
        let mut e = ER {
            graph,
            prob,
            ids,
            rng,
        };
        e.random();
        e
    }

    pub fn get_graph(&self) -> &Graph<T> {
        &self.graph
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::node::TestNode;

    #[test]
    fn test_graph_construction() {
        let rng = Pcg64::seed_from_u64(76);
        let e = ER::<TestNode, Pcg64>::new(20, 0.3, rng);
        assert_eq!(e.get_graph().edge_count(), 56);
        assert_eq!(20, e.get_graph().vertex_count());
    }

    #[test]
    fn test_rand_step() {
        let rng = Pcg64::seed_from_u64(76);
        let mut e = ER::<TestNode, Pcg64>::new(20, 2.0, rng);
        assert_eq!(20, e.get_graph().vertex_count());
        assert_eq!(190, e.get_graph().edge_count() );
        e.random_step();
    }
}
