use crate::SwGraph;
use crate::traits::*;
use core::cmp::{min,max};
use crate::SwChangeState;

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

    /// r_prob is probability of rewire
    pub fn new(n: u32, r_prob: f64, rng: R) -> Self {
        let graph = SwGraph::new(n);
        let mut result =
            SwEnsemble {
                graph,
                r_prob,
                rng,
            };
        result.randomize();
        result
    }

    fn draw_remaining(&mut self, index0: u32, index1: u32, high: u32) -> u32 {
        let num = self.rng.gen_range(0, high);
        let min = min(index0, index1);
        let max = max(index0, index1);

        if num < min {
            num
        } else if num + 1 < max {
            num + 1
        } else {
            num + 2
        }
    }

    fn randomize_edge(&mut self, index0: u32, index1: u32) -> SwChangeState {
        let vertex_count = self.graph.vertex_count();

        if self.rng.gen::<f64>() <= self.r_prob {
            let rewire_index = self.
            draw_remaining(index0, index1, vertex_count - 2);
            self.graph.rewire_edge(index0, index1, rewire_index)
        }else {
            self.graph.reset_edge(index0, index1)
        }
    }

    pub fn randomize(&mut self){
        self.graph
            .init_ring_2();
        let count = self.graph
            .vertex_count();

        for i in 0..count {
            let vertex = self.graph
                .get_mut_unchecked(i as usize);

            let mut root_iter = vertex
                .iter_raw_edges()
                .filter(|edge| edge.is_root())
                .map(|edge| edge.to());

            let first   = *root_iter.next().unwrap();
            let second  = *root_iter.next().unwrap();
            debug_assert!(root_iter.next().is_none());
            self.randomize_edge(i, first);
            self.randomize_edge(i, second);
        }
    }

    pub fn graph(&self) -> &SwGraph<T> {
        &self.graph
    }

}
