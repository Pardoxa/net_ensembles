use crate::SwGraph;
use crate::traits::*;


/// # Implementssmall-world graph ensemble
#[derive(Debug, Clone)]
pub struct SwEnsemble<T: Node, R: rand::Rng> {
    graph: SwGraph<T>,
    r_prob: f64,
    rng: R,
}

impl <T, R> SwEnsemble<T, R>
    where   T: Node,
            R: rand::Rng,
{
    pub fn new(n: u32, r_prob: f64, rng: R) -> Self {
        let mut graph = SwGraph::new(n);
        graph.init_ring_2();
        SwEnsemble {
            graph,
            r_prob,
            rng,
        }
    }

    pub fn graph(&self) -> &SwGraph<T> {
        &self.graph
    }

}
