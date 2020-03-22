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
    rng: R,
}
/// This is going to become an Erdős-Rényi graph
#[allow(dead_code)]
impl<T: Node, R: rand::Rng + SeedableRng> ER<T, R> {
    fn random(&mut self) {
        if self.graph.edge_count() > 0 {
            panic!("cant randomize graph which already has edges in it -> not implemented yet")
        }
        for i in 0..self.graph.vertex_count() {
            for j in i+1..self.graph.vertex_count() {
                println!("i,j {}, {}", i, j);
                if self.rng.gen::<f64>() <= self.prob {
                    println!("yes!");
                    self.graph.add_edge(i, j).unwrap();
                }
            }
        }
    }

    pub fn random_step(&mut self) {
        let (e1, e2) = draw_two_from_range(&mut self.rng, self.graph.vertex_count());
        println!("(e1: {}, e2: {})", e1, e2);

        // should remove edge here if result is err
        self.graph.add_edge(e1,e2);
        //self.graph.add_edge(&sample[0], &sample[1]).unwrap();
    }

    pub fn new(size: u32, prob: f64, rng: R) -> Self {
        let graph: Graph<T> = Graph::new(size);
        let mut e = ER {
            graph,
            prob,
            rng,
        };
        e.random();
        e
    }

    pub fn get_graph(&self) -> &Graph<T> {
        &self.graph
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
