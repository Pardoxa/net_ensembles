use crate::*;
use crate::spacial::*;
use rand::Rng;
use std::f64::consts::PI;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SpacialEnsemble<T, R> 
where T: Node {
    graph: SpacialGraph<T>,
    rng: R,
    f: f64,
    alpha: f64,
    sqrt_n_pi: f64,
}


impl<T, R> SpacialEnsemble<T, R> 
    where 
        T: Node, 
        R: Rng,
{
    /// Generate a new Spacial ensemble with 
    /// * `n` nodes
    /// * `rng` as random number generator
    /// * `f` - see paper
    /// * `alpha` - see paper
    pub fn new(n: usize, mut rng: R, f: f64, alpha: f64) -> Self
    {
        let mut graph = SpacialGraph::new(n);

        graph.vertices
            .iter_mut()
            .for_each(|v|
                {
                    v.x = rng.gen();
                    v.y = rng.gen();
                }
            );

        let mut res = Self{
            graph,
            rng,
            alpha,
            f,
            sqrt_n_pi: (n as f64 * PI).sqrt()
        };
        res.randomize();
        res
    }
}


impl<T, R> SimpleSample for SpacialEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng,
{
    /// # Randomizes the edges according to Er probabilities
    /// * this is used by `ErEnsembleC::new` to create the initial topology
    /// * you can use this for sampling the ensemble
    /// * runs in `O(vertices * vertices)`
    fn randomize(&mut self) {
        self.graph.clear_edges();
        // iterate over all possible edges once
        for i in 0..self.graph.vertex_count() {
            for j in i+1..self.graph.vertex_count() {
                let distance = unsafe{
                    self.graph
                        .vertices
                        .get_unchecked(i)
                        .distance(self.graph.vertices.get_unchecked(j))
                };
                let prob = self.f * 
                    (1.0 + self.sqrt_n_pi * distance / self.alpha)
                    .powf(-self.alpha);
                if self.rng.gen::<f64>() <= prob {
                    // in these circumstances equivalent to 
                    // self.graph.add_edge(i, j).unwrap();
                    // but without checking for existing edges and other errors -> a bit more efficient
                    unsafe{
                        self.graph
                            .vertices
                            .get_unchecked_mut(i)
                            .adj
                            .push(j);
                        self.graph
                            .vertices
                            .get_unchecked_mut(j)
                            .adj
                            .push(i);
                    }
                    
                    self.graph.edge_count += 1;
                }
            }
        }
    }
}