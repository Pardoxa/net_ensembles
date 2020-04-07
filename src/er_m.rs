//! Erdős-Rényi with constant number of edges
use crate::graph::Graph;
use crate::Node;
use crate::traits::{Ensemble, EnsembleRng};
use crate::ErStepC;
use rand::seq::SliceRandom;

/// # Implements Erdős-Rényi graph ensemble
#[derive(Debug, Clone)]
pub struct ErEnsembleM<T: Node, R: rand::Rng> {
    graph: Graph<T>,
    m: usize,
    rng: R,
    possible_edges: Vec<(u32, u32)>,
    remaining: Vec<(u32, u32)>
}

impl <T, R> Ensemble<ErStepC, ErStepC> for ErEnsembleM<T, R>
    where   T: Node,
            R: rand::Rng,
{
    /// * undo a monte carlo step, return result-state
    /// * if you want to undo more than one step
    /// see [`undo_steps`](#method.undo_steps)
    fn undo_step(&mut self, step: ErStepC) -> ErStepC{
        unimplemented!()
    }

    /// * undo a monte carlo step, **panic** on invalid result state
    /// * for undoing multiple steps see [`undo_steps_quiet`](#method.undo_steps_quiet)
    fn undo_step_quiet(&mut self, step: ErStepC) -> (){
        unimplemented!()
    }

    /// # Randomizes self according to  model
    /// * this is intended for creation of initial sample
    /// * used in [`simple_sample`](#method.simple_sample)
    /// and [`simple_sample_vec`](#method.simple_sample_vec)
    fn randomize(&mut self){
        self.graph.clear_edges();
        // I enumerate the edges mentally
        let vertex_count = self.graph().vertex_count();
        // possible edges per vertex
        let p_edges_pv = vertex_count - 1;
        // Number of possible edges in a complete graph
        let p_edges = (vertex_count * p_edges_pv) / 2;
        self.shuffle_possible_edges();

        let random_edges = &self.possible_edges[..self.m];

        for edge in random_edges {
            self.graph
                .add_edge(edge.0, edge.1)
                .unwrap();
        }

        // uses mem-copy
        self.remaining
            .copy_from_slice(&self.possible_edges[self.m..])
    }

    /// # Monte Carlo step
    /// * use this to perform a Monte Carlo step
    /// * for doing multiple mc steps at once, use [`mc_steps`](#method.mc_steps)
    fn mc_step(&mut self) -> ErStepC{
        
        unimplemented!()
    }
}

impl<T: Node, R: rand::Rng> ErEnsembleM<T, R> {
    /// # Initialize
    /// * create new ErEnsembleM graph with `n` vertices
    /// * `m` edges
    /// * `rng` is consumed and used as random number generator in the following
    /// * internally uses `Graph<T>::new(n)`
    /// * generates random edges according to ErEnsembleC probability (see `ErEnsembleC::randomize`)
    pub fn new(n: u32, m: usize, rng: R) -> Self {
        let graph: Graph<T> = Graph::new(n);

        let p_edges = (n * (n - 1)) / 2;
        let mut vec = Vec::with_capacity(p_edges as usize);
        for i in 0..n {
            for j in i+1..n {
                vec.push((i, j));
            }
        }

        let mut e = ErEnsembleM {
            graph,
            m,
            rng,
            possible_edges: vec,
            remaining: vec![(0, 0); p_edges as usize - m]   // randomize will mem_copy - slice needs to be big enough
        };
        e.randomize();
        e
    }

    /// # Sort adjecency lists
    /// If you depend on the order of the adjecency lists, you can sort them
    /// # Performance
    /// * internally uses [pattern-defeating quicksort](https://github.com/orlp/pdqsort)
    /// as long as that is the standard
    /// * sorts an adjecency list with length `d` in worst-case: `O(d log(d))`
    /// * is called for each adjecency list, i.e., `self.vertex_count()` times
    pub fn sort_adj(&mut self) {
        self.graph_mut().sort_adj();
    }

    /// returns reference to the underlying topology aka, the `Graph<T>`
    ///
    /// Use this to call functions regarding the topology
    pub fn graph(&self) -> &Graph<T> {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut Graph<T> {
        &mut self.graph
    }

    /// Return total number of edges
    pub fn get_m(&self) -> usize {
        self.m
    }

    fn shuffle_possible_edges(&mut self) -> () {
        self.possible_edges
            .shuffle(&mut self.rng);
    }
}
